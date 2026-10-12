use std::borrow::Borrow;
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// 编译 pass 与运行时注册表共用的 executor 身份。
/// 图编译只存此身份，提交前必须能在当前注册表中找到对应实现。
pub struct RenderPassExecutorId(String);

impl RenderPassExecutorId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for RenderPassExecutorId {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for RenderPassExecutorId {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl Borrow<str> for RenderPassExecutorId {
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for RenderPassExecutorId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}
