use bytes::Bytes;
use std::collections::HashMap;
use thiserror::Error;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;

#[derive(Error, Debug)]
pub enum ScgiError {
    #[error("IO Error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Protocol Error: {0}")]
    Protocol(String),
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

        let content_len = self.body.len().to_string();
        headers_data.extend_from_slice(b"CONTENT_LENGTH");
        headers_data.push(0);
        headers_data.extend_from_slice(content_len.as_bytes());
        headers_data.push(0);

        headers_data.extend_from_slice(b"SCGI");
        headers_data.push(0);
        headers_data.extend_from_slice(b"1");
        headers_data.push(0);

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

pub async fn send_request(socket_path: &str, request: ScgiRequest) -> Result<Bytes, ScgiError> {
    let mut stream = UnixStream::connect(socket_path).await?;
    let data = request.encode();
    stream.write_all(&data).await?;

    let mut response = Vec::new();
    stream.read_to_end(&mut response).await?;

    let double_newline = b"\r\n\r\n";
    if let Some(pos) = response
        .windows(double_newline.len())
        .position(|window| window == double_newline)
    {
        Ok(Bytes::from(response.split_off(pos + double_newline.len())))
    } else {
        Ok(Bytes::from(response))
    }
}
