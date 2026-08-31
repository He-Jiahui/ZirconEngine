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
mod tests {
    use crate::core::framework::render::{
        RenderCameraTargetGraphImportReport, RenderCameraTargetWritebackReport,
    };
    use crate::core::math::UVec2;
    use crate::core::resource::{ResourceHandle, ResourceId, TextureMarker};
    use crate::graphics::types::{FRAMEWORK_OUTPUT_FORMAT_LABEL, ViewportRenderOutputTarget};

    use super::OutputTargetFramePlan;

    #[test]
    fn frame_plan_retains_the_exact_output_target_identity() {
        let target = ViewportRenderOutputTarget::Texture {
            handle: ResourceHandle::<TextureMarker>::new(ResourceId::from_stable_label(
                "tests/output-target/frame-plan",
            )),
            size: UVec2::new(96, 54),
            format: FRAMEWORK_OUTPUT_FORMAT_LABEL,
        };
        let plan = OutputTargetFramePlan::new(
            target,
            RenderCameraTargetGraphImportReport::ready_for_direct_import(UVec2::new(96, 54)),
            RenderCameraTargetWritebackReport::skipped_direct_import(UVec2::new(96, 54)),
            RenderCameraTargetWritebackReport::ready_for_copy(UVec2::new(96, 54)),
        );

        assert_eq!(plan.target(), target);
        assert_eq!(
            plan.compiled_graph_writeback_plan(),
            RenderCameraTargetWritebackReport::skipped_direct_import(UVec2::new(96, 54))
        );
        assert_eq!(
            plan.direct_submission_writeback_plan(),
            RenderCameraTargetWritebackReport::ready_for_copy(UVec2::new(96, 54))
        );
    }

    #[test]
    fn frame_plan_has_no_post_resolution_mutator() {
        let source = include_str!("output_target_frame_plan.rs")
            .split_once("#[cfg(test)]")
            .map(|(production, _)| production)
            .expect("output target frame plan test boundary");

        assert!(!source.contains("set_graph_import_report"));
        assert!(!source.contains("set_writeback_plan"));
    }
}
