use zr_rhi::{
    RenderDevice, RenderQueueClass, RhiError, SubmissionTicket, TextureDesc, TextureFormat,
    TextureHandle, TextureUsage,
};

use super::mvp_triangle_pipeline::MvpTrianglePipeline;
use super::WgpuRenderDevice;

/// Smallest reusable offscreen product frame expressed wholly through neutral RHI objects.
///
/// The output remains a generation-qualified texture handle. Presentation, readback, and resource
/// retirement therefore continue through the owning [`WgpuRenderDevice`] instead of exposing a
/// second native WGPU resource path.
pub struct WgpuMvpOffscreenTriangle {
    target: TextureHandle,
    depth_target: TextureHandle,
    pipeline: MvpTrianglePipeline,
}

impl WgpuMvpOffscreenTriangle {
    /// Allocates the persistent resources for one offscreen triangle frame owner.
    pub fn new(device: &WgpuRenderDevice, width: u32, height: u32) -> Result<Self, RhiError> {
        let target = device.create_texture(&TextureDesc::new(
            "zircon-mvp-offscreen-target",
            width,
            height,
            TextureFormat::Rgba8Unorm,
            TextureUsage::RENDER_ATTACHMENT | TextureUsage::COPY_SRC,
        ))?;
        let depth_target = match device.create_texture(&TextureDesc::new(
            "zircon-mvp-offscreen-depth",
            width,
            height,
            TextureFormat::Depth24Plus,
            TextureUsage::RENDER_ATTACHMENT,
        )) {
            Ok(handle) => handle,
            Err(error) => {
                let _ = device.destroy_texture(target);
                return Err(error);
            }
        };
        let pipeline = match MvpTrianglePipeline::new(device, TextureFormat::Rgba8Unorm) {
            Ok(pipeline) => pipeline,
            Err(error) => {
                let _ = device.destroy_texture(depth_target);
                let _ = device.destroy_texture(target);
                return Err(error);
            }
        };

        Ok(Self {
            target,
            depth_target,
            pipeline,
        })
    }

    /// Returns the offscreen output for neutral readback or later graph composition.
    pub const fn target(&self) -> TextureHandle {
        self.target
    }

    /// Returns the depth attachment allocated with the color target for this frame generation.
    pub const fn depth_target(&self) -> TextureHandle {
        self.depth_target
    }

    /// Records and submits one graphics command list through the device's sole submission owner.
    pub fn submit(&self, device: &WgpuRenderDevice) -> Result<SubmissionTicket, RhiError> {
        let mut command_list =
            device.create_command_list(RenderQueueClass::Graphics, "zircon-mvp-triangle-frame")?;
        self.pipeline.record_draw(
            &mut *command_list,
            MvpTrianglePipeline::color_attachment(self.target),
            self.depth_target,
        );
        device.submit(command_list)
    }

    /// Releases persistent frame resources in dependency order.
    pub fn destroy(self, device: &WgpuRenderDevice) -> Result<(), RhiError> {
        self.pipeline.destroy(device)?;
        device.destroy_texture(self.depth_target)?;
        device.destroy_texture(self.target)
    }
}
