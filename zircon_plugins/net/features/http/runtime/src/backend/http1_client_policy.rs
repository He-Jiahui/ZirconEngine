use http_body_util::Full;
use hyper::body::Bytes;
use hyper_util::client::legacy::{connect::HttpConnector, Client};
use hyper_util::rt::TokioExecutor;

type PlainHttpRequestBody = Full<Bytes>;
type PlainHttpClient = Client<HttpConnector, PlainHttpRequestBody>;

pub(super) fn plain_http_client() -> PlainHttpClient {
    Client::builder(TokioExecutor::new()).build_http()
}
