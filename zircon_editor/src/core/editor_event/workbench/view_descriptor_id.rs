use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// 可注册视图类型的身份；菜单 OpenView 先按此查描述符，再由注册表创建或复用实例。
pub struct ViewDescriptorId(pub(crate) String);

impl ViewDescriptorId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}
