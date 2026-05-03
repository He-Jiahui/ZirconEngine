use crate::{
    plugin::PluginFeatureBundleManifest, plugin::PluginFeatureDependency,
    plugin::PluginModuleManifest, RuntimePluginId, RuntimeTargetMode,
};

use super::RuntimePluginDescriptor;

impl RuntimePluginDescriptor {
    pub fn builtin_catalog() -> Vec<Self> {
        [
            (
                "physics",
                "Physics",
                RuntimePluginId::Physics,
                "zircon_plugin_physics_runtime",
                "runtime.plugin.physics",
                &[
                    RuntimeTargetMode::ClientRuntime,
                    RuntimeTargetMode::ServerRuntime,
                    RuntimeTargetMode::EditorHost,
                ][..],
            ),
            (
                "sound",
                "Sound",
                RuntimePluginId::Sound,
                "zircon_plugin_sound_runtime",
                "runtime.plugin.sound",
                &[
                    RuntimeTargetMode::ClientRuntime,
                    RuntimeTargetMode::EditorHost,
                ][..],
            ),
            (
                "texture",
                "Texture",
                RuntimePluginId::Texture,
                "zircon_plugin_texture_runtime",
                "runtime.plugin.texture",
                &[
                    RuntimeTargetMode::ClientRuntime,
                    RuntimeTargetMode::EditorHost,
                ][..],
            ),
            (
                "net",
                "Network",
                RuntimePluginId::Net,
                "zircon_plugin_net_runtime",
                "runtime.plugin.net",
                &[
                    RuntimeTargetMode::ServerRuntime,
                    RuntimeTargetMode::ClientRuntime,
                ][..],
            ),
            (
                "navigation",
                "Navigation",
                RuntimePluginId::Navigation,
                "zircon_plugin_navigation_runtime",
                "runtime.plugin.navigation",
                &[
                    RuntimeTargetMode::ClientRuntime,
                    RuntimeTargetMode::ServerRuntime,
                    RuntimeTargetMode::EditorHost,
                ][..],
            ),
            (
                "particles",
                "Particles",
                RuntimePluginId::Particles,
                "zircon_plugin_particles_runtime",
                "runtime.plugin.particles",
                &[
                    RuntimeTargetMode::ClientRuntime,
                    RuntimeTargetMode::EditorHost,
                ][..],
            ),
            (
                "animation",
                "Animation",
                RuntimePluginId::Animation,
                "zircon_plugin_animation_runtime",
                "runtime.plugin.animation",
                &[
                    RuntimeTargetMode::ClientRuntime,
                    RuntimeTargetMode::ServerRuntime,
                    RuntimeTargetMode::EditorHost,
                ][..],
            ),
            (
                "terrain",
                "Terrain",
                RuntimePluginId::Terrain,
                "zircon_plugin_terrain_runtime",
                "runtime.plugin.terrain",
                &[
                    RuntimeTargetMode::ClientRuntime,
                    RuntimeTargetMode::EditorHost,
                ][..],
            ),
            (
                "tilemap_2d",
                "Tilemap 2D",
                RuntimePluginId::Tilemap2d,
                "zircon_plugin_tilemap_2d_runtime",
                "runtime.plugin.tilemap_2d",
                &[
                    RuntimeTargetMode::ClientRuntime,
                    RuntimeTargetMode::EditorHost,
                ][..],
            ),
            (
                "prefab_tools",
                "Prefab Tools",
                RuntimePluginId::PrefabTools,
                "zircon_plugin_prefab_tools_runtime",
                "runtime.plugin.prefab_tools",
                &[
                    RuntimeTargetMode::ClientRuntime,
                    RuntimeTargetMode::EditorHost,
                ][..],
            ),
            (
                "gltf_importer",
                "glTF Importer",
                RuntimePluginId::GltfImporter,
                "zircon_plugin_gltf_importer_runtime",
                "runtime.plugin.gltf_importer",
                &[
                    RuntimeTargetMode::ClientRuntime,
                    RuntimeTargetMode::EditorHost,
                ][..],
            ),
            (
                "obj_importer",
                "OBJ Importer",
                RuntimePluginId::ObjImporter,
                "zircon_plugin_obj_importer_runtime",
                "runtime.plugin.obj_importer",
                &[
                    RuntimeTargetMode::ClientRuntime,
                    RuntimeTargetMode::EditorHost,
                ][..],
            ),
            (
                "texture_importer",
                "Texture Importer",
                RuntimePluginId::TextureImporter,
                "zircon_plugin_texture_importer_runtime",
                "runtime.plugin.texture_importer",
                &[
                    RuntimeTargetMode::ClientRuntime,
                    RuntimeTargetMode::EditorHost,
                ][..],
            ),
            (
                "audio_importer",
                "Audio Importer",
                RuntimePluginId::AudioImporter,
                "zircon_plugin_audio_importer_runtime",
                "runtime.plugin.audio_importer",
                &[
                    RuntimeTargetMode::ClientRuntime,
                    RuntimeTargetMode::EditorHost,
                ][..],
            ),
            (
                "shader_wgsl_importer",
                "WGSL Shader Importer",
                RuntimePluginId::ShaderWgslImporter,
                "zircon_plugin_shader_wgsl_importer_runtime",
                "runtime.plugin.shader_wgsl_importer",
                &[
                    RuntimeTargetMode::ClientRuntime,
                    RuntimeTargetMode::EditorHost,
                ][..],
            ),
            (
                "ui_document_importer",
                "UI Document Importer",
                RuntimePluginId::UiDocumentImporter,
                "zircon_plugin_ui_document_importer_runtime",
                "runtime.plugin.ui_document_importer",
                &[
                    RuntimeTargetMode::ClientRuntime,
                    RuntimeTargetMode::EditorHost,
                ][..],
            ),
            (
                "rendering",
                "Rendering",
                RuntimePluginId::Rendering,
                "zircon_plugin_rendering_runtime",
                "runtime.plugin.rendering",
                &[
                    RuntimeTargetMode::ClientRuntime,
                    RuntimeTargetMode::EditorHost,
                ][..],
            ),
            (
                "virtual_geometry",
                "Virtual Geometry",
                RuntimePluginId::VirtualGeometry,
                "zircon_plugin_virtual_geometry_runtime",
                "runtime.plugin.virtual_geometry",
                &[
                    RuntimeTargetMode::ClientRuntime,
                    RuntimeTargetMode::EditorHost,
                ][..],
            ),
            (
                "hybrid_gi",
                "Hybrid GI",
                RuntimePluginId::HybridGi,
                "zircon_plugin_hybrid_gi_runtime",
                "runtime.plugin.hybrid_gi",
                &[
                    RuntimeTargetMode::ClientRuntime,
                    RuntimeTargetMode::EditorHost,
                ][..],
            ),
        ]
        .into_iter()
        .map(
            |(id, name, runtime_id, crate_name, capability, target_modes)| {
                Self::new(id, name, runtime_id, crate_name)
                    .with_target_modes(target_modes.iter().copied())
                    .with_capability(capability)
            },
        )
        .map(|descriptor| match descriptor.package_id.as_str() {
            "animation" => {
                descriptor.with_capability("runtime.feature.animation.timeline_event_track")
            }
            "terrain" | "tilemap_2d" | "prefab_tools" => descriptor.with_category("authoring"),
            "physics" => descriptor.with_capability("runtime.capability.physics.raycast"),
            "gltf_importer" => descriptor
                .with_category("asset_importer")
                .with_capability("runtime.asset.importer.model.gltf"),
            "obj_importer" => descriptor
                .with_category("asset_importer")
                .with_capability("runtime.asset.importer.model.obj"),
            "texture_importer" => descriptor
                .with_category("asset_importer")
                .with_capability("runtime.asset.importer.texture.image"),
            "audio_importer" => descriptor
                .with_category("asset_importer")
                .with_capability("runtime.asset.importer.audio.wav"),
            "shader_wgsl_importer" => descriptor
                .with_category("asset_importer")
                .with_capability("runtime.asset.importer.shader.wgsl"),
            "ui_document_importer" => descriptor
                .with_category("asset_importer")
                .with_capability("runtime.asset.importer.ui_document"),
            "sound" => descriptor
                .with_optional_feature(sound_timeline_animation_track_feature())
                .with_optional_feature(sound_ray_traced_convolution_reverb_feature()),
            "rendering" => descriptor
                .with_category("rendering")
                .with_optional_feature(rendering_feature(
                    "post_process",
                    "Post Process",
                    true,
                    Vec::new(),
                ))
                .with_optional_feature(rendering_feature("ssao", "SSAO", true, Vec::new()))
                .with_optional_feature(rendering_feature("decals", "Decals", false, Vec::new()))
                .with_optional_feature(rendering_feature(
                    "reflection_probes",
                    "Reflection Probes",
                    true,
                    Vec::new(),
                ))
                .with_optional_feature(rendering_feature(
                    "baked_lighting",
                    "Baked Lighting",
                    true,
                    Vec::new(),
                ))
                .with_optional_feature(rendering_feature(
                    "ray_tracing_policy",
                    "Ray Tracing Policy",
                    false,
                    Vec::new(),
                ))
                .with_optional_feature(rendering_feature(
                    "shader_graph",
                    "Shader Graph",
                    false,
                    Vec::new(),
                ))
                .with_optional_feature(rendering_feature(
                    "vfx_graph",
                    "VFX Graph",
                    false,
                    vec![
                        PluginFeatureDependency::required("particles", "runtime.plugin.particles"),
                        PluginFeatureDependency::required(
                            "rendering",
                            "runtime.feature.rendering.shader_graph",
                        ),
                    ],
                )),
            _ => descriptor,
        })
        .collect()
    }
}

