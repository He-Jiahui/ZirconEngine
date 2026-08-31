use crate::core::framework::render::RenderBloomSettings;
use crate::core::math::UVec2;
use zr_rhi_wgpu::{WgpuBufferUpload, WgpuBufferUploadBatch};

use super::super::super::bloom_params::BloomParams;
use super::super::super::clear_render_target::clear_render_target;
use super::super::super::scene_post_process_resources::ScenePostProcessResources;

impl ScenePostProcessResources {
    pub(crate) fn execute_bloom(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        viewport_size: UVec2,
        scene_color_origin: [u32; 2],
        scene_color_view: &wgpu::TextureView,
        bloom_view: &wgpu::TextureView,
        settings: RenderBloomSettings,
        enabled: bool,
    ) -> WgpuBufferUploadBatch {
        if !enabled || settings.intensity <= f32::EPSILON {
            clear_render_target(encoder, "ClearBloomPass", bloom_view, wgpu::Color::BLACK);
            return WgpuBufferUploadBatch::new();
        }

        let params = BloomParams {
            viewport: [
                viewport_size.x.max(1),
                viewport_size.y.max(1),
                scene_color_origin[0],
                scene_color_origin[1],
            ],
            tuning: [
                settings.threshold.clamp(0.0, 4.0),
                settings.intensity.max(0.0),
                settings.radius.max(0.0),
                0.0,
            ],
        };
        let params_uploads = WgpuBufferUploadBatch::from(WgpuBufferUpload::from_bytes(
            self.bloom_params_buffer.clone(),
            0,
            bytemuck::bytes_of(&params),
        ));

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("zircon-bloom-bind-group"),
            layout: &self.bloom_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(scene_color_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: self.bloom_params_buffer.as_entire_binding(),
                },
            ],
        });

        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("BloomPass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: bloom_view,
                resolve_target: None,
                depth_slice: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
            multiview_mask: None,
        });
        pass.set_viewport(
            0.0,
            0.0,
            viewport_size.x.max(1) as f32,
            viewport_size.y.max(1) as f32,
            0.0,
            1.0,
        );
        pass.set_scissor_rect(0, 0, viewport_size.x.max(1), viewport_size.y.max(1));
        pass.set_pipeline(&self.bloom_pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.draw(0..3, 0..1);
        params_uploads
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn bloom_params_are_returned_as_pre_submit_uploads() {
        let source = include_str!("execute_bloom.rs");
        let production = source
            .split("#[cfg(test)]")
            .next()
            .expect("bloom production source");

        assert!(!production.contains("queue.write_buffer"));
        assert!(production.contains("WgpuBufferUpload::from_bytes("));
        assert!(production.contains("WgpuBufferUploadBatch"));
    }
}
