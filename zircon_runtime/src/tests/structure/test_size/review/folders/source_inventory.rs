//! 核对审查守卫汇总的父子职责、源码清单及迁移锚点；子串与计数约束保留结构快照，后续合法新增需同步复核预期。
use super::super::super::*;

#[path = "sources/budgets.rs"]
mod budgets;
#[path = "../../../../test_size/review/folders/sources/delegation.rs"]
mod delegation;
#[path = "sources/model.rs"]
mod model;
#[path = "sources/reads.rs"]
mod reads;

pub(super) use model::CodeReviewFindingsSources;

pub(super) const FOLDER_BACKED_SUMMARY_CHILD: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary.rs";
pub(super) const SOURCE_INVENTORY_CHILD: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/source_inventory.rs";
pub(super) const CODE_REVIEW_FINDINGS_LINE_BUDGET: usize = 800;

pub(super) const SOURCE_INVENTORY_FOLDER_BACKED_SPLIT_NAME: &str =
    "Runtime 15 M3 code review findings source inventory folder-backed split";
pub(super) const SOURCE_INVENTORY_FOLDER_BACKED_SPLIT_ID: &str =
    "runtime_15_code_review_findings_source_inventory_folder_backed_static_passed_cargo_deferred";
pub(super) const SOURCE_INVENTORY_STATUS_MIRROR_CHILD_OWNER_SPLIT_NAME: &str =
    "Runtime 15 M3 code review findings source inventory status-mirror child-owner split";
pub(super) const SOURCE_INVENTORY_STATUS_MIRROR_CHILD_OWNER_SPLIT_ID: &str = "runtime_15_code_review_findings_source_inventory_status_mirror_child_owner_split_static_passed_cargo_deferred";
pub(super) const SOURCE_INVENTORY_STATUS_MIRROR_CHILD_OWNER_DATE: &str = "2026-07-04";
pub(super) const SOURCE_INVENTORY_MAP_SOURCE_SYNC_SPLIT_NAME: &str =
    "Runtime 15 M3 code review findings source inventory map-source sync";
pub(super) const SOURCE_INVENTORY_MAP_SOURCE_SYNC_SPLIT_ID: &str =
    "runtime_15_code_review_findings_source_inventory_map_source_sync_static_passed_cargo_deferred";
pub(super) const SOURCE_INVENTORY_MAP_SOURCE_SYNC_DATE: &str = "2026-07-07";

pub(super) const SOURCE_INVENTORY_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "model",
        "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/sources/model.rs",
        "struct CodeReviewFindingsSources",
    ),
    (
        "reads",
        "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/sources/reads.rs",
        "pub(super) fn code_review_findings_sources",
    ),
    (
        "budgets",
        "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/sources/budgets.rs",
        "pub(super) fn assert_code_review_findings_line_budgets",
    ),
    (
        "delegation",
        "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/sources/delegation.rs",
        "runtime_15_code_review_findings_source_inventory_is_child_owner",
    ),
];

pub(super) fn code_review_findings_sources() -> CodeReviewFindingsSources {
    reads::code_review_findings_sources()
}

pub(super) fn assert_code_review_findings_line_budgets(sources: &CodeReviewFindingsSources) {
    budgets::assert_code_review_findings_line_budgets(sources);
}

pub(super) fn source_inventory_child_sources() -> Vec<(&'static str, String)> {
    SOURCE_INVENTORY_CHILDREN
        .iter()
        .map(|(_, path, _)| (*path, read_runtime_src(path)))
        .collect()
}

pub(super) fn source_inventory_child_source_blob() -> String {
    let mut blob = String::new();
    for (_, source) in source_inventory_child_sources() {
        blob.push_str(&source);
        blob.push('\n');
    }
    blob
}

