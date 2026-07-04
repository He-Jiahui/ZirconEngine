use crate::core::math::Vec3;
use crate::graphics::types::ViewportRenderRegion;
use crate::render_graph::{RenderGraphAttachmentOps, RenderGraphResourceAccessKind};

use super::RenderPassGpuExecutionContext;

pub struct ParticleGpuTransparentDrawContext<'a, 'b> {
    pub device: &'a wgpu::Device,
    pub queue: &'a wgpu::Queue,
    pub encoder: &'b mut wgpu::CommandEncoder,
    pub color_view: &'a wgpu::TextureView,
    pub depth_view: &'a wgpu::TextureView,
    pub scene_bind_group: &'a wgpu::BindGroup,
    pub scene_bind_group_layout: &'a wgpu::BindGroupLayout,
    pub target_format: wgpu::TextureFormat,
    pub depth_format: wgpu::TextureFormat,
    pub render_region: ViewportRenderRegion,
    pub camera_right: Vec3,
    pub camera_up: Vec3,
}

impl RenderPassGpuExecutionContext<'_> {
    pub fn record_particle_billboards_to_resources(
        &mut self,
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
        let particle_renderer = self.particle_renderer.ok_or_else(|| {
            format!(
                "particle graph executor requires particle renderer context for resources `{color_resource_name}` and `{depth_resource_name}`"
            )
        })?;
        particle_renderer.record(
            self.device,
            self.encoder,
            color_view,
            depth_view,
            self.scene_bind_group,
            self.frame,
            render_region,
        );
        Ok(())
    }

    pub fn record_particle_gpu_transparent_to_resources(
        &mut self,
        color_resource_name: &str,
        depth_resource_name: &str,
        mut record_gpu_draw: impl FnMut(
            ParticleGpuTransparentDrawContext<'_, '_>,
        ) -> Result<bool, String>,
    ) -> Result<bool, String> {
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
        let (camera_right, camera_up) = {
            let camera = &self.frame_extract().view.camera;
            (camera.transform.right(), camera.transform.up())
        };
        let render_region = self.render_region_for_write_resource(color_resource_name);
        record_gpu_draw(ParticleGpuTransparentDrawContext {
            device: self.device,
            queue: self.queue,
            encoder: self.encoder,
            color_view,
            depth_view,
            scene_bind_group: self.scene_bind_group,
            scene_bind_group_layout: self.scene_bind_group_layout,
            target_format: self.target_format,
            depth_format: self.depth_format,
            render_region,
            camera_right,
            camera_up,
        })
    }

    pub(in crate::graphics::scene::scene_renderer) fn record_particle_velocity_to_resource(
        &mut self,
        pass_name: &str,
        velocity_resource_name: &str,
        depth_resource_name: &str,
        attachment_ops: RenderGraphAttachmentOps,
    ) -> Result<(), String> {
        let resources = &*self.resources;
        let resource_resolver = self.resource_resolver;
        let velocity_view = Self::require_texture_view_by_name(
            resources,
            resource_resolver,
            velocity_resource_name,
            RenderGraphResourceAccessKind::Write,
        )?;
        let depth_view = Self::require_texture_view_by_name(
            resources,
            resource_resolver,
            depth_resource_name,
            RenderGraphResourceAccessKind::Read,
        )?;
        let render_region = self.render_region_for_write_resource(velocity_resource_name);
        let particle_renderer = self.particle_renderer.ok_or_else(|| {
            format!(
                "particle velocity graph executor for pass `{pass_name}` requires particle renderer context for resources `{velocity_resource_name}` and `{depth_resource_name}`"
            )
        })?;
        particle_renderer.record_velocity(
            self.device,
            self.encoder,
            pass_name,
            velocity_view,
            depth_view,
            self.scene_bind_group,
            self.frame,
            render_region,
            attachment_ops,
        );
        Ok(())
    }
}
