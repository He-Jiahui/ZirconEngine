use crate::core::framework::render::MotionVectorCameraStatus;
use crate::graphics::scene::scene_renderer::graph_execution::{
    RenderGraphComputeDispatchRecord, RenderGraphLightGridReport,
};
use crate::graphics::visibility::HzbOcclusionCullReport;

use super::RenderPassGpuExecutionContext;

impl<'a> RenderPassGpuExecutionContext<'a> {
    pub(in crate::graphics::scene::scene_renderer) fn take_compute_dispatches(
        &mut self,
    ) -> Vec<RenderGraphComputeDispatchRecord> {
        std::mem::take(&mut self.compute_dispatches)
    }

    pub fn record_compute_dispatch(
        &mut self,
        pass_name: impl Into<String>,
        executor_id: impl Into<String>,
        pipeline_label: impl Into<String>,
        workgroup_size: [u32; 3],
        dispatch_groups: [u32; 3],
        storage_write_resources: Vec<String>,
    ) {
        self.compute_dispatches
            .push(RenderGraphComputeDispatchRecord::new(
                pass_name,
                executor_id,
                pipeline_label,
                workgroup_size,
                dispatch_groups,
                storage_write_resources,
            ));
    }

    pub(in crate::graphics::scene::scene_renderer) fn take_hzb_occlusion_cull_report(
        &mut self,
    ) -> Option<HzbOcclusionCullReport> {
        self.hzb_occlusion_cull_report.take()
    }

    pub(in crate::graphics::scene::scene_renderer) fn take_light_grid_report(
        &mut self,
    ) -> Option<RenderGraphLightGridReport> {
        self.light_grid_report.take()
    }

    pub(in crate::graphics::scene::scene_renderer) fn motion_vector_camera_status(
        &self,
    ) -> MotionVectorCameraStatus {
        self.motion_vector_camera_status
    }
}
