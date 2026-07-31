use crate::graphics::scene::scene_renderer::SceneRendererDeferredLightingProfile;
use crate::graphics::scene::scene_renderer::advanced_lighting::irradiance_volume::IrradianceVolumeResources;
use crate::graphics::scene::scene_renderer::advanced_lighting::light_cookie::LightCookieAtlasResources;
use crate::graphics::scene::scene_renderer::attachment_ops::color_attachment_operations;
use crate::graphics::scene::scene_renderer::shadow::atlas::{
    SHADOW_ATLAS_BINDING, SHADOW_ATLAS_SAMPLER_BINDING, SHADOW_ATLAS_SLOT_BUFFER_BINDING,
    SHADOW_GLOBALS_BINDING, ShadowAtlasResources,
};
use crate::graphics::types::ViewportRenderFrame;
use crate::graphics::types::ViewportRenderRegion;
use crate::render_graph::RenderGraphAttachmentOps;

use super::DeferredSceneResources;

// 12 base entries + 5 probes + 3 lightmap + 3 volumetric + 2 cookies + 3 irradiance volume.
const DEFERRED_LIGHTING_BIND_GROUP_ENTRY_CAPACITY: usize = 28;
const ENVIRONMENT_ONLY_PBR_BIND_GROUP_ENTRY_CAPACITY: usize = 10;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct DeferredLightingExecutionPlan {
    full_lighting_bind_group: bool,
    bind_group_entry_capacity: usize,
    subsurface_mrt: bool,
    color_attachment_count: usize,
    uses_gpu_scene: bool,
}

impl DeferredLightingExecutionPlan {
    const fn new(
        profile: SceneRendererDeferredLightingProfile,
        has_subsurface_diffuse_target: bool,
        has_subsurface_retained_target: bool,
    ) -> Self {
        let full_lighting_bind_group = profile.uses_full_lighting_bind_group();
        let subsurface_mrt = full_lighting_bind_group
            && has_subsurface_diffuse_target
            && has_subsurface_retained_target;
        Self {
            full_lighting_bind_group,
            bind_group_entry_capacity: if full_lighting_bind_group {
                DEFERRED_LIGHTING_BIND_GROUP_ENTRY_CAPACITY
            } else {
                ENVIRONMENT_ONLY_PBR_BIND_GROUP_ENTRY_CAPACITY
            },
            subsurface_mrt,
            color_attachment_count: if subsurface_mrt { 3 } else { 1 },
            uses_gpu_scene: profile.uses_gpu_scene(),
        }
    }
}

