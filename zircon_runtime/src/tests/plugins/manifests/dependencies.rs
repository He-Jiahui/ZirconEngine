//! 静态清单的依赖包身份、能力解析与主依赖约束回归；经共享读取和本领域断言检查当前包声明，不执行插件行为。
#[path = "dependencies/capabilities.rs"]
mod capabilities;
#[path = "dependencies/capability_assertions.rs"]
mod capability_assertions;
#[path = "dependencies/optional_features.rs"]
mod optional_features;
#[path = "dependencies/package.rs"]
mod package;
#[path = "dependencies/provider_ids.rs"]
mod provider_ids;
#[path = "dependencies/provider_tokens.rs"]
mod provider_tokens;
#[path = "dependencies/required_gates.rs"]
mod required_gates;
#[path = "dependencies/resolution.rs"]
mod resolution;
