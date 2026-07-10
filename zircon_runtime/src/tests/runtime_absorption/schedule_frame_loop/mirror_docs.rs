use super::runtime_anchors::assert_runtime_03_sources_and_anchors;

const RUNTIME_03_PLAN: &str = include_str!(
    "../../../../../docs/plans/zircon_runtime/runtime/03-schedule-and-frame-loop-alignment.md"
);
const FRAME_SCHEDULE_DOC: &str =
    include_str!("../../../../../docs/zircon_runtime/core/frame_schedule.md");
const RUNTIME_INDEX: &str =
    include_str!("../../../../../docs/plans/zircon_runtime/runtime/index.md");
const M0_REVIEW: &str =
    include_str!("../../../../../docs/engine-architecture/runtime-architecture-review-m0.md");
const INTERFACE_CONVERGENCE: &str =
    include_str!("../../../../../docs/engine-architecture/runtime-interface-convergence.md");

#[test]
fn runtime_03_schedule_frame_loop_mirror_docs_match_structure_audit_counts() {
    let runtime_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    assert_runtime_03_sources_and_anchors(runtime_root);
    assert_mirror_docs_match_structure_audit();
}

fn assert_mirror_docs_match_structure_audit() {
    for (doc_name, doc_source) in [
        ("Runtime 03 plan", RUNTIME_03_PLAN),
        ("frame schedule doc", FRAME_SCHEDULE_DOC),
        ("runtime index", RUNTIME_INDEX),
        ("M0 review", M0_REVIEW),
        ("interface convergence", INTERFACE_CONVERGENCE),
    ] {
        for required_anchor in [
            "schedule_frame_loop_boundary",
            "source files 19/19",
            "guard/test files 11/11",
            "`SystemStage` count and variants 9/9",
            "fixed-loop stages 3/3",
            "dynamic-session `.tick_time(...)` calls 1/1",
            "Runtime 03 guard anchors 14/14",
            "behavior_test_anchor_count = 13",
            "missing_behavior_test_anchors = []",
            "doc_anchors = 10/10",
            "no `WorldDriver` second `advance_time_by(...)` references",
            "no dynamic-session raw-delta level tick references",
            "risks = []",
            "runtime_03_schedule_frame_loop_mirror_docs_match_structure_audit_counts",
        ] {
            assert!(
                doc_source.contains(required_anchor),
                "{doc_name} should mirror Runtime 03 schedule/frame-loop audit anchor `{required_anchor}`"
            );
        }
    }
}
