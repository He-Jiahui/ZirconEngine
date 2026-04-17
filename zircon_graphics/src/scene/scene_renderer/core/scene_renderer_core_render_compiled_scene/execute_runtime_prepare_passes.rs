use crate::scene::scene_renderer::{HybridGiGpuPendingReadback, VirtualGeometryGpuPendingReadback};
use crate::types::{EditorOrRuntimeFrame, GraphicsError};

use super::super::scene_renderer_core::SceneRendererCore;

impl SceneRendererCore {
    pub(super) fn execute_runtime_prepare_passes(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        frame: &EditorOrRuntimeFrame,
    ) -> Result<
        (
            Option<HybridGiGpuPendingReadback>,
            Option<VirtualGeometryGpuPendingReadback>,
        ),
        GraphicsError,
    > {
        let hybrid_gi_gpu_readback = self.hybrid_gi.execute_prepare(
            device,
            queue,
            encoder,
            frame.hybrid_gi_prepare.as_ref(),
            frame.extract.lighting.hybrid_global_illumination.as_ref(),
            &frame.extract.lighting.directional_lights,
            frame
                .extract
                .lighting
                .hybrid_global_illumination
                .as_ref()
                .map(|hybrid_gi| hybrid_gi.probe_budget),
            frame
                .extract
                .lighting
                .hybrid_global_illumination
                .as_ref()
                .map(|hybrid_gi| hybrid_gi.tracing_budget),
        )?;
        let virtual_geometry_gpu_readback = self.virtual_geometry.execute_prepare(
            device,
            queue,
            encoder,
            frame.virtual_geometry_prepare.as_ref(),
            frame
                .extract
                .geometry
                .virtual_geometry
                .as_ref()
                .map(|virtual_geometry| virtual_geometry.page_budget),
        )?;

        Ok((hybrid_gi_gpu_readback, virtual_geometry_gpu_readback))
    }
}
