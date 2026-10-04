use serde::{Deserialize, Serialize};

/// 场景携带的字段元数据；编辑权限由此声明，写入仍须经过运行时反射校验。
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComponentPropertyDescriptor {
    pub name: String,
    pub value_type: String,
    pub editable: bool,
}
