use super::super::support::{frontmatter_status, runtime_plan_source_with_archive};

#[test]
fn runtime_05_closeout_status_waits_for_full_scene_cargo_gate() {
    let source = runtime_plan_source_with_archive(
        "05",
        include_str!(
        "../../../../../../docs/plans/zircon_runtime/runtime/05-scene-editor-boundary-closeout.md"
    ),
    );
    let source = source.as_str();

    assert_eq!(
        frontmatter_status(source),
        Some("in_progress"),
        "Runtime 05 should not be completed until the full scene:: Cargo gate closes"
    );
    for required_anchor in [
        "pending_full_scene_cargo",
        "cargo test -p zircon_runtime --lib scene:: --locked",
        "frontmatter 从 `completed` 修正为 `in_progress`",
    ] {
        assert!(
            source.contains(required_anchor),
            "Runtime 05 closeout plan should record `{required_anchor}`"
        );
    }
}
