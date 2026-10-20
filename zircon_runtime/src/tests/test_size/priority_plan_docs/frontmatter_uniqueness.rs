//! 检查优先计划文档的头部段落、条目与来源链接；解析遵循本模块的行式约定，文档锚点不承载运行验收结论。
use super::*;

#[test]
fn runtime_15_priority_plan_docs_frontmatter_sections_have_unique_entries() {
    for (label, path) in PRIORITY_PLAN_DOCS {
        let source = read_repo(path);
        assert_priority_plan_doc_frontmatter_entries_are_unique(label, path, &source);
    }
}

fn assert_priority_plan_doc_frontmatter_entries_are_unique(label: &str, path: &str, source: &str) {
    let frontmatter = frontmatter_lines(label, path, source);

    for section in [
        "related_code",
        "implementation_files",
        "plan_sources",
        "tests",
    ] {
        let entries = frontmatter_section_items(&frontmatter, section);
        let mut seen = std::collections::BTreeSet::new();
        let mut duplicates = Vec::new();

        for entry in entries {
            if !seen.insert(entry) {
                duplicates.push(entry);
            }
        }

        assert!(
            duplicates.is_empty(),
            "{label} priority plan doc `{path}` should keep `{section}:` duplicate-free; duplicates: {:?}",
            duplicates
        );
    }
}
