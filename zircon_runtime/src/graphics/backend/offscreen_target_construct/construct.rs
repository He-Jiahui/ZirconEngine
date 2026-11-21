use crate::core::framework::render::RenderViewportHandle;
use crate::core::math::UVec2;
use crate::graphics::types::ViewportRenderOutputTarget;

use crate::graphics::scene::{
    cluster_buffer_bytes_for_size, cluster_dimensions_for_size, create_depth_texture,
    FINAL_COLOR_FORMAT, GBUFFER_ALBEDO_FORMAT, GBUFFER_EMISSIVE_FORMAT, GBUFFER_MATERIAL_FORMAT,
    NORMAL_FORMAT, SCENE_COLOR_HDR_FORMAT,
};

use super::super::offscreen_target::OffscreenTarget;
use super::create_cluster_buffer::create_cluster_buffer;
use super::create_texture_bundle::create_texture_bundle;
use super::texture_bundle::TextureBundle;

impl OffscreenTarget {
    #[cfg(test)]
    pub(crate) fn new(device: &wgpu::Device, size: UVec2) -> Self {
        Self::new_with_render_size(device, size, size, None, None, None)
    }

    /// The retained final color may be moved across scene-only allocation changes, but never
    /// across display extents or viewport owners. `output == None` means the graph has a direct
    /// asset import and does not write the renderer-owned final attachment.
    pub(crate) fn ensure_allocations(
        device: &wgpu::Device,
        active: &mut Option<Self>,
        size: UVec2,
        render_size: UVec2,
        viewport: Option<RenderViewportHandle>,
        output: Option<ViewportRenderOutputTarget>,
    ) -> bool {
        let retain_final = active.as_ref().filter(|target| {
            target.size == size
                && target.final_color_viewport == viewport
                && (output.is_none()
                    || target.final_color_output.is_none()
                    || target.final_color_output == output)
        });
        let rebuild_scene = active
            .as_ref()
            .is_none_or(|target| target.size != size || target.render_size != render_size);
        if rebuild_scene {
            *active = Some(Self::new_with_render_size(
                device,
                size,
                render_size,
                viewport,
                output,
                retain_final,
            ));
        } else {
            let final_reused = retain_final.is_some();
            if let Some(target) = active.as_mut() {
                if !final_reused {
                    let final_color = create_final_color(device, size);
                    target.final_color = final_color.texture;
                    target.final_color_view = final_color.view;
                    target.final_color_viewport = viewport;
                    target.final_color_output = output;
                } else if output.is_some() {
                    target.final_color_output = output;
                }
            }
        }
        // Callers use this flag to invalidate bindings to scene-sized targets, not to final color.
        rebuild_scene
    }

    fn new_with_render_size(
        device: &wgpu::Device,
        size: UVec2,
        render_size: UVec2,
        viewport: Option<RenderViewportHandle>,
        output: Option<ViewportRenderOutputTarget>,
        retain_final: Option<&Self>,
    ) -> Self {
        let final_color = retain_final.map_or_else(
            || create_final_color(device, size),
            |target| TextureBundle {
                texture: target.final_color.clone(),
                view: target.final_color_view.clone(),
            },
        );
        let scene_color = create_texture_bundle(
            device,
            "zircon-offscreen-scene-color",
            render_size,
            SCENE_COLOR_HDR_FORMAT,
            wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_SRC,
        );
        let global_illumination = create_texture_bundle(
            device,
            "zircon-offscreen-global-illumination",
            render_size,
            SCENE_COLOR_HDR_FORMAT,
            wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_SRC,
        );
        let bloom = create_texture_bundle(
            device,
            "zircon-offscreen-bloom",
            render_size,
            SCENE_COLOR_HDR_FORMAT,
            wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_SRC,
        );
        let gbuffer_albedo = create_texture_bundle(
            device,
            "zircon-offscreen-gbuffer-albedo",
            render_size,
            GBUFFER_ALBEDO_FORMAT,
            wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        );
        let gbuffer_emissive = create_texture_bundle(
            device,
            "zircon-offscreen-gbuffer-emissive",
            render_size,
            GBUFFER_EMISSIVE_FORMAT,
            wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        );
        let sss_diagnostic_gbuffer_usage =
            wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING;
        #[cfg(test)]
        let sss_diagnostic_gbuffer_usage =
            sss_diagnostic_gbuffer_usage | wgpu::TextureUsages::COPY_SRC;
        let gbuffer_material = create_texture_bundle(
            device,
            "zircon-offscreen-gbuffer-material",
            render_size,
            GBUFFER_MATERIAL_FORMAT,
            sss_diagnostic_gbuffer_usage,
        );
        let normal = create_texture_bundle(
            device,
            "zircon-offscreen-normal",
            render_size,
            NORMAL_FORMAT,
            sss_diagnostic_gbuffer_usage,
        );
        let ambient_occlusion = create_texture_bundle(
            device,
            "zircon-offscreen-ambient-occlusion",
            render_size,
            wgpu::TextureFormat::Rgba8Unorm,
            wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::STORAGE_BINDING
                | wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::COPY_SRC,
        );
        let depth = create_depth_texture(device, render_size);
        let depth_view = depth.create_view(&wgpu::TextureViewDescriptor::default());
        let cluster_dimensions = cluster_dimensions_for_size(render_size);
        let cluster_buffer_bytes = cluster_buffer_bytes_for_size(render_size);
        let cluster_buffer = create_cluster_buffer(device, cluster_buffer_bytes);

        Self {
            size,
            render_size,
            final_color: final_color.texture,
            final_color_view: final_color.view,
            final_color_viewport: viewport,
            final_color_output: output
                .or_else(|| retain_final.and_then(|target| target.final_color_output)),
            global_illumination: global_illumination.texture,
            global_illumination_view: global_illumination.view,
            scene_color: scene_color.texture,
            scene_color_view: scene_color.view,
            scene_color_identity: crate::graphics::resource_identity::SampledTextureIdentity::new(),
            scene_velocity: None,
            bloom: bloom.texture,
            bloom_view: bloom.view,
            gbuffer_albedo: gbuffer_albedo.texture,
            gbuffer_albedo_view: gbuffer_albedo.view,
            gbuffer_emissive: gbuffer_emissive.texture,
            gbuffer_emissive_view: gbuffer_emissive.view,
            gbuffer_material: gbuffer_material.texture,
            gbuffer_material_view: gbuffer_material.view,
            normal: normal.texture,
            normal_view: normal.view,
            ambient_occlusion: ambient_occlusion.texture,
            ambient_occlusion_view: ambient_occlusion.view,
            depth,
            depth_view,
            depth_identity: crate::graphics::resource_identity::SampledTextureIdentity::new(),
            cluster_dimensions,
            cluster_buffer,
            cluster_buffer_bytes,
        }
    }
}

fn create_final_color(device: &wgpu::Device, size: UVec2) -> TextureBundle {
    create_texture_bundle(
        device,
        "zircon-offscreen-final-color",
        size,
        FINAL_COLOR_FORMAT,
        wgpu::TextureUsages::RENDER_ATTACHMENT
            | wgpu::TextureUsages::COPY_SRC
            | wgpu::TextureUsages::TEXTURE_BINDING,
    )
}
