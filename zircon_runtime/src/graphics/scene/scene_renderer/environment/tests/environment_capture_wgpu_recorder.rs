use super::skipped_environment_capture_command_count;

const SOURCE: &str = include_str!("../environment_capture_wgpu_recorder.rs");

fn production_source() -> &'static str {
    SOURCE
        .split_once("#[cfg(test)]")
        .map(|(production, _)| production)
        .expect("environment capture recorder must retain a test boundary")
}

#[test]
fn recorder_builds_one_opaque_command_set_for_all_six_faces() {
    let source = production_source();

    assert_eq!(
        source
            .matches("build_environment_capture_command_buffers(")
            .count(),
        1
    );
    assert!(source.contains("for capture_pass in render_plan.passes()"));
    assert!(source.contains("command_buffers.opaque().commands()"));
    assert!(source.contains("command_buffers.alpha_mask().commands()"));
    assert!(source.contains("command_buffers.advanced_pbr_opaque().commands()"));
    assert_eq!(
        source
            .matches("create_environment_capture_forward_receiver_bind_group(")
            .count(),
        1
    );
    assert!(!source.contains("build_mesh_pass_command_buffers("));
    assert!(!source.contains("command_buffers.transparent().commands()"));
    assert!(!source.contains("command_buffers.transmission().commands()"));
}

#[test]
fn recorder_selects_face_owned_bindings_and_attachments() {
    let source = production_source();

    assert!(source.contains("scene_batch.select_face(capture_pass.face())"));
    assert!(source.contains("uniform_workspace.bind_group(capture_pass.face())"));
    assert!(source.contains("light_grid_workspace"));
    assert!(source.contains("bind_group(capture_pass.face())"));
    assert!(source.contains("target.color_face(capture_pass.face())"));
    assert!(source.contains("target.depth_view()"));
    assert!(source.contains("record_preview_sky("));
    assert!(source.contains("record_environment_capture_commands_with_attachment_ops("));
    assert!(!source.contains("BaseScenePass.record_commands_with_attachment_ops("));
    assert!(source.contains("RenderGraphAttachmentOps::load_store()"));
}

#[test]
fn recorder_reuses_one_disabled_receiver_or_builds_six_lit_receivers() {
    let source = production_source();

    assert!(source.contains("EnvironmentCaptureForwardReceiverBindGroups::Shared"));
    assert!(source.contains("EnvironmentCaptureForwardReceiverBindGroups::PerFace"));
    assert!(source.contains("CubemapFace::ALL.map"));
}

#[test]
fn recorder_publishes_existing_structural_report_to_the_profiler() {
    let source = production_source();

    for counter in [
        "environment_capture_face_pass_count",
        "environment_capture_command_build_count",
        "environment_capture_commands_per_face",
        "environment_capture_draw_call_count",
        "environment_capture_state_change_count",
        "environment_capture_bind_skip_count",
    ] {
        assert!(
            source.contains(counter),
            "missing profile counter {counter}"
        );
    }
    assert!(source.contains("report.emit_profile_counters()"));
}

#[test]
fn incomplete_face_replay_is_not_a_successful_capture() {
    assert_eq!(skipped_environment_capture_command_count(3, 6, 18), 0);
    assert_eq!(skipped_environment_capture_command_count(3, 6, 17), 1);
    assert_eq!(skipped_environment_capture_command_count(3, 6, 0), 18);
}
