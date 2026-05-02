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
pub struct NetHttpRequestDescriptor {
    pub request: NetRequestId,
    pub method: NetHttpMethod,
    pub url: String,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
    pub timeout_ms: u64,
    pub security: NetSecurityPolicy,
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
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
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
