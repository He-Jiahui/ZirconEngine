use crate::{plugin::CapabilityStatus, plugin::PluginMaturity};

use super::super::super::RuntimePluginDescriptor;
use super::super::capability_status::capability_status;

pub(in crate::plugin::runtime_plugin::builtin_catalog) fn is_language_descriptor(
    package_id: &str,
) -> bool {
    package_id == "zr_vm_language"
}

pub(in crate::plugin::runtime_plugin::builtin_catalog) fn classify_language_descriptor(
    package_id: &str,
    descriptor: RuntimePluginDescriptor,
) -> RuntimePluginDescriptor {
    if package_id == "zr_vm_language" {
        return descriptor
            .with_maturity(PluginMaturity::Experimental)
            .with_capability_status(capability_status(
                "runtime.plugin.zr_vm_language",
                CapabilityStatus::Partial,
            ))
            .with_capability_status(capability_status(
                "runtime.script.backend.zr_vm_project",
                CapabilityStatus::Partial,
            ));
    }
    descriptor
}
