use crate::graphics::pipeline::RenderPassStage;
use crate::graphics::scene::scene_renderer::attachment_ops::depth_attachment_operations;
use crate::render_graph::{RenderGraphAttachmentOps, RenderGraphResourceAccessKind};

use super::RenderPassGpuExecutionContext;

pub(super) fn record_depth_clear_pass(
    encoder: &mut wgpu::CommandEncoder,
    pass_name: &str,
    depth_view: &wgpu::TextureView,
    attachment_ops: RenderGraphAttachmentOps,
) {
    let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some(pass_name),
        color_attachments: &[],
        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
            view: depth_view,
            depth_ops: Some(depth_attachment_operations(attachment_ops, 1.0)),
            stencil_ops: None,
        }),
        timestamp_writes: None,
        occlusion_query_set: None,
        multiview_mask: None,
    });
}

impl<'a> RenderPassGpuExecutionContext<'a> {
    pub(in crate::graphics::scene::scene_renderer) fn record_surface_present(
        &mut self,
        source_resource_name: &str,
    ) -> Result<(), String> {
        let source_view = Self::require_texture_view_by_name(
            self.resources,
            self.resource_resolver,
            source_resource_name,
            RenderGraphResourceAccessKind::Read,
        )?;
        let Some((surface, surface_target)) = self.surface_frame else {
            let error = crate::graphics::types::GraphicsError::SurfaceStatus(
                "surface-present graph pass requires an acquired surface frame target",
            );
            let reason = error.to_string();
            self.surface_present_error = Some(error);
            return Err(reason);
        };
        match surface.record_frame_target_blit(self.encoder, source_view, surface_target) {
            Ok(()) => Ok(()),
            Err(error) => {
                let reason = error.to_string();
                self.surface_present_error = Some(error);
                Err(reason)
            }
        }
    }

    pub(in crate::graphics::scene::scene_renderer) fn record_transmission_scene_color_copy(
        &mut self,
        source_resource_name: &str,
        destination_resource_name: &str,
    ) -> Result<(), String> {
        let source_region = self.render_region_for_write_resource(source_resource_name);
        let source_origin = source_region.physical_position();
        let copy_size = source_region.local_size();
        let resources = &*self.resources;
        let resolver = self.resource_resolver;
        let source_desc = Self::require_texture_desc_by_name(
            resources,
            resolver,
            source_resource_name,
            RenderGraphResourceAccessKind::Read,
        )?;
        let destination_desc = Self::require_texture_desc_by_name(
            resources,
            resolver,
            destination_resource_name,
            RenderGraphResourceAccessKind::Write,
        )?;
        if source_desc.depth != destination_desc.depth
            || source_desc.format != destination_desc.format
        {
            return Err(format!(
                "transmission scene copy texture mismatch: source={source_desc:?}, destination={destination_desc:?}"
            ));
        }
        if source_desc.sample_count != 1 {
            return Err(format!(
                "transmission scene copy source must be single-sampled, got {} samples",
                source_desc.sample_count
            ));
        }
        if destination_desc.sample_count != 1 {
            return Err(format!(
                "transmission scene copy destination must be single-sampled, got {} samples",
                destination_desc.sample_count
            ));
        }
        if destination_desc.width != copy_size.x || destination_desc.height != copy_size.y {
            return Err(format!(
                "transmission scene copy destination extent must match local render size {}x{}, got {}x{}",
                copy_size.x, copy_size.y, destination_desc.width, destination_desc.height
            ));
        }
        if source_origin.x.saturating_add(copy_size.x) > source_desc.width
            || source_origin.y.saturating_add(copy_size.y) > source_desc.height
        {
            return Err(format!(
                "transmission scene copy source region origin=({}, {}) extent={}x{} exceeds {}x{}",
                source_origin.x,
                source_origin.y,
                copy_size.x,
                copy_size.y,
                source_desc.width,
                source_desc.height
            ));
        }

        let source = Self::require_physical_texture_by_name(
            resources,
            resolver,
            source_resource_name,
            RenderGraphResourceAccessKind::Read,
        )?;
        let destination = Self::require_owned_texture_by_name(
            resources,
            resolver,
            destination_resource_name,
            RenderGraphResourceAccessKind::Write,
        )?;
        self.encoder.copy_texture_to_texture(
            wgpu::TexelCopyTextureInfo {
                texture: source,
                mip_level: 0,
                origin: wgpu::Origin3d {
                    x: source_origin.x,
                    y: source_origin.y,
                    z: 0,
                },
                aspect: wgpu::TextureAspect::All,
            },
            destination.as_image_copy(),
            wgpu::Extent3d {
                width: copy_size.x,
                height: copy_size.y,
                depth_or_array_layers: destination_desc.depth,
            },
        );
        Ok(())
    }

