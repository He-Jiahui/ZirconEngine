//! 公开内容下载 feature 的能力名，供宿主 feature 选择和依赖解析使用。
//! 能力声明不代表有可执行 HTTP 后端或已完成下载校验。

pub use crate::feature::NET_CONTENT_DOWNLOAD_FEATURE_CAPABILITY;

pub const RUNTIME_CAPABILITIES: &[&str] = &[NET_CONTENT_DOWNLOAD_FEATURE_CAPABILITY];
