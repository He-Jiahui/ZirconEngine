use serde::{Deserialize, Serialize};

use super::{ReflectFieldId, ReflectObjectAddress, ReflectedValue};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
/// 稳定字段标识、当前显示名与字段值组成的传输记录。
pub struct ReflectFieldValue {
    /// 字段的稳定 UUID 标识；运行时用它解析当前 schema slot。
    pub field_id: ReflectFieldId,
    /// Current schema name retained for authoring and diagnostics; never used as identity.
    /// 本次 schema 中的字段名，仅用于编辑器展示和诊断，不参与寻址。
    pub field_name: String,
    /// 当前字段值；预算与字段类型的运行时校验发生在读写路径中。
    pub value: ReflectedValue,
}

impl ReflectFieldValue {
    /// 组装一条字段值传输记录；不执行字段存在性或值类型校验。
    pub fn new(
        field_id: ReflectFieldId,
        field_name: impl Into<String>,
        value: ReflectedValue,
    ) -> Self {
        Self {
            field_id,
            field_name: field_name.into(),
            value,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// 请求读取某个对象当前全部反射字段的消息。
pub struct ReflectFieldsRequest {
    /// 要读取的组件或资源地址。
    pub address: ReflectObjectAddress,
}

impl ReflectFieldsRequest {
    /// 用对象地址构造字段枚举请求；地址解析由运行时处理。
    pub fn new(address: ReflectObjectAddress) -> Self {
        Self { address }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
/// 对象当前全部反射字段及其稳定标识和当前 schema 名称。
pub struct ReflectFieldsResponse {
    /// 产生此快照的对象地址。
    pub address: ReflectObjectAddress,
    /// 按运行时 schema slot 顺序枚举的字段记录。
    pub fields: Vec<ReflectFieldValue>,
}

impl ReflectFieldsResponse {
    /// 组装字段枚举结果；不会重新读取或校验字段值。
    pub fn new(address: ReflectObjectAddress, fields: Vec<ReflectFieldValue>) -> Self {
        Self { address, fields }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
/// 通过对象地址和稳定字段标识读取单个字段的请求。
pub struct ReflectReadRequest {
    /// 要读取的组件或资源地址。
    pub address: ReflectObjectAddress,
    /// 要解析到当前 schema slot 的稳定字段标识。
    pub field_id: ReflectFieldId,
}

impl ReflectReadRequest {
    /// 组装单字段读取请求；字段存在性由运行时在访问对象时判定。
    pub fn new(address: ReflectObjectAddress, field_id: ReflectFieldId) -> Self {
        Self { address, field_id }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
/// 单字段读取结果，携带当前 schema 名称和值。
pub struct ReflectReadResponse {
    /// 实际完成读取的对象地址。
    pub address: ReflectObjectAddress,
    /// 从当前对象适配器读取并通过运行时值预算检查的字段。
    pub field: ReflectFieldValue,
}

impl ReflectReadResponse {
    /// 组装单字段读取结果；不执行读操作本身。
    pub fn new(address: ReflectObjectAddress, field: ReflectFieldValue) -> Self {
        Self { address, field }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
/// 通过对象地址和稳定字段标识提交单字段写入的请求。
pub struct ReflectWriteRequest {
    /// 要修改的组件或资源地址。
    pub address: ReflectObjectAddress,
    /// 要解析到当前 schema slot 的稳定字段标识。
    pub field_id: ReflectFieldId,
    /// 待提交的新值；运行时先校验预算，再交给对象适配器验证并写入。
    pub value: ReflectedValue,
}

impl ReflectWriteRequest {
    /// 组装单字段写入请求；字段可写性和新值类型由运行时适配器路径判定。
    pub fn new(
        address: ReflectObjectAddress,
        field_id: ReflectFieldId,
        value: ReflectedValue,
    ) -> Self {
        Self {
            address,
            field_id,
            value,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
/// 单字段写入的接受结果。
pub struct ReflectWriteResponse {
    /// 实际完成写入的对象地址。
    pub address: ReflectObjectAddress,
    /// The accepted request field, not an implicit post-publication readback.
    pub field: ReflectFieldValue,
    /// 是否发生了实际变更；值由对应对象适配器返回，具体发布策略由适配器实现决定。
    pub changed: bool,
}

impl ReflectWriteResponse {
    /// 组装写入应答；不会执行提交，也不会隐式读取写入后的值。
    pub fn new(address: ReflectObjectAddress, field: ReflectFieldValue, changed: bool) -> Self {
        Self {
            address,
            field,
            changed,
        }
    }
}
