//! 静态清单的顶层和嵌套字段的显式准入集合回归；经共享读取和本领域断言检查当前包声明，不执行插件行为。
#[path = "manifest_schema/assertions.rs"]
mod assertions;
#[path = "manifest_schema/field_sets.rs"]
mod field_sets;
#[path = "manifest_schema/nested.rs"]
mod nested;
#[path = "manifest_schema/top_level.rs"]
mod top_level;
