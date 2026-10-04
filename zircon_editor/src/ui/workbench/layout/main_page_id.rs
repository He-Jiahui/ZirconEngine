use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// 页面/浮动工作区命令的关联键；须连同目标类别解释，构造不验证存在性。
pub struct MainPageId(pub(crate) String);

impl MainPageId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn workbench() -> Self {
        Self::new("workbench")
    }
}
