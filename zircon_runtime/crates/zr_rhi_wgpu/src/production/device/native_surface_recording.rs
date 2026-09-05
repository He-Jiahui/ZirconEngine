use std::sync::Arc;

use zr_rhi::{
    RenderDevice, RhiError, SubmissionTicket, SurfaceFrameId, SurfaceFrameLease,
    SurfacePresentReceipt,
};

use super::WgpuRenderDevice;

/// One acquired neutral surface target retained until present or automatic discard.
#[must_use = "an acquired native surface target must join a submission or be discarded"]
pub struct WgpuNativeSurfaceFrameTarget {
    owner: Arc<WgpuRenderDevice>,
    frame: Option<SurfaceFrameLease>,
    target_view: wgpu::TextureView,
}

impl WgpuNativeSurfaceFrameTarget {
    pub fn frame(&self) -> SurfaceFrameId {
        self.frame
            .as_ref()
            .expect("live native surface target")
            .frame()
    }

    pub fn record<E>(
        &self,
        owner: &WgpuRenderDevice,
        encoder: &mut wgpu::CommandEncoder,
        record: impl FnOnce(
            &wgpu::Device,
            &wgpu::TextureView,
            &mut wgpu::CommandEncoder,
        ) -> Result<(), E>,
    ) -> Result<(), E>
    where
        E: From<RhiError>,
    {
        self.validate_owner(owner).map_err(E::from)?;
        record(&owner.device, &self.target_view, encoder)
    }

    pub(super) fn frame_lease(&self) -> &SurfaceFrameLease {
        self.frame.as_ref().expect("live native surface target")
    }

    pub(super) fn validate_owner(&self, owner: &WgpuRenderDevice) -> Result<(), RhiError> {
        if std::ptr::eq(Arc::as_ptr(&self.owner), owner) {
            return Ok(());
        }
        Err(RhiError::SubmissionPacketDeviceMismatch {
            packet_device_id: self.owner.device_id(),
            packet_generation: self.owner.generation(),
            device_id: owner.device_id(),
            generation: owner.generation(),
        })
    }

    pub fn present(
        &mut self,
        submission: SubmissionTicket,
    ) -> Result<SurfacePresentReceipt, RhiError> {
        let frame = self.frame_lease().clone();
        let receipt = self.owner.present_surface_frame(frame, submission)?;
        self.frame.take();
        Ok(receipt)
    }

    pub fn discard(mut self) -> Result<(), RhiError> {
        let frame = self.frame_lease().clone();
        self.owner.discard_surface_frame(frame)?;
        self.frame.take();
        Ok(())
    }
}

impl Drop for WgpuNativeSurfaceFrameTarget {
    fn drop(&mut self) {
        if let Some(frame) = self.frame.take() {
            let _ = self.owner.discard_surface_frame(frame);
        }
    }
}

impl WgpuRenderDevice {
    pub fn prepare_native_surface_frame_target(
        self: &Arc<Self>,
        frame: SurfaceFrameLease,
    ) -> Result<WgpuNativeSurfaceFrameTarget, RhiError> {
        let target_view = (|| {
            self.ensure_admission()?;
            let surfaces = self.lock_surfaces();
            let (target, default_view) = surfaces.validate_frame_lease(&frame)?;
            let registry = self.lock_registry();
            if registry.texture_desc(target)? != *frame.desc() {
                return Err(RhiError::SurfaceFrameLeaseMismatch {
                    frame: frame.frame(),
                });
            }
            let target_view: wgpu::TextureView = registry.texture_view(default_view)?.clone();
            Ok(target_view)
        })();
        let target_view = match target_view {
            Ok(target_view) => target_view,
            Err(error) => {
                let frame_id = frame.frame();
                return match self.discard_surface_frame(frame) {
                    Ok(()) => Err(error),
                    Err(cleanup) => Err(RhiError::SurfaceFrameCleanupFailed {
                        frame: frame_id,
                        cleanup: Box::new(cleanup),
                        source: Box::new(error),
                    }),
                };
            }
        };
        Ok(WgpuNativeSurfaceFrameTarget {
            owner: Arc::clone(self),
            frame: Some(frame),
            target_view,
        })
    }

    pub(super) fn register_native_surface_frame_use(
        &self,
        frame: &SurfaceFrameLease,
        ticket: SubmissionTicket,
    ) -> Result<(), RhiError> {
        let surfaces = self.lock_surfaces();
        let (target, default_view) = surfaces.validate_frame_lease(frame)?;
        let mut registry = self.lock_registry();
        if registry.texture_desc(target)? != *frame.desc() {
            return Err(RhiError::SurfaceFrameLeaseMismatch {
                frame: frame.frame(),
            });
        }
        registry.mark_native_surface_frame_use(frame.frame(), target, default_view, ticket)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn prepared_surface_target_discards_unpresented_leases() {
        let source = include_str!("native_surface_recording.rs");
        let source = source.split("mod tests {").next().unwrap();

        assert!(source.contains("pub struct WgpuNativeSurfaceFrameTarget"));
        assert!(source.contains("impl Drop for WgpuNativeSurfaceFrameTarget"));
        assert!(source.contains("std::ptr::eq(Arc::as_ptr(&self.owner), owner)"));
        assert!(source.contains("self.validate_owner(owner).map_err(E::from)?"));
        assert!(source.contains("record(&owner.device, &self.target_view, encoder)"));
        assert!(!source.contains("pub fn target_view"));
        assert!(source.contains("self.owner.discard_surface_frame(frame)"));
        assert!(source.contains("self.owner.present_surface_frame(frame, submission)?"));
        assert!(source.contains("self.owner.discard_surface_frame(frame)?"));
        assert!(source.contains("self.frame.take();"));
        assert!(source.contains("RhiError::SurfaceFrameCleanupFailed"));
    }
}
