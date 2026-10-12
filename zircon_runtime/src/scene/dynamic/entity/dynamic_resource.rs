use serde::{Deserialize, Serialize};
use zircon_runtime_interface::reflect::ReflectFieldValue;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
/// 场景级反射资源写入意图；应用时按目标世界的注册模式解析，不能凭此快照创建未注册资源类型。
pub struct DynamicResource {
    pub type_path: String,
    #[serde(default)]
    pub fields: Vec<ReflectFieldValue>,
}

impl DynamicResource {
    pub fn new(type_path: impl Into<String>, fields: Vec<ReflectFieldValue>) -> Self {
        Self {
            type_path: type_path.into(),
            fields,
        }
    }
}
