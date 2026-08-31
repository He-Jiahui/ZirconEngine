use zr_rhi::{
    DeviceGeneration, DeviceId, RenderDevice, RenderPassTextureViewDesc, RenderQueueClass,
    RhiError, SubmissionTicket, SurfaceFrameLease, SurfacePresentReceipt, SwapchainDesc,
    TextureDesc, TextureHandle, TextureUsage,
};

use super::mvp_triangle_pipeline::MvpTrianglePipeline;
use super::WgpuRenderDevice;

/// Minimal direct-to-surface triangle frame owner expressed entirely through neutral RHI.
///
/// It consumes an acquired surface lease as its color attachment, submits exactly one packet, and
/// presents with the packet's ticket. It does not create an offscreen color target, native surface
/// view, command encoder, or additional queue submission.
pub struct WgpuMvpSurfaceTriangle {
    device_id: DeviceId,
    generation: DeviceGeneration,
    swapchain: SwapchainDesc,
    depth_target: TextureHandle,
    pipeline: MvpTrianglePipeline,
}

impl WgpuMvpSurfaceTriangle {
    pub fn new(device: &WgpuRenderDevice, swapchain: &SwapchainDesc) -> Result<Self, RhiError> {
        Self::new_for_device(device, swapchain)
    }

    pub(crate) fn new_for_device(
        device: &dyn RenderDevice,
        swapchain: &SwapchainDesc,
    ) -> Result<Self, RhiError> {
        if swapchain.width == 0 || swapchain.height == 0 {
            return Err(RhiError::InvalidSurfaceDescriptor {
                label: Some("zircon-mvp-surface-triangle".to_string()),
                reason: "direct surface triangle requires a renderable swapchain extent"
                    .to_string(),
            });
        }
        let depth_target = device.create_texture(&TextureDesc::new(
            "zircon-mvp-surface-depth",
            swapchain.width,
            swapchain.height,
            zr_rhi::TextureFormat::Depth24Plus,
            TextureUsage::RENDER_ATTACHMENT,
        ))?;
        let pipeline = match MvpTrianglePipeline::new(device, swapchain.format) {
            Ok(pipeline) => pipeline,
            Err(error) => {
                let _ = device.destroy_texture(depth_target);
                return Err(error);
            }
        };

        Ok(Self {
            device_id: device.device_id(),
            generation: device.generation(),
            swapchain: swapchain.clone(),
            depth_target,
            pipeline,
        })
    }

    pub const fn depth_target(&self) -> TextureHandle {
        self.depth_target
    }

    /// Records one packet that references the acquired target, then presents it through the same
    /// device generation. A failed packet or present attempt discards the lease exactly once.
    pub fn render_and_present(
        &self,
        device: &WgpuRenderDevice,
        frame: SurfaceFrameLease,
    ) -> Result<SurfacePresentReceipt, RhiError> {
        self.render_and_present_for_device(device, frame)
    }

    pub(crate) fn render_and_present_for_device(
        &self,
        device: &dyn RenderDevice,
        frame: SurfaceFrameLease,
    ) -> Result<SurfacePresentReceipt, RhiError> {
        let result = self
            .submit_surface_frame(device, &frame)
            .and_then(|ticket| device.present_surface_frame(frame.clone(), ticket));
        match result {
            Ok(receipt) => Ok(receipt),
            Err(error) => {
                let _ = device.discard_surface_frame(frame);
                Err(error)
            }
        }
    }

    pub fn destroy(self, device: &WgpuRenderDevice) -> Result<(), RhiError> {
        self.destroy_for_device(device)
    }

    pub(crate) fn destroy_for_device(self, device: &dyn RenderDevice) -> Result<(), RhiError> {
        self.pipeline.destroy(device)?;
        device.destroy_texture(self.depth_target)
    }

    fn submit_surface_frame(
        &self,
        device: &dyn RenderDevice,
        frame: &SurfaceFrameLease,
    ) -> Result<SubmissionTicket, RhiError> {
        self.validate_device(device)?;
        self.validate_frame(frame)?;
        let mut command_list = device.create_command_list(
            RenderQueueClass::Graphics,
            "zircon-mvp-surface-triangle-frame",
        )?;
        let color_attachment = MvpTrianglePipeline::color_attachment(frame.target()).with_view(
            RenderPassTextureViewDesc::new(frame.target())
                .with_registered_view(frame.default_view()),
        );
        self.pipeline
            .record_draw(&mut *command_list, color_attachment, self.depth_target);
        device.submit(command_list)
    }

    fn validate_frame(&self, frame: &SurfaceFrameLease) -> Result<(), RhiError> {
        if self.device_id != frame.frame().device_id()
            || self.generation != frame.frame().generation()
        {
            return Err(RhiError::SurfaceUnavailable(
                "acquired surface lease belongs to another WGPU device generation".to_string(),
            ));
        }
        let desc = frame.desc();
        if desc.width != self.swapchain.width
            || desc.height != self.swapchain.height
            || desc.format != self.swapchain.format
        {
            return Err(RhiError::InvalidSurfaceDescriptor {
                label: Some("zircon-mvp-surface-triangle".to_string()),
                reason: "acquired surface lease does not match the triangle swapchain receipt"
                    .to_string(),
            });
        }
        Ok(())
    }

    fn validate_device(&self, device: &dyn RenderDevice) -> Result<(), RhiError> {
        if self.device_id != device.device_id() || self.generation != device.generation() {
            return Err(RhiError::SurfaceUnavailable(
                "direct surface triangle resources belong to another WGPU device generation"
                    .to_string(),
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn direct_surface_triangle_has_no_native_queue_or_offscreen_color_owner() {
        let source = include_str!("mvp_surface_triangle.rs");

        assert!(source.contains("frame.target()"));
        assert!(source.contains("frame.default_view()"));
        assert!(source.contains("frame.frame().device_id()"));
        assert!(source.contains("device.device_id()"));
        assert!(source.contains("device.submit(command_list)"));
        assert!(source.contains("device.present_surface_frame(frame.clone(), ticket)"));
        assert!(source.contains("device.discard_surface_frame(frame)"));
        assert!(!source.contains(concat!("wgpu", "::")));
        assert!(!source.contains(concat!("TextureUsage", "::COPY_SRC")));
    }
}
