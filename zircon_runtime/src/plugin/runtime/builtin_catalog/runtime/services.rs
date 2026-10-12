use crate::{plugin::CapabilityStatus, plugin::PluginMaturity};

use super::super::super::capability_status::capability_status;
use super::super::super::BuiltinCatalogDescriptorBuilder;

pub(super) fn is_runtime_service_descriptor(package_id: &str) -> bool {
    matches!(package_id, "physics" | "sound" | "texture" | "net")
}

pub(super) fn classify_runtime_service_descriptor(
    package_id: &str,
    descriptor: BuiltinCatalogDescriptorBuilder,
) -> BuiltinCatalogDescriptorBuilder {
    match package_id {
        "physics" => descriptor
            .with_maturity(PluginMaturity::Experimental)
            .with_capability_status(capability_status(
                "runtime.plugin.physics",
                CapabilityStatus::Partial,
            ))
            .with_capability_status(capability_status(
                "runtime.capability.physics.raycast",
                CapabilityStatus::Partial,
            ))
            .with_capability_status(capability_status(
                "runtime.capability.physics.overlap",
                CapabilityStatus::Partial,
            ))
            .with_capability_status(capability_status(
                "runtime.capability.physics.shape_cast",
                CapabilityStatus::Partial,
            ))
            .with_capability_status(capability_status(
                "runtime.capability.physics.trigger_events",
                CapabilityStatus::Partial,
            ))
            .with_capability_status(capability_status(
                "runtime.capability.physics.constraints",
                CapabilityStatus::Partial,
            ))
            .with_capability_status(capability_status(
                "runtime.capability.physics.skeletal_joints",
                CapabilityStatus::Partial,
            )),
        "sound" => descriptor
            .with_maturity(PluginMaturity::Beta)
            .with_capability_status(
                capability_status("runtime.plugin.sound", CapabilityStatus::Partial)
                    .with_bevy_reference("dev/bevy/crates/bevy_audio/src/lib.rs"),
            ),
        "texture" => descriptor
            .with_maturity(PluginMaturity::Stable)
            .with_capability_status(capability_status(
                "runtime.plugin.texture",
                CapabilityStatus::Complete,
            )),
        "net" => descriptor
            .with_maturity(PluginMaturity::Beta)
            .with_capability_status(
                capability_status("runtime.plugin.net", CapabilityStatus::Partial)
                    .with_bevy_reference("dev/bevy/crates/bevy_remote/src/lib.rs"),
            ),
        _ => descriptor,
    }
}
