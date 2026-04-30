use super::super::builtin_render_feature::BuiltinRenderFeature;
use super::super::feature_descriptors::{
    baked_lighting, bloom, clustered_lighting, color_grading, debug_overlay, deferred_geometry,
    deferred_lighting, history_resolve, mesh, post_process, ray_tracing, reflection_probes,
    screen_space_ambient_occlusion, shadows,
};
use super::super::render_feature_descriptor::RenderFeatureDescriptor;
use crate::graphics::feature::RenderFeatureCapabilityRequirement;

pub(super) fn descriptor_for(feature: BuiltinRenderFeature) -> RenderFeatureDescriptor {
    match feature {
        BuiltinRenderFeature::Mesh => mesh::descriptor(),
        BuiltinRenderFeature::DeferredGeometry => deferred_geometry::descriptor(),
        BuiltinRenderFeature::DeferredLighting => deferred_lighting::descriptor(),
        BuiltinRenderFeature::ClusteredLighting => clustered_lighting::descriptor(),
        BuiltinRenderFeature::ScreenSpaceAmbientOcclusion => {
            screen_space_ambient_occlusion::descriptor()
        }
        BuiltinRenderFeature::Bloom => bloom::descriptor(),
        BuiltinRenderFeature::ColorGrading => color_grading::descriptor(),
        BuiltinRenderFeature::ReflectionProbes => reflection_probes::descriptor(),
        BuiltinRenderFeature::BakedLighting => baked_lighting::descriptor(),
        BuiltinRenderFeature::HistoryResolve => history_resolve::descriptor(),
        BuiltinRenderFeature::Shadows => shadows::descriptor(),
        BuiltinRenderFeature::PostProcess => post_process::descriptor(),
        BuiltinRenderFeature::DebugOverlay => debug_overlay::descriptor(),
        BuiltinRenderFeature::Particle => externalized_optional_plugin_descriptor("particle"),
        BuiltinRenderFeature::GlobalIllumination => {
            externalized_advanced_plugin_descriptor("global_illumination")
        }
        BuiltinRenderFeature::RayTracing => ray_tracing::descriptor(),
        BuiltinRenderFeature::VirtualGeometry => {
            externalized_advanced_plugin_descriptor("virtual_geometry")
        }
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

fn externalized_optional_plugin_descriptor(name: &str) -> RenderFeatureDescriptor {
    RenderFeatureDescriptor::new(name, Vec::new(), Vec::new(), Vec::new())
}
