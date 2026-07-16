const PARENT_SOURCE: &str = include_str!("../compatibility_shells.rs");
const NESTED_CRATES_SOURCE: &str = include_str!("nested_crates.rs");
const SPLIT_LAYOUT_SOURCE: &str = include_str!("split_layout.rs");

const FRAMEWORKS_02_OUTPUT_RECORDS: &str = include_str!(
    "../../../../../docs/plans/zircon_runtime/frameworks/02/2026-07-09-module-kernel-and-lifecycle-unification-output-records.md"
);
const RUNTIME_15_OUTPUT_RECORDS: &str = include_str!(
    "../../../../../docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-code-structure-and-module-conventions-output-records.md"
);
const MODULE_CONVENTION_DOC: &str =
    include_str!("../../../../../docs/zircon_runtime/structure/module-convention.md");

#[test]
fn runtime_15_compatibility_shells_route_owner_is_folder_backed() {
    assert_contains_all(
        "compatibility_shells route owner",
        PARENT_SOURCE,
        &[
            "#[path = \"compatibility_shells/nested_crates.rs\"]",
            "#[path = \"compatibility_shells/split_layout.rs\"]",
        ],
    );
    assert_parent_route_only();
    assert_child_owner_is_focused();
    assert_line_budget();
    assert_docs_and_status_mirror_split();
}

fn assert_parent_route_only() {
    assert!(
        !PARENT_SOURCE.contains("#[test]"),
        "compatibility_shells.rs should route child owners instead of owning tests"
    );
    for forbidden in [
        "runtime_root.join(\"crates\")",
        "CARGO_MANIFEST_DIR",
        "include_str!(",
    ] {
        assert!(
            !PARENT_SOURCE.contains(forbidden),
            "compatibility_shells.rs should not retain `{forbidden}`"
        );
    }
}

fn assert_child_owner_is_focused() {
    assert_contains_all(
        "nested crates child",
        NESTED_CRATES_SOURCE,
        &[
            "runtime_absorption_does_not_keep_nested_compatibility_shells",
            "runtime_root.join(\"crates\")",
            "nested compatibility crates",
        ],
    );
}

fn assert_line_budget() {
    for (label, source, max_lines) in [
        ("parent route owner", PARENT_SOURCE, 6),
        ("nested crates child", NESTED_CRATES_SOURCE, 16),
        ("split layout guard", SPLIT_LAYOUT_SOURCE, 180),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count <= max_lines,
            "{label} has {line_count} lines; expected at most {max_lines}"
        );
    }
}

fn assert_docs_and_status_mirror_split() {
    assert!(
        RUNTIME_15_OUTPUT_RECORDS.contains(
            "runtime_15_compatibility_shells_route_owner_split_static_passed_cargo_deferred"
        ),
        "Runtime 15 output records should own the compatibility_shells route-owner split status"
    );
    assert_contains_all(
        "module convention doc",
        MODULE_CONVENTION_DOC,
        &[
            "compatibility_shells/nested_crates.rs",
            "compatibility_shells/split_layout.rs",
            "runtime_15_compatibility_shells_route_owner_is_folder_backed",
        ],
    );
    assert_contains_all(
        "Frameworks 02 output records",
        FRAMEWORKS_02_OUTPUT_RECORDS,
        &[
            "frameworks_02_m3_compatibility_shells_route_owner_split_static_passed_cargo_deferred",
            "Runtime 15 M3 compatibility-shells route-owner split",
        ],
    );
}

fn assert_contains_all(label: &str, source: &str, required: &[&str]) {
    for anchor in required {
        assert!(
            source.contains(anchor),
            "{label} should contain split anchor `{anchor}`"
        );
    }
}
