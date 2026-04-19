use super::super::builtin_render_feature::BuiltinRenderFeature;
use super::super::feature_descriptors::{
    baked_lighting, bloom, clustered_lighting, color_grading, debug_overlay, deferred_geometry,
    deferred_lighting, global_illumination, history_resolve, mesh, particle, post_process,
    ray_tracing, reflection_probes, screen_space_ambient_occlusion, shadows, virtual_geometry,
};
use super::super::render_feature_descriptor::RenderFeatureDescriptor;

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
        BuiltinRenderFeature::Particle => particle::descriptor(),
        BuiltinRenderFeature::GlobalIllumination => global_illumination::descriptor(),
        BuiltinRenderFeature::RayTracing => ray_tracing::descriptor(),
        BuiltinRenderFeature::VirtualGeometry => virtual_geometry::descriptor(),
    }
}
