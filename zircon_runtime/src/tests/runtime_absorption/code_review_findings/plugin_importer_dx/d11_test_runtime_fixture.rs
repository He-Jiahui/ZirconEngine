const ANIMATION_PHYSICS_TEST: &str = include_str!(
    "../../../../../../zircon_plugins/animation/runtime/tests/runtime_physics_animation_tick_contract.rs"
);
const ANIMATION_ASSETS_FIXTURE: &str = include_str!(
    "../../../../../../zircon_plugins/animation/runtime/tests/runtime_physics_animation_tick_contract/animation_assets.rs"
);
const RUNTIME_HELPERS_FIXTURE: &str = include_str!(
    "../../../../../../zircon_plugins/animation/runtime/tests/runtime_physics_animation_tick_contract/runtime_helpers.rs"
);
const TARGET_RESOLUTION_FIXTURE: &str = include_str!(
    "../../../../../../zircon_plugins/animation/runtime/tests/runtime_physics_animation_tick_contract/target_resolution.rs"
);
const PLUGIN_SDK_TEST_RUNTIME: &str =
    include_str!("../../../../../../zircon_plugins/plugin_sdk/src/test.rs");
const PLUGIN_SDK_LIB: &str = include_str!("../../../../../../zircon_plugins/plugin_sdk/src/lib.rs");
const PLUGIN_SDK_PRELUDE: &str =
    include_str!("../../../../../../zircon_plugins/plugin_sdk/src/prelude.rs");

