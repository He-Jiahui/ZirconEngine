//! 注册报告的一次性清单索引，为包校验、内嵌特性校验及注册结果核对共享重复和成员关系。
//! 索引借用原清单；所有行号都属于构建时的原始数组顺序，不能搭配另一份清单使用。

mod build;
mod duplicate_identity;
mod duplicate_occurrence;
mod lookup;
mod metrics;

use std::cell::Cell;
use std::collections::HashSet;

use duplicate_occurrence::DuplicateOccurrence;
pub(in crate::plugin::runtime_plugin) use duplicate_occurrence::EmbeddedFeatureKind;
pub(in crate::plugin::runtime_plugin) use metrics::RuntimePluginPackageValidationMetrics;
#[cfg(test)]
pub(in crate::plugin::runtime_plugin) use metrics::{
    begin_package_projection_build_observation, observed_package_projection_builds,
};

/// 与一份不可变包清单同寿命的校验视图；有序行用于稳定诊断，集合用于重复和成员查询。
/// 此视图描述声明关系，实际模块是否导出接口或注册系统仍由注册报告核对。
pub(in crate::plugin::runtime_plugin) struct RuntimePluginPackageValidationProjection<'a> {
    duplicates: HashSet<DuplicateOccurrence>,
    // 状态归属限于包与可选特性的能力；模块能力和扩展特性能力不自动取得这一归属。
    owned_capabilities: HashSet<&'a str>,
    // 有序视图保留清单顺序及重复行，成员视图供注册表快速核对，避免散列顺序进入诊断。
    runtime_module_names: Vec<&'a str>,
    runtime_module_name_membership: HashSet<&'a str>,
    provided_interface_ids: Vec<&'a str>,
    provided_interface_membership: HashSet<&'a str>,
    dependency_interface_ids: Vec<&'a str>,
    dependency_interface_membership: HashSet<&'a str>,
    runtime_system_anchors: Vec<(&'a str, &'a str)>,
    identity_rows_indexed: usize,
    membership_probes: Cell<usize>,
}

#[cfg(test)]
#[path = "projection/tests/cases.rs"]
mod tests;
