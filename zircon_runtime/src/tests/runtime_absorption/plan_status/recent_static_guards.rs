#[path = "recent_static_guards/document_sources.rs"]
mod document_sources;
#[path = "recent_static_guards/parent_routing.rs"]
mod parent_routing;
#[path = "recent_static_guards/runtime_01_to_04.rs"]
mod runtime_01_to_04;
#[path = "recent_static_guards/runtime_05_to_08.rs"]
mod runtime_05_to_08;
#[path = "recent_static_guards/runtime_09_to_12.rs"]
mod runtime_09_to_12;
#[path = "recent_static_guards/runtime_13_14_review_index.rs"]
mod runtime_13_14_review_index;
#[path = "recent_static_guards/split_layout.rs"]
mod split_layout;

#[test]
fn runtime_recent_static_guard_anchors_stay_recorded_across_plan_docs() {
    let sources = document_sources::RecentStaticGuardSources::load();
    sources.assert_parent_routing();

    runtime_01_to_04::assert_runtime_01_to_04_anchors(&sources);
    runtime_05_to_08::assert_runtime_05_to_08_anchors(&sources);
    runtime_09_to_12::assert_runtime_09_to_12_anchors(&sources);
    runtime_13_14_review_index::assert_runtime_13_14_review_index_anchors(&sources);
}
