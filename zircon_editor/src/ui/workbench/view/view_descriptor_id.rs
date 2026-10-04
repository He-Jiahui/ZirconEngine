//! 视图种类的稳定身份；注册、能力判断与工作区实例引用此键，构造本身不证明视图存在。
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// 视图种类键；一个描述符可有多个实例，不能以此值定位具体pane。
pub struct ViewDescriptorId(pub(crate) String);

impl ViewDescriptorId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}
