use super::super::assert_contains_all;
use super::{read_repo, read_runtime_src, DEAD_CODE_ALLOW_ATTRIBUTE};

#[test]
fn runtime_15_script_host_value_descriptors_do_not_suppress_dead_code() {
    let builtin_host_modules = read_runtime_src("script/vm/host/builtin_host_modules.rs");
    let script_host_ledger = read_repo("docs/zircon_runtime/script/vm/host/function_ledger.md");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");

    assert!(
        !builtin_host_modules.contains(DEAD_CODE_ALLOW_ATTRIBUTE),
        "script host descriptors should keep value layouts live without dead-code suppression"
    );
    assert_contains_all(
        "script host value descriptor layout sentinel",
        &builtin_host_modules,
        &[
            "struct Vec3",
            "struct ColorRgba",
            "derive(crate::ZirconScriptType)",
            "const _: ((f64, f64, f64), (f64, f64, f64, f64))",
            "(vec3.x, vec3.y, vec3.z)",
            "(color.r, color.g, color.b, color.a)",
            "vec3_length",
            "vec3_dot",
        ],
    );
    assert_contains_all(
        "script host ledger keeps descriptor counts stable",
        &script_host_ledger,
        &[
            "6 host modules, 52 fixed host functions, and 2 fixed script type descriptors",
            "Type `Vec3`",
            "Type `ColorRgba`",
        ],
    );

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("script host ledger", script_host_ledger.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 F12 script host value descriptor dead-code cleanup",
                "runtime_15_script_host_value_descriptors_coremin_check_passed",
                "runtime_15_script_host_value_descriptors_do_not_suppress_dead_code",
            ],
        );
    }
}

#[test]
fn runtime_15_script_reflection_macro_fixtures_do_not_suppress_dead_code() {
    let reflection_docs = read_runtime_src("script/vm/tests/reflection_docs.rs");
    let vm_tests_doc = read_repo("docs/zircon_runtime/script/vm/tests.md");
    let host_reflection_doc = read_repo("docs/zircon_runtime/script/vm/zr_vm_host_reflection.md");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");

    assert!(
        !reflection_docs.contains(DEAD_CODE_ALLOW_ATTRIBUTE),
        "reflection_docs macro fixtures should be exercised by test assertions instead of dead-code suppression"
    );
    assert_contains_all(
        "script reflection macro fixture live reads",
        &reflection_docs,
        &[
            "let test_vec3 = TestVec3",
            "test_vec3.x + test_vec3.y + test_vec3.z",
            "matches!(TestEnum::A, TestEnum::A)",
            "pub fn point_fixture_x() -> f64",
            "Point { x: 3.5 }.x",
            "macro_math::point_fixture_x()",
        ],
    );

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("script VM tests doc", vm_tests_doc.as_str()),
        ("host reflection doc", host_reflection_doc.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 F12 script reflection macro fixture dead-code cleanup",
                "runtime_15_script_reflection_macro_fixture_dead_code_cleanup_static_passed_cargo_deferred",
                "runtime_15_script_reflection_macro_fixtures_do_not_suppress_dead_code",
            ],
        );
    }
}