#[test]
fn review_d11_animation_physics_tests_use_sdk_test_runtime_fixture() {
    let review_findings = include_str!(
        "../../../../../../docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-review-findings-output-records.md"
    );
    let structure_convention = include_str!(
        "../../../../../../docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-structure-output-records.md"
    );
    let plugins_12 = include_str!(
        "../../../../../../docs/plans/zircon_plugins/12/2026-07-09-plugin-dx-and-structure-framework-output-records.md"
    );
    let plugin_sdk_doc = include_str!("../../../../../../docs/zircon_plugins/plugin-sdk.md");
    let animation_doc = include_str!("../../../../../../docs/zircon_plugins/animation/runtime.md");
    let runtime_15 = crate::tests::runtime_absorption::current_source_fixture::RUNTIME_ARCHITECTURE_IMPLEMENTATION_OUTPUT;
    let runtime_index = include_str!(
        "../../../../../../docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-runtime-index-output-records.md"
    );
    let module_convention =
        include_str!("../../../../../../docs/zircon_runtime/structure/module-convention.md");

    assert_contains_all(
        "animation/physics integration test should use SDK TestRuntime",
        ANIMATION_PHYSICS_TEST,
        &[
            "#[path = \"runtime_physics_animation_tick_contract/animation_assets.rs\"]",
            "#[path = \"runtime_physics_animation_tick_contract/runtime_helpers.rs\"]",
            "#[path = \"runtime_physics_animation_tick_contract/target_resolution.rs\"]",
            "runtime.create_default_level().unwrap()",
            "runtime.tick_level_seconds(&level,",
        ],
    );
    assert!(
        ANIMATION_PHYSICS_TEST.lines().count() < 1000,
        "animation/physics test owner should stay below the test-file budget"
    );

    assert_contains_all(
        "runtime helper fixture should be SDK-backed",
        RUNTIME_HELPERS_FIXTURE,
        &[
            "use zircon_plugin_sdk::{TestRuntime, WeakBridge};",
            "TestRuntime::builder()",
            ".with_runtime_plugin(&physics_plugin)",
            ".with_runtime_plugin(&animation_plugin)",
            "TestRuntime::builder().build().unwrap()",
            "asset::project_asset_manager_handle(core)",
            "resolve_manager_service(",
        ],
    );
    assert_not_contains_any(
        "runtime helper fixture should not rebuild CoreRuntime manually",
        RUNTIME_HELPERS_FIXTURE,
        &[
            "CoreRuntime::new",
            "RuntimePluginCatalog",
            "RuntimePluginRegistrationReport",
            "RuntimePluginFeatureRegistrationReport",
            "foundation::module_descriptor",
            "scene::create_default_level",
            "register_module(",
            "activate_module(",
            "install_world_runtime_extensions",
            "TEST_MAX_FIXED_STEPS",
            "TEST_FIXED_TIMESTEP_NANOS",
            "fn tick_level",
            "asset::PROJECT_ASSET_MANAGER_NAME",
        ],
    );

    assert_contains_all(
        "animation asset fixture child should own large data constructors",
        ANIMATION_ASSETS_FIXTURE,
        &[
            "pub(super) fn sequence_asset_for_entity",
            "pub(super) fn register_animation_blend_assets",
            "pub(super) fn two_bone_skeleton",
            "pub(super) fn single_hand_translation_clip",
            "pub(super) fn timed_transition_state_machine",
            "ComponentPropertyPath::parse(\"Transform.translation\")",
        ],
    );
    assert!(
        ANIMATION_ASSETS_FIXTURE.lines().count() < 800,
        "animation asset fixture child should stay under the child-file budget"
    );
    assert!(
        RUNTIME_HELPERS_FIXTURE.lines().count() < 800,
        "runtime helper fixture child should stay under the child-file budget"
    );
    assert_contains_all(
        "target resolution fixture child should own target-id fallback tests",
        TARGET_RESOLUTION_FIXTURE,
        &[
            "fn clip_sampling_resolves_track_target_id_before_bone_name_fallback",
            "fn sequence_runtime_resolves_target_id_before_entity_path_fallback",
            "target_id",
            "Wrong/Path",
        ],
    );
    assert!(
        TARGET_RESOLUTION_FIXTURE.lines().count() < 200,
        "target resolution fixture child should stay under the focused child-file budget"
    );

    assert_contains_all(
        "plugin SDK test runtime should expose fixture APIs",
        PLUGIN_SDK_TEST_RUNTIME,
        &[
            "pub struct TestRuntime",
            "pub struct TestRuntimeBuilder",
            "pub fn with_runtime_plugin(mut self, plugin: &dyn RuntimePlugin) -> Self",
            "pub fn create_default_level(&self)",
            "pub fn tick_level_seconds(&self, level: &scene::LevelSystem, seconds: f64)",
            "world_runtime_extension_plan",
            "install_world_runtime_extension_plan",
        ],
    );
    assert_contains_all(
        "plugin SDK root exports should include test runtime fixture",
        &format!("{PLUGIN_SDK_LIB}\n{PLUGIN_SDK_PRELUDE}"),
        &["TestRuntime", "TestRuntimeBuilder", "TestRuntimeBaseModule"],
    );

    let d11_row = review_findings
        .lines()
        .find(|line| line.starts_with("| D11 |"))
        .expect("D11 review finding row should exist");
    assert_contains_all(
        "D11 review row should record migrated state",
        d11_row,
        &[
            "animation/physics 跨插件测试已迁移到 SDK TestRuntime fixture",
            "runtime_physics_animation_tick_contract.rs`（969 行）",
            "runtime_physics_animation_tick_contract/animation_assets.rs`（288 行）",
            "runtime_physics_animation_tick_contract/runtime_helpers.rs`（34 行）",
            "runtime_physics_animation_tick_contract/target_resolution.rs",
            "d11_animation_physics_test_runtime_fixture_static_passed_cargo_deferred",
            "review_d11_animation_physics_tests_use_sdk_test_runtime_fixture",
            "M2 / closed",
        ],
    );
    assert_not_contains_any(
        "D11 review row should not keep stale fixture debt wording",
        d11_row,
        &["手搭 1354 行 CoreRuntime+Scene fixture", "（1354 行）"],
    );

    for (doc_label, doc) in [
        ("review findings", review_findings),
        ("structure convention", structure_convention),
        ("Plugins 12 plan", plugins_12),
        ("plugin SDK doc", plugin_sdk_doc),
        ("animation plugin doc", animation_doc),
        ("Runtime 15 plan", runtime_15),
        ("Runtime index", runtime_index),
        ("module convention", module_convention),
    ] {
        assert_contains_all(
            doc_label,
            doc,
            &[
                "D11 animation/physics TestRuntime fixture migration",
                "d11_animation_physics_test_runtime_fixture_static_passed_cargo_deferred",
                "review_d11_animation_physics_tests_use_sdk_test_runtime_fixture",
                "runtime_physics_animation_tick_contract/runtime_helpers.rs",
                "runtime_physics_animation_tick_contract/target_resolution.rs",
            ],
        );
    }
}

fn assert_contains_all(label: &str, source: &str, needles: &[&str]) {
    for needle in needles {
        assert!(source.contains(needle), "{label} should contain `{needle}`");
    }
}

fn assert_not_contains_any(label: &str, source: &str, needles: &[&str]) {
    for needle in needles {
        assert!(
            !source.contains(needle),
            "{label} should not contain `{needle}`"
        );
    }
}
