use crate::core::framework::render::{
    CubemapFace, RenderEnvironmentCaptureRequest, RENDER_ENVIRONMENT_CAPTURE_WORK_ITEM_COUNT,
};

use super::environment_capture_gpu_target::EnvironmentCaptureGpuTargetPlan;

const CUBEMAP_FACE_COUNT: usize = 6;

/// CPU-side contract consumed by the environment-capture recorder.
///
/// The plan deliberately contains no WGPU handles. Resource creation and command recording stay
/// in the renderer owner, while this value makes the six-pass ownership and ordering explicit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::graphics) struct EnvironmentCaptureRenderPlan {
    target: EnvironmentCaptureGpuTargetPlan,
    passes: [EnvironmentCaptureRenderPass; CUBEMAP_FACE_COUNT],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::graphics) struct EnvironmentCaptureRenderPass {
    face: CubemapFace,
    color_array_layer: u32,
    uniform_slot: u32,
    reverse_raster_winding: bool,
    opaque_only: bool,
}

impl EnvironmentCaptureRenderPlan {
    pub(in crate::graphics) fn from_request(request: &RenderEnvironmentCaptureRequest) -> Self {
        let target = EnvironmentCaptureGpuTargetPlan::from_request(request);
        let passes = CubemapFace::ALL.map(|face| EnvironmentCaptureRenderPass {
            face,
            color_array_layer: face.index() as u32,
            uniform_slot: face.index() as u32,
            reverse_raster_winding: true,
            opaque_only: true,
        });
        debug_assert_eq!(
            RENDER_ENVIRONMENT_CAPTURE_WORK_ITEM_COUNT as usize,
            CUBEMAP_FACE_COUNT
        );
        Self { target, passes }
    }

    pub(in crate::graphics) fn target(&self) -> EnvironmentCaptureGpuTargetPlan {
        self.target
    }

    pub(in crate::graphics) fn passes(
        &self,
    ) -> &[EnvironmentCaptureRenderPass; CUBEMAP_FACE_COUNT] {
        &self.passes
    }

    pub(in crate::graphics) fn pass(&self, face: CubemapFace) -> EnvironmentCaptureRenderPass {
        self.passes[face.index()]
    }

    pub(in crate::graphics) fn total_pass_count(&self) -> usize {
        self.passes.len()
    }
}

impl EnvironmentCaptureRenderPass {
    pub(in crate::graphics) fn face(self) -> CubemapFace {
        self.face
    }

    pub(in crate::graphics) fn color_array_layer(self) -> u32 {
        self.color_array_layer
    }

    pub(in crate::graphics) fn uniform_slot(self) -> u32 {
        self.uniform_slot
    }

    pub(in crate::graphics) fn reverse_raster_winding(self) -> bool {
        self.reverse_raster_winding
    }

    pub(in crate::graphics) fn opaque_only(self) -> bool {
        self.opaque_only
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plan_has_one_opaque_winding_safe_pass_per_cubemap_face() {
        let request = RenderEnvironmentCaptureRequest::new("probe", [0.0; 3], 1)
            .unwrap()
            .with_face_size(256)
            .unwrap();
        let plan = EnvironmentCaptureRenderPlan::from_request(&request);

        assert_eq!(plan.total_pass_count(), 6);
        for (index, pass) in plan.passes().iter().copied().enumerate() {
            assert_eq!(pass.face().index(), index);
            assert_eq!(pass.color_array_layer(), index as u32);
            assert_eq!(pass.uniform_slot(), index as u32);
            assert!(pass.reverse_raster_winding());
            assert!(pass.opaque_only());
        }
        assert_eq!(plan.pass(CubemapFace::NegativeZ).uniform_slot(), 5);
        assert_eq!(plan.target().face_size(), 256);
    }

    #[test]
    fn plan_keeps_target_mip_budget_and_capture_pass_count_together() {
        let request = RenderEnvironmentCaptureRequest::new("probe", [0.0; 3], 1)
            .unwrap()
            .with_face_size(1024)
            .unwrap();
        let plan = EnvironmentCaptureRenderPlan::from_request(&request);

        assert_eq!(plan.target().source_mip_count(), 11);
        assert_eq!(plan.target().total_texture_bytes(), 71_303_152);
        assert_eq!(
            plan.total_pass_count() as u32,
            RENDER_ENVIRONMENT_CAPTURE_WORK_ITEM_COUNT
        );
    }
}
