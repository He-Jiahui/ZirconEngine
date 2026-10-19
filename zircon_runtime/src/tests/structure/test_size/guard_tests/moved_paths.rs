//! 约束优先计划文档守卫的入口清单与子文件归属；保留名称的文本证据需随职责迁移更新，归档文字不等于当前验证结果。
use super::*;

const MOVED_PRIORITY_PLAN_DOC_GUARD_PATHS: &[&str] = &[
    "priority_plan_docs/code_paths.rs::runtime_15_priority_plan_docs_code_paths_stay_current",
    "priority_plan_docs/test_paths.rs::runtime_15_priority_plan_docs_test_paths_stay_current",
    "priority_plan_docs/frontmatter_uniqueness.rs::runtime_15_priority_plan_docs_frontmatter_sections_have_unique_entries",
    "priority_plan_docs/header_sections.rs::runtime_15_priority_plan_docs_required_header_sections_stay_complete",
    "priority_plan_docs/plan_sources.rs::runtime_15_priority_plan_docs_plan_sources_stay_cross_linked",
    "priority_plan_docs/guard_tests/listing.rs::runtime_15_priority_plan_docs_guard_tests_stay_listed",
    "priority_plan_docs/guard_tests/child_layout.rs::runtime_15_priority_plan_docs_guard_children_are_folder_backed",
    "priority_plan_docs/guard_tests/inventory_sync.rs::runtime_15_priority_plan_docs_guard_inventory_uses_child_row_data_sources",
    "priority_plan_docs/guard_tests/inventory_sync.rs::runtime_15_priority_plan_docs_listing_prose_names_full_inventory",
    "priority_plan_docs/guard_tests/nested_layout.rs::runtime_15_priority_plan_docs_guard_test_children_are_folder_backed",
    "priority_plan_docs/guard_tests/nested_layout.rs::runtime_15_priority_plan_docs_guard_test_child_prose_names_full_inventory",
    "priority_plan_docs/guard_tests/moved_paths.rs::runtime_15_priority_plan_docs_moved_guard_paths_stay_current",
    "priority_plan_docs/guard_tests/moved_paths.rs::runtime_15_priority_plan_docs_moved_mirror_names_full_inventory",
];
const MOVED_MIRROR_SLICE: &str =
    "Runtime 15 M3 priority plan docs moved mirror full inventory sync";
const MOVED_MIRROR_STATUS: &str =
    "runtime_15_priority_plan_docs_moved_mirror_full_inventory_sync_static_passed_cargo_deferred";
const MOVED_MIRROR_GUARD: &str = "runtime_15_priority_plan_docs_moved_mirror_names_full_inventory";
const MOVED_MIRROR_GUARD_PATH: &str = "priority_plan_docs/guard_tests/moved_paths.rs::runtime_15_priority_plan_docs_moved_mirror_names_full_inventory";

#[test]
fn runtime_15_priority_plan_docs_moved_guard_paths_stay_current() {
    for (label, source) in super::inventory_sync::independent_listing_mirror_sources() {
        assert_contains_all(label, &source, MOVED_PRIORITY_PLAN_DOC_GUARD_PATHS);
    }
    let mut current_names = std::collections::BTreeSet::new();
    for entry in PRIORITY_PLAN_DOC_GUARDS {
        let (path, name) = entry
            .split_once("::")
            .expect("current guard file::fn identity");
        assert!(current_names.insert(name), "duplicate current guard {name}");
        let relative = path
            .strip_prefix("zircon_runtime/src/")
            .expect("runtime guard owner");
        let source = read_runtime_src(relative);
        assert!(
            source.contains(&format!("#[test]\nfn {name}(")),
            "current mounted owner {path} must define actual test {name}"
        );
    }
    let mut historical_names = std::collections::BTreeSet::new();
    for entry in MOVED_PRIORITY_PLAN_DOC_GUARD_PATHS {
        let (_, name) = entry
            .split_once("::")
            .expect("historical guard file::fn identity");
        assert!(
            historical_names.insert(name),
            "duplicate historical guard {name}"
        );
    }
    assert_eq!(
        current_names, historical_names,
        "all thirteen moved guards retain their actual test identities"
    );
}

#[test]
fn runtime_15_priority_plan_docs_moved_mirror_names_full_inventory() {
    let current_owner_archive = priority_plan_doc_current_owner_archive_source();
    assert_moved_mirror_windows_name_full_inventory(
        "priority-plan-doc current-owner archive",
        &current_owner_archive,
    );
}

fn assert_moved_mirror_windows_name_full_inventory(label: &str, source: &str) {
    let lines: Vec<&str> = source.lines().collect();
    let anchor_indexes: Vec<usize> = lines
        .iter()
        .enumerate()
        .filter_map(|(index, line)| line.contains("runtime_15_priority_plan_docs_moved_guard_path_mirror_static_passed_cargo_deferred").then_some(index))
        .collect();

    assert!(
        !anchor_indexes.is_empty(),
        "{label} should keep a moved guard path mirror status anchor"
    );

    for index in anchor_indexes {
        let window = lines[index..usize::min(index + 40, lines.len())].join("\n");
        assert_contains_all(
            label,
            &window,
            &[
                "full priority-plan-doc moved guard inventory",
                "priority_plan_docs/frontmatter_uniqueness.rs::runtime_15_priority_plan_docs_frontmatter_sections_have_unique_entries",
                "priority_plan_docs/guard_tests/inventory_sync.rs::runtime_15_priority_plan_docs_guard_inventory_uses_child_row_data_sources",
                "priority_plan_docs/guard_tests/inventory_sync.rs::runtime_15_priority_plan_docs_listing_prose_names_full_inventory",
                MOVED_MIRROR_GUARD_PATH,
            ],
        );
    }
}
