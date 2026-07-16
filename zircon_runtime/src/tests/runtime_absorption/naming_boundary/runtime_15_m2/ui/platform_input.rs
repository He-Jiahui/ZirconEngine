use std::path::Path;

use super::*;

#[test]
fn runtime_15_platform_input_uses_dom_keycode_names() {
    let manifest_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let keyboard_map = read_text(
        &manifest_root.join("src/ui/platform_input/keyboard_map.rs"),
        "platform input keyboard map should be readable",
    );
    let winit_translation = read_text(
        &manifest_root.join("src/ui/platform_input/winit_translation.rs"),
        "platform input winit translation should be readable",
    );
    let runtime_15_plan = read_repo_text(
        manifest_root,
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-code-structure-and-module-conventions-output-records.md",
    );
    let runtime_index = read_repo_text(
        manifest_root,
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-runtime-index-output-records.md",
    );
    let review_findings = read_repo_text(
        manifest_root,
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-review-findings-output-records.md",
    );
    let structure_convention = read_repo_text(
        manifest_root,
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-structure-output-records.md",
    );
    let module_doc = read_repo_text(
        manifest_root,
        "docs/zircon_runtime/structure/module-convention.md",
    );
    let platform_input_doc =
        read_repo_text(manifest_root, "docs/zircon_runtime/ui/platform_input.md");
    let status_rows = read_runtime_15_naming_status_rows(manifest_root);
    let status_slice = read_runtime_15_naming_status_map(manifest_root);
    let date_slice = read_runtime_15_naming_date_map(manifest_root);

    assert_contains_all(
        "platform input keyboard map",
        &keyboard_map,
        &[
            "pub(super) fn dom_key_code",
            "Key::Character(text) => dom_character_key_code(text)",
            "fn dom_character_key_code",
        ],
    );
    assert!(
        !keyboard_map.contains("legacy_key_code")
            && !keyboard_map.contains("legacy_character_key_code"),
        "platform input keyboard map should not keep legacy key-code helper names"
    );
    assert_contains_all(
        "platform input winit translation",
        &winit_translation,
        &[
            "dom_key_code(&event.logical_key)",
            "const PIXEL_SCROLL_LINE_DELTA_SCALE",
            "translate_winit_wheel_preserves_precise_delta_and_line_delta_scale",
        ],
    );
    assert!(
        !winit_translation.contains("legacy_key_code")
            && !winit_translation.contains("PIXEL_SCROLL_LEGACY_LINE_SCALE")
            && !winit_translation.contains("legacy_scalar"),
        "platform input winit translation should not keep legacy naming"
    );

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan),
        ("runtime index", runtime_index),
        ("review findings", review_findings),
        ("structure convention", structure_convention),
        ("module convention doc", module_doc),
        ("platform input doc", platform_input_doc),
        ("status row data", status_rows),
        ("status slice", status_slice),
        ("date slice", date_slice),
    ] {
        assert_contains_all(
            label,
            &source,
            &[
                "Runtime 15 M2 platform input DOM keycode naming hard cutover",
                "runtime_15_platform_input_dom_keycode_naming_hard_cutover_static_passed_cargo_timeout_no_result",
                "ui/platform_input/keyboard_map.rs",
                "dom_key_code",
                "runtime_15_platform_input_uses_dom_keycode_names",
            ],
        );
    }
}

#[test]
fn runtime_15_platform_input_winit_tests_use_runtime_input_baseline_names() {
    let manifest_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let winit_translation = read_text(
        &manifest_root.join("src/ui/platform_input/winit_translation.rs"),
        "platform input winit translation should be readable",
    );
    let runtime_15_plan = read_repo_text(
        manifest_root,
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-code-structure-and-module-conventions-output-records.md",
    );
    let runtime_index = read_repo_text(
        manifest_root,
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-runtime-index-output-records.md",
    );
    let review_findings = read_repo_text(
        manifest_root,
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-review-findings-output-records.md",
    );
    let structure_convention = read_repo_text(
        manifest_root,
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-structure-output-records.md",
    );
    let module_doc = read_repo_text(
        manifest_root,
        "docs/zircon_runtime/structure/module-convention.md",
    );
    let platform_input_doc =
        read_repo_text(manifest_root, "docs/zircon_runtime/ui/platform_input.md");
    let status_rows = read_runtime_15_naming_status_rows(manifest_root);
    let status_slice = read_runtime_15_naming_status_map(manifest_root);
    let date_slice = read_runtime_15_naming_date_map(manifest_root);

    assert_contains_all(
        "platform input winit translation runtime-input baseline tests",
        &winit_translation,
        &[
            "translate_winit_keyboard_matrix_matches_runtime_input_baseline",
            "translate_winit_ime_preedit_commit_and_disable_match_runtime_input_baseline",
        ],
    );
    assert!(
        !winit_translation.contains("editor_baseline"),
        "platform input runtime tests should not use editor_baseline names inside the runtime owner"
    );

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan),
        ("runtime index", runtime_index),
        ("review findings", review_findings),
        ("structure convention", structure_convention),
        ("module convention doc", module_doc),
        ("platform input doc", platform_input_doc),
        ("status row data", status_rows),
        ("status slice", status_slice),
        ("date slice", date_slice),
    ] {
        assert_contains_all(
            label,
            &source,
            &[
                "Runtime 15 M2 platform input runtime baseline test naming hard cutover",
                "runtime_15_platform_input_runtime_baseline_test_naming_hard_cutover_static_passed_cargo_deferred",
                "ui/platform_input/winit_translation.rs",
                "runtime_input_baseline",
                "runtime_15_platform_input_winit_tests_use_runtime_input_baseline_names",
            ],
        );
    }
}
