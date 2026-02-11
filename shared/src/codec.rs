use leptos::prelude::*;
use leptos::server_fn::codec::{Encoding, FromReq, FromRes, IntoReq, IntoRes};
use leptos::server_fn::request::{ClientReq, Req};
use leptos::server_fn::response::{ClientRes, Res, TryRes};
use http::Method;
use bytes::Bytes;
use serde::{de::DeserializeOwned, Serialize};
use std::future::Future;

pub struct MessagePack;

impl leptos::server_fn::ContentType for MessagePack {
    const CONTENT_TYPE: &'static str = "application/msgpack";
}

impl Encoding for MessagePack {
    const METHOD: Method = Method::POST;
}

#[cfg(any(feature = "ssr", feature = "hydrate"))]
impl<T, Request, Error> IntoReq<MessagePack, Request, Error> for T
where
    Request: ClientReq<Error>,
    T: Serialize + Send,
    Error: Send,
{
    fn into_req(self, path: &str, accepts: &str) -> Result<Request, Error> {
        let data = rmp_serde::to_vec(&self)
            .map_err(|e| ServerFnError::new(e.to_string()).into())?;
            
        // Use try_new_post_bytes which should be available on ClientReq trait
        Request::try_new_post_bytes(
            path,
            MessagePack::CONTENT_TYPE,
            accepts,
            Bytes::from(data)
        )
    }
}

#[cfg(any(feature = "ssr", feature = "hydrate"))]
impl<T, Response, Error> FromRes<MessagePack, Response, Error> for T
where
    Response: ClientRes<Error> + Send,
    T: DeserializeOwned + Send,
    Error: Send,
{
    fn from_res(res: Response) -> impl Future<Output = Result<Self, Error>> + Send {
        async move {
            let data = res.try_into_bytes().await?;
            rmp_serde::from_slice(&data)
                .map_err(|e| ServerFnError::new(e.to_string()).into())
        }
    }
}

#[cfg(feature = "ssr")]
impl<T, Request, Error> FromReq<MessagePack, Request, Error> for T
where
    Request: Req<Error> + Send,
    T: DeserializeOwned,
    Error: Send,
{
    fn from_req(req: Request) -> impl Future<Output = Result<Self, Error>> + Send {
        async move {
            let data = req.try_into_bytes().await?;
            rmp_serde::from_slice(&data)
                .map_err(|e| ServerFnError::new(e.to_string()).into())
        }
    }
}

#[cfg(feature = "ssr")]
impl<T, Response, Error> IntoRes<MessagePack, Response, Error> for T
where
    Response: TryRes<Error> + Send,
    T: Serialize + Send,
    Error: Send,
{
    fn into_res(self) -> impl Future<Output = Result<Response, Error>> + Send {
        async move {
            let data = rmp_serde::to_vec(&self)
                .map_err(|e| ServerFnError::new(e.to_string()).into())?;
            
            Response::try_from_bytes(MessagePack::CONTENT_TYPE, Bytes::from(data))
        }
    }
}
