use crate::{plugin::CapabilityStatus, plugin::PluginMaturity};

use super::capability_status::capability_status;
use super::BuiltinCatalogDescriptorBuilder;

pub(super) fn is_advanced_render_descriptor(package_id: &str) -> bool {
    matches!(package_id, "virtual_geometry" | "hybrid_gi" | "solari")
}

pub(super) fn classify_advanced_render_descriptor(
    package_id: &str,
    descriptor: BuiltinCatalogDescriptorBuilder,
) -> BuiltinCatalogDescriptorBuilder {
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

#[cfg(test)]
#[path = "tests/advanced_rendering_optimization_tests.rs"]
mod optimization_tests;
