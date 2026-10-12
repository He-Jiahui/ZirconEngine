//! 静态清单的依赖和能力门槛行的统一遍历回归；经共享读取和本领域断言检查当前包声明，不执行插件行为。
#[path = "dependency_rows/feature.rs"]
mod feature;
#[path = "dependency_rows/package.rs"]
mod package;
#[path = "../dependency_rows/required_capabilities.rs"]
mod required_capabilities;

pub(super) use feature::visit_feature_dependency_rows;
pub(super) use package::{visit_package_dependency_ids, visit_package_dependency_rows};
pub(super) use required_capabilities::{
    visit_asset_importer_required_capabilities, visit_option_required_capabilities,
};
