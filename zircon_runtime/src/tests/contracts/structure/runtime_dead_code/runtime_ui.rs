use super::super::assert_contains_all;
use super::{read_runtime_src, runtime_source_path, DEAD_CODE_ALLOW_ATTRIBUTE};

#[test]
fn runtime_15_runtime_ui_dead_code_surface_is_test_support() {
    let ui_mod = read_runtime_src("ui/mod.rs");
    let public_runtime_frame = read_runtime_src("ui/public_runtime_frame.rs");
    let graphics_types_mod = read_runtime_src("graphics/types/mod.rs");
    let runtime_ui_support_mod = read_runtime_src("ui/tests/runtime_ui_support/mod.rs");
    let runtime_ui_manager = read_runtime_src("ui/tests/runtime_ui_support/runtime_ui_manager.rs");

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
            "pub ui: Option<Arc<UiRenderSubmission>>",
        ],
    );
    assert!(
        !runtime_source_path("graphics/types/viewport_render_frame_from_public_runtime.rs")
            .exists(),
        "graphics must not retain a conversion owner that imports the UI domain"
    );
    assert!(
        !graphics_types_mod.contains("viewport_render_frame_from_public_runtime"),
        "graphics types root must not mount the retired UI conversion owner"
    );
    assert!(
        !public_runtime_frame.contains("crate::graphics"),
        "UI public runtime frame must remain a neutral extract DTO instead of importing graphics internals"
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
}
