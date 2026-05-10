use serde::{Deserialize, Serialize};

use super::{ReflectObjectAddress, ReflectedValue};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ReflectFieldValue {
    pub field_name: String,
    pub value: ReflectedValue,
}

impl ReflectFieldValue {
    pub fn new(field_name: impl Into<String>, value: ReflectedValue) -> Self {
        Self {
            field_name: field_name.into(),
            value,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReflectFieldsRequest {
    pub address: ReflectObjectAddress,
}

impl ReflectFieldsRequest {
    pub fn new(address: ReflectObjectAddress) -> Self {
        Self { address }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ReflectFieldsResponse {
    pub address: ReflectObjectAddress,
    pub fields: Vec<ReflectFieldValue>,
}

impl ReflectFieldsResponse {
    pub fn new(address: ReflectObjectAddress, fields: Vec<ReflectFieldValue>) -> Self {
        Self { address, fields }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReflectReadRequest {
    pub address: ReflectObjectAddress,
    pub field_name: String,
}

impl ReflectReadRequest {
    pub fn new(address: ReflectObjectAddress, field_name: impl Into<String>) -> Self {
        Self {
            address,
            field_name: field_name.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ReflectReadResponse {
    pub address: ReflectObjectAddress,
    pub field: ReflectFieldValue,
}

impl ReflectReadResponse {
    pub fn new(address: ReflectObjectAddress, field: ReflectFieldValue) -> Self {
        Self { address, field }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ReflectWriteRequest {
    pub address: ReflectObjectAddress,
    pub field_name: String,
    pub value: ReflectedValue,
}

impl ReflectWriteRequest {
    pub fn new(
        address: ReflectObjectAddress,
        field_name: impl Into<String>,
        value: ReflectedValue,
    ) -> Self {
        Self {
            address,
            field_name: field_name.into(),
            value,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ReflectWriteResponse {
    pub address: ReflectObjectAddress,
    pub field: ReflectFieldValue,
    pub changed: bool,
}

impl ReflectWriteResponse {
    pub fn new(address: ReflectObjectAddress, field: ReflectFieldValue, changed: bool) -> Self {
        Self {
            address,
            field,
            changed,
        }
    }
}
