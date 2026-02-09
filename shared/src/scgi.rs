#![cfg(feature = "ssr")]

use bytes::Bytes;
use std::collections::HashMap;
use thiserror::Error;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;

#[derive(Error, Debug)]
pub enum ScgiError {
    #[error("IO Error: {0}")]
    Io(#[from] std::io::Error),
    #[allow(dead_code)]
    #[error("Protocol Error: {0}")]
    Protocol(String),
    #[error("Timeout: SCGI request took too long")]
    Timeout,
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
    let perform_request = async {
        let mut stream = UnixStream::connect(socket_path).await?;
        let data = request.encode();
        stream.write_all(&data).await?;

        let mut response = Vec::new();
        stream.read_to_end(&mut response).await?;
        Ok::<Vec<u8>, std::io::Error>(response)
    };

    let response = tokio::time::timeout(std::time::Duration::from_secs(10), perform_request)
        .await
        .map_err(|_| ScgiError::Timeout)??;

    let mut response_vec = response;
    
    // Improved header stripping: find the first occurrence of "<?xml" OR double newline
    let patterns = [
        &b"\r\n\r\n"[..],
        &b"\n\n"[..],
        &b"<?xml"[..] // If headers are missing or weird, find start of XML
    ];

    let mut found_pos = None;
    for (i, pattern) in patterns.iter().enumerate() {
        if let Some(pos) = response_vec
            .windows(pattern.len())
            .position(|window| window == *pattern)
        {
            // For XML pattern, we keep it. For newlines, we skip them.
            if i == 2 {
                found_pos = Some(pos);
            } else {
                found_pos = Some(pos + pattern.len());
            }
            break;
        }
    }

    if let Some(pos) = found_pos {
        Ok(Bytes::from(response_vec.split_off(pos)))
    } else {
        Ok(Bytes::from(response_vec))
    }
}