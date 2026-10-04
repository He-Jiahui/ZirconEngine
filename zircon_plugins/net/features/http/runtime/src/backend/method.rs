//! 在引擎 HTTP method 和 Hyper/Reqwest 类型间保持同一方法语义；server 不支持的方法不命中路由。
//! 此转换边界不会替业务决定重试幂等性或路由权限。

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

pub(super) fn method_to_hyper(method: NetHttpMethod) -> hyper::Method {
    match method {
        NetHttpMethod::Get => hyper::Method::GET,
        NetHttpMethod::Post => hyper::Method::POST,
        NetHttpMethod::Put => hyper::Method::PUT,
        NetHttpMethod::Patch => hyper::Method::PATCH,
        NetHttpMethod::Delete => hyper::Method::DELETE,
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
