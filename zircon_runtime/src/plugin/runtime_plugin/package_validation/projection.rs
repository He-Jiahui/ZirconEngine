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

pub(in crate::plugin::runtime_plugin) struct RuntimePluginPackageValidationProjection<'a> {
    duplicates: HashSet<DuplicateOccurrence>,
    owned_capabilities: HashSet<&'a str>,
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
mod tests;
