//! 具体pane实例的关联键，串联布局叶节点、焦点、注册表和项目工作区快照。
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// 实例身份需在同一工作区唯一；此类型只封装文本，调用方须防止碰撞，当前恢复入口尚未完成该校验。
pub struct ViewInstanceId(pub(crate) String);

impl ViewInstanceId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}
