use std::sync::Arc;
use std::time::Instant;

use zr_rhi::RenderDeviceProfile;

use crate::asset::ProjectAssetManagerAccess;
use crate::core::framework::render::{GeometrySourceDescriptor, ShadingModelDescriptor};
use crate::graphics::backend::SystemTextureGenerationLease;
use crate::graphics::scene::gpu_scene::GpuScene;
use crate::graphics::scene::scene_renderer::SceneRendererDeferredLightingProfile;
use crate::graphics::{
    GraphicsError, RenderFeatureDescriptor, RuntimePrepareCollectorRegistration,
};
use crate::text::font::FontCollectionService;

use super::super::super::super::deferred::DeferredSceneResources;
use super::super::super::super::environment::realtime_ibl_capture_wgpu::RealtimeIblCaptureWgpuPipelines;
use super::super::super::super::environment::{IblBakeWgpuPipelineCache, RealtimeIblRuntime};
use super::super::super::super::hzb::{HzbOcclusionCuller, hzb_occlusion_supported_by_limits};
use super::super::super::super::mesh::skinning::{
    create_empty_skinned_joint_palette_arena_buffer, skinned_joint_palette_arena_min_binding_size,
};
use super::super::super::super::mesh::{
    CachedMeshDrawCommands, MeshIndirectDrawWorkspace, MeshPipelineCache,
};
use super::super::super::super::overlay::{ViewportIconSource, ViewportOverlayRenderer};
use super::super::super::super::particle::ParticleRenderer;
use super::super::super::super::post_process::ScenePostProcessResources;
use super::super::super::super::scene_clear::SceneRegionClearResources;
use super::super::super::super::shadow::ShadowMapRenderer;
use super::super::super::super::shadow::atlas::{
    SHADOW_ATLAS_DEFAULT_CSM_ROW_HEIGHT, ShadowAtlasAllocator, ShadowAtlasConfig,
    ShadowAtlasResourceConfig, ShadowAtlasResources,
};
use super::super::super::super::sprite::SpriteRenderer;
use super::super::super::super::ui::ScreenSpaceUiRenderer;
use super::super::super::constants::{DEPTH_FORMAT, SCENE_COLOR_HDR_FORMAT};
use super::super::super::scene_renderer::{
    ScenePostProcessStartupMode, SceneRendererCoreStartupReport,
};
use super::super::super::scene_renderer_core::{
    SceneRendererAdvancedPluginResources, SceneRendererCore,
};
use super::super::layouts::{
    create_material_texture_bind_group_layout, create_texture_bind_group_layout,
};
use super::super::scene_bind_group_bundle::create_scene_bind_group_bundle;

const ENVIRONMENT_ONLY_SHADOW_ATLAS_PLACEHOLDER: ShadowAtlasResourceConfig =
    ShadowAtlasResourceConfig::new(1, 1, 1);

