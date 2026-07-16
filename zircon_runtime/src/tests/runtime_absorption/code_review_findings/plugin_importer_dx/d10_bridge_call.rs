const PHYSICS_FRAMEWORK_MOD: &str =
    include_str!("../../../../../../zircon_runtime/src/core/framework/physics/mod.rs");
const PHYSICS_QUERY_INTERFACE: &str =
    include_str!("../../../../../../zircon_runtime/src/core/framework/physics/query_interface.rs");
const PHYSICS_PLUGIN_SOURCE: &str =
    include_str!("../../../../../../zircon_plugins/physics/runtime/src/plugin.rs");
const PHYSICS_LIB_SOURCE: &str =
    include_str!("../../../../../../zircon_plugins/physics/runtime/src/lib.rs");
const PLUGIN_SDK_REGISTRATION: &str =
    include_str!("../../../../../../zircon_plugins/plugin_sdk/src/registration.rs");
const PLUGIN_SDK_LIB: &str = include_str!("../../../../../../zircon_plugins/plugin_sdk/src/lib.rs");
const PLUGIN_SDK_PRELUDE: &str =
    include_str!("../../../../../../zircon_plugins/plugin_sdk/src/prelude.rs");
const ANIMATION_PHYSICS_TEST: &str = include_str!(
    "../../../../../../zircon_plugins/animation/runtime/tests/runtime_physics_animation_tick_contract.rs"
);
const RUNTIME_HELPERS_FIXTURE: &str = include_str!(
    "../../../../../../zircon_plugins/animation/runtime/tests/runtime_physics_animation_tick_contract/runtime_helpers.rs"
);

