use super::super::assert_contains_all;
use super::helpers::{assert_frontmatter_section_has_unique_entries, read_repo};

const MODULE_DOC_FRONTMATTER_UNIQUENESS_STATUS: &str =
    "runtime_15_module_convention_module_doc_frontmatter_uniqueness_static_passed_cargo_deferred";
const MODULE_DOC_FRONTMATTER_UNIQUENESS_SLICE: &str =
    "Runtime 15 M3 module convention module-doc frontmatter uniqueness guard";
const MODULE_DOC_FRONTMATTER_UNIQUENESS_GUARD: &str =
    "runtime_15_module_convention_module_doc_frontmatter_has_unique_entries";

#[test]
fn runtime_15_module_convention_module_doc_frontmatter_has_unique_entries() {
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");

    for section in [
        "related_code",
        "implementation_files",
        "plan_sources",
        "tests",
    ] {
        assert_frontmatter_section_has_unique_entries(
            "module convention docs",
            &module_doc,
            section,
        );
    }

    assert_contains_all(
        "module convention doc frontmatter records status owners",
        &module_doc,
        &[
            "zircon_runtime/src/tests/runtime_absorption/structure_convention/module_convention_gate.rs",
            "zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/module_convention_status.rs",
            "zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/module_convention_status/frontmatter_and_gate_rows.rs",
            "zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support.rs",
            "zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support.rs",
            MODULE_DOC_FRONTMATTER_UNIQUENESS_GUARD,
        ],
    );

    for doc in [
        "docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md",
        "docs/plans/zircon_runtime/runtime/index.md",
        "docs/plans/engine-code-structure-convention.md",
        "docs/plans/engine-code-review-findings-2026-06.md",
        "docs/zircon_runtime/structure/module-convention.md",
        "zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/module_convention_status/frontmatter_and_gate_rows.rs",
    ] {
        let source = read_repo(doc);
        assert_contains_all(
            doc,
            &source,
            &[
                MODULE_DOC_FRONTMATTER_UNIQUENESS_SLICE,
                MODULE_DOC_FRONTMATTER_UNIQUENESS_STATUS,
                MODULE_DOC_FRONTMATTER_UNIQUENESS_GUARD,
                "frontmatter duplicate count 0",
            ],
        );
    }

    let status_map = format!(
        "{}\n{}",
        read_repo(
            "zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support.rs",
        ),
        read_repo(
            "zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/lock_poison_module_maps.rs",
        )
    );
    assert_contains_all(
        "Runtime 15 M3 structure-support status map",
        &status_map,
        &[
            MODULE_DOC_FRONTMATTER_UNIQUENESS_SLICE,
            MODULE_DOC_FRONTMATTER_UNIQUENESS_STATUS,
        ],
    );

    let date_map = format!(
        "{}\n{}",
        read_repo(
            "zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support.rs",
        ),
        read_repo(
            "zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/lock_poison_module_maps.rs",
        )
    );
    assert_contains_all(
        "Runtime 15 M3 structure-support date map",
        &date_map,
        &[MODULE_DOC_FRONTMATTER_UNIQUENESS_SLICE, "2026-07-03"],
    );
}