fn rendering_feature(
    id_suffix: &str,
    display_name: &str,
    enabled_by_default: bool,
    extra_dependencies: Vec<PluginFeatureDependency>,
) -> PluginFeatureBundleManifest {
    let feature_id = format!("rendering.{id_suffix}");
    let capability = format!("runtime.feature.rendering.{id_suffix}");
    let runtime_crate = format!("zircon_plugin_rendering_{id_suffix}_runtime");
    let editor_crate = format!("zircon_plugin_rendering_{id_suffix}_editor");
    let mut manifest =
        PluginFeatureBundleManifest::new(feature_id.clone(), display_name, "rendering")
            .with_dependency(PluginFeatureDependency::primary(
                "rendering",
                "runtime.plugin.rendering",
            ))
            .with_capability(capability.clone())
            .with_runtime_module(
                PluginModuleManifest::runtime(format!("{feature_id}.runtime"), runtime_crate)
                    .with_target_modes([
                        RuntimeTargetMode::ClientRuntime,
                        RuntimeTargetMode::EditorHost,
                    ])
                    .with_capabilities([capability.clone()]),
            )
            .with_editor_module(PluginModuleManifest::editor(
                format!("{feature_id}.editor"),
                editor_crate,
            ))
            .enabled_by_default(enabled_by_default);
    for dependency in extra_dependencies {
        manifest = manifest.with_dependency(dependency);
    }
    manifest
}

