use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// 主页面的布局身份；工作台内置页面使用 workbench()，布局恢复与活动页切换按该身份查找。
pub struct MainPageId(pub(crate) String);

impl MainPageId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn workbench() -> Self {
        Self::new("workbench")
    }
}