fn review_source_items(path: &str, source: &str) -> (Vec<(String, Vec<String>)>, Vec<String>) {
    use crate::tests::runtime_absorption::structure_convention::rust_source_view::lexical_code_view;
    let code = lexical_code_view(source);
    let test_attribute = concat!("#[", "test]");
    let mut depth = 0isize;
    let mut attributes = Vec::new();
    let mut attribute = String::new();
    let mut attribute_code = String::new();
    let mut attribute_depth = 0isize;
    let mut attribute_opened = false;
    let mut modules = Vec::new();
    let mut tests = Vec::new();
    for (raw, masked) in source.lines().zip(code.lines()) {
        let line = masked.trim();
        if depth == 0 && !line.is_empty() {
            if !attribute.is_empty() || line.starts_with('#') {
                if !attribute.is_empty() {
                    attribute.push('\n');
                    attribute_code.push('\n');
                }
                attribute.push_str(raw.trim());
                attribute_code.push_str(line);
                attribute_opened |= line.bytes().any(|byte| byte == b'[');
                attribute_depth += line.bytes().filter(|byte| *byte == b'[').count() as isize;
                attribute_depth -= line.bytes().filter(|byte| *byte == b']').count() as isize;
                assert!(attribute_depth >= 0, "balanced review attribute");
                if attribute_opened && attribute_depth == 0 {
                    let compact: String = attribute_code
                        .chars()
                        .filter(|character| !character.is_whitespace())
                        .collect();
                    if let Some(meta) = compact.strip_prefix("#![") {
                        let name = meta.split(['(', ']']).next().expect("inner attribute name");
                        if matches!(name.trim_start_matches("r#"), "cfg" | "cfg_attr") {
                            assert!(
                                path == "lib.rs"
                                    && attribute == "#![cfg_attr(all(test, windows), recursion_limit = \"256\")]",
                                "review owners must not hide behind inner cfg/cfg_attr"
                            );
                        }
                        attribute.clear();
                    } else {
                        assert!(compact.starts_with("#["), "complete Rust item attribute");
                        attributes.push(std::mem::take(&mut attribute));
                    }
                    attribute_code.clear();
                    attribute_opened = false;
                }
            } else {
                let item = if let Some(rest) = line.strip_prefix("pub(") {
                    rest.split_once(')')
                        .expect("complete item visibility")
                        .1
                        .trim_start()
                } else {
                    line.strip_prefix("pub ").unwrap_or(line)
                };
                if let Some(module) = item.strip_prefix("mod ") {
                    let name = module
                        .strip_suffix(';')
                        .expect("review modules must be out of line");
                    modules.push((name.trim().to_owned(), attributes.clone()));
                }
                if let Some(function) = item.strip_prefix("fn ") {
                    if attributes
                        .iter()
                        .any(|attribute| attribute == test_attribute)
                    {
                        assert_eq!(
                            attributes,
                            vec![test_attribute.to_owned()],
                            "review tests must have no cfg or ignore"
                        );
                        let name = function.split_once('(').expect("test function signature").0;
                        tests.push(name.trim().to_owned());
                    }
                }
                attributes.clear();
            }
        }
        depth += line.bytes().filter(|byte| *byte == b'{').count() as isize;
        depth -= line.bytes().filter(|byte| *byte == b'}').count() as isize;
        assert!(depth >= 0, "balanced review source");
    }
    assert_eq!(depth, 0, "balanced review source");
    assert_eq!(attribute_depth, 0, "complete review attributes");
    assert!(
        attributes.is_empty() && attribute.is_empty() && attribute_code.is_empty(),
        "attributes must bind an item"
    );
    (modules, tests)
}

fn assert_review_owner_contract(
    path: &str,
    logical: &str,
    expected_modules: &[(&str, &str)],
    expected_tests: &[&str],
    identities: &mut Vec<String>,
) {
    let source = read_runtime_src(path);
    assert!(
        source.lines().count() < 800,
        "{path} must retain its original line budget"
    );
    let (modules, tests) = review_source_items(path, &source);
    let expected_modules: Vec<_> = expected_modules
        .iter()
        .map(|(item, attribute)| (item.to_string(), vec![attribute.to_string()]))
        .collect();
    assert_eq!(
        modules, expected_modules,
        "{path}: exact cfg/path/item module contract"
    );
    let expected_tests: Vec<String> = expected_tests.iter().map(|name| name.to_string()).collect();
    assert_eq!(
        tests, expected_tests,
        "{path}: exact original review function identities"
    );
    identities.extend(tests.into_iter().map(|name| format!("{logical}::{name}")));
}

