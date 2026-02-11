use leptos::server_fn::codec::{Encoding, FromReq, FromRes, IntoReq, IntoRes};
use leptos::server_fn::request::{ClientReq, Req};
use leptos::server_fn::response::{ClientRes, Res};
use leptos::server_fn::error::ServerFnError;
use serde::{de::DeserializeOwned, Serialize};
use std::future::Future;

pub struct MessagePack;

impl leptos::server_fn::codec::ContentType for MessagePack {
    const CONTENT_TYPE: &'static str = "application/msgpack";
}

impl Encoding for MessagePack {
    const METHOD: leptos::server_fn::request::Method = leptos::server_fn::request::Method::POST;
}

#[cfg(any(feature = "ssr", feature = "hydrate"))]
impl<Input, Output> IntoReq<MessagePack, Input, Output> for MessagePack
where
    Input: Serialize + Send,
    Output: Send,
{
    fn into_req(self, args: Input, path: &str) -> Result<ClientReq, ServerFnError> {
        let data = rmp_serde::to_vec(&args)
            .map_err(|e| ServerFnError::Serialization(e.to_string()))?;

        ClientReq::try_new(
            leptos::server_fn::request::Method::POST,
            path,
            leptos::server_fn::request::Payload::Binary(bytes::Bytes::from(data))
        )
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
            
            Res::try_from_bytes(
                bytes::Bytes::from(data), 
                "application/msgpack"
            )
        }
    }
}
