//! 归档的实现输出记录是运行时结构守卫共享的只读证据。以结果断言检查当前接口或源码快照对应的边界。
pub(crate) const RUNTIME_ARCHITECTURE_IMPLEMENTATION_OUTPUT: &str = include_str!(
    "../../../../docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-code-structure-and-module-conventions-output-records.md"
);

#[test]
fn runtime_architecture_implementation_output_is_tracked_plan_evidence() {
    for anchor in [
        "# 15-code-structure-and-module-conventions 产出记录归档",
        "runtime_15_tech_stack_route_owner_split_static_passed_cargo_deferred",
        "d10_animation_physics_bridge_call_static_passed_cargo_deferred",
        "d11_animation_physics_test_runtime_fixture_static_passed_cargo_deferred",
    ] {
        assert!(
            RUNTIME_ARCHITECTURE_IMPLEMENTATION_OUTPUT.contains(anchor),
            "tracked runtime architecture output should contain `{anchor}`"
        );
    }
}
