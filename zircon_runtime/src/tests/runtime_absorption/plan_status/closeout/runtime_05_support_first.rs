#[test]
fn runtime_05_full_scene_failure_clusters_keep_support_first_triage_visible() {
    let runtime_05_plan = include_str!(
        "../../../../../../docs/plans/zircon_runtime/runtime/05-scene-editor-boundary-closeout.md"
    );
    let runtime_index =
        include_str!("../../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let review = include_str!(
        "../../../../../../docs/engine-architecture/runtime-architecture-review-m0.md"
    );
    let convergence =
        include_str!("../../../../../../docs/engine-architecture/runtime-interface-convergence.md");

    let required_anchors = [
        "Runtime 05 scene:: failure support-first triage",
        "graphics-scene-lower-layer-candidate",
        "scene-asset-project-io-lower-layer-candidate",
        "ecs-scene-lower-layer-candidate",
        "support-first-scene-closeout-triage-before-owner-edits",
    ];

    for (label, source) in [
        ("Runtime 05 closeout plan", runtime_05_plan),
        ("runtime index", runtime_index),
        ("M0 architecture review", review),
        ("runtime-interface convergence", convergence),
    ] {
        for required_anchor in required_anchors.iter().copied() {
            assert!(
                source.contains(required_anchor),
                "{label} should keep Runtime 05 support-first triage anchor `{required_anchor}`"
            );
        }
    }
}
