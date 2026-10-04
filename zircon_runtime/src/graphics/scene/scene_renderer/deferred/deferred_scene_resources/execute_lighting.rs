use crate::graphics::scene::scene_renderer::advanced_lighting::irradiance_volume::IrradianceVolumeResources;
use crate::graphics::scene::scene_renderer::advanced_lighting::light_cookie::LightCookieAtlasResources;
use crate::graphics::scene::scene_renderer::attachment_ops::color_attachment_operations;
use crate::graphics::scene::scene_renderer::shadow::atlas::{
    ShadowAtlasResources, SHADOW_ATLAS_BINDING, SHADOW_ATLAS_SAMPLER_BINDING,
    SHADOW_ATLAS_SLOT_BUFFER_BINDING, SHADOW_GLOBALS_BINDING,
};
use crate::graphics::scene::scene_renderer::SceneRendererDeferredLightingProfile;
use crate::graphics::types::ViewportRenderFrame;
use crate::graphics::types::ViewportRenderRegion;
use crate::render_graph::RenderGraphAttachmentOps;

use super::DeferredSceneResources;

// 13 base entries (including AO) + 5 probes + 3 lightmap + 3 volumetric + 2 cookies + 3 irradiance volume.
const DEFERRED_LIGHTING_BIND_GROUP_ENTRY_CAPACITY: usize = 29;
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
        ambient_occlusion_view: Option<&wgpu::TextureView>,
        gbuffer_material_view: &wgpu::TextureView,
        gbuffer_emissive_view: &wgpu::TextureView,
        scene_depth_view: &wgpu::TextureView,
        shadow_atlas_resources: Option<&ShadowAtlasResources>,
        light_grid_params_buffer: wgpu::BufferBinding<'_>,
        light_zbins_buffer: wgpu::BufferBinding<'_>,
        light_tile_masks_buffer: wgpu::BufferBinding<'_>,
        integrated_volumetric_view: Option<&wgpu::TextureView>,
        light_cookies: &LightCookieAtlasResources,
        irradiance_volume: &IrradianceVolumeResources,
        frame: &ViewportRenderFrame,
        scene_color_view: &wgpu::TextureView,
        subsurface_diffuse_view: Option<&wgpu::TextureView>,
        subsurface_retained_view: Option<&wgpu::TextureView>,
        attachment_ops: RenderGraphAttachmentOps,
        render_region: ViewportRenderRegion,
    ) -> Result<(), String> {
        // profile 同时决定绑定容量与 GPU-scene 组；使用完整绑定且两个 subsurface 目标都存在时才启用三路 MRT，
        // 资源绑定、颜色附件和缓存管线共用同一计划。
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
            let ambient_occlusion_view = ambient_occlusion_view.ok_or_else(|| {
                "full deferred lighting requires an AO texture or neutral fallback".to_string()
            })?;
            entries.push(wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::TextureView(ambient_occlusion_view),
            });
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
            let Some(volumetric_params_buffer) = volumetric_params_buffer.as_ref() else {
                return Err(
                    "deferred lighting execution plan requires volumetric parameters".to_string(),
                );
            };
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
                    resource: wgpu::BindingResource::Buffer(light_grid_params_buffer),
                },
                wgpu::BindGroupEntry {
                    binding: 21,
                    resource: wgpu::BindingResource::Buffer(light_zbins_buffer),
                },
                wgpu::BindGroupEntry {
                    binding: 22,
                    resource: wgpu::BindingResource::Buffer(light_tile_masks_buffer),
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
            let (Some(diffuse_view), Some(retained_view)) =
                (subsurface_diffuse_view, subsurface_retained_view)
            else {
                return Err(
                    "deferred lighting subsurface MRT requires diffuse and retained targets"
                        .to_string(),
                );
            };
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
            return Ok(());
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
        Ok(())
    }
}

#[cfg(test)]
#[path = "tests/execute_lighting.rs"]
mod tests;