impl DeferredSceneResources {
    pub(crate) fn execute_lighting(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        scene_bind_group: &wgpu::BindGroup,
        gpu_scene_bind_group: &wgpu::BindGroup,
        gbuffer_albedo_view: &wgpu::TextureView,
        normal_view: &wgpu::TextureView,
        gbuffer_material_view: &wgpu::TextureView,
        gbuffer_emissive_view: &wgpu::TextureView,
        scene_depth_view: &wgpu::TextureView,
        shadow_atlas_resources: Option<&ShadowAtlasResources>,
        light_grid_params_buffer: &wgpu::Buffer,
        light_zbins_buffer: &wgpu::Buffer,
        light_tile_masks_buffer: &wgpu::Buffer,
        integrated_volumetric_view: Option<&wgpu::TextureView>,
        light_cookies: &LightCookieAtlasResources,
        irradiance_volume: &IrradianceVolumeResources,
        frame: &ViewportRenderFrame,
        scene_color_view: &wgpu::TextureView,
        subsurface_diffuse_view: Option<&wgpu::TextureView>,
        subsurface_retained_view: Option<&wgpu::TextureView>,
        attachment_ops: RenderGraphAttachmentOps,
        render_region: ViewportRenderRegion,
    ) {
        let execution_plan = DeferredLightingExecutionPlan::new(
            self.deferred_lighting_profile,
            subsurface_diffuse_view.is_some(),
            subsurface_retained_view.is_some(),
        );
        let mut entries = Vec::with_capacity(execution_plan.bind_group_entry_capacity);
        entries.extend([
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(gbuffer_albedo_view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(normal_view),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: wgpu::BindingResource::TextureView(gbuffer_material_view),
            },
            wgpu::BindGroupEntry {
                binding: 4,
                resource: wgpu::BindingResource::TextureView(scene_depth_view),
            },
            wgpu::BindGroupEntry {
                binding: 5,
                resource: wgpu::BindingResource::TextureView(gbuffer_emissive_view),
            },
        ]);
        entries.extend(self.reflection_probe_bindings.bind_group_entries());
        // Entries retain a reference until create_bind_group below, so the owned
        // params buffer must outlive the direct-light conditional branch.
        let volumetric_params_buffer = execution_plan.full_lighting_bind_group.then(|| {
            self.volumetric_apply.create_params_buffer(
                device,
                frame,
                render_region.local_render_region(),
                integrated_volumetric_view.is_some(),
                "zircon-deferred-volumetric-params",
            )
        });
        if execution_plan.full_lighting_bind_group {
            let shadow_atlas_view = shadow_atlas_resources
                .map(ShadowAtlasResources::atlas_view)
                .unwrap_or(&self.shadow_atlas_fallback_view);
            let shadow_atlas_sampler = shadow_atlas_resources
                .map(ShadowAtlasResources::compare_sampler)
                .unwrap_or(&self.shadow_compare_sampler);
            let shadow_atlas_slot_buffer = shadow_atlas_resources
                .map(ShadowAtlasResources::slot_buffer)
                .unwrap_or(&self.shadow_atlas_fallback_slot_buffer);
            let shadow_atlas_globals_buffer = shadow_atlas_resources
                .map(ShadowAtlasResources::globals_buffer)
                .unwrap_or(&self.shadow_atlas_fallback_globals_buffer);
            let volumetric_params_buffer = volumetric_params_buffer
                .as_ref()
                .expect("full deferred lighting creates volumetric parameters");
            entries.extend([
                wgpu::BindGroupEntry {
                    binding: SHADOW_ATLAS_BINDING,
                    resource: wgpu::BindingResource::TextureView(shadow_atlas_view),
                },
                wgpu::BindGroupEntry {
                    binding: SHADOW_ATLAS_SAMPLER_BINDING,
                    resource: wgpu::BindingResource::Sampler(shadow_atlas_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: SHADOW_ATLAS_SLOT_BUFFER_BINDING,
                    resource: shadow_atlas_slot_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: SHADOW_GLOBALS_BINDING,
                    resource: shadow_atlas_globals_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 20,
                    resource: light_grid_params_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 21,
                    resource: light_zbins_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 22,
                    resource: light_tile_masks_buffer.as_entire_binding(),
                },
            ]);
            entries.extend(self.lightmap_bindings.bind_group_entries());
            entries.extend(
                self.volumetric_apply
                    .bind_group_entries(volumetric_params_buffer, integrated_volumetric_view),
            );
            entries.extend(light_cookies.bind_group_entries());
            entries.extend(irradiance_volume.bind_group_entries());
        }
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("zircon-deferred-lighting-bind-group"),
            layout: &self.lighting_bind_group_layout,
            entries: &entries,
        });

