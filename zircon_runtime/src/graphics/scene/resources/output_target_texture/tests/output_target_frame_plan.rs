use crate::core::framework::render::{
    RenderCameraTargetGraphImportReport, RenderCameraTargetWritebackReport,
};
use crate::core::math::UVec2;
use crate::core::resource::{ResourceHandle, ResourceId, TextureMarker};
use crate::graphics::types::{ViewportRenderOutputTarget, FRAMEWORK_OUTPUT_FORMAT_LABEL};

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
    let source = include_str!("../output_target_frame_plan.rs")
        .split_once("#[cfg(test)]")
        .map(|(production, _)| production)
        .expect("output target frame plan test boundary");

    assert!(!source.contains("set_graph_import_report"));
    assert!(!source.contains("set_writeback_plan"));
}
