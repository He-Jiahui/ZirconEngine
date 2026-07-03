#[path = "m3_structure_support/naming_guard_maps.rs"]
mod naming_guard_maps;
#[path = "m3_structure_support/review_guard_maps.rs"]
mod review_guard_maps;
#[path = "m3_structure_support/status_support_maps.rs"]
mod status_support_maps;

pub(super) fn expected_date_for_slice(slice: &str) -> Option<&'static str> {
    if let Some(date) = review_guard_maps::expected_date_for_slice(slice)
        .or_else(|| naming_guard_maps::expected_date_for_slice(slice))
        .or_else(|| status_support_maps::expected_date_for_slice(slice))
    {
        return Some(date);
    }

    // runtime_15_foundation_guards_row_data_owner_child_split_static_passed_cargo_deferred
    // plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/foundation_guards.rs
    // plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/foundation_guards/dead_code_surface.rs
    // plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/foundation_guards/runtime_structure_tests.rs
    // plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/foundation_guards/plugin_importer_review.rs
    // plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/foundation_guards/plugin_importer_migrations.rs
    // plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/foundation_guards/runtime_absorption_followups.rs
    // runtime_15_foundation_guards_row_data_owner_is_child_backed
    if slice == "Runtime 15 M3 foundation-guards row-data owner child split" {
        Some("2026-07-02")
    // runtime_15_scene_script_row_data_owner_child_split_static_passed_cargo_deferred
    // plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/scene_script_tests.rs
    // plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/scene_script_tests/runtime_07_performance.rs
    // plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/scene_script_tests/script_vm_runtime.rs
    // plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/scene_script_tests/plugin_extension_tests.rs
    // plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/scene_script_tests/script_vm_gameplay_shader.rs
    // plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/scene_script_tests/scene_ecs_tests.rs
    // plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/scene_script_tests/scene_asset_world.rs
    // runtime_15_scene_script_row_data_owner_is_child_backed
    } else if slice == "Runtime 15 M3 scene-script row-data owner child split" {
        Some("2026-07-02")
    // runtime_15_lock_poison_status_row_data_owner_child_split_static_passed_cargo_deferred
    // plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/lock_poison_status.rs
    // plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/lock_poison_status/status_rows.rs
    // plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/lock_poison_status/policy_guards.rs
    // plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/lock_poison_status/core_runtime_recovery.rs
    // plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/lock_poison_status/runtime_services_recovery.rs
    // plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/lock_poison_status/resource_render_input_recovery.rs
    // plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/lock_poison_status/script_vm_recovery.rs
    // runtime_15_lock_poison_status_row_data_owner_is_child_backed
    } else if slice == "Runtime 15 M3 lock-poison status row-data owner child split" {
        Some("2026-07-02")
    } else if slice == "Runtime 15 M3 graphics dead-code guard module split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 graphics dead-code guard child-owner split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 graphics dead-code guard forbidden attribute literal cleanup"
    {
        Some("2026-06-27")
    } else if slice == "Runtime 15 M3 provider boilerplate guard module split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 provider boilerplate guard child-owner split" {
        Some("2026-06-24")
    } else if slice == "Runtime 15 M3 facade surface guard module split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 runtime dead-code guard module split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 runtime dead-code guard forbidden attribute literal cleanup" {
        Some("2026-06-24")
    } else if slice == "Runtime 15 M3 runtime dead-code guard child-owner split" {
        Some("2026-06-29")
    } else if slice == "Runtime 15 M3 runtime dead-code documentation anchor cleanup" {
        Some("2026-06-29")
    } else if slice == "Runtime 15 M3 runtime dead-code module-gate status wording cleanup" {
        Some("2026-06-29")
    } else if slice == "Runtime 15 M3 runtime dead-code production-gate status wording cleanup" {
        Some("2026-06-29")
    } else if slice == "Runtime 15 M3 diagnostics guard module split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 core framework test folder split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 picking test folder split" {
        Some("2026-06-24")
    } else if slice == "Runtime 15 M3 core runtime deactivation blocked test folder split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 lock-poison status row-data child-owner split" {
        Some("2026-06-28")
    } else if slice == "Runtime 15 M3 asset/render/input lock-poison guard child-owner split" {
        Some("2026-07-01")
    } else if slice == "Runtime 15 M3 runtime services lock-poison guard child-owner split" {
        Some("2026-07-01")
    } else if slice == "Runtime 15 M3 module-convention status row-data child-owner split" {
        Some("2026-06-28")
    } else if slice == "Runtime 15 M3 module convention module-doc frontmatter uniqueness guard" {
        Some("2026-07-03")
    } else if slice == "Runtime 15 M3 module convention gate output contract" {
        Some("2026-06-27")
    } else if slice == "Runtime 15 M3 module convention non-render debt guard" {
        Some("2026-06-27")
    } else if slice == "Runtime 15 M3 render-scoped migration debt handoff gate" {
        Some("2026-06-27")
    } else if slice == "Runtime 15 M3 hard-cutover allowed Hyper policy risk cleanup" {
        Some("2026-06-27")
    } else if slice == "Runtime 15 M3 module convention gate audit-clear status mirror" {
        Some("2026-06-27")
    } else if slice == "Runtime 15 M3 module convention zero-debt revalidation" {
        Some("2026-06-30")
    } else if slice == "Runtime 15 M3 module convention audit script family naming cleanup" {
        Some("2026-06-27")
    } else if slice == "Runtime 15 M3 dynamic scene absorption guard folder split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 input manager test folder split" {
        Some("2026-06-24")
    } else if slice == "Runtime 15 M3 UI architecture test folder split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 UI v2 asset test folder split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 UI shared core test folder split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 UI shared core guard child-owner split" {
        Some("2026-06-25")
    } else if slice == "Runtime 15 M3 UI shared core input visibility child folder split" {
        Some("2026-06-24")
    } else if slice == "Runtime 15 M3 UI shared core scroll mutation child folder split" {
        Some("2026-06-24")
    } else if slice == "Runtime 15 M3 UI shared core layout surface child folder split" {
        Some("2026-06-24")
    } else if slice == "Runtime 15 M3 UI accessibility test folder split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 UI accessibility widget actions test folder split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 UI layout slots test folder split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 UI surface-frame authority test folder split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 UI surface dirty domains test folder split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 UI material layout test folder split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 UI template test folder split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 UI component catalog test folder split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 UI boundary test folder split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 UI boundary ZUI surface projection guard sync" {
        Some("2026-07-03")
    } else if slice == "Runtime 15 M3 UI component state test folder split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 UI component state keyboard test folder split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 UI Material foundation test folder split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 UI event routing test folder split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 UI runtime input reply routes test folder split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 UI runtime input reply route child folder split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 UI runtime input reply table pointer route folder split" {
        Some("2026-06-24")
    } else if slice == "Runtime 15 M3 UI runtime input reply route guard child-owner split" {
        Some("2026-06-25")
    } else if slice == "Runtime 15 M3 runtime diagnostics test folder split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 RHI command list test folder split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 RHI device contract test folder split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 asset pack test folder split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 asset facade test folder split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 asset project zmeta test folder split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 asset project zmeta current 12-test guard sync" {
        Some("2026-07-03")
    } else if slice == "Runtime 15 M3 asset project manager test folder split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 asset project manager current 11-test guard sync" {
        Some("2026-07-03")
    } else if slice == "Runtime 15 M3 asset project flow sample test folder split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 asset project example vampire test folder split" {
        Some("2026-06-24")
    } else if slice == "Runtime 15 M3 asset artifact store test folder split" {
        Some("2026-06-24")
    } else if slice == "Runtime 15 M3 asset material test folder split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 asset mesh test root split" {
        Some("2026-06-24")
    } else if slice == "Runtime 15 M3 asset glTF importer test folder split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 asset glTF primitive fixture folder split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 asset importer test folder split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 asset scene test folder split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 asset UI test folder split" {
        Some("2026-06-24")
    } else if slice == "Runtime 15 M3 asset pipeline manager test folder split" {
        Some("2026-06-24")
    } else if slice == "Runtime 15 M3 test file budget guard folder split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 test file budget guard root mod cutover" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 no oversized test files global gate" {
        Some("2026-06-27")
    } else if slice == "Runtime 15 M3 render product mesh-cache morph tests child-owner split" {
        Some("2026-07-01")
    } else if slice == "Runtime 15 M3 UI text layout folder-backed owner split" {
        Some("2026-07-03")
    } else if slice == "Runtime 15 M3 Runtime 07 performance hotspot guard folder split" {
        Some("2026-06-23")
    } else if slice
        == "Runtime 15 M3 Runtime 07 owner-budget virtual-geometry guard child-owner split"
    {
        Some("2026-07-01")
    } else if slice == "Runtime 15 M3 Runtime 07 owner-budget large-file gate child-owner split" {
        Some("2026-07-01")
    } else if slice == "Runtime 15 M3 Runtime 07 owner-budget mirror-docs child-owner split" {
        Some("2026-07-01")
    } else if slice == "Runtime 15 M3 script VM test folder split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 script VM primary guard child-owner split" {
        Some("2026-06-30")
    } else if slice == "Runtime 15 M3 script VM hot-reload coordinator test folder split" {
        Some("2026-06-24")
    } else if slice == "Runtime 15 M3 script VM hot-reload guard child-owner split" {
        Some("2026-06-30")
    } else if slice == "Runtime 15 M3 native live-host tests folder split" {
        Some("2026-06-24")
    } else if slice == "Runtime 15 M3 native plugin loader real fixture test folder split" {
        Some("2026-06-24")
    } else if slice == "Runtime 15 M3 extension registry bridge test folder split" {
        Some("2026-06-24")
    } else if slice == "Runtime 15 M3 manifest contributions test folder split" {
        Some("2026-06-24")
    } else if slice == "Runtime 15 M3 manifest contributions runtime-family test child-owner split"
    {
        Some("2026-07-01")
    } else if slice == "Runtime 15 M3 runtime plugin package manifest test folder split" {
        Some("2026-06-24")
    } else if slice
        == "Runtime 15 M3 runtime plugin package manifest capability-status test child-owner split"
    {
        Some("2026-07-01")
    } else if slice
        == "Runtime 15 M3 runtime plugin catalog feature-dependency report test child-owner split"
    {
        Some("2026-07-01")
    } else if slice == "Runtime 15 M3 runtime plugin lifecycle fixture child-owner split" {
        Some("2026-07-03")
    } else if slice == "Runtime 15 M3 export build plan test folder split" {
        Some("2026-06-24")
    } else if slice
        == "Runtime 15 M3 export build plan profile feature matrix test child-owner split"
    {
        Some("2026-07-01")
    } else if slice == "Runtime 15 M3 export build plan platform test folder split" {
        Some("2026-06-24")
    } else if slice
        == "Runtime 15 M3 export build plan platform release-adapter test child-owner split"
    {
        Some("2026-07-01")
    } else if slice == "Runtime 15 M3 gameplay host test folder split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 script VM gameplay host guard child-owner split" {
        Some("2026-06-30")
    } else if slice == "Runtime 15 M3 shader prewarm manifest test folder split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 scene ECS schedule test folder split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 scene ECS schedule conflict graph child folder split" {
        Some("2026-06-24")
    } else if slice == "Runtime 15 M3 scene ECS systems test folder split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 scene ECS query test folder split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 scene ECS query structure test folder split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 scene derived-state test folder split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 dynamic scene session path-management test folder split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 scene component-structure test folder split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 scene ECS reflect foundation test folder split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 dynamic scene root test folder split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 scene render extract test folder split" {
        Some("2026-06-24")
    } else if slice == "Runtime 15 M3 scene asset integration test folder split" {
        Some("2026-06-24")
    } else if slice == "Runtime 15 M3 scene world basics test folder split" {
        Some("2026-06-24")
    } else if slice == "Runtime 15 M3 scene property paths test folder split" {
        Some("2026-06-24")
    } else {
        None
    }
}
