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
#[path = "tests/environment_capture_render_plan.rs"]
mod tests;
