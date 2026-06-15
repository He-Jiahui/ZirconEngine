use super::super::builtin_render_feature::{descriptor_only_advanced_slot, BuiltinRenderFeature};
use super::super::feature_descriptors::{
    advanced_slot, anti_alias, bloom, clustered_lighting, color_grading, debug_overlay,
    deferred_geometry, deferred_lighting, hzb, mesh, neural_compute, post_process, ray_tracing,
    screen_space_ambient_occlusion, shadows, sprite, temporal, ui,
};
use super::super::render_feature_descriptor::RenderFeatureDescriptor;
use crate::graphics::feature::RenderFeatureCapabilityRequirement;

pub(super) fn descriptor_for(feature: BuiltinRenderFeature) -> RenderFeatureDescriptor {
    if let Some(slot) = descriptor_only_advanced_slot(feature) {
        return advanced_slot::descriptor(slot);
    }

    match feature {
        BuiltinRenderFeature::Mesh => mesh::descriptor(),
        BuiltinRenderFeature::Sprite => sprite::descriptor(),
        BuiltinRenderFeature::DeferredGeometry => deferred_geometry::descriptor(),
        BuiltinRenderFeature::DeferredLighting => deferred_lighting::descriptor(),
        BuiltinRenderFeature::ClusteredLighting => clustered_lighting::descriptor(),
        BuiltinRenderFeature::Hzb => hzb::descriptor(),
        BuiltinRenderFeature::ScreenSpaceAmbientOcclusion => {
            screen_space_ambient_occlusion::descriptor()
        }
        BuiltinRenderFeature::Bloom => bloom::descriptor(),
        BuiltinRenderFeature::ColorGrading => color_grading::descriptor(),
        BuiltinRenderFeature::Temporal => temporal::descriptor(),
        BuiltinRenderFeature::AntiAlias => anti_alias::descriptor(),
        BuiltinRenderFeature::Shadows => shadows::descriptor(),
        BuiltinRenderFeature::PostProcess => post_process::descriptor(),
        BuiltinRenderFeature::Ui => ui::descriptor(),
        BuiltinRenderFeature::DebugOverlay => debug_overlay::descriptor(),
        BuiltinRenderFeature::GlobalIllumination => {
            externalized_advanced_plugin_descriptor("global_illumination")
        }
        BuiltinRenderFeature::RayTracing => ray_tracing::descriptor(),
        BuiltinRenderFeature::NeuralCompute => neural_compute::descriptor(),
        BuiltinRenderFeature::VirtualGeometry => {
            externalized_advanced_plugin_descriptor("virtual_geometry")
        }
        descriptor_only_slot => unreachable!(
            "descriptor-only advanced slot {:?} must be registered in the advanced slot catalog",
            descriptor_only_slot
        ),
    }
}

fn externalized_advanced_plugin_descriptor(name: &str) -> RenderFeatureDescriptor {
    let descriptor = RenderFeatureDescriptor::new(name, Vec::new(), Vec::new(), Vec::new());
    match name {
        "virtual_geometry" => descriptor
            .with_capability_requirement(RenderFeatureCapabilityRequirement::VirtualGeometry),
        "global_illumination" => descriptor.with_capability_requirement(
            RenderFeatureCapabilityRequirement::HybridGlobalIllumination,
        ),
        _ => descriptor,
    }
}
