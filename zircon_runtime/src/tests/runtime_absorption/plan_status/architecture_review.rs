use super::support::{
    markdown_frontmatter_and_body, runtime_absorption_guard_modules,
    runtime_absorption_plan_status_support_files,
};

#[test]
fn runtime_architecture_review_documents_all_absorption_guards() {
    let review =
        include_str!("../../../../../docs/engine-architecture/runtime-architecture-review-m0.md");
    let (review_frontmatter, review_body) = markdown_frontmatter_and_body(review);
    let guard_modules = runtime_absorption_guard_modules();
    let harness_short_anchor = "runtime_absorption/mod.rs";
    let harness_full_anchor = "zircon_runtime/src/tests/runtime_absorption/mod.rs";

    assert!(
        !guard_modules.is_empty(),
        "runtime_absorption/mod.rs should expose at least one guard module"
    );
    assert!(
        review_frontmatter.contains(harness_full_anchor),
        "runtime architecture review frontmatter should list absorption harness `{harness_full_anchor}`"
    );
    assert!(
        review_body.contains(harness_short_anchor),
        "runtime architecture review body should document absorption harness `{harness_short_anchor}`"
    );
    for module in guard_modules {
        let short_anchor = format!("runtime_absorption/{module}.rs");
        let full_anchor = format!("zircon_runtime/src/tests/{short_anchor}");
        assert!(
            review_frontmatter.contains(&full_anchor),
            "runtime architecture review frontmatter should list guard module `{full_anchor}`"
        );
        assert!(
            review_body.contains(&short_anchor),
            "runtime architecture review body should document guard module `{short_anchor}`"
        );
    }
    for short_anchor in runtime_absorption_plan_status_support_files() {
        let full_anchor = format!("zircon_runtime/src/tests/{short_anchor}");
        assert!(
            review_frontmatter.contains(&full_anchor),
            "runtime architecture review frontmatter should list plan-status support file `{full_anchor}`"
        );
        assert!(
            review_body.contains(&short_anchor),
            "runtime architecture review body should document plan-status support file `{short_anchor}`"
        );
    }
}
