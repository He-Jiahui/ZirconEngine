//! 静态清单的坐标字段与最终包身份的解析回归；经共享读取和本领域断言检查当前包声明，不执行插件行为。
#[path = "package_coordinates/coordinates.rs"]
mod coordinates;
#[path = "../package_coordinates/coordinate_resolution.rs"]
mod package_coordinate_resolution;
#[path = "package_coordinates/resolved_ids.rs"]
mod resolved_ids;