impl SceneRendererCore {
    pub(in crate::graphics::scene::scene_renderer::core) fn new_with_icon_source(
        asset_manager: ProjectAssetManagerAccess,
        device: &wgpu::Device,
        system_textures: SystemTextureGenerationLease,
        final_color_format: wgpu::TextureFormat,
        backend_name: &str,
        device_profile: &RenderDeviceProfile,
        icon_source: Arc<dyn ViewportIconSource>,
        render_features: &[RenderFeatureDescriptor],
        plugin_geometry_sources: impl IntoIterator<Item = GeometrySourceDescriptor>,
        plugin_shading_models: impl IntoIterator<Item = ShadingModelDescriptor>,
        runtime_prepare_collectors: impl IntoIterator<Item = RuntimePrepareCollectorRegistration>,
        deferred_lighting_profile: SceneRendererDeferredLightingProfile,
        font_collection: Arc<FontCollectionService>,
    ) -> Result<(Self, SceneRendererCoreStartupReport), GraphicsError> {
        if system_textures.device_id() != device_profile.device_id()
            || system_textures.generation() != device_profile.generation()
        {
            return Err(GraphicsError::WgpuValidation(format!(
                "system texture lease belongs to device {} generation {}, expected device {} generation {}",
                system_textures.device_id().raw(),
                system_textures.generation().raw(),
                device_profile.device_id().raw(),
                device_profile.generation().raw(),
            )));
        }
        let setup_started = Instant::now();
        let resolved_asset_manager = asset_manager
            .resolve()
            .map_err(|error| GraphicsError::Asset(error.to_string()))?;
        #[cfg(test)]
        let project_root = resolved_asset_manager
            .current_project_manager()
            .map(|project| project.paths().root().to_path_buf())
            // Graphics unit tests can intentionally construct a renderer before a project opens.
            // This test-only root preserves their isolated, non-product fixture behavior.
            .unwrap_or_else(|| {
                std::env::current_dir().expect("graphics unit tests require a working directory")
            });
        #[cfg(not(test))]
        let project_root = resolved_asset_manager
            .current_project_manager()
            .ok_or_else(|| {
                GraphicsError::Asset(
                    "scene renderer requires an active project before pipeline cache construction"
                        .to_owned(),
                )
            })?
            .paths()
            .root()
            .to_path_buf();
        let plugin_shading_models = plugin_shading_models.into_iter().collect::<Vec<_>>();
        let scene_bind_group_bundle = create_scene_bind_group_bundle(device, &system_textures);
        let skinned_joint_palette_arena_buffer =
            create_empty_skinned_joint_palette_arena_buffer(device);
        let texture_bind_group_layout = create_texture_bind_group_layout(device);
        let material_texture_bind_group_layout = create_material_texture_bind_group_layout(device);
        let gpu_scene = GpuScene::new(
            device,
            Arc::clone(&skinned_joint_palette_arena_buffer),
            skinned_joint_palette_arena_min_binding_size(),
        );
        let advanced_plugin_resources =
            SceneRendererAdvancedPluginResources::new(render_features, runtime_prepare_collectors);
        let volumetric_fog_enabled = advanced_plugin_resources.volumetric_fog_enabled();
        let setup = setup_started.elapsed();

        let mesh_and_environment_started = Instant::now();
        let scene_color_format = SCENE_COLOR_HDR_FORMAT;
        let mut mesh_pipelines = MeshPipelineCache::new_with_adapter_facts(
            device,
            &system_textures,
            scene_color_format,
            device_profile.adapter(),
            &project_root,
            &scene_bind_group_bundle.layout,
            &material_texture_bind_group_layout,
            gpu_scene.scene_bind_group_layout(),
            deferred_lighting_profile.defers_local_reflection_provider_resources(),
        );
        for descriptor in plugin_geometry_sources {
            mesh_pipelines.register_geometry_source_descriptor(descriptor);
        }
        let ibl_bake_pipeline_cache = IblBakeWgpuPipelineCache::new(device);
        let environment_capture_mip_pipelines = RealtimeIblCaptureWgpuPipelines::new(device);
        let realtime_ibl = RealtimeIblRuntime::new();
        let scene_clear = deferred_lighting_profile
            .supports_compiled_scene_graph()
            .then(|| SceneRegionClearResources::new(device, scene_color_format, DEPTH_FORMAT));
        let mesh_and_environment = mesh_and_environment_started.elapsed();

        let shadows_started = Instant::now();
        let shadow_map_renderer = deferred_lighting_profile
            .uses_shadow_map_renderer()
            .then(|| {
                ShadowMapRenderer::new(
                    &scene_bind_group_bundle.layout,
                    scene_bind_group_bundle.shadow_environment_binding_lease(),
                )
            });
        // The zero-direct-light preview retains valid shared bindings without reserving the
        // full 4096-square shadow atlas used by scene-capable renderer profiles.
        let shadow_atlas_resource_config =
            if deferred_lighting_profile.uses_full_shadow_atlas_resources() {
                ShadowAtlasResourceConfig::default()
            } else {
                ENVIRONMENT_ONLY_SHADOW_ATLAS_PLACEHOLDER
            };
        let shadow_atlas_resources =
            ShadowAtlasResources::new(device, shadow_atlas_resource_config);
        let shadow_atlas_resource_config = shadow_atlas_resources.config();
        let shadow_atlas_allocator = ShadowAtlasAllocator::new(ShadowAtlasConfig {
            width: shadow_atlas_resource_config.width,
            height: shadow_atlas_resource_config.height,
            reserved_top_px: SHADOW_ATLAS_DEFAULT_CSM_ROW_HEIGHT
                .min(shadow_atlas_resource_config.height),
            ..ShadowAtlasConfig::default()
        });
        let shadows = shadows_started.elapsed();

        let deferred_started = Instant::now();
        let (deferred, deferred_startup_report) = DeferredSceneResources::new(
            device,
            resolved_asset_manager.as_ref(),
            &scene_bind_group_bundle.layout,
            &material_texture_bind_group_layout,
            gpu_scene.scene_bind_group_layout(),
            mesh_pipelines.reflection_probes.bindings(),
            mesh_pipelines.lightmaps.bindings(),
            scene_color_format,
            &plugin_shading_models,
            volumetric_fog_enabled,
            deferred_lighting_profile,
        )?;
        let deferred_startup = deferred_started.elapsed();

        let uses_auxiliary_scene_effects = deferred_lighting_profile.uses_auxiliary_scene_effects();
        let scene_effects_started = Instant::now();
        let scene_effects_particles_started = Instant::now();
        let particle_renderer = uses_auxiliary_scene_effects.then(|| {
            ParticleRenderer::new(device, &scene_bind_group_bundle.layout, scene_color_format)
        });
        let scene_effects_particles = scene_effects_particles_started.elapsed();
        let scene_effects_sprites_started = Instant::now();
        let sprite_renderer = uses_auxiliary_scene_effects.then(|| {
            SpriteRenderer::new(
                device,
                &scene_bind_group_bundle.layout,
                &texture_bind_group_layout,
                scene_color_format,
            )
        });
        let scene_effects_sprites = scene_effects_sprites_started.elapsed();
        let scene_effects_hzb_started = Instant::now();
        let hzb_occlusion_culler = uses_auxiliary_scene_effects
            .then(|| {
                hzb_occlusion_supported_by_limits(&device.limits()).then(|| {
                    HzbOcclusionCuller::new(
                        device,
                        &scene_bind_group_bundle.layout,
                        gpu_scene.scene_bind_group_layout(),
                    )
                })
            })
            .flatten();
        let scene_effects_hzb = scene_effects_hzb_started.elapsed();
        let scene_effects_post_process_started = Instant::now();
        let post_process = match deferred_lighting_profile.post_process_startup_mode() {
            ScenePostProcessStartupMode::Full => ScenePostProcessResources::new(
                device,
                &system_textures,
                final_color_format,
                backend_name,
            )?,
            ScenePostProcessStartupMode::OutputTransferOnly => {
                ScenePostProcessResources::output_transfer_only(device, final_color_format)?
            }
        };
        let scene_effects_post_process = scene_effects_post_process_started.elapsed();
        let scene_effects = scene_effects_started.elapsed();

        let overlay_and_ui_started = Instant::now();
        let overlay_renderer = ViewportOverlayRenderer::new(
            device,
            scene_color_format,
            final_color_format,
            &scene_bind_group_bundle.layout,
            &texture_bind_group_layout,
            icon_source,
            volumetric_fog_enabled,
            deferred_lighting_profile.uses_interaction_overlays(),
        );
        let screen_space_ui_renderer = if deferred_lighting_profile.uses_screen_space_ui() {
            Some(ScreenSpaceUiRenderer::new_with_font_collection(
                asset_manager,
                device,
                final_color_format,
                font_collection,
            )?)
        } else {
            None
        };
        let overlay_and_ui = overlay_and_ui_started.elapsed();
        Ok((
            Self {
                device_id: device_profile.device_id(),
                device_generation: device_profile.generation(),
                texture_bind_group_layout,
                scene_bind_group_layout: scene_bind_group_bundle.layout,
                scene_uniform_buffer: scene_bind_group_bundle.uniform_buffer,
                scene_environment_sh9_buffer: scene_bind_group_bundle.environment_sh9_buffer,
                scene_environment_cubemap: scene_bind_group_bundle.environment_cubemap,
                scene_environment_brdf_lut: scene_bind_group_bundle.environment_brdf_lut,
                scene_bind_group: scene_bind_group_bundle.bind_group,
                scene_color_format,
                final_color_format,
                depth_format: DEPTH_FORMAT,
                global_material_mip_bias: 0.0,
                mesh_command_generation: 0,
                material_texture_bind_group_layout,
                mesh_pipelines,
                ibl_bake_pipeline_cache,
                environment_capture_mip_pipelines,
                realtime_ibl,
                scene_bind_group_realtime_ibl_slot: None,
                cached_mesh_draw_commands: CachedMeshDrawCommands::default(),
                mesh_indirect_draw_workspace: MeshIndirectDrawWorkspace::default(),
                neutral_graph_buffers: Default::default(),
                gpu_scene,
                hit_proxy_resources: Default::default(),
                hzb_occlusion_culler,
                scene_clear,
                deferred_lighting_profile,
                shadow_map_renderer,
                shadow_atlas_allocator,
                shadow_atlas_resources,
                deferred,
                particle_renderer,
                sprite_renderer,
                post_process,
                overlay_renderer,
                screen_space_ui_renderer,
                transient_resource_pool: Default::default(),
                diagnostic_frame_index: 0,
                ibl_bake_runtime_writebacks: Default::default(),
                advanced_plugin_resources,
            },
            SceneRendererCoreStartupReport {
                setup,
                mesh_and_environment,
                shadows,
                deferred: deferred_startup,
                deferred_lighting_pipelines: deferred_startup_report.lighting_pipelines(),
                deferred_lighting_shader_source_assembly: deferred_startup_report
                    .lighting_shader_source_assembly(),
                deferred_lighting_pipeline_foundation: deferred_startup_report
                    .lighting_pipeline_foundation(),
                deferred_lighting_standard_pipeline: deferred_startup_report
                    .lighting_standard_pipeline(),
                deferred_fallback_resources: deferred_startup_report.fallback_resources(),
                scene_effects,
                scene_effects_particles,
                scene_effects_sprites,
                scene_effects_hzb,
                scene_effects_post_process,
                overlay_and_ui,
            },
        ))
    }
}
