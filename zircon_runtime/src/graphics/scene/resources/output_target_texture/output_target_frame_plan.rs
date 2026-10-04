use crate::core::framework::render::{
    RenderCameraTargetGraphImportReport, RenderCameraTargetWritebackReport,
};
use crate::graphics::types::ViewportRenderOutputTarget;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(in crate::graphics::scene) struct OutputTargetFramePlan {
    target: ViewportRenderOutputTarget,
    graph_import_report: RenderCameraTargetGraphImportReport,
    compiled_graph_writeback_plan: RenderCameraTargetWritebackReport,
    direct_submission_writeback_plan: RenderCameraTargetWritebackReport,
}

impl OutputTargetFramePlan {
    pub(in crate::graphics::scene) const fn new(
        target: ViewportRenderOutputTarget,
        graph_import_report: RenderCameraTargetGraphImportReport,
        compiled_graph_writeback_plan: RenderCameraTargetWritebackReport,
        direct_submission_writeback_plan: RenderCameraTargetWritebackReport,
    ) -> Self {
        Self {
            target,
            graph_import_report,
            compiled_graph_writeback_plan,
            direct_submission_writeback_plan,
        }
    }

    pub(in crate::graphics::scene) fn not_requested(target: ViewportRenderOutputTarget) -> Self {
        Self::new(
            target,
            RenderCameraTargetGraphImportReport::not_requested(target.kind()),
            RenderCameraTargetWritebackReport::not_requested(target.kind()),
            RenderCameraTargetWritebackReport::not_requested(target.kind()),
        )
    }

    pub(in crate::graphics::scene) const fn target(self) -> ViewportRenderOutputTarget {
        self.target
    }

    pub(in crate::graphics::scene) const fn graph_import_report(
        self,
    ) -> RenderCameraTargetGraphImportReport {
        self.graph_import_report
    }

    pub(in crate::graphics::scene) const fn compiled_graph_writeback_plan(
        self,
    ) -> RenderCameraTargetWritebackReport {
        self.compiled_graph_writeback_plan
    }

    pub(in crate::graphics::scene) const fn direct_submission_writeback_plan(
        self,
    ) -> RenderCameraTargetWritebackReport {
        self.direct_submission_writeback_plan
    }
}

#[cfg(test)]
#[path = "tests/output_target_frame_plan.rs"]
mod tests;
