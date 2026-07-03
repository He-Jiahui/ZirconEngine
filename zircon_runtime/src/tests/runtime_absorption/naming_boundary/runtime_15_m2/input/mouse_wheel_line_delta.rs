use super::*;

const OLD_PIXEL_SCROLL_SCALE_NAME: &str = concat!("LEGACY_", "PIXEL_SCROLL_SCALE");
const OLD_VERTICAL_DELTA_HELPER_NAME: &str = concat!("legacy_", "vertical_delta");

#[test]
fn runtime_15_input_mouse_wheel_line_delta_uses_current_names() {
    let manifest_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let framework_input_dir = manifest_root.join("src/core/framework/input");
    let mouse_wheel = read_text(
        &framework_input_dir.join("mouse_wheel.rs"),
        "framework mouse-wheel owner should be readable",
    );
    let framework_input_mod = read_text(
        &framework_input_dir.join("mod.rs"),
        "framework input module entry should be readable",
    );
    let input_mod = read_text(
        &manifest_root.join("src/input/mod.rs"),
        "runtime input module entry should be readable",
    );
    let prelude = read_text(
        &manifest_root.join("src/prelude.rs"),
        "runtime prelude should be readable",
    );
    let default_input_manager = read_text(
        &manifest_root.join("src/input/runtime/default_input_manager.rs"),
        "default input manager should be readable",
    );
    let dynamic_events = read_text(
        &manifest_root.join("src/dynamic_api/session/events.rs"),
        "dynamic API session event owner should be readable",
    );
    let dynamic_input_events_test = read_text(
        &manifest_root.join("src/dynamic_api/tests/input_events.rs"),
        "dynamic API input event tests should be readable",
    );
    let input_manager_tests = read_text(
        &manifest_root.join("src/input/tests/input_manager.rs"),
        "input manager test parent should be readable",
    );
    let frame_state_tests = read_text(
        &manifest_root.join("src/input/tests/input_manager/frame_state.rs"),
        "input manager frame-state tests should be readable",
    );
    let runtime_15_plan = read_repo_text(
        manifest_root,
        "docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md",
    );
    let runtime_index = read_repo_text(manifest_root, "docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo_text(
        manifest_root,
        "docs/plans/engine-code-review-findings-2026-06.md",
    );
    let structure_convention = read_repo_text(
        manifest_root,
        "docs/plans/engine-code-structure-convention.md",
    );
    let module_doc = read_repo_text(
        manifest_root,
        "docs/zircon_runtime/structure/module-convention.md",
    );
    let input_state_doc = read_repo_text(manifest_root, "docs/zircon_runtime/input/input_state.md");
    let dynamic_api_doc =
        read_repo_text(manifest_root, "docs/zircon_runtime/dynamic_api/session.md");
    let prelude_doc = read_repo_text(manifest_root, "docs/zircon_runtime/prelude.md");
    let status_rows = read_text(
        &manifest_root.join(
            "src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/foundation.rs",
        ),
        "Runtime 15 status rows should be readable",
    );
    let expected_status = read_text(
        &manifest_root.join(
            "src/tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/naming_boundary.rs",
        ),
        "Runtime 15 expected naming-boundary status map should be readable",
    );
    let expected_date = read_text(
        &manifest_root.join(
            "src/tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/naming_boundary.rs",
        ),
        "Runtime 15 expected naming-boundary date map should be readable",
    );

    assert_contains_all(
        "framework mouse-wheel owner",
        &mouse_wheel,
        &[
            "pub const PIXEL_SCROLL_LINE_DELTA_SCALE: f32 = 0.1;",
            "pub fn vertical_line_delta(self) -> f32",
            "MouseScrollUnit::Pixel => self.y * PIXEL_SCROLL_LINE_DELTA_SCALE",
        ],
    );
    assert_contains_all(
        "framework input module entry",
        &framework_input_mod,
        &["PIXEL_SCROLL_LINE_DELTA_SCALE"],
    );
    assert_contains_all(
        "runtime input module entry",
        &input_mod,
        &["PIXEL_SCROLL_LINE_DELTA_SCALE"],
    );
    assert_contains_all(
        "runtime prelude",
        &prelude,
        &["PIXEL_SCROLL_LINE_DELTA_SCALE"],
    );
    assert_contains_all(
        "default input manager",
        &default_input_manager,
        &["wheel.vertical_line_delta()"],
    );
    assert_contains_all(
        "dynamic API session events",
        &dynamic_events,
        &["wheel.vertical_line_delta()"],
    );
    assert_contains_all(
        "dynamic API input events test",
        &dynamic_input_events_test,
        &["wheel.vertical_line_delta()"],
    );
    assert_contains_all(
        "input manager tests",
        &input_manager_tests,
        &["PIXEL_SCROLL_LINE_DELTA_SCALE"],
    );
    assert_contains_all(
        "input frame-state tests",
        &frame_state_tests,
        &[
            "MouseWheelEvent::pixels(4.0, 20.0).vertical_line_delta()",
            "20.0 * PIXEL_SCROLL_LINE_DELTA_SCALE",
        ],
    );

    for (label, source) in [
        ("framework mouse-wheel owner", mouse_wheel.as_str()),
        ("framework input module entry", framework_input_mod.as_str()),
        ("runtime input module entry", input_mod.as_str()),
        ("runtime prelude", prelude.as_str()),
        ("default input manager", default_input_manager.as_str()),
        ("dynamic API session events", dynamic_events.as_str()),
    ] {
        assert!(
            !source.contains(OLD_PIXEL_SCROLL_SCALE_NAME),
            "{label} should not preserve the old pixel-scroll scale name"
        );
        assert!(
            !source.contains(OLD_VERTICAL_DELTA_HELPER_NAME),
            "{label} should not preserve the old vertical delta helper name"
        );
    }

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("input state doc", input_state_doc.as_str()),
        ("dynamic API doc", dynamic_api_doc.as_str()),
        ("prelude doc", prelude_doc.as_str()),
        ("status-output row data", status_rows.as_str()),
        ("expected status map", expected_status.as_str()),
        ("expected date map", expected_date.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M2 input mouse-wheel line-delta naming hard cutover",
                "runtime_15_input_mouse_wheel_line_delta_naming_hard_cutover_static_passed_cargo_deferred",
                "core/framework/input/mouse_wheel.rs",
                "vertical_line_delta",
                "runtime_15_input_mouse_wheel_line_delta_uses_current_names",
            ],
        );
    }
}
