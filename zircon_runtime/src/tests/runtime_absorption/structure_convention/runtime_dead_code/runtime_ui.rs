use super::super::assert_contains_all;
use super::{read_repo, read_runtime_src, runtime_source_path, DEAD_CODE_ALLOW_ATTRIBUTE};

#[test]
fn runtime_15_runtime_ui_dead_code_surface_is_test_support() {
    let ui_mod = read_runtime_src("ui/mod.rs");
    let public_runtime_frame = read_runtime_src("ui/public_runtime_frame.rs");
    let viewport_conversion =
        read_runtime_src("graphics/types/viewport_render_frame_from_public_runtime.rs");
    let runtime_ui_support_mod = read_runtime_src("ui/tests/runtime_ui_support/mod.rs");
    let runtime_ui_manager = read_runtime_src("ui/tests/runtime_ui_support/runtime_ui_manager.rs");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");

    assert_contains_all(
        "runtime UI production frame surface",
        &ui_mod,
        &[
            "mod public_runtime_frame;",
            "pub(crate) use public_runtime_frame::PublicRuntimeFrame;",
            "#[cfg(test)]",
            "#[path = \"tests/runtime_ui_support/mod.rs\"]",
            "mod runtime_ui_support;",
            "pub(crate) use runtime_ui_support::{RuntimeUiFixture, RuntimeUiManager};",
        ],
    );
    assert!(
        !ui_mod.contains(DEAD_CODE_ALLOW_ATTRIBUTE),
        "ui root should not hide runtime UI dead code behind allow(dead_code)"
    );
    assert!(
        !ui_mod.contains("mod runtime_ui;"),
        "runtime UI manager support should not remain a production ui::runtime_ui module"
    );
    assert!(
        !runtime_source_path("ui/runtime_ui/mod.rs").exists(),
        "old production ui/runtime_ui module directory should be removed"
    );

    assert_contains_all(
        "public runtime frame owner",
        &public_runtime_frame,
        &[
            "pub(crate) struct PublicRuntimeFrame",
            "pub extract: RenderFrameExtract",
            "pub viewport_size: UVec2",
            "pub ui: Option<UiRenderExtract>",
        ],
    );
    assert_contains_all(
        "graphics public runtime frame conversion",
        &viewport_conversion,
        &[
            "use crate::ui::PublicRuntimeFrame;",
            "impl From<PublicRuntimeFrame> for ViewportRenderFrame",
            "extract: Arc::new(frame.extract)",
        ],
    );
    assert_contains_all(
        "runtime UI test support owner",
        &runtime_ui_support_mod,
        &[
            "mod runtime_ui_fixture;",
            "mod runtime_ui_manager;",
            "pub(crate) use runtime_ui_fixture::RuntimeUiFixture;",
            "pub(crate) use runtime_ui_manager::RuntimeUiManager;",
        ],
    );
    assert_contains_all(
        "runtime UI manager test support frame import",
        &runtime_ui_manager,
        &[
            "use crate::ui::{dispatch::UiInputManager, PublicRuntimeFrame};",
            "pub(crate) fn build_frame(&self) -> PublicRuntimeFrame",
        ],
    );

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 runtime UI dead-code support split",
                "runtime_15_runtime_ui_dead_code_support_split_coremin_check_passed",
                "runtime_15_runtime_ui_dead_code_surface_is_test_support",
                "f8_f9_f10_runtime_surface_top_row_closed_status_static_passed_cargo_deferred",
            ],
        );
    }
    let f10_row = review_findings
        .lines()
        .find(|line| line.starts_with("| F10 |"))
        .expect("F10 review findings top row");
    assert!(
        f10_row.contains(
            "f8_f9_f10_runtime_surface_top_row_closed_status_static_passed_cargo_deferred"
        ) && f10_row.ends_with("| Runtime 09 + Runtime 15 / review closed |"),
        "F10 top row should record runtime surface review closed status"
    );
}
