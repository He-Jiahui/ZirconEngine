use zr_rhi::{RenderDevice, RenderQueueClass, RhiError, SubmissionTicket};

use super::{WgpuUiExternalImage, WgpuUiSurfaceContext};

/// A generation-stable UI image and the submission that initialized its GPU contents.
pub struct WgpuUiExternalImageCopyReceipt {
    image: WgpuUiExternalImage,
    submission: SubmissionTicket,
}

/// Destination allocated before scene recording and initialized inside the scene packet.
pub struct WgpuUiExternalImageCopyTarget {
    image: WgpuUiExternalImage,
    device_id: zr_rhi::DeviceId,
    device_generation: zr_rhi::DeviceGeneration,
}

impl WgpuUiExternalImageCopyTarget {
    pub fn encode_copy(&self, encoder: &mut wgpu::CommandEncoder, source: &wgpu::Texture) {
        encoder.copy_texture_to_texture(
            source.as_image_copy(),
            self.image.texture.as_image_copy(),
            wgpu::Extent3d {
                width: self.image.width.max(1),
                height: self.image.height.max(1),
                depth_or_array_layers: 1,
            },
        );
    }

    pub fn complete(
        self,
        submission: SubmissionTicket,
    ) -> Result<WgpuUiExternalImageCopyReceipt, RhiError> {
        if submission.device_id() != self.device_id
            || submission.generation() != self.device_generation
        {
            return Err(RhiError::UnknownSubmissionTicket(submission));
        }
        Ok(WgpuUiExternalImageCopyReceipt {
            image: self.image,
            submission,
        })
    }
}

impl WgpuUiExternalImageCopyReceipt {
    pub const fn submission(&self) -> SubmissionTicket {
        self.submission
    }

    pub fn into_image(self) -> WgpuUiExternalImage {
        self.image
    }

    pub const fn width(&self) -> u32 {
        self.image.width
    }

    pub const fn height(&self) -> u32 {
        self.image.height
    }

    pub const fn generation(&self) -> u64 {
        self.image.generation
    }
}

impl WgpuUiSurfaceContext {
    pub fn prepare_texture_for_external_image(
        &self,
        width: u32,
        height: u32,
        format: wgpu::TextureFormat,
        generation: u64,
    ) -> Result<WgpuUiExternalImageCopyTarget, RhiError> {
        let render_device = self.external_image_render_device()?;
        let width = width.max(1);
        let height = height.max(1);
        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("zircon-ui-external-product"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        Ok(WgpuUiExternalImageCopyTarget {
            image: WgpuUiExternalImage::new_opaque(texture, width, height, generation),
            device_id: render_device.device_id(),
            device_generation: render_device.generation(),
        })
    }

    /// Copies a renderer-owned output into one generation-stable UI texture.
    ///
    /// External product publication requires the shared render-device owner. The copy receives a
    /// real ticket from that owner's fault, admission, history, flush, and completion timeline; a
    /// raw context cannot silently create an untracked product submission.
    pub fn copy_texture_for_external_image(
        &self,
        source: &wgpu::Texture,
        width: u32,
        height: u32,
        format: wgpu::TextureFormat,
        generation: u64,
    ) -> Result<WgpuUiExternalImageCopyReceipt, RhiError> {
        let render_device = self.external_image_render_device()?;
        let target = self.prepare_texture_for_external_image(width, height, format, generation)?;
        let mut recording = render_device.begin_native_recording(RenderQueueClass::Graphics)?;
        recording.record_command_buffer(
            "zircon-ui-external-product-copy",
            |_device, encoder| {
                target.encode_copy(encoder, source);
                Ok::<(), RhiError>(())
            },
        )?;
        let packet = recording.finish()?;
        let submission = render_device.submit_native_recording_packet(packet)?;
        target.complete(submission)
    }
}
