//! 静态清单的导入器身份、资源种类、选择器与能力门槛回归；经共享读取和本领域断言检查当前包声明，不执行插件行为。
#[path = "asset_importers/ids.rs"]
mod ids;
#[path = "asset_importers/kinds.rs"]
mod kinds;
#[path = "asset_importers/rows.rs"]
mod rows;
#[path = "asset_importers/selectors.rs"]
mod selectors;
#[path = "asset_importers/traversal.rs"]
mod traversal;
