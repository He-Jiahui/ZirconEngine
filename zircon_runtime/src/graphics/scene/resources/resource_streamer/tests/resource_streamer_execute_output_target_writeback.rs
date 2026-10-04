use crate::core::framework::render::{
    RenderCameraTargetWritebackReport, RenderCameraTargetWritebackStatus,
};
use crate::core::math::UVec2;
use crate::core::resource::{ResourceHandle, ResourceId, TextureMarker};
use crate::graphics::types::{
    ViewportRenderOutputTarget, ViewportTextureWritebackStatus, FRAMEWORK_OUTPUT_FORMAT_LABEL,
    LINEAR_OUTPUT_FORMAT_LABEL,
};

use super::{
    output_target_writeback_extent, output_target_writeback_report_for_plan,
    should_execute_output_target_writeback, suppressed_output_target_writeback_report,
};

#[test]
fn output_target_writeback_executes_ready_copy_and_conversion_plans() {
    let texture = texture_handle("tests/writeback/ready");
    let target = ViewportRenderOutputTarget::Texture {
        handle: texture,
        size: UVec2::new(128, 72),
        format: FRAMEWORK_OUTPUT_FORMAT_LABEL,
    };
    let conversion_target = ViewportRenderOutputTarget::Texture {
        handle: texture,
        size: UVec2::new(128, 72),
        format: LINEAR_OUTPUT_FORMAT_LABEL,
    };
    let ready = target.writeback_plan(Some(FRAMEWORK_OUTPUT_FORMAT_LABEL));
    let conversion = conversion_target.writeback_plan(Some(LINEAR_OUTPUT_FORMAT_LABEL));
    let blocked = target.writeback_plan(Some("rgba16float"));
    let non_texture = ViewportRenderOutputTarget::PrimarySurface
        .writeback_plan(Some(FRAMEWORK_OUTPUT_FORMAT_LABEL));

    assert_eq!(
        ready.status(),
        ViewportTextureWritebackStatus::ReadyForSrgbCopy
    );
    assert!(should_execute_output_target_writeback(&ready));
    assert_eq!(
        conversion.status(),
        ViewportTextureWritebackStatus::ReadyForConversion
    );
    assert!(should_execute_output_target_writeback(&conversion));
    assert!(!should_execute_output_target_writeback(&blocked));
    assert!(!should_execute_output_target_writeback(&non_texture));
}

#[test]
fn output_target_writeback_only_encodes_into_its_caller_frame_packet() {
    let source = include_str!("../resource_streamer_execute_output_target_writeback.rs");
    let production = source
        .split_once("#[cfg(test)]")
        .map(|(production, _)| production)
        .expect("writeback source must retain its test-module boundary");

    assert!(production.contains("fn encode_output_target_writeback"));
    assert!(production.contains("encoder.copy_texture_to_texture"));
    assert!(production.contains("encode_linear_rgba_conversion"));
    assert!(!production.contains("wgpu::Queue"));
    assert!(!production.contains("queue.submit"));
}

#[test]
fn direct_writeback_consumes_the_direct_submission_branch_of_the_frame_plan() {
    let source = include_str!("../resource_streamer_execute_output_target_writeback.rs");
    let production = source
        .split_once("#[cfg(test)]")
        .map(|(production, _)| production)
        .expect("writeback source must retain its test-module boundary");

    assert!(production.contains("let report = plan.direct_submission_writeback_plan();"));
    assert!(!production.contains("let report = plan.compiled_graph_writeback_plan();"));
}

