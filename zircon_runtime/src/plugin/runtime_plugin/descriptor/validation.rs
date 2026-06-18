mod crate_name;
mod display;
mod target_modes;

use super::super::{
    package_validation::{
        validate_runtime_plugin_default_packaging, validate_runtime_plugin_package_id,
    },
    RuntimePlugin,
};
use crate::builtin::RuntimeTargetMode;

pub(in crate::plugin::runtime_plugin) fn validate_runtime_plugin_descriptor(
    plugin: &dyn RuntimePlugin,
    diagnostics: &mut Vec<String>,
) {
    let descriptor = plugin.descriptor();
    validate_runtime_plugin_package_id(
        "runtime plugin descriptor",
        "package_id",
        &descriptor.package_id,
        diagnostics,
    );
    display::validate_runtime_plugin_display_field(
        "display_name",
        &descriptor.display_name,
        diagnostics,
    );
    crate_name::validate_runtime_plugin_descriptor_crate_name(&descriptor.crate_name, diagnostics);
    target_modes::validate_runtime_plugin_descriptor_target_modes(
        &descriptor.target_modes,
        diagnostics,
    );
    validate_runtime_plugin_default_packaging(
        "descriptor",
        &descriptor.default_packaging,
        diagnostics,
    );
}

pub(super) type DescriptorTargetMode = RuntimeTargetMode;
