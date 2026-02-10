use leptos::server_fn::codec::{Encoding, FromReq, FromRes, IntoReq, IntoRes};
use leptos::server_fn::request::{ClientReq, Req};
use leptos::server_fn::response::{ClientRes, Res};
use leptos::server_fn::error::ServerFnError;
use serde::{de::DeserializeOwned, Serialize};
use std::future::Future;

pub struct MessagePack;

impl Encoding for MessagePack {
    const CONTENT_TYPE: &'static str = "application/msgpack";
    const METHOD: leptos::server_fn::request::Method = leptos::server_fn::request::Method::POST;
}

#[cfg(any(feature = "ssr", feature = "hydrate"))]
impl<Input, Output> IntoReq<MessagePack, Input, Output> for MessagePack
where
    Input: Serialize + Send,
    Output: Send,
{
    fn into_req(args: Input, path: &str) -> Result<ClientReq, ServerFnError> {
        let data = rmp_serde::to_vec(&args)
            .map_err(|e| ServerFnError::Serialization(e.to_string()))?;

        // ClientReq is typically http::Request or similar.
        // If ClientReq::new(method, path) doesn't exist, check if ClientReq is alias for Request.
        // In leptos 0.8+, ClientReq::try_new(method, uri, body) is available via trait extension usually or direct impl.
        // Actually, ClientReq IS http::Request in server/ssr, and gloo_net::Request in hydrate (often).
        
        // Let's assume ClientReq::post(path).body(...) or similar builders.
        // Or if it's http::Request:
        // http::Request::builder().method("POST").uri(path).header(...).body(data)
        
        // But ClientReq type differs between Features.
        // Let's try to use `ClientReq` assuming it has `try_new` as seen in other codecs (Json).
        // If that fails, I will use conditional compilation for specific types.
        
        // The error "expected a type, found a trait" suggests `ClientReq` handles differently.
        // Let's look at `Json` codec usage pattern if possible - I can't read source.
        
        // Let's try constructing via builder if available.
        // Or better, let's look at what `ClientReq` is.
        // In `leptos_server_fn::request`, `ClientReq` is public type alias.
        
        // If I use `ClientReq::try_new`, I need it to be available.
        // Let's try `ClientReq::new` again but verify imports.
        // Maybe I need to import `http::Method`?
        
        // Let's try using `http::Request` explicitly if possible, or just construct it.
        // If `ClientReq` is `http::Request`, `ClientReq::builder()` works.
        
        let req = ClientReq::builder()
            .method("POST")
            .uri(path)
            .header("Content-Type", "application/msgpack")
            .header("Accept", "application/msgpack")
            .body(bytes::Bytes::from(data))
            .map_err(|e| ServerFnError::Request(e.to_string()))?;
            
        Ok(req)
    }
}

#[cfg(any(feature = "ssr", feature = "hydrate"))]
impl<Input, Output> FromRes<MessagePack, Input, Output> for MessagePack
where
    Input: Send,
    Output: DeserializeOwned + Send,
{
    fn from_res(res: ClientRes) -> impl Future<Output = Result<Output, ServerFnError>> + Send {
        async move {
            let data = res.try_into_bytes().await?;
            rmp_serde::from_slice(&data)
                .map_err(|e| ServerFnError::Deserialization(e.to_string()))
        }
    }
}

#[cfg(feature = "ssr")]
impl<Input, Output> FromReq<MessagePack, Input, Output> for MessagePack
where
    Input: DeserializeOwned + Send,
    Output: Send,
{
    fn from_req(req: Req) -> impl Future<Output = Result<Input, ServerFnError>> + Send {
        async move {
            let body_bytes = req.try_into_bytes().await?;
            rmp_serde::from_slice(&body_bytes)
                .map_err(|e| ServerFnError::Args(e.to_string()))
        }
    }
}

#[cfg(feature = "ssr")]
impl<Input, Output> IntoRes<MessagePack, Input, Output> for MessagePack
where
    Input: Send,
    Output: Serialize + Send,
{
    fn into_res(res: Output) -> impl Future<Output = Result<Res, ServerFnError>> + Send {
        async move {
            let data = rmp_serde::to_vec(&res)
            .map_err(|e| ServerFnError::Serialization(e.to_string()))?;
            
            let mut res = Res::new(200);
            res.try_set_header("Content-Type", "application/msgpack")?;
            res.try_set_body(bytes::Bytes::from(data))?;
            Ok(res)
        }
    }
}
