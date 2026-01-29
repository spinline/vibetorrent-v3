use crate::scgi::{send_request, ScgiRequest};
use quick_xml::events::Event;
use quick_xml::reader::Reader;


// Simple helper to build an XML-RPC method call
pub fn build_method_call(method: &str, params: &[&str]) -> String {
    let mut xml = String::from("<?xml version=\"1.0\"?>\n<methodCall>\n");
    xml.push_str(&format!("<methodName>{}</methodName>\n<params>\n", method));
    for param in params {
        xml.push_str("<param><value><string><![CDATA[");
        xml.push_str(param);
        xml.push_str("]]></string></value></param>\n");
    }
    xml.push_str("</params>\n</methodCall>");
    xml
}

pub struct RtorrentClient {
    socket_path: String,
}

impl RtorrentClient {
    pub fn new(socket_path: &str) -> Self {
        Self {
            socket_path: socket_path.to_string(),
        }
    }

    pub async fn call(&self, method: &str, params: &[&str]) -> Result<String, String> {
        let xml = build_method_call(method, params);
        let req = ScgiRequest::new().body(xml.into_bytes());
        
        match send_request(&self.socket_path, req).await {
            Ok(bytes) => {
                let s = String::from_utf8_lossy(&bytes).to_string();
                Ok(s)
            }
            Err(e) => Err(format!("{:?}", e)),
        }
    }
}

// Specialized parser for d.multicall2 response
// Expected structure:
// <methodResponse><params><param><value><array><data>
//   <value><array><data>
//     <value><string>HASH</string></value>
//     <value><string>NAME</string></value>
//     ...
//   </data></array></value>
// ...
// </data></array></value></param></params></methodResponse>

pub fn parse_multicall_response(xml: &str) -> Result<Vec<Vec<String>>, String> {
    let mut reader = Reader::from_str(xml);
    reader.trim_text(true);

    let mut buf = Vec::new();
    let mut results = Vec::new();
    let mut current_row = Vec::new();
    let mut inside_value = false;
    let mut current_text = String::new();

    // Loop through events
    // Strategy: We look for <data> inside the outer array. 
    // The outer array contains values which are arrays (rows).
    // Each row array contains values (columns).
    
    // Simplified logic: flatten all <value>... content, but respect structure? 
    // Actually, handling nested arrays properly with a streaming parser is tricky.
    // Let's rely on the fact that d.multicall2 returns a 2D array.
    // Depth 0: methodResponse/params/param/value/array/data
    // Depth 1: value (row) / array / data
    // Depth 2: value (col) / type (string/i8/i4)
    
    // We can count <array> depth.
    
    let mut array_depth = 0;
    
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                match e.name().as_ref() {
                    b"array" => array_depth += 1,
                    b"value" => inside_value = true,
                    _ => (),
                }
            }
            Ok(Event::End(ref e)) => {
                match e.name().as_ref() {
                    b"array" => {
                        array_depth -= 1;
                        // If we just finished a row (depth 1 which means the inner array of the main list)
                        if array_depth == 1 {
                             if !current_row.is_empty() {
                                 results.push(current_row.clone());
                                 current_row.clear();
                             }
                        }
                    },
                    b"value" => {
                        inside_value = false;
                        // If we are at depth 2 (inside a column value)
                        if array_depth == 2 && !current_text.is_empty() {
                             current_row.push(current_text.clone());
                             current_text.clear();
                        } else if array_depth == 2 {
                             // Empty value or non-text?
                             // Sometimes values are empty, e.g. empty string
                             // We should push it if we just closed a value at depth 2
                             // But wait, the text event handles the content.
                             // Logic: If we closed value at depth 2, we push the collected text (which might be empty).
                             // To handle empty text correctly, we should clear text at Start(value) or use a flag.
                             if inside_value == false { // we just closed it
                                 current_row.push(current_text.clone());
                                 current_text.clear();
                             }
                        }
                    }
                    _ => (),
                }
            }
            Ok(Event::Text(e)) => {
                if inside_value && array_depth == 2 {
                    current_text = e.unescape().unwrap().into_owned();
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(format!("Parse error: {:?}", e)),
            _ => (),
        }
        buf.clear();
    }

    Ok(results)
}

// Parse a simple string response from a method call
// Expected: <methodResponse><params><param><value><string>RESULT</string></value></param></params></methodResponse>
pub fn parse_string_response(xml: &str) -> Result<String, String> {
    let mut reader = Reader::from_str(xml);
    reader.trim_text(true);
    let mut buf = Vec::new();
    let mut result = String::new();
    let mut inside_string = false;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                if e.name().as_ref() == b"string" {
                    inside_string = true;
                }
            }
            Ok(Event::Text(e)) => {
                if inside_string {
                    result = e.unescape().unwrap().into_owned();
                }
            }
            Ok(Event::End(ref e)) => {
                if e.name().as_ref() == b"string" {
                     // inside_string = false;
                     // Assuming only one string in the response which matters
                    break;
                }
            }
            Ok(Event::Eof) => break,
            _ => (),
        }
    }
    
    if result.is_empty() {
        // It might be empty string or we didn't find it.
        // If xml contains "fault", we should verify. 
        if xml.contains("fault") {
             return Err("RPC Fault detected".to_string());
        }
    }

    Ok(result)
}
