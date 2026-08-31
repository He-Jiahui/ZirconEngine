use crate::core::framework::render::{
    MotionVectorCameraStatus, RenderPassNativeResourceCreateMetrics,
};
use crate::graphics::scene::scene_renderer::graph_execution::{
    RenderGraphComputeDispatchRecord, RenderGraphLightGridReport,
};
use crate::graphics::visibility::HzbOcclusionCullReport;

use crate::graphics::FrameHistorySlot;
use crate::graphics::scene::scene_renderer::history::{
    SceneHistoryDomain, SceneHistoryWriteIntent,
};

use super::RenderPassGpuExecutionContext;

impl<'a> RenderPassGpuExecutionContext<'a> {
    pub fn record_frame_history_write(&mut self, slot: FrameHistorySlot) {
        self.record_history_write(scene_history_domain_for_frame_slot(slot));
    }

    pub(in crate::graphics::scene::scene_renderer) fn record_history_write(
        &mut self,
        domain: SceneHistoryDomain,
    ) {
        self.history_writes.record(domain, true);
    }

    pub(in crate::graphics::scene::scene_renderer) fn take_history_writes(
        &mut self,
    ) -> SceneHistoryWriteIntent {
        std::mem::take(&mut self.history_writes)
    }

    pub(in crate::graphics::scene::scene_renderer) fn take_compute_dispatches(
        &mut self,
    ) -> Vec<RenderGraphComputeDispatchRecord> {
        std::mem::take(&mut self.compute_dispatches)
    }

    pub(in crate::graphics::scene::scene_renderer) fn take_native_resource_creates(
        &self,
    ) -> RenderPassNativeResourceCreateMetrics {
        self.native_resource_creates
            .replace(RenderPassNativeResourceCreateMetrics::default())
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
        self.record_compute_dispatch_with_uploaded_bytes(
            pass_name,
            executor_id,
            pipeline_label,
            workgroup_size,
            dispatch_groups,
            0,
            storage_write_resources,
        );
    }

    pub fn record_indirect_compute_dispatch(
        &mut self,
        pass_name: impl Into<String>,
        executor_id: impl Into<String>,
        pipeline_label: impl Into<String>,
        workgroup_size: [u32; 3],
        storage_write_resources: Vec<String>,
    ) {
        self.compute_dispatches.push(
            RenderGraphComputeDispatchRecord::new(
                pass_name,
                executor_id,
                pipeline_label,
                workgroup_size,
                [0, 1, 1],
                storage_write_resources,
            )
            .with_gpu_indirect_dispatch_groups(),
        );
    }

    #[allow(clippy::too_many_arguments)]
    pub fn record_compute_dispatch_with_uploaded_bytes(
        &mut self,
        pass_name: impl Into<String>,
        executor_id: impl Into<String>,
        pipeline_label: impl Into<String>,
        workgroup_size: [u32; 3],
        dispatch_groups: [u32; 3],
        uploaded_bytes: u64,
        storage_write_resources: Vec<String>,
    ) {
        self.compute_dispatches.push(
            RenderGraphComputeDispatchRecord::new(
                pass_name,
                executor_id,
                pipeline_label,
                workgroup_size,
                dispatch_groups,
                storage_write_resources,
            )
            .with_uploaded_bytes(uploaded_bytes),
        );
    }

    pub(in crate::graphics::scene::scene_renderer) fn push_compute_dispatch_record(
        &mut self,
        dispatch: RenderGraphComputeDispatchRecord,
    ) {
        self.compute_dispatches.push(dispatch);
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

    pub(in crate::graphics::scene::scene_renderer) fn record_taa_reactive_mask_encoding(
        &mut self,
        size: crate::core::math::UVec2,
    ) {
        self.taa_reactive_mask_encoded_pass_count =
            self.taa_reactive_mask_encoded_pass_count.saturating_add(1);
        self.taa_reactive_mask_encoded_write_bytes = self
            .taa_reactive_mask_encoded_write_bytes
            .saturating_add(u64::from(size.x) * u64::from(size.y));
    }

    pub(in crate::graphics::scene::scene_renderer) fn taa_reactive_mask_encoding(
        &self,
    ) -> (usize, u64) {
        (
            self.taa_reactive_mask_encoded_pass_count,
            self.taa_reactive_mask_encoded_write_bytes,
        )
    }

    pub(in crate::graphics::scene::scene_renderer) fn record_taa_resolve_bind_group_create(
        &mut self,
    ) {
        self.taa_resolve_bind_group_create_count =
            self.taa_resolve_bind_group_create_count.saturating_add(1);
    }

    pub(in crate::graphics::scene::scene_renderer) fn taa_resolve_bind_group_create_count(
        &self,
    ) -> usize {
        self.taa_resolve_bind_group_create_count
    }

    pub(in crate::graphics::scene::scene_renderer) fn motion_vector_camera_status(
        &self,
    ) -> MotionVectorCameraStatus {
        self.motion_vector_camera_status
    }
}

const fn scene_history_domain_for_frame_slot(slot: FrameHistorySlot) -> SceneHistoryDomain {
    match slot {
        FrameHistorySlot::AmbientOcclusion => SceneHistoryDomain::AmbientOcclusion,
        FrameHistorySlot::GlobalIllumination => SceneHistoryDomain::HybridGlobalIllumination,
        FrameHistorySlot::HzbFurthest => SceneHistoryDomain::HzbFurthest,
        FrameHistorySlot::TaaSceneColor => SceneHistoryDomain::TaaSceneColor,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn public_frame_history_slots_map_to_scene_history_domains() {
        assert_eq!(
            scene_history_domain_for_frame_slot(FrameHistorySlot::AmbientOcclusion),
            SceneHistoryDomain::AmbientOcclusion
        );
        assert_eq!(
            scene_history_domain_for_frame_slot(FrameHistorySlot::GlobalIllumination),
            SceneHistoryDomain::HybridGlobalIllumination
        );
        assert_eq!(
            scene_history_domain_for_frame_slot(FrameHistorySlot::HzbFurthest),
            SceneHistoryDomain::HzbFurthest
        );
        assert_eq!(
            scene_history_domain_for_frame_slot(FrameHistorySlot::TaaSceneColor),
            SceneHistoryDomain::TaaSceneColor
        );
    }
}
