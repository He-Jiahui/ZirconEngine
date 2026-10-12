//! 静态清单的包种类与特性行结构的一致性回归；经共享读取和本领域断言检查当前包声明，不执行插件行为。
#[path = "package_kind/feature_rows.rs"]
mod feature_rows;
#[path = "package_kind/package_kind_fields.rs"]
mod package_kind_fields;
#[path = "package_kind/values.rs"]
mod values;
