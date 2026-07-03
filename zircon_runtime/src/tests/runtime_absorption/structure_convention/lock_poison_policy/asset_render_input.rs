use super::*;

#[path = "asset_render_input/asset_pipeline.rs"]
mod asset_pipeline;
#[path = "asset_render_input/input_script.rs"]
mod input_script;
#[path = "asset_render_input/render_animation.rs"]
mod render_animation;

const TEST_ATTRIBUTE: &str = concat!("#[", "test", "]");

#[test]
fn runtime_15_asset_render_input_lock_poison_guard_child_owner_split() {
    let parent = read_runtime_src(
        "tests/runtime_absorption/structure_convention/lock_poison_policy/asset_render_input.rs",
    );
    let asset_pipeline = read_runtime_src(
        "tests/runtime_absorption/structure_convention/lock_poison_policy/asset_render_input/asset_pipeline.rs",
    );
    let render_animation = read_runtime_src(
        "tests/runtime_absorption/structure_convention/lock_poison_policy/asset_render_input/render_animation.rs",
    );
    let input_script = read_runtime_src(
        "tests/runtime_absorption/structure_convention/lock_poison_policy/asset_render_input/input_script.rs",
    );

    assert_contains_all(
        "asset/render/input lock-poison parent mounts child owners",
        &parent,
        &[
            "mod asset_pipeline;",
            "mod input_script;",
            "mod render_animation;",
        ],
    );

    for moved_guard in [
        concat!(
            "fn ",
            "runtime_15_asset_project_manager_lock_poison_recovery_guard_covers_project_asset_manager"
        ),
        concat!(
            "fn ",
            "runtime_15_asset_worker_pool_lock_poison_recovery_guard_covers_asset_worker_pool"
        ),
        concat!(
            "fn ",
            "runtime_15_wgpu_render_framework_lock_poison_recovery_guard_covers_wgpu_framework"
        ),
        concat!(
            "fn ",
            "runtime_15_animation_manager_lock_poison_recovery_guard_covers_playback_settings"
        ),
        concat!(
            "fn ",
            "runtime_15_input_runtime_manager_lock_poison_recovery_guard_covers_input_state"
        ),
        concat!(
            "fn ",
            "runtime_15_script_vm_registry_lock_poison_recovery_guard_covers_vm_registries"
        ),
        concat!(
            "fn ",
            "runtime_15_zr_vm_real_backend_runtime_lock_poison_recovery_guard_covers_global_runtime_lock"
        ),
    ] {
        assert!(
            !parent.contains(moved_guard),
            "asset_render_input.rs should mount child owners instead of defining {moved_guard}"
        );
    }

    assert_contains_all(
        "asset pipeline child owns asset lock-poison guards",
        &asset_pipeline,
        &[
            concat!(
                "fn ",
                "runtime_15_asset_project_manager_lock_poison_recovery_guard_covers_project_asset_manager"
            ),
            concat!(
                "fn ",
                "runtime_15_asset_worker_pool_lock_poison_recovery_guard_covers_asset_worker_pool"
            ),
        ],
    );
    assert_contains_all(
        "render animation child owns render and animation lock-poison guards",
        &render_animation,
        &[
            concat!(
                "fn ",
                "runtime_15_wgpu_render_framework_lock_poison_recovery_guard_covers_wgpu_framework"
            ),
            concat!(
                "fn ",
                "runtime_15_animation_manager_lock_poison_recovery_guard_covers_playback_settings"
            ),
        ],
    );
    assert_contains_all(
        "input script child owns input and VM lock-poison guards",
        &input_script,
        &[
            concat!(
                "fn ",
                "runtime_15_input_runtime_manager_lock_poison_recovery_guard_covers_input_state"
            ),
            concat!(
                "fn ",
                "runtime_15_script_vm_registry_lock_poison_recovery_guard_covers_vm_registries"
            ),
            concat!(
                "fn ",
                "runtime_15_zr_vm_real_backend_runtime_lock_poison_recovery_guard_covers_global_runtime_lock"
            ),
        ],
    );

    let child_test_total = [
        parent.as_str(),
        asset_pipeline.as_str(),
        render_animation.as_str(),
        input_script.as_str(),
    ]
    .into_iter()
    .map(|source| source.matches(TEST_ATTRIBUTE).count())
    .sum::<usize>();
    assert_eq!(
        child_test_total, 8,
        "asset/render/input lock-poison children should preserve seven existing guards plus the new split guard"
    );

    for (path, source) in [
        (
            "structure_convention/lock_poison_policy/asset_render_input.rs",
            parent.as_str(),
        ),
        (
            "structure_convention/lock_poison_policy/asset_render_input/asset_pipeline.rs",
            asset_pipeline.as_str(),
        ),
        (
            "structure_convention/lock_poison_policy/asset_render_input/render_animation.rs",
            render_animation.as_str(),
        ),
        (
            "structure_convention/lock_poison_policy/asset_render_input/input_script.rs",
            input_script.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 400,
            "{path} should stay below the focused child-owner budget; got {line_count} lines"
        );
    }

    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let session_note =
        read_repo(".codex/sessions/20260612-0847-runtime-architecture-implementation.md");
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/lock_poison_status.rs",
    );
    let status_map = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support.rs",
    );
    let date_map = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support.rs",
    );

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("session note", session_note.as_str()),
        (
            "status-output M3 lock-poison row data",
            status_rows.as_str(),
        ),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M3 asset/render/input lock-poison guard child-owner split",
                "runtime_15_asset_render_input_lock_poison_guard_child_owner_split_static_passed_cargo_deferred",
                "structure_convention/lock_poison_policy/asset_render_input.rs",
                "structure_convention/lock_poison_policy/asset_render_input/asset_pipeline.rs",
                "structure_convention/lock_poison_policy/asset_render_input/render_animation.rs",
                "structure_convention/lock_poison_policy/asset_render_input/input_script.rs",
                "runtime_15_asset_render_input_lock_poison_guard_child_owner_split",
            ],
        );
    }
    assert_contains_all(
        "status-output status map",
        &status_map,
        &[
            "Runtime 15 M3 asset/render/input lock-poison guard child-owner split",
            "runtime_15_asset_render_input_lock_poison_guard_child_owner_split_static_passed_cargo_deferred",
        ],
    );
    assert_contains_all(
        "status-output date map",
        &date_map,
        &[
            "Runtime 15 M3 asset/render/input lock-poison guard child-owner split",
            "2026-07-01",
        ],
    );
}