fn sound_timeline_animation_track_feature() -> PluginFeatureBundleManifest {
    PluginFeatureBundleManifest::new(
        "sound.timeline_animation_track",
        "Sound Timeline Animation Track",
        "sound",
    )
    .with_dependency(PluginFeatureDependency::primary(
        "sound",
        "runtime.plugin.sound",
    ))
    .with_dependency(PluginFeatureDependency::required(
        "animation",
        "runtime.feature.animation.timeline_event_track",
    ))
    .with_capability("runtime.feature.sound.timeline_animation_track")
    .with_runtime_module(
        PluginModuleManifest::runtime(
            "sound.timeline_animation_track.runtime",
            "zircon_plugin_sound_timeline_animation_runtime",
        )
        .with_target_modes([
            RuntimeTargetMode::ClientRuntime,
            RuntimeTargetMode::EditorHost,
        ])
        .with_capabilities(["runtime.feature.sound.timeline_animation_track".to_string()]),
    )
    .with_editor_module(PluginModuleManifest::editor(
        "sound.timeline_animation_track.editor",
        "zircon_plugin_sound_timeline_animation_editor",
    ))
}

fn sound_ray_traced_convolution_reverb_feature() -> PluginFeatureBundleManifest {
    PluginFeatureBundleManifest::new(
        "sound.ray_traced_convolution_reverb",
        "Ray Traced Convolution Reverb",
        "sound",
    )
    .with_dependency(PluginFeatureDependency::primary(
        "sound",
        "runtime.plugin.sound",
    ))
    .with_dependency(PluginFeatureDependency::required(
        "physics",
        "runtime.plugin.physics",
    ))
    .with_dependency(PluginFeatureDependency::required(
        "physics",
        "runtime.capability.physics.raycast",
    ))
    .with_capability("runtime.feature.sound.ray_traced_convolution_reverb")
    .with_runtime_module(
        PluginModuleManifest::runtime(
            "sound.ray_traced_convolution_reverb.runtime",
            "zircon_plugin_sound_ray_traced_convolution_runtime",
        )
        .with_target_modes([
            RuntimeTargetMode::ClientRuntime,
            RuntimeTargetMode::EditorHost,
        ])
        .with_capabilities(["runtime.feature.sound.ray_traced_convolution_reverb".to_string()]),
    )
    .with_editor_module(PluginModuleManifest::editor(
        "sound.ray_traced_convolution_reverb.editor",
        "zircon_plugin_sound_ray_traced_convolution_editor",
    ))
}
