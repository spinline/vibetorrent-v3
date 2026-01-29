use bytes::Bytes;
use std::collections::HashMap;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;

#[derive(Debug)]
pub enum ScgiError {
    #[allow(dead_code)]
    Io(std::io::Error),
    #[allow(dead_code)]
    Protocol(String),
}

impl From<std::io::Error> for ScgiError {
    fn from(err: std::io::Error) -> Self {
        ScgiError::Io(err)
    }
}

pub struct ScgiRequest {
    headers: HashMap<String, String>,
    body: Vec<u8>,
}

impl ScgiRequest {
    pub fn new() -> Self {
        let mut headers = HashMap::new();
        headers.insert("SCGI".to_string(), "1".to_string());
        Self {
            headers,
            body: Vec::new(),
        }
    }

    pub fn _header(mut self, key: &str, value: &str) -> Self {
        self.headers.insert(key.to_string(), value.to_string());
        self
    }

    pub fn body(mut self, body: Vec<u8>) -> Self {
        self.body = body;
        self.headers
            .insert("CONTENT_LENGTH".to_string(), self.body.len().to_string());
        self
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut headers_data = Vec::new();
        
        // SCGI Spec: The first header must be "CONTENT_LENGTH"
        // The second header must be "SCGI" with value "1"
        
        // We handle CONTENT_LENGTH and SCGI explicitly first
        let content_len = self.body.len().to_string();
        headers_data.extend_from_slice(b"CONTENT_LENGTH");
        headers_data.push(0);
        headers_data.extend_from_slice(content_len.as_bytes());
        headers_data.push(0);
        
        headers_data.extend_from_slice(b"SCGI");
        headers_data.push(0);
        headers_data.extend_from_slice(b"1");
        headers_data.push(0);

        // Add remaining headers (excluding the ones we just added if they exist in the map)
        for (k, v) in &self.headers {
            if k == "CONTENT_LENGTH" || k == "SCGI" {
                continue;
            }
            headers_data.extend_from_slice(k.as_bytes());
            headers_data.push(0);
            headers_data.extend_from_slice(v.as_bytes());
            headers_data.push(0);
        }

        let headers_len = headers_data.len();
        let mut packet = Vec::new();
        let len_str = headers_len.to_string();
        packet.extend_from_slice(len_str.as_bytes());
        packet.push(b':');
        packet.extend(headers_data);
        packet.push(b',');
        packet.extend(&self.body);

        packet
    }
}

pub async fn send_request(
    socket_path: &str,
    request: ScgiRequest,
) -> Result<Bytes, ScgiError> {
    let mut stream = UnixStream::connect(socket_path).await?;
    let data = request.encode();
    stream.write_all(&data).await?;

    let mut response = Vec::new();
    stream.read_to_end(&mut response).await?;

    // The response is usually HTTP-like: headers\r\n\r\nbody
    // We strictly want the body for XML-RPC
    // Find double newline
    let double_newline = b"\r\n\r\n";
    if let Some(pos) = response
        .windows(double_newline.len())
        .position(|window| window == double_newline)
    {
        Ok(Bytes::from(response.split_off(pos + double_newline.len())))
    } else {
         // Fallback: rTorrent sometimes sends raw XML without headers if configured poorly, 
         // but SCGI usually implies headers.
         // If we don't find headers, maybe it's all body? 
         // But usually there's at least "Status: 200 OK"
         // Let's return everything if we can't find the split, or error.
         // For now, assume everything is body if no headers found might be unsafe, 
         // but valid for simple XML-RPC dumping.
         Ok(Bytes::from(response))
    }
}
pub async fn system_call(
    socket_path: &str,
    method: &str,
    params: Vec<&str>,
) -> Result<String, ScgiError> {
    // Construct XML-RPC payload manually for simplicity
    // <methodCall><methodName>method</methodName><params><param><value><string>val</string></value></param>...</params></methodCall>
    let mut xml = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>");
    xml.push_str(&format!("<methodCall><methodName>{}</methodName><params>", method));
    for param in params {
        // Use CDATA for safety with special chars in magnet links
        xml.push_str(&format!("<param><value><string><![CDATA[{}]]></string></value></param>", param));
    }
    xml.push_str("</params></methodCall>");

    println!("Sending XML-RPC Payload: {}", xml); // Debug logging

    let req = ScgiRequest::new().body(xml.clone().into_bytes());
    let response_bytes = send_request(socket_path, req).await?;
    let response_str = String::from_utf8_lossy(&response_bytes).to_string();

    // Ideally parse the response, but for actions we just check if it executed without SCGI error
    // rTorrent usually returns <value><i8>0</i8></value> for success or fault.
    // For now, returning the raw string is fine for debugging/logging in main.
    
    Ok(response_str)
}
