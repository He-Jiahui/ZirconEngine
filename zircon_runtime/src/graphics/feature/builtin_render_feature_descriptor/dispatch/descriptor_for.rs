use super::super::builtin_render_feature::BuiltinRenderFeature;
use super::super::feature_descriptors::{
    advanced_slot, anti_alias, baked_lighting, bloom, clustered_lighting, color_grading,
    debug_overlay, deferred_geometry, deferred_lighting, history_resolve, mesh, neural_compute,
    post_process, ray_tracing, reflection_probes, screen_space_ambient_occlusion, shadows, sprite,
    ui,
};
use super::super::render_feature_descriptor::RenderFeatureDescriptor;
use crate::graphics::feature::RenderFeatureCapabilityRequirement;

pub(super) fn descriptor_for(feature: BuiltinRenderFeature) -> RenderFeatureDescriptor {
    match feature {
        BuiltinRenderFeature::Mesh => mesh::descriptor(),
        BuiltinRenderFeature::Sprite => sprite::descriptor(),
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
        BuiltinRenderFeature::AntiAlias => anti_alias::descriptor(),
        BuiltinRenderFeature::Shadows => shadows::descriptor(),
        BuiltinRenderFeature::PostProcess => post_process::descriptor(),
        BuiltinRenderFeature::Ui => ui::descriptor(),
        BuiltinRenderFeature::DebugOverlay => debug_overlay::descriptor(),
        BuiltinRenderFeature::Particle => advanced_slot::descriptor("particle", "particles"),
        BuiltinRenderFeature::GlobalIllumination => {
            externalized_advanced_plugin_descriptor("global_illumination")
        }
        BuiltinRenderFeature::RayTracing => ray_tracing::descriptor(),
        BuiltinRenderFeature::NeuralCompute => neural_compute::descriptor(),
        BuiltinRenderFeature::SparseTexture => {
            advanced_slot::descriptor("sparse_texture", "sparse_texture")
                .with_capability_requirement(RenderFeatureCapabilityRequirement::SparseTexture)
        }
        BuiltinRenderFeature::Terrain => advanced_slot::descriptor("terrain", "terrain"),
        BuiltinRenderFeature::Tree => advanced_slot::descriptor("tree", "tree"),
        BuiltinRenderFeature::Projector => advanced_slot::descriptor("projector", "projector"),
        BuiltinRenderFeature::Halo => advanced_slot::descriptor("halo", "halo"),
        BuiltinRenderFeature::LensFlare => advanced_slot::descriptor("lens_flare", "lens_flare"),
        BuiltinRenderFeature::Trail => advanced_slot::descriptor("trail", "trail"),
        BuiltinRenderFeature::Billboard => advanced_slot::descriptor("billboard", "billboard"),
        BuiltinRenderFeature::Tilemap => advanced_slot::descriptor("tilemap", "tilemap"),
        BuiltinRenderFeature::TextShaping => {
            advanced_slot::descriptor("text_shaping", "text_shaping")
        }
        BuiltinRenderFeature::Skybox => advanced_slot::descriptor("skybox", "skybox"),
        BuiltinRenderFeature::Cubemap => advanced_slot::descriptor("cubemap", "cubemap"),
        BuiltinRenderFeature::Texture2dArray => {
            advanced_slot::descriptor("texture_2d_array", "texture_2d_array")
        }
        BuiltinRenderFeature::NormalMap => advanced_slot::descriptor("normal_map", "normal_map"),
        BuiltinRenderFeature::Mipmap => advanced_slot::descriptor("mipmap", "mipmap"),
        BuiltinRenderFeature::ColorSpace => advanced_slot::descriptor("color_space", "color_space"),
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
