use crate::{plugin::CapabilityStatus, plugin::PluginMaturity};

use super::super::RuntimePluginDescriptor;
use super::capability_status::capability_status;

const ADVANCED_RENDER_PACKAGE_IDS: &[&str] = &["virtual_geometry", "hybrid_gi", "solari"];

pub(super) fn is_advanced_render_descriptor(package_id: &str) -> bool {
    ADVANCED_RENDER_PACKAGE_IDS.contains(&package_id)
}

pub(super) fn classify_advanced_render_descriptor(
    package_id: &str,
    descriptor: RuntimePluginDescriptor,
) -> RuntimePluginDescriptor {
    match package_id {
        "virtual_geometry" => descriptor
            .with_maturity(PluginMaturity::Experimental)
            .with_capability("runtime.render.advanced.virtual_geometry")
            .with_capability_status(capability_status(
                "runtime.plugin.virtual_geometry",
                CapabilityStatus::Partial,
            ))
            .with_capability_status(
                capability_status(
                    "runtime.render.advanced.virtual_geometry",
                    CapabilityStatus::Partial,
                )
                .with_note(
                    "AdvancedRender provider path; default render profiles do not require it.",
                ),
            ),
        "hybrid_gi" => descriptor
            .with_maturity(PluginMaturity::Experimental)
            .with_capability("runtime.render.advanced.hybrid_gi")
            .with_capability_status(capability_status(
                "runtime.plugin.hybrid_gi",
                CapabilityStatus::Partial,
            ))
            .with_capability_status(
                capability_status(
                    "runtime.render.advanced.hybrid_gi",
                    CapabilityStatus::Partial,
                )
                .with_note(
                    "AdvancedRender provider path; default render profiles do not require it.",
                ),
            ),
        "solari" => descriptor
            .with_maturity(PluginMaturity::Experimental)
            .with_capability("runtime.render.experimental.solari")
            .with_capability_status(capability_status(
                "runtime.plugin.solari",
                CapabilityStatus::Partial,
            ))
            .with_capability_status(
                capability_status(
                    "runtime.render.experimental.solari",
                    CapabilityStatus::Partial,
                )
                .with_note(
                    "Solari realtime raytraced lighting pass executor is not implemented yet",
                ),
            ),
        _ => descriptor,
    }
}
