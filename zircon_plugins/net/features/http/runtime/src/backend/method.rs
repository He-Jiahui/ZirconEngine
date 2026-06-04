use zircon_runtime::core::framework::net::NetHttpMethod;

pub(super) fn method_to_reqwest(method: NetHttpMethod) -> reqwest::Method {
    match method {
        NetHttpMethod::Get => reqwest::Method::GET,
        NetHttpMethod::Post => reqwest::Method::POST,
        NetHttpMethod::Put => reqwest::Method::PUT,
        NetHttpMethod::Patch => reqwest::Method::PATCH,
        NetHttpMethod::Delete => reqwest::Method::DELETE,
    }
}

pub(super) fn http_method_from_hyper(method: &hyper::Method) -> Option<NetHttpMethod> {
    if method == hyper::Method::GET {
        Some(NetHttpMethod::Get)
    } else if method == hyper::Method::POST {
        Some(NetHttpMethod::Post)
    } else if method == hyper::Method::PUT {
        Some(NetHttpMethod::Put)
    } else if method == hyper::Method::PATCH {
        Some(NetHttpMethod::Patch)
    } else if method == hyper::Method::DELETE {
        Some(NetHttpMethod::Delete)
    } else {
        None
    }
}
