//! 静态清单的公开文字、分类、目标及数组唯一性回归；经共享读取和本领域断言检查当前包声明，不执行插件行为。
#[path = "package_metadata/arrays.rs"]
mod arrays;
#[path = "package_metadata/classification.rs"]
mod classification;
#[path = "package_metadata/display.rs"]
mod display;
#[path = "package_metadata/public_fields.rs"]
mod public_fields;
#[path = "package_metadata/targets.rs"]
mod targets;
