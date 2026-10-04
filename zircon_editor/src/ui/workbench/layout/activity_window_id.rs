use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// 活动窗口的稳定键；主页面、视图实例和原生宿主句柄使用各自身份。
pub struct ActivityWindowId(pub(crate) String);

impl ActivityWindowId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn workbench() -> Self {
        Self::new("window:workbench")
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}
