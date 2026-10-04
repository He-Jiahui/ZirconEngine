//! 集中声明明文 HTTP/1 client 的 body 与 executor，供 HTTP backend 的明文请求分支使用。
//! TLS 与证书 pin 走独立客户端配置，调用者必须先校验请求安全策略。

use http_body_util::Full;
use hyper::body::Bytes;
use hyper_util::client::legacy::{connect::HttpConnector, Client};
use hyper_util::rt::TokioExecutor;

type PlainHttpRequestBody = Full<Bytes>;
type PlainHttpClient = Client<HttpConnector, PlainHttpRequestBody>;

pub(super) fn plain_http_client() -> PlainHttpClient {
    Client::builder(TokioExecutor::new()).build_http()
}
