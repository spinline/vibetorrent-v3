use crate::scgi::{send_request, ScgiRequest};
use quick_xml::de::from_str;
use quick_xml::se::to_string;
use serde::{Deserialize, Serialize};

// --- Request Models ---

#[derive(Debug, Serialize)]
#[serde(rename = "methodCall")]
struct MethodCall<'a> {
    #[serde(rename = "methodName")]
    method_name: &'a str,
    params: RequestParams<'a>,
}

#[derive(Debug, Serialize)]
struct RequestParams<'a> {
    param: Vec<RequestParam<'a>>,
}

#[derive(Debug, Serialize)]
struct RequestParam<'a> {
    value: RequestValueInner<'a>,
}

#[derive(Debug, Serialize)]
struct RequestValueInner<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    string: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    i4: Option<i32>,
}

// --- Response Models for d.multicall2 ---

#[derive(Debug, Deserialize)]
#[serde(rename = "methodResponse")]
struct MulticallResponse {
    params: MulticallResponseParams,
}

#[derive(Debug, Deserialize)]
struct MulticallResponseParams {
    param: MulticallResponseParam,
}

#[derive(Debug, Deserialize)]
struct MulticallResponseParam {
    value: MulticallResponseValueArray,
}

// Top level array in d.multicall2 response
#[derive(Debug, Deserialize)]
struct MulticallResponseValueArray {
    array: MulticallResponseDataOuter,
}

#[derive(Debug, Deserialize)]
struct MulticallResponseDataOuter {
    data: MulticallResponseDataOuterValue,
}

#[derive(Debug, Deserialize)]
struct MulticallResponseDataOuterValue {
    #[serde(rename = "value", default)]
    values: Vec<MulticallRowValue>,
}

// Each row in the response
#[derive(Debug, Deserialize)]
struct MulticallRowValue {
    array: MulticallResponseDataInner,
}

#[derive(Debug, Deserialize)]
struct MulticallResponseDataInner {
    data: MulticallResponseDataInnerValue,
}

#[derive(Debug, Deserialize)]
struct MulticallResponseDataInnerValue {
    #[serde(rename = "value", default)]
    values: Vec<MulticallItemValue>,
}

// Each item in a row (column)
#[derive(Debug, Deserialize)]
struct MulticallItemValue {
    #[serde(rename = "string", default)]
    string: Option<String>,
    #[serde(rename = "i4", default)]
    i4: Option<i64>,
    #[serde(rename = "i8", default)]
    i8: Option<i64>,
}

impl MulticallItemValue {
    fn to_string_lossy(&self) -> String {
        if let Some(s) = &self.string {
            s.clone()
        } else if let Some(i) = self.i4 {
            i.to_string()
        } else if let Some(i) = self.i8 {
            i.to_string()
        } else {
            String::new()
        }
    }
}

// --- Response Model for simple string ---

#[derive(Debug, Deserialize)]
#[serde(rename = "methodResponse")]
struct StringResponse {
    params: StringResponseParams,
}

#[derive(Debug, Deserialize)]
struct StringResponseParams {
    param: StringResponseParam,
}

#[derive(Debug, Deserialize)]
struct StringResponseParam {
    value: StringResponseValue,
}

#[derive(Debug, Deserialize)]
struct StringResponseValue {
    string: String,
}

// --- Client Implementation ---

pub struct RtorrentClient {
    socket_path: String,
}

impl RtorrentClient {
    pub fn new(socket_path: &str) -> Self {
        Self {
            socket_path: socket_path.to_string(),
        }
    }

    /// Helper to build and serialize XML-RPC method call
    fn build_method_call(&self, method: &str, params: &[&str]) -> Result<String, String> {
        let req_params = RequestParams {
            param: params
                .iter()
                .map(|p| RequestParam {
                    value: RequestValueInner {
                        string: Some(p),
                        i4: None,
                    },
                })
                .collect(),
        };

        let call = MethodCall {
            method_name: method,
            params: req_params,
        };

        let xml_body = to_string(&call).map_err(|e| format!("Serialization error: {}", e))?;
        Ok(format!("<?xml version=\"1.0\"?>\n{}", xml_body))
    }

    pub async fn call(&self, method: &str, params: &[&str]) -> Result<String, String> {
        let xml = self.build_method_call(method, params)?;
        let req = ScgiRequest::new().body(xml.into_bytes());

        match send_request(&self.socket_path, req).await {
            Ok(bytes) => {
                let s = String::from_utf8_lossy(&bytes).to_string();
                Ok(s)
            }
            Err(e) => Err(format!("SCGI Error: {:?}", e)),
        }
    }
}

pub fn parse_multicall_response(xml: &str) -> Result<Vec<Vec<String>>, String> {
    let response: MulticallResponse =
        from_str(xml).map_err(|e| format!("XML Parse Error: {}", e))?;

    let mut result = Vec::new();

    for row in response.params.param.value.array.data.values {
        let mut row_vec = Vec::new();
        for item in row.array.data.values {
            row_vec.push(item.to_string_lossy());
        }
        result.push(row_vec);
    }

    Ok(result)
}

pub fn parse_string_response(xml: &str) -> Result<String, String> {
    let response: StringResponse = from_str(xml).map_err(|e| format!("XML Parse Error: {}", e))?;
    Ok(response.params.param.value.string)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_method_call() {
        let client = RtorrentClient::new("dummy");
        let xml = client
            .build_method_call("d.multicall2", &["", "main", "d.name="])
            .unwrap();

        println!("Generated XML: {}", xml);

        assert!(xml.contains("<methodName>d.multicall2</methodName>"));
        // With struct option serialization, it should produce <value><string>...</string></value>
        assert!(xml.contains("<value><string>main</string></value>"));
    }

    #[test]
    fn test_parse_multicall_response() {
        let xml = r#"<methodResponse>
<params>
<param>
<value>
<array>
<data>
<value>
<array>
<data>
<value><string>HASH123</string></value>
<value><string>Ubuntu ISO</string></value>
<value><i4>1024</i4></value>
</data>
</array>
</value>
</data>
</array>
</value>
</param>
</params>
</methodResponse>
"#;
        let result = parse_multicall_response(xml).expect("Failed to parse");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0][0], "HASH123");
        assert_eq!(result[0][1], "Ubuntu ISO");
        assert_eq!(result[0][2], "1024");
    }
}
