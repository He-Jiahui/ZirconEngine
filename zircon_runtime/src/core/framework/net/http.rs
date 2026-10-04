use serde::{Deserialize, Serialize};

use super::{NetEndpoint, NetRequestId, NetSecurityPolicy};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum NetHttpMethod {
    Get,
    Post,
    Put,
    Patch,
    Delete,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// HTTP 客户端请求的跨插件描述符；构造后按调用场景显式设置超时、重试和安全策略，再交给 NetManager。
pub struct NetHttpRequestDescriptor {
    pub request: NetRequestId,
    pub method: NetHttpMethod,
    pub url: String,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
    pub timeout_ms: u64,
    pub security: NetSecurityPolicy,
    pub max_retry_attempts: u8,
}

impl NetHttpRequestDescriptor {
    pub fn new(request: NetRequestId, method: NetHttpMethod, url: impl Into<String>) -> Self {
        Self {
            request,
            method,
            url: url.into(),
            headers: Vec::new(),
            body: Vec::new(),
            timeout_ms: 30_000,
            security: NetSecurityPolicy::default(),
            max_retry_attempts: 0,
        }
    }

    pub fn with_header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.push((name.into(), value.into()));
        self
    }

    pub fn with_body(mut self, body: impl Into<Vec<u8>>) -> Self {
        self.body = body.into();
        self
    }

    pub fn with_max_retry_attempts(mut self, attempts: u8) -> Self {
        self.max_retry_attempts = attempts;
        self
    }

    /// 为分块下载设置闭区间 Range；调用方须先验证起点不大于终点及资源边界。
    pub fn with_byte_range(mut self, start: u64, end_inclusive: u64) -> Self {
        self.headers
            .retain(|(name, _)| !name.eq_ignore_ascii_case("range"));
        self.headers.push((
            "range".to_string(),
            byte_range_header_value(start, end_inclusive),
        ));
        self
    }
}

fn byte_range_header_value(start: u64, end_inclusive: u64) -> String {
    let mut value = String::with_capacity("bytes=".len() + 20 + 1 + 20);
    value.push_str("bytes=");
    push_u64_decimal(&mut value, start);
    value.push('-');
    push_u64_decimal(&mut value, end_inclusive);
    value
}

fn push_u64_decimal(output: &mut String, mut value: u64) {
    let mut digits = [0_u8; 20];
    let mut start = digits.len();
    loop {
        start -= 1;
        digits[start] = b'0' + (value % 10) as u8;
        value /= 10;
        if value == 0 {
            break;
        }
    }
    for digit in &digits[start..] {
        output.push(char::from(*digit));
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// HTTP 完成结果；request 用于关联请求，body_bytes 应随 body 保持一致，for_request 会按当前 body 重算。
pub struct NetHttpResponseDescriptor {
    pub request: NetRequestId,
    pub status_code: u16,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
    pub body_bytes: usize,
}

impl NetHttpResponseDescriptor {
    pub fn new(request: NetRequestId, status_code: u16, body: impl Into<Vec<u8>>) -> Self {
        let body = body.into();
        Self {
            request,
            status_code,
            headers: Vec::new(),
            body_bytes: body.len(),
            body,
        }
    }

    pub fn with_header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.push((name.into(), value.into()));
        self
    }

    pub fn for_request(mut self, request: NetRequestId) -> Self {
        self.request = request;
        self.body_bytes = self.body.len();
        self
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NetHttpRouteDescriptor {
    pub path: String,
    pub methods: Vec<NetHttpMethod>,
    pub endpoint: Option<NetEndpoint>,
}

impl NetHttpRouteDescriptor {
    pub fn new(path: impl Into<String>, methods: impl IntoIterator<Item = NetHttpMethod>) -> Self {
        Self {
            path: path.into(),
            methods: methods.into_iter().collect(),
            endpoint: None,
        }
    }
}

#[cfg(test)]
#[path = "tests/http_optimization_batch_fa_tests.rs"]
mod optimization_batch_fa_tests;
