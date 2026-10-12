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
}
