const SLICE: &str = "Runtime 15 M3 script host ledger guard folder-backed split";
const STATUS: &str =
    "runtime_15_script_host_ledger_guard_folder_backed_static_passed_cargo_deferred";
const FRAMEWORKS_STATUS: &str =
    "frameworks_02_m3_script_host_ledger_guard_folder_backed_static_passed_cargo_deferred";
const GUARD: &str = "runtime_15_script_host_ledger_guard_is_folder_backed";

const PARENT_PATH: &str = "script_host_ledger.rs";
const CHILD_PATHS: &[&str] = &[
    "script_host_ledger/capability.rs",
    "script_host_ledger/capability_fixture.rs",
    "script_host_ledger/catalog.rs",
    "script_host_ledger/ecs_facade.rs",
    "script_host_ledger/ledger.rs",
    "script_host_ledger/split_layout.rs",
];

#[test]
fn runtime_15_script_host_ledger_guard_is_folder_backed() {
    let parent = include_str!("../script_host_ledger.rs");
    let children = [
        include_str!("capability.rs"),
        include_str!("capability_fixture.rs"),
        include_str!("catalog.rs"),
        include_str!("ecs_facade.rs"),
        include_str!("ledger.rs"),
        include_str!("split_layout.rs"),
    ];

    assert_contains_all(
        "script host ledger parent routes child owners",
        parent,
        &[
            "mod capability;",
            "mod capability_fixture;",
            "mod catalog;",
            "mod ecs_facade;",
            "mod ledger;",
            "mod split_layout;",
        ],
    );

    for moved_anchor in [
        "FIXED_HOST_FUNCTIONS",
        "host_function_registry_matches_documented_ledger",
        "host_capability_representatives_are_declared_on_registered_modules",
        "script_ecs_access_path_stays_on_gameplay_facade_not_native_ecs_abi",
        "registered_bridge_exports",
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "script host ledger parent should not retain moved owner `{moved_anchor}`"
        );
        assert!(
            children.iter().any(|source| source.contains(moved_anchor)),
            "script host ledger children should own moved owner `{moved_anchor}`"
        );
    }

    for (path, source, max_lines) in [
        (PARENT_PATH, parent, 30usize),
        (CHILD_PATHS[0], children[0], 70),
        (CHILD_PATHS[1], children[1], 220),
        (CHILD_PATHS[2], children[2], 130),
        (CHILD_PATHS[3], children[3], 90),
        (CHILD_PATHS[4], children[4], 130),
        (CHILD_PATHS[5], children[5], 170),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count <= max_lines,
            "{path} has {line_count} lines, expected <= {max_lines}"
        );
    }

    #[rustfmt::skip]
    let numbered_records = concat!(
        include_str!("../../../../../docs/plans/zircon_runtime/runtime/13/2026-07-09-script-binding-and-reflection-output-records.md"),
        include_str!("../../../../../docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-code-structure-and-module-conventions-output-records.md"),
        include_str!("../../../../../docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-structure-output-records.md"),
        include_str!("../../../../../docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-review-findings-output-records.md"),
        include_str!("../../../../../docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-runtime-index-output-records.md"),
        include_str!("../../../../../docs/plans/zircon_runtime/frameworks/02/2026-07-09-module-kernel-and-lifecycle-unification-output-records.md")
    );
    assert_contains_all(
        "numbered output records",
        numbered_records,
        &[SLICE, STATUS, GUARD, CHILD_PATHS[5], FRAMEWORKS_STATUS],
    );
    for (label, source) in [(
        "module convention doc",
        include_str!("../../../../../docs/zircon_runtime/structure/module-convention.md"),
    )] {
        assert_contains_all(label, source, &[SLICE, STATUS, GUARD, CHILD_PATHS[5]]);
    }
}

fn assert_contains_all(label: &str, source: &str, needles: &[&str]) {
    let missing = needles
        .iter()
        .copied()
        .filter(|needle| !source.contains(needle))
        .collect::<Vec<_>>();
    assert!(
        missing.is_empty(),
        "{label} missing expected anchors:\n{}",
        missing.join("\n")
    );
}
