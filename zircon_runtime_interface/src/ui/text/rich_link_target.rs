use core::fmt;
use std::sync::Arc;

use serde::{de::Error as _, Deserialize, Deserializer, Serialize, Serializer};

use crate::resource::{ResourceLocator, ResourceLocatorError, ResourceScheme};

/// Parser-approved engine-local destination carried unchanged to the UI host.
///
/// 目标只经过词法规范化与资源 scheme 白名单检查，不证明资源存在、物理路径包含关系或授权。
/// 克隆共享 `Arc` 中的定位值，不保活实际资源；Serde 使用显示字符串，读取时重新解析。
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UiRichLinkTarget {
    locator: Arc<ResourceLocator>,
}

impl UiRichLinkTarget {
    /// 将文本解析为受支持的引擎资源目标，不执行资源访问。
    ///
    /// # 参数
    /// `value` 会先去除首尾空白；省略 scheme 时按 `res` 处理，支持 `res/lib/package/builtin`。
    /// 输入借用不被保留，路径与可选标签由返回值持有。
    ///
    /// # 返回
    /// 规范化目标，或空文本、非法定位、非允许 scheme 的错误。
    pub fn parse(value: &str) -> Result<Self, UiRichLinkTargetError> {
        let value = value.trim();
        if value.is_empty() {
            return Err(UiRichLinkTargetError::Empty);
        }
        let locator = if value.contains("://") {
            ResourceLocator::parse(value)
        } else {
            let (path, label) = value
                .split_once('#')
                .map_or((value, None), |(path, label)| {
                    (path, Some(label.to_owned()))
                });
            ResourceLocator::new(ResourceScheme::Res, path, label)
        }
        .map_err(UiRichLinkTargetError::InvalidLocator)?;
        Self::from_locator(locator)
    }

    /// 接收已有定位值，仅检查 scheme 白名单后保存其分量。
    /// 不承诺任意分字段结构都能经显示字符串无损往返；本函数不重新解析路径文本。
    pub fn from_locator(locator: ResourceLocator) -> Result<Self, UiRichLinkTargetError> {
        if !matches!(
            locator.scheme(),
            ResourceScheme::Res
                | ResourceScheme::Library
                | ResourceScheme::Package
                | ResourceScheme::Builtin
        ) {
            return Err(UiRichLinkTargetError::UnsupportedScheme(locator.scheme()));
        }
        Ok(Self {
            locator: Arc::new(locator),
        })
    }

    /// 借用内部定位值；引用有效期受 `self` 约束，不代表实际资源的存续或访问权。
    pub fn locator(&self) -> &ResourceLocator {
        self.locator.as_ref()
    }

    pub fn matches_display(&self, value: &str) -> bool {
        self.locator.matches_display(value)
    }

    /// Heap bytes retained by the canonical locator components.
    /// 返回路径与标签的 UTF-8 字节长度之和；不计容量、Arc 控制块或分配器开销。
    pub fn retained_heap_bytes(&self) -> usize {
        self.locator
            .path()
            .len()
            .saturating_add(self.locator.label().map_or(0, str::len))
    }
}

impl fmt::Display for UiRichLinkTarget {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.locator.fmt(formatter)
    }
}

impl Serialize for UiRichLinkTarget {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for UiRichLinkTarget {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::parse(&value).map_err(D::Error::custom)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UiRichLinkTargetError {
    Empty,
    InvalidLocator(ResourceLocatorError),
    UnsupportedScheme(ResourceScheme),
}

impl fmt::Display for UiRichLinkTargetError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => formatter.write_str("rich link target cannot be empty"),
            Self::InvalidLocator(error) => error.fmt(formatter),
            Self::UnsupportedScheme(scheme) => {
                write!(
                    formatter,
                    "rich link target scheme is not allowed: {scheme:?}"
                )
            }
        }
    }
}

impl std::error::Error for UiRichLinkTargetError {}

#[cfg(test)]
#[path = "tests/rich_link_target.rs"]
mod tests;