fn assert_review_mother_edge(path: &str, item: &str, expected_attributes: &[&str]) {
    let source = read_runtime_src(path);
    assert!(
        source.lines().count() < 800,
        "{path} must retain its original line budget"
    );
    let (modules, _) = review_source_items(path, &source);
    let matching: Vec<_> = modules.iter().filter(|(name, _)| name == item).collect();
    assert_eq!(
        matching.len(),
        1,
        "{path}: unique mother module item {item}"
    );
    let expected: Vec<String> = expected_attributes
        .iter()
        .map(|attribute| attribute.to_string())
        .collect();
    assert_eq!(
        matching[0].1, expected,
        "{path}: exact mother cfg/path/item"
    );
}

pub(super) fn assert_complete_review_graph_contract() {
    super::direct_review_assertions::assert_guard_module_graph_contract();
    let mut identities = Vec::new();
    reads::assert_review_graph_part(&mut identities);
    model::assert_review_graph_part(&mut identities);
    budgets::assert_review_graph_part(&mut identities);
    assert_eq!(
        identities.len(),
        84,
        "all original mounted review functions must remain"
    );
    let distinct: std::collections::BTreeSet<_> = identities.iter().collect();
    assert_eq!(
        distinct.len(),
        84,
        "review identities must not be counted twice"
    );
    assert_review_mother_edge(
        "lib.rs",
        "tests",
        &[
            "#[cfg(all(test, feature = \"native-host\"))]",
            "#[path = \"tests/mod.rs\"]",
        ],
    );
    assert_review_mother_edge(
        "tests/mod.rs",
        "runtime_absorption",
        &["#[path = \"contracts/mod.rs\"]"],
    );
    assert_review_mother_edge(
        "tests/contracts/mod.rs",
        "code_review_findings",
        &["#[path = \"review.rs\"]"],
    );
    assert_review_mother_edge(
        "tests/contracts/mod.rs",
        "structure_convention",
        &["#[path = \"structure.rs\"]"],
    );
    assert_review_mother_edge(
        "tests/contracts/structure.rs",
        "test_file_budget",
        &["#[path = \"structure/test_size/mod.rs\"]"],
    );
    assert_review_mother_edge(
        "tests/contracts/structure/test_size/mod.rs",
        "code_review_findings",
        &["#[path = \"review.rs\"]"],
    );
    assert_review_mother_edge(
        "tests/contracts/structure.rs",
        "rust_source_view",
        &["#[path = \"structure/rust_source_view.rs\"]"],
    );
    for path in &[
        "tests/contracts/structure/rust_source_view.rs",
        "tests/contracts/structure/test_size/review.rs",
        "tests/contracts/structure/test_size/review/folders.rs",
        "tests/review/errors/convergence_mounts/asset_parents.rs",
        "tests/review/errors/convergence_mounts/root_child_rows.rs",
        "tests/review/errors/convergence_mounts/root_paths.rs",
        "tests/review/errors/convergence_mounts/root_sources.rs",
        "tests/review/errors/convergence_mounts/runtime_parents.rs",
        "tests/review/errors/loader/routes/lifecycle_paths.rs",
        "tests/review/errors/loader/routes/manifest_sources.rs",
        "tests/review/errors/loader/routes/plugin_descriptor.rs",
        "tests/review/errors/moved_guard_absence/preserved_guards.rs",
        "tests/review/errors/moved_guard_absence/root_paths.rs",
        "tests/review/errors/structure/convergence_mounts/budgets.rs",
        "tests/review/errors/structure/convergence_mounts/top_level.rs",
        "tests/review/errors/structure/loader/routes/abi_surfaces.rs",
        "tests/review/errors/structure/loader/routes/live_host.rs",
        "tests/review/errors/structure/loader/routes/replay_runtime.rs",
        "tests/review/errors/structure/loader/routes/top_level.rs",
        "tests/review/errors/structure/moved_guard_absence.rs",
        "tests/review/errors/structure/parent_backflow/guard_body.rs",
        "tests/review/errors/structure/parent_backflow/guard_names.rs",
        "tests/review/errors/structure/parent_backflow/parent_paths.rs",
        "tests/review/errors/structure/parent_backflow/sources.rs",
        "tests/review/importers/structure/d13_sdk/parent_mounts.rs",
        "tests/review/importers/structure/d13_sdk/review_children.rs",
        "tests/review/importers/structure/review_mounts/paths.rs",
        "tests/review/late_api_cleanup_owners/root_child_rows.rs",
        "tests/review/late_api_cleanup_owners/root_sources.rs",
        "tests/review/late_api_cleanup_owners/route_ownership.rs",
        "tests/review/structure/d13_sdk/parent_mounts/review_mounts.rs",
        "tests/review/structure/review_mounts/parent_mounts.rs",
        "tests/review/structure/review_mounts/review_children.rs",
        "tests/structure/test_size/review/errors/source_inventory.rs",
        "tests/structure/test_size/review/errors/sources/budgets.rs",
        "tests/structure/test_size/review/errors/sources/paths.rs",
        "tests/structure/test_size/review/errors/sources/reads.rs",
        "tests/structure/test_size/review/errors/structure/loader.rs",
        "tests/structure/test_size/review/folders/assertions.rs",
        "tests/structure/test_size/review/folders/assertions/f12.rs",
        "tests/structure/test_size/review/folders/assertions/f8.rs",
        "tests/structure/test_size/review/folders/assertions/p0.rs",
        "tests/structure/test_size/review/folders/assertions/render.rs",
        "tests/structure/test_size/review/folders/f12/review_guard.rs",
        "tests/structure/test_size/review/folders/f8/parent_mounts.rs",
        "tests/structure/test_size/review/folders/p0/parent_mounts.rs",
        "tests/structure/test_size/review/folders/source_inventory.rs",
        "tests/structure/test_size/review/folders/sources/budgets.rs",
        "tests/structure/test_size/review/folders/sources/model.rs",
        "tests/structure/test_size/review/folders/sources/reads.rs",
        "tests/structure/test_size/review/importers/sources/budgets.rs",
        "tests/structure/test_size/review/importers/sources/paths.rs",
        "tests/structure/test_size/review/importers/sources/reads.rs",
        "tests/structure/test_size/review/typed_error_child_owners.rs",
        "tests/test_size/review/errors/structure/convergence_mounts.rs",
        "tests/test_size/review/errors/structure/loader/metadata.rs",
        "tests/test_size/review/errors/structure/loader/routes.rs",
        "tests/test_size/review/errors/structure/loader/sources.rs",
        "tests/test_size/review/errors/structure_assertions.rs",
        "tests/test_size/review/folders/assertions/root_parent.rs",
        "tests/test_size/review/folders/f8/review_children.rs",
        "tests/test_size/review/folders/p0/review_children.rs",
        "tests/test_size/review/folders/render/review_guard.rs",
        "tests/test_size/review/folders/root_parent/backflow.rs",
        "tests/test_size/review/folders/root_parent/parent_mounts.rs",
        "tests/test_size/review/importers/source_inventory.rs",
        "tests/test_size/review/importers/structure/d13_sdk.rs",
        "tests/test_size/review/importers/structure/d13_sdk/paths.rs",
        "tests/test_size/review/importers/structure/review_mounts.rs",
        "tests/test_size/review/importers/structure_assertions.rs",
        "tests/test_size/review/late_api_cleanup_child_owners.rs",
        "tests/test_size/review/late_api_cleanup_owners/root_paths.rs",
        "tests/test_size/review/plugin_importer_dx_child_owners.rs",
    ] {
        assert!(
            read_runtime_src(path).lines().count() < 800,
            "{path} must retain its original guard budget"
        );
    }
}

pub(super) fn assert_guard_owner_contract(path: &str, expected_modules: &[(&str, &str)]) {
    let source = read_runtime_src(path);
    assert!(
        source.lines().count() < 800,
        "{path} must retain its original guard budget"
    );
    let (modules, _) = review_source_items(path, &source);
    let expected: Vec<_> = expected_modules
        .iter()
        .map(|(item, attribute)| (item.to_string(), vec![attribute.to_string()]))
        .collect();
    assert_eq!(
        modules, expected,
        "{path}: exact reachable guard cfg/path/item contract"
    );
}