    pub(in crate::graphics::scene::scene_renderer) fn record_sprite_stage_to_resources(
        &mut self,
        color_resource_name: &str,
        depth_resource_name: &str,
        stage: RenderPassStage,
        attachment_ops: RenderGraphAttachmentOps,
        depth_attachment_ops: RenderGraphAttachmentOps,
    ) -> Result<(), String> {
        let resources = &*self.resources;
        let resource_resolver = self.resource_resolver;
        let color_view = Self::require_texture_view_by_name(
            resources,
            resource_resolver,
            color_resource_name,
            RenderGraphResourceAccessKind::Write,
        )?;
        let depth_view = Self::require_texture_view_by_name(
            resources,
            resource_resolver,
            depth_resource_name,
            RenderGraphResourceAccessKind::Read,
        )?;
        let render_region = self.render_region_for_write_resource(color_resource_name);
        let sprite_renderer = self.sprite_renderer.ok_or_else(|| {
            format!("sprite graph executor for stage `{stage:?}` requires sprite renderer context")
        })?;
        let streamer = self.streamer.ok_or_else(|| {
            format!(
                "sprite graph executor for stage `{stage:?}` requires resource streamer context"
            )
        })?;
        sprite_renderer.record(
            self.device,
            self.encoder,
            color_view,
            depth_view,
            self.scene_bind_group,
            streamer,
            self.frame,
            stage,
            render_region,
            attachment_ops,
            depth_attachment_ops,
        );
        Ok(())
    }

    pub(in crate::graphics::scene::scene_renderer) fn record_screen_space_ui_to_resource(
        &mut self,
        resource_name: &str,
        attachment_ops: RenderGraphAttachmentOps,
    ) -> Result<(), String> {
        let resources = &*self.resources;
        let color_view = Self::require_texture_view_by_name(
            resources,
            self.resource_resolver,
            resource_name,
            RenderGraphResourceAccessKind::Write,
        )?;
        let mut prepared_upload = self
            .screen_space_ui_renderer
            .as_deref_mut()
            .ok_or_else(|| {
                format!(
                    "screen-space UI graph executor for resource `{resource_name}` requires UI renderer context"
                )
            })?
            .record(
                self.device,
                self.encoder,
                color_view,
                self.frame,
                attachment_ops,
                self.streamer,
            )
            .map_err(|error| error.to_string())?;
        let appended = self
            .screen_space_ui_renderer
            .as_deref()
            .is_some_and(|renderer| {
                prepared_upload.append_to(
                    renderer,
                    &mut self.buffer_uploads,
                    &mut self.texture_uploads,
                )
            });
        if !appended {
            return Err(format!(
                "screen-space UI graph executor for resource `{resource_name}` could not attach its resource upload transaction"
            ));
        }
        self.push_screen_space_ui_upload_commit(prepared_upload);
        Ok(())
    }

    pub(in crate::graphics::scene::scene_renderer) fn record_preview_sky_to_resources(
        &mut self,
        pass_name: &str,
        color_resource_name: &str,
        depth_resource_name: &str,
        color_attachment_ops: RenderGraphAttachmentOps,
        depth_attachment_ops: RenderGraphAttachmentOps,
    ) -> Result<(), String> {
        let resources = &*self.resources;
        let resource_resolver = self.resource_resolver;
        let color_view = Self::require_texture_view_by_name(
            resources,
            resource_resolver,
            color_resource_name,
            RenderGraphResourceAccessKind::Write,
        )?;
        let depth_view = Self::require_texture_view_by_name(
            resources,
            resource_resolver,
            depth_resource_name,
            RenderGraphResourceAccessKind::Read,
        )?;
        let integrated_volumetric_view = Self::optional_texture_view_by_name(
            resources,
            resource_resolver,
            crate::core::framework::render::PostProcessGraphResourceNames::VOLUMETRIC_INTEGRATED,
            RenderGraphResourceAccessKind::Read,
        )?;
        let render_region = self.render_region_for_write_resource(color_resource_name);
        let overlay_renderer = self.overlay_renderer.as_deref_mut().ok_or_else(|| {
            format!(
                "preview sky graph executor for pass `{pass_name}` requires preview sky renderer context"
            )
        })?;
        overlay_renderer.record_preview_sky_with_attachment_ops(
            self.encoder,
            self.device,
            color_view,
            depth_view,
            self.scene_bind_group,
            self.frame,
            render_region,
            color_attachment_ops,
            depth_attachment_ops,
            integrated_volumetric_view,
        );
        Ok(())
    }

    pub(in crate::graphics::scene::scene_renderer) fn record_overlay_to_resources(
        &mut self,
        pass_name: &str,
        color_resource_name: &str,
        depth_resource_name: &str,
    ) -> Result<(), String> {
        let resources = &*self.resources;
        let resource_resolver = self.resource_resolver;
        let color_view = Self::require_texture_view_by_name(
            resources,
            resource_resolver,
            color_resource_name,
            RenderGraphResourceAccessKind::Write,
        )?;
        let depth_view = Self::require_texture_view_by_name(
            resources,
            resource_resolver,
            depth_resource_name,
            RenderGraphResourceAccessKind::Read,
        )?;
        let render_region = self.render_region_for_write_resource(color_resource_name);
        let overlay_renderer = self.overlay_renderer.as_deref_mut().ok_or_else(|| {
            format!(
                "overlay graph executor for pass `{pass_name}` requires overlay renderer context"
            )
        })?;
        let prepared_overlays = self.prepared_overlays.ok_or_else(|| {
            format!(
                "overlay graph executor for pass `{pass_name}` requires prepared overlay buffers"
            )
        })?;
        overlay_renderer.record_overlays(
            self.encoder,
            color_view,
            depth_view,
            self.scene_bind_group,
            self.frame,
            prepared_overlays,
            render_region,
        );
        Ok(())
    }
}
