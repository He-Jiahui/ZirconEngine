//! 静态清单的包内可选特性、外部特性和模块的统一遍历回归；经共享读取和本领域断言检查当前包声明，不执行插件行为。
mod feature_bundle_rows;
mod feature_rows;
mod module_rows;

pub(super) use feature_rows::{for_each_feature_extension, for_each_optional_feature};
pub(super) use module_rows::{for_each_module_row, visit_module_rows};