#[test]
fn output_target_writeback_report_maps_ready_and_blocked_plans() {
    let texture = texture_handle("tests/writeback/report");
    let target = ViewportRenderOutputTarget::Texture {
        handle: texture,
        size: UVec2::new(128, 72),
        format: FRAMEWORK_OUTPUT_FORMAT_LABEL,
    };
    let conversion_target = ViewportRenderOutputTarget::Texture {
        handle: texture,
        size: UVec2::new(128, 72),
        format: LINEAR_OUTPUT_FORMAT_LABEL,
    };
    let ready = output_target_writeback_report_for_plan(
        &target.writeback_plan(Some(FRAMEWORK_OUTPUT_FORMAT_LABEL)),
    );
    let conversion = output_target_writeback_report_for_plan(
        &conversion_target.writeback_plan(Some(LINEAR_OUTPUT_FORMAT_LABEL)),
    );
    let blocked =
        output_target_writeback_report_for_plan(&target.writeback_plan(Some("rgba16float")));

    assert_eq!(
        ready.status,
        RenderCameraTargetWritebackStatus::ReadyForCopy
    );
    assert_eq!(ready.target_size, UVec2::new(128, 72));
    assert_eq!(ready.copied_count, 0);
    assert!(!ready.debug_marker_emitted);
    assert_eq!(
        conversion.status,
        RenderCameraTargetWritebackStatus::ReadyForConversion
    );
    assert_eq!(conversion.target_size, UVec2::new(128, 72));
    assert_eq!(conversion.converted_count, 0);
    assert!(!conversion.debug_marker_emitted);
    assert_eq!(
        blocked.status,
        RenderCameraTargetWritebackStatus::BlockedFormatMismatch
    );
    assert_eq!(blocked.target_size, UVec2::new(128, 72));
    assert!(!blocked.debug_marker_emitted);
    let copied = RenderCameraTargetWritebackReport::copied(UVec2::new(128, 72));
    let converted = RenderCameraTargetWritebackReport::converted(UVec2::new(128, 72));
    assert!(copied.debug_marker_emitted);
    assert!(!copied.conversion_debug_marker_emitted);
    assert!(!converted.debug_marker_emitted);
    assert!(converted.conversion_debug_marker_emitted);
    assert_eq!(converted.converted_count, 1);
}

#[test]
fn suppressed_output_target_writeback_report_is_texture_only() {
    let texture = ViewportRenderOutputTarget::Texture {
        handle: texture_handle("tests/writeback/suppressed"),
        size: UVec2::new(128, 72),
        format: FRAMEWORK_OUTPUT_FORMAT_LABEL,
    };
    let texture_report = suppressed_output_target_writeback_report(texture);
    let primary_report =
        suppressed_output_target_writeback_report(ViewportRenderOutputTarget::PrimarySurface);

    assert_eq!(
        texture_report.status,
        RenderCameraTargetWritebackStatus::SuppressedByCameraStack
    );
    assert_eq!(texture_report.target_size, UVec2::new(128, 72));
    assert_eq!(
        primary_report.status,
        RenderCameraTargetWritebackStatus::NotRequested
    );
}

#[test]
fn output_target_writeback_extent_accepts_matching_source_and_destination() {
    let plan = ready_plan(UVec2::new(64, 32));

    let extent =
        output_target_writeback_extent(&plan, UVec2::new(64, 32), UVec2::new(64, 32)).unwrap();

    assert_eq!(extent.width, 64);
    assert_eq!(extent.height, 32);
    assert_eq!(extent.depth_or_array_layers, 1);
}

#[test]
fn output_target_writeback_extent_rejects_source_size_mismatch() {
    let plan = ready_plan(UVec2::new(64, 32));

    let error =
        output_target_writeback_extent(&plan, UVec2::new(63, 32), UVec2::new(64, 32)).unwrap_err();

    assert!(
        matches!(error, crate::graphics::types::GraphicsError::Asset(message) if message.contains("source extent"))
    );
}

#[test]
fn output_target_writeback_extent_rejects_destination_size_mismatch() {
    let plan = ready_plan(UVec2::new(64, 32));

    let error =
        output_target_writeback_extent(&plan, UVec2::new(64, 32), UVec2::new(64, 31)).unwrap_err();

    assert!(
        matches!(error, crate::graphics::types::GraphicsError::Asset(message) if message.contains("destination extent"))
    );
}

fn ready_plan(size: UVec2) -> crate::graphics::types::ViewportTextureWritebackPlan {
    ViewportRenderOutputTarget::Texture {
        handle: texture_handle("tests/writeback/extent"),
        size,
        format: FRAMEWORK_OUTPUT_FORMAT_LABEL,
    }
    .writeback_plan(Some(FRAMEWORK_OUTPUT_FORMAT_LABEL))
}

fn texture_handle(label: &str) -> ResourceHandle<TextureMarker> {
    ResourceHandle::new(ResourceId::from_stable_label(label))
}
