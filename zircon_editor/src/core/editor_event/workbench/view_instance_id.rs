use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// 打开的视图实例身份，可与同一描述符的其他实例并存；布局、脏标记和焦点均按实例定位。
pub struct ViewInstanceId(pub(crate) String);

impl ViewInstanceId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}
