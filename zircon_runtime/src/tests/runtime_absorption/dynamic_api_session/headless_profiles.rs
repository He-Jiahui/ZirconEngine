use super::shared::slices::slice_between;

#[test]
fn runtime_10_headless_profiles_keep_render_bridge_optional_and_noop_surfaces() {
    let session_source = include_str!("../../../dynamic_api/session/state.rs");
    let construction_source = include_str!("../../../dynamic_api/session/construction.rs");
    let session_profile_source = include_str!("../../../dynamic_api/session/profile.rs");
    let lifecycle_tests = include_str!("../../../dynamic_api/tests/session_lifecycle.rs");
    let session_profile_tests = include_str!("../../../dynamic_api/tests/session_profiles.rs");
    let session_entry_point_tests =
        include_str!("../../../dynamic_api/tests/session_entry_points.rs");
    let session_doc = include_str!("../../../../../docs/zircon_runtime/dynamic_api/session.md");
    let runtime_10_output = include_str!(
        "../../../../../docs/plans/zircon_runtime/runtime/10/2026-07-09-dynamic-api-and-interface-convergence-output-records.md"
    );
    let runtime_index_output = include_str!(
        "../../../../../docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-runtime-index-output-records.md"
    );

    for required_source_anchor in ["render_bridge: Option<RuntimeRenderBridge>"] {
        assert!(
            session_source.contains(required_source_anchor),
            "dynamic session source should keep Runtime 10 headless anchor `{required_source_anchor}`"
        );
    }
    for required_profile_anchor in [
        "RUNTIME_SESSION_PROFILE_MINIMAL",
        "RUNTIME_SESSION_PROFILE_HEADLESS",
        "fn uses_render_bridge(self) -> bool",
    ] {
        assert!(
            session_profile_source.contains(required_profile_anchor),
            "dynamic session profile source should keep Runtime 10 headless anchor `{required_profile_anchor}`"
        );
    }

    let uses_render_bridge = slice_between(
        session_profile_source,
        "fn uses_render_bridge(self) -> bool",
        "\n    }\n}",
    );
    assert!(
        uses_render_bridge.contains("matches!(self, Self::Runtime | Self::Editor | Self::Dev)"),
        "only rendered runtime/editor/dev profiles should create RuntimeRenderBridge"
    );
    let rendered_profiles = uses_render_bridge
        .split("matches!(self,")
        .nth(1)
        .and_then(|source| source.split(')').next())
        .expect("uses_render_bridge should use an explicit matches! profile set");
    for forbidden_profile in ["Self::Minimal", "Self::Headless"] {
        assert!(
            !rendered_profiles.contains(forbidden_profile),
            "headless lifecycle profiles must not re-enter RuntimeRenderBridge creation through `{forbidden_profile}`"
        );
    }

    let construction = slice_between(
        construction_source,
        "let render_bridge = if profile.uses_render_bridge()",
        "let level = {",
    );
    for required_construction_anchor in [
        "RuntimeRenderBridge::new(&core)",
        "Some(render_bridge)",
        "runtime_dynamic_session_render_bridge_skipped",
        "None",
    ] {
        assert!(
            construction.contains(required_construction_anchor),
            "dynamic session construction should keep optional render bridge anchor `{required_construction_anchor}`"
        );
    }

    let capture_frame = slice_between(
        session_source,
        "    pub(super) fn capture_frame(",
        "    pub(super) fn bind_viewport_surface",
    );
    for required_capture_anchor in [
        "if let Some(render_bridge) = &mut self.render_bridge",
        "submit_extract_with_ui",
        "empty_captured_frame(requested)",
    ] {
        assert!(
            capture_frame.contains(required_capture_anchor),
            "headless capture should keep empty-frame fallback anchor `{required_capture_anchor}`"
        );
    }

    for (method_start, method_end) in [
        (
            "    pub(super) fn bind_viewport_surface(",
            "    pub(super) fn unbind_viewport_surface(",
        ),
        (
            "    pub(super) fn unbind_viewport_surface(",
            "    pub(super) fn present_viewport(",
        ),
        (
            "    pub(super) fn present_viewport(",
            "    pub(super) fn capture_accessibility_tree(",
        ),
    ] {
        let method = slice_between(session_source, method_start, method_end);
        assert!(
            method.contains("let Some(render_bridge) = &mut self.render_bridge else"),
            "`{method_start}` should gate WGPU work on an installed RuntimeRenderBridge"
        );
        assert!(
            method.contains("return Ok(());"),
            "`{method_start}` should be a no-op when headless/minimal skipped the render bridge"
        );
    }

    for (test_source, required_test_anchor) in [
        (
            session_profile_tests,
            "create_session_accepts_named_headless_profile_without_render_bridge",
        ),
        (
            session_profile_tests,
            "minimal_and_headless_profiles_skip_render_bridge_bootstrap",
        ),
        (
            session_entry_point_tests,
            "destroyed_headless_session_entry_points_reject_old_handle",
        ),
        (
            lifecycle_tests,
            "session_destroy_reports_explicit_not_found_after_headless_destroy",
        ),
    ] {
        assert!(
            test_source.contains(required_test_anchor),
            "dynamic API lifecycle tests should keep Runtime 10 evidence `{required_test_anchor}`"
        );
    }

    for required_doc_anchor in [
        "minimal` and `headless` profiles now skip `RuntimeRenderBridge` creation",
        "frame capture returns an empty encoded frame",
        "surface bind/unbind/present operations are no-ops",
        "runtime_10_headless_profiles_keep_render_bridge_optional_and_noop_surfaces",
    ] {
        assert!(
            session_doc.contains(required_doc_anchor),
            "dynamic API session docs should record `{required_doc_anchor}`"
        );
    }

    for required_plan_anchor in [
        "headless/minimal 跳过 bridge",
        "capture 返回空帧",
        "surface bind/unbind/present 为 no-op",
        "runtime_10_headless_profiles_keep_render_bridge_optional_and_noop_surfaces",
    ] {
        assert!(
            runtime_10_output.contains(required_plan_anchor)
                || runtime_index_output.contains(required_plan_anchor),
            "Runtime 10 output status should record `{required_plan_anchor}`"
        );
    }
}
