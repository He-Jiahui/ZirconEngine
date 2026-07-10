use super::super::support::runtime_numbered_archive_sources;

#[test]
fn runtime_05_full_scene_failure_clusters_keep_support_first_triage_visible() {
    let archive_source = runtime_numbered_archive_sources();
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
        ("runtime numbered archives", archive_source.as_str()),
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
