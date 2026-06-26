use std::path::Path;

#[test]
fn runtime_10_dynamic_session_test_owner_split_keeps_focused_modules() {
    let runtime_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let repo_root = runtime_root
        .parent()
        .expect("runtime crate should live under the repository root");
    let old_tests_path = repo_root.join("zircon_runtime/src/dynamic_api/session/tests.rs");
    let tests_mod = include_str!("../../../dynamic_api/session/tests/mod.rs");
    let vampire_runtime_support =
        include_str!("../../../dynamic_api/session/tests/vampire_runtime_support.rs");
    let gameplay = include_str!("../../../dynamic_api/session/tests/vampire_gameplay.rs");
    let menu = include_str!("../../../dynamic_api/session/tests/vampire_menu.rs");
    let hud = include_str!("../../../dynamic_api/session/tests/vampire_hud.rs");
    let frame_diagnostics = include_str!("../../../dynamic_api/session/tests/frame_diagnostics.rs");
    let runtime_errors = include_str!("../../../dynamic_api/session/tests/runtime_errors.rs");
    let session_doc = include_str!("../../../../../docs/zircon_runtime/dynamic_api/session.md");
    let runtime_10_plan = include_str!(
        "../../../../../docs/plans/zircon_runtime/runtime/10-dynamic-api-and-interface-convergence.md"
    );
    let runtime_index = include_str!("../../../../../docs/plans/zircon_runtime/runtime/index.md");

    assert!(
        !old_tests_path.exists(),
        "dynamic session tests should not drift back into the removed monolithic tests.rs"
    );

    for module_decl in [
        "mod frame_diagnostics;",
        "mod lock_poison;",
        "mod runtime_errors;",
        "mod vampire_gameplay;",
        "mod vampire_hud;",
        "mod vampire_menu;",
        "mod vampire_runtime_support;",
    ] {
        assert!(
            tests_mod.contains(module_decl),
            "dynamic session tests/mod.rs should keep module declaration `{module_decl}`"
        );
    }
    assert!(
        !tests_mod.contains("#[test]") && !tests_mod.contains("fn "),
        "dynamic session tests/mod.rs should stay navigational only"
    );

    for helper_anchor in [
        "pub(super) fn vampire_project_config",
        "pub(super) fn start_vampire_game",
        "pub(super) fn click_vampire_menu_button",
        "pub(super) fn small_headless_frame_request",
        "pub(super) fn summarize_hud_region",
    ] {
        assert!(
            vampire_runtime_support.contains(helper_anchor),
            "dynamic session vampire_runtime_support.rs should retain shared support `{helper_anchor}`"
        );
    }

    for (owner_name, owner_source, expected_anchor) in [
        (
            "vampire_gameplay",
            gameplay,
            "vampire_project_session_w_key_moves_player_before_input_clear",
        ),
        (
            "vampire_gameplay",
            gameplay,
            "vampire_project_session_auto_blood_bolt_damages_nearest_enemy",
        ),
        (
            "vampire_menu",
            menu,
            "vampire_project_session_game_over_menu_retries_to_playing",
        ),
        (
            "vampire_hud",
            hud,
            "vampire_project_session_capture_frame_draws_world_hud_bars",
        ),
        (
            "frame_diagnostics",
            frame_diagnostics,
            "headless_session_capture_records_frame_extract_diagnostics",
        ),
        (
            "frame_diagnostics",
            frame_diagnostics,
            "frame_extract_rebuild_skips_unchanged_entities",
        ),
        (
            "frame_diagnostics",
            frame_diagnostics,
            "vampire_project_session_reports_runtime_fps_and_render_work",
        ),
        (
            "runtime_errors",
            runtime_errors,
            "runtime_session_error_preserves_step_when_inner_error_is_empty",
        ),
    ] {
        assert!(
            owner_source.contains(expected_anchor),
            "dynamic session tests/{owner_name}.rs should own `{expected_anchor}`"
        );
    }

    for doc_anchor in [
        "Dynamic Session Test Owner Split",
        "session/tests/{mod,vampire_runtime_support,vampire_gameplay,vampire_menu,vampire_hud,frame_diagnostics,runtime_errors,lock_poison}.rs",
        "runtime_10_dynamic_session_test_owner_split_keeps_focused_modules",
        "session/tests/frame_diagnostics.rs",
    ] {
        assert!(
            session_doc.contains(doc_anchor)
                || runtime_10_plan.contains(doc_anchor)
                || runtime_index.contains(doc_anchor),
            "Runtime 10 dynamic session test split docs should retain `{doc_anchor}`"
        );
    }
}