        let mut color_attachments = [
            Some(wgpu::RenderPassColorAttachment {
                view: scene_color_view,
                resolve_target: None,
                depth_slice: None,
                ops: color_attachment_operations(attachment_ops, wgpu::Color::BLACK),
            }),
            None,
            None,
        ];
        if execution_plan.subsurface_mrt {
            let (diffuse_view, retained_view) = (
                subsurface_diffuse_view.expect("subsurface MRT requires a diffuse target"),
                subsurface_retained_view.expect("subsurface MRT requires a retained target"),
            );
            color_attachments[1] = Some(wgpu::RenderPassColorAttachment {
                view: diffuse_view,
                resolve_target: None,
                depth_slice: None,
                ops: color_attachment_operations(
                    RenderGraphAttachmentOps::clear_store(),
                    wgpu::Color::TRANSPARENT,
                ),
            });
            color_attachments[2] = Some(wgpu::RenderPassColorAttachment {
                view: retained_view,
                resolve_target: None,
                depth_slice: None,
                ops: color_attachment_operations(
                    RenderGraphAttachmentOps::clear_store(),
                    wgpu::Color::TRANSPARENT,
                ),
            });
        }
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("DeferredLightingPass"),
            color_attachments: &color_attachments[..execution_plan.color_attachment_count],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
            multiview_mask: None,
        });
        if !render_region.apply_local_to_render_pass(&mut pass) {
            return;
        }
        pass.set_pipeline(self.lighting_pipelines.pipeline(
            device,
            &self.lighting_bind_group_layout,
            execution_plan.subsurface_mrt,
        ));
        pass.set_bind_group(0, scene_bind_group, &[]);
        pass.set_bind_group(1, &bind_group, &[]);
        if execution_plan.uses_gpu_scene {
            pass.set_bind_group(3, gpu_scene_bind_group, &[]);
        }
        pass.draw(0..3, 0..1);
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::asset::{ProjectAssetManager, ProjectAssetManagerAccess};
    use crate::core::framework::render::{
        EnvironmentExtract, FallbackSkyboxKind, PreviewEnvironmentExtract, RenderOverlayExtract,
        RenderSceneGeometryExtract, RenderSceneSnapshot, ViewportCameraSnapshot,
    };
    use crate::core::math::{UVec2, Vec4};
    use crate::graphics::scene::scene_renderer::{SceneRenderer, SceneRendererStartupOptions};

    fn empty_lit_snapshot() -> RenderSceneSnapshot {
        RenderSceneSnapshot {
            scene: RenderSceneGeometryExtract {
                camera: ViewportCameraSnapshot::default(),
                meshes: Vec::new(),
                directional_lights: Vec::new(),
                point_lights: Vec::new(),
                spot_lights: Vec::new(),
                ambient_lights: Vec::new(),
                rect_lights: Vec::new(),
            },
            overlays: RenderOverlayExtract::default(),
            environment: EnvironmentExtract::default(),
            preview: PreviewEnvironmentExtract {
                lighting_enabled: true,
                skybox_enabled: false,
                fallback_skybox: FallbackSkyboxKind::None,
                clear_color: Vec4::ZERO,
            },
            virtual_geometry_debug: None,
        }
    }

    #[test]
    fn deferred_lighting_color_attachments_use_fixed_stack_storage() {
        let source = include_str!("execute_lighting.rs");
        let implementation = source
            .split("#[cfg(test)]")
            .next()
            .expect("deferred lighting implementation");

        assert!(!implementation.contains("let mut color_attachments = vec!["));
        assert!(implementation.contains("let mut color_attachments = ["));
        assert!(
            implementation.contains("color_attachments[..execution_plan.color_attachment_count]")
        );
    }

    #[test]
    fn deferred_lighting_selects_a_cached_pipeline_for_the_active_mrt_shape() {
        let source = include_str!("execute_lighting.rs");
        let implementation = source
            .split("#[cfg(test)]")
            .next()
            .expect("deferred lighting implementation");

        assert!(implementation.contains("self.lighting_pipelines.pipeline("));
        assert!(implementation.contains("execution_plan.subsurface_mrt,"));
    }

    #[test]
    fn deferred_lighting_preallocates_the_fixed_bind_group_entry_count() {
        let source = include_str!("execute_lighting.rs");
        let implementation = source
            .split("#[cfg(test)]")
            .next()
            .expect("deferred lighting implementation");

        assert!(
            implementation
                .contains("const DEFERRED_LIGHTING_BIND_GROUP_ENTRY_CAPACITY: usize = 28")
        );
        assert!(implementation.contains("ENVIRONMENT_ONLY_PBR_BIND_GROUP_ENTRY_CAPACITY"));
        assert!(
            implementation.contains("Vec::with_capacity(execution_plan.bind_group_entry_capacity)")
        );
    }

    #[test]
    fn full_scene_lighting_plan_builds_the_complete_bind_group_shape() {
        let plan = DeferredLightingExecutionPlan::new(
            SceneRendererDeferredLightingProfile::FullScene,
            true,
            true,
        );

        assert!(plan.full_lighting_bind_group);
        assert_eq!(
            plan.bind_group_entry_capacity,
            DEFERRED_LIGHTING_BIND_GROUP_ENTRY_CAPACITY
        );
        assert!(plan.subsurface_mrt);
        assert_eq!(plan.color_attachment_count, 3);
        assert!(plan.uses_gpu_scene);
    }

    #[test]
    fn environment_only_lighting_plan_omits_direct_lighting_resources() {
        let plan = DeferredLightingExecutionPlan::new(
            SceneRendererDeferredLightingProfile::EnvironmentOnlyPbrPreview,
            true,
            true,
        );

        assert!(!plan.full_lighting_bind_group);
        assert_eq!(
            plan.bind_group_entry_capacity,
            ENVIRONMENT_ONLY_PBR_BIND_GROUP_ENTRY_CAPACITY
        );
        assert!(!plan.subsurface_mrt);
        assert_eq!(plan.color_attachment_count, 1);
        assert!(!plan.uses_gpu_scene);
    }

    #[test]
    fn deferred_lighting_profiles_construct_bind_groups_during_real_render() {
        let asset_manager = Arc::new(ProjectAssetManager::default());
        for (profile_name, startup_options) in [
            ("full-scene", SceneRendererStartupOptions::default()),
            (
                "environment-only",
                SceneRendererStartupOptions::environment_only_pbr_preview(),
            ),
        ] {
            let (mut renderer, _) = SceneRenderer::new_with_startup_options_and_report(
                ProjectAssetManagerAccess::for_test(Arc::clone(&asset_manager)),
                startup_options,
            )
            .unwrap_or_else(|error| panic!("{profile_name} renderer startup failed: {error}"));

            renderer
                .render(empty_lit_snapshot(), UVec2::new(8, 8))
                .unwrap_or_else(|error| {
                    panic!("{profile_name} deferred bind-group construction failed: {error}")
                });
        }
    }
}
