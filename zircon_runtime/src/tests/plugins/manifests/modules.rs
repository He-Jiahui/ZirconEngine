//! 静态清单的模块身份、所属包、目标和系统锚点回归；经共享读取和本领域断言检查当前包声明，不执行插件行为。
#[path = "modules/capabilities.rs"]
mod capabilities;
#[path = "modules/crates.rs"]
mod crates;
#[path = "modules/identity.rs"]
mod identity;
#[path = "modules/names.rs"]
mod names;
#[path = "modules/system_anchors.rs"]
mod system_anchors;
#[path = "modules/targets.rs"]
mod targets;
