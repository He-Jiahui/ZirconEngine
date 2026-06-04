use crate::{plugin::CapabilityStatus, plugin::PluginMaturity};

use super::super::super::super::RuntimePluginDescriptor;
use super::super::super::capability_status::capability_status;

pub(super) fn is_runtime_system_descriptor(package_id: &str) -> bool {
    matches!(package_id, "ai" | "navigation" | "particles" | "animation")
}

pub(super) fn classify_runtime_system_descriptor(
    package_id: &str,
    descriptor: RuntimePluginDescriptor,
) -> RuntimePluginDescriptor {
    match package_id {
        "ai" => descriptor
            .with_maturity(PluginMaturity::Experimental)
            .with_capability_status(
                capability_status("runtime.plugin.ai", CapabilityStatus::Partial).with_note(
                    "Foundational AI runtime package; behavior-tree execution is intentionally staged behind manager contracts.",
                ),
            )
            .with_capability_status(capability_status(
                "runtime.feature.ai.behavior_tree",
                CapabilityStatus::Partial,
            ))
            .with_capability_status(capability_status(
                "runtime.feature.ai.blackboard",
                CapabilityStatus::Partial,
            ))
            .with_capability_status(capability_status(
                "runtime.feature.ai.perception",
                CapabilityStatus::Partial,
            )),
        "navigation" => descriptor
            .with_maturity(PluginMaturity::Beta)
            .with_capability_status(
                capability_status("runtime.plugin.navigation", CapabilityStatus::Partial)
                    .with_note(
                    "Gameplay navmesh/pathfinding is optional; UI navigation parity is separate.",
                ),
            ),
        "particles" => descriptor
            .with_maturity(PluginMaturity::Experimental)
            .with_capability_status(
                capability_status("runtime.plugin.particles", CapabilityStatus::Partial).with_note(
                    "Advanced optional VFX capability; not a Bevy default parity blocker.",
                ),
            ),
        "animation" => descriptor
            .with_maturity(PluginMaturity::Beta)
            .with_capability_status(
                capability_status("runtime.plugin.animation", CapabilityStatus::Partial)
                    .with_bevy_reference("dev/bevy/crates/bevy_animation/src/lib.rs"),
            )
            .with_capability_status(capability_status(
                "runtime.feature.animation.timeline_event_track",
                CapabilityStatus::Partial,
            )),
        _ => descriptor,
    }
}
