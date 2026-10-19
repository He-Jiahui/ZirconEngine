//! 约束优先计划文档守卫的入口清单与子文件归属；保留名称的文本证据需随职责迁移更新，归档文字不等于当前验证结果。
use super::*;

#[test]
fn runtime_15_priority_plan_docs_guard_tests_stay_listed() {
    for (label, path) in PRIORITY_PLAN_DOCS {
        let source = read_repo(path);
        assert_priority_plan_doc_tests_list_guard_functions(label, path, &source);
    }
}

fn assert_priority_plan_doc_tests_list_guard_functions(label: &str, path: &str, source: &str) {
    let frontmatter = frontmatter_lines(label, path, source);
    let test_entries = frontmatter_section_items(&frontmatter, "tests");

    for entry in PRIORITY_PLAN_DOC_GUARDS {
        assert!(
            test_entries.iter().any(|candidate| candidate == entry),
            "{label} priority plan doc `{path}` should list current guard `{entry}` in frontmatter tests"
        );
    }
}
