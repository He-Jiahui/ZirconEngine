use crate::core::framework::render::RenderVirtualGeometryReadbackOutputs;
use crate::graphics::VirtualGeometryGpuCompletion;

use super::super::scene_renderer::SceneRenderer;

impl SceneRenderer {
    #[cfg(feature = "test-support")]
    pub(in crate::graphics) fn read_virtual_geometry_native_probe(
        &self,
        handle: &crate::graphics::scene::VirtualGeometryReadyReadback,
        kind: u32,
    ) -> Result<Vec<u8>, String> {
        handle.read_native_probe(&self.backend, kind)
    }
    pub(crate) fn virtual_geometry_device_epoch(
        &self,
    ) -> crate::graphics::scene::RenderPassDeviceEpoch {
        crate::graphics::scene::RenderPassDeviceEpoch::from_profile(self.backend.device_profile())
    }

    #[cfg(feature = "test-support")]
    pub(crate) fn finish_virtual_geometry_native_readbacks_for_test(
        &mut self,
    ) -> Result<(), crate::graphics::types::GraphicsError> {
        self.poll_frame_submission_completions()?;
        Ok(())
    }

    #[cfg(feature = "test-support")]
    pub(crate) fn record_completed_virtual_geometry_native_readback_for_test(
        &mut self,
        outputs: &RenderVirtualGeometryReadbackOutputs,
        inspection: Option<
            &crate::core::framework::render::RenderVirtualGeometryNativeReadbackInspection,
        >,
    ) {
        self.last_virtual_geometry_readback_for_test = Some((outputs.clone(), inspection.cloned()));
        self.last_virtual_geometry_completion_for_test =
            VirtualGeometryGpuCompletion::from_readback_outputs(outputs.clone());
    }

    pub(crate) fn take_ticketed_virtual_geometry_readbacks(
        &mut self,
    ) -> Vec<crate::graphics::scene::VirtualGeometryTicketedReadback> {
        std::mem::take(&mut self.virtual_geometry_native_handoff)
    }

    pub(crate) fn observe_virtual_geometry_readback(
        &self,
        packet: &mut crate::graphics::scene::VirtualGeometryTicketedReadback,
    ) {
        packet.observe_status(self.backend.submission_status(packet.ticket).ok());
    }

    pub(crate) fn take_last_virtual_geometry_readback_outputs(
        &mut self,
    ) -> RenderVirtualGeometryReadbackOutputs {
        self.advanced_plugin_outputs
            .take_virtual_geometry_readback_outputs()
    }

    #[cfg(feature = "test-support")]
    pub fn test_support_last_virtual_geometry_readback_outputs(
        &self,
    ) -> Option<RenderVirtualGeometryReadbackOutputs> {
        self.last_virtual_geometry_readback_for_test
            .as_ref()
            .map(|(outputs, _)| outputs.clone())
    }

    #[cfg(feature = "test-support")]
    pub fn test_support_last_virtual_geometry_native_inspection(
        &self,
    ) -> Option<crate::core::framework::render::RenderVirtualGeometryNativeReadbackInspection> {
        self.last_virtual_geometry_readback_for_test
            .as_ref()
            .and_then(|(_, inspection)| inspection.clone())
    }

    #[cfg(feature = "test-support")]
    pub fn test_support_last_virtual_geometry_gpu_completion(
        &self,
    ) -> Option<VirtualGeometryGpuCompletion> {
        self.last_virtual_geometry_completion_for_test.clone()
    }
}