#[test]
fn review_d10_animation_physics_tests_use_sdk_bridge_call() {
    let review_findings = include_str!(
        "../../../../../../docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-review-findings-output-records.md"
    );
    let structure_convention = include_str!(
        "../../../../../../docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-structure-output-records.md"
    );
    let plugins_11 =
        include_str!("../../../../../../docs/plans/zircon_plugins/11-plugin-call-bridge.md");
    let plugins_12 = include_str!(
        "../../../../../../docs/plans/zircon_plugins/12/2026-07-09-plugin-dx-and-structure-framework-output-records.md"
    );
    let plugin_sdk_doc = include_str!("../../../../../../docs/zircon_plugins/plugin-sdk.md");
    let physics_doc = include_str!("../../../../../../docs/zircon_plugins/physics/runtime.md");
    let animation_doc = include_str!("../../../../../../docs/zircon_plugins/animation/runtime.md");
    let runtime_15 = crate::tests::runtime_absorption::current_source_fixture::RUNTIME_ARCHITECTURE_IMPLEMENTATION_OUTPUT;
    let runtime_index = include_str!(
        "../../../../../../docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-runtime-index-output-records.md"
    );
    let module_convention =
        include_str!("../../../../../../docs/zircon_runtime/structure/module-convention.md");

    assert_contains_all(
        "runtime physics framework should expose the bridge interface contract",
        PHYSICS_FRAMEWORK_MOD,
        &[
            "mod query_interface;",
            "pub use query_interface::{PhysicsQueryInterface, PHYSICS_QUERY_INTERFACE_ID};",
        ],
    );
    assert_contains_all(
        "physics query bridge interface should be the shared runtime-owned contract",
        PHYSICS_QUERY_INTERFACE,
        &[
            "pub trait PhysicsQueryInterface",
            "pub const PHYSICS_QUERY_INTERFACE_ID: &str = \"physics.query.v1\"",
            "impl PluginInterface for dyn PhysicsQueryInterface",
            "const INTERFACE_ID: &'static str = PHYSICS_QUERY_INTERFACE_ID",
            "fn ray_cast(&self, query: &PhysicsRayCastQuery) -> Vec<PhysicsRayCastHit>;",
            "fn shape_overlap(&self, query: &PhysicsShapeOverlapQuery) -> Vec<PhysicsShapeOverlapHit>;",
            "fn shape_cast(&self, query: &PhysicsShapeCastQuery) -> Vec<PhysicsShapeCastHit>;",
        ],
    );

    assert_contains_all(
        "SDK registration helper should expose owner-tracked bridge export",
        PLUGIN_SDK_REGISTRATION,
        &[
            "pub fn export_interface<T>",
            "T: PluginInterface + ?Sized",
            "self.registry",
            ".export_interface::<T>(self.owner, implementation)",
        ],
    );
    assert_contains_all(
        "SDK root/prelude should expose bridge call types",
        &format!("{PLUGIN_SDK_LIB}\n{PLUGIN_SDK_PRELUDE}"),
        &["PluginInterface", "WeakBridge", "BridgeError"],
    );

    assert_contains_all(
        "physics plugin should declare and export physics.query.v1 through SDK bridge",
        PHYSICS_PLUGIN_SOURCE,
        &[
            "PhysicsQueryInterface",
            ".with_provided_interface_id(PHYSICS_QUERY_INTERFACE_ID)",
            ".export_interface::<dyn PhysicsQueryInterface>(",
            "let manager: Arc<dyn PhysicsQueryInterface>",
        ],
    );
    assert_contains_all(
        "physics runtime public surface should expose the bridge interface id",
        PHYSICS_LIB_SOURCE,
        &["PhysicsQueryInterface"],
    );

    assert_contains_all(
        "animation/physics runtime helper should expose weak bridge fixture helpers",
        RUNTIME_HELPERS_FIXTURE,
        &[
            "pub(super) fn runtime_physics_query_bridge",
            "RuntimeExtensionCatalogReport",
            "WeakBridge<dyn PhysicsQueryInterface>",
            ".resolve_weak::<dyn PhysicsQueryInterface>()",
        ],
    );
    assert_not_contains_any(
        "animation/physics runtime helper should not resolve concrete physics manager",
        RUNTIME_HELPERS_FIXTURE,
        &[
            "runtime_physics_manager(",
            "DefaultPhysicsManager",
            "DEFAULT_PHYSICS_MANAGER_NAME",
            "resolve_manager::<zircon_plugin_physics_runtime::DefaultPhysicsManager>",
        ],
    );
    assert_contains_all(
        "animation/physics contract test should call physics through the weak bridge",
        ANIMATION_PHYSICS_TEST,
        &[
            "let physics_query = runtime_physics_query_bridge(runtime.extension_report());",
            "physics_query",
            ".call(|physics| {",
            "physics.ray_cast(&PhysicsRayCastQuery",
            "physics.shape_overlap(&PhysicsShapeOverlapQuery",
            "physics.shape_cast(&PhysicsShapeCastQuery",
        ],
    );
    assert_not_contains_any(
        "animation/physics contract test should not reach the concrete physics manager",
        ANIMATION_PHYSICS_TEST,
        &[
            "runtime_physics_manager(",
            "DefaultPhysicsManager",
            "DEFAULT_PHYSICS_MANAGER_NAME",
        ],
    );

    let d10_row = review_findings
        .lines()
        .find(|line| line.starts_with("| D10 |"))
        .expect("D10 review finding row should exist");
    assert_contains_all(
        "D10 review row should record bridge-call convergence",
        d10_row,
        &[
            "插件间调用 bridge 已覆盖 animation/physics 查询路径",
            "physics.query.v1",
            "WeakBridge<dyn PhysicsQueryInterface>",
            "d10_animation_physics_bridge_call_static_passed_cargo_deferred",
            "review_d10_animation_physics_tests_use_sdk_bridge_call",
            "M4 / closed",
        ],
    );
    assert_not_contains_any(
        "D10 review row should not keep stale no-bridge wording",
        d10_row,
        &["插件间调用无\"桥\"", "resolve_xxx_manager"],
    );

    for (doc_label, doc) in [
        ("review findings", review_findings),
        ("structure convention", structure_convention),
        ("Plugins 11 plan", plugins_11),
        ("Plugins 12 plan", plugins_12),
        ("plugin SDK doc", plugin_sdk_doc),
        ("physics plugin doc", physics_doc),
        ("animation plugin doc", animation_doc),
        ("Runtime 15 plan", runtime_15),
        ("Runtime index", runtime_index),
        ("module convention", module_convention),
    ] {
        assert_contains_all(
            doc_label,
            doc,
            &[
                "D10 animation/physics bridge call migration",
                "d10_animation_physics_bridge_call_static_passed_cargo_deferred",
                "review_d10_animation_physics_tests_use_sdk_bridge_call",
                "physics.query.v1",
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
