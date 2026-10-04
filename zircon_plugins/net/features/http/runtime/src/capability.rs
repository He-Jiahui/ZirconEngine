//! 公开 HTTP feature 能力名，供根 net 清单与宿主依赖解析使用；后端需在 feature 服务启动后安装。

pub use crate::feature::NET_HTTP_FEATURE_CAPABILITY;

pub const RUNTIME_CAPABILITIES: &[&str] = &[NET_HTTP_FEATURE_CAPABILITY];
