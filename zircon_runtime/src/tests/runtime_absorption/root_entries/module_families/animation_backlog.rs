#[test]
fn runtime_animation_backlog_boundary_requires_doc_update() {
    let animation_source = include_str!("../../../../animation/mod.rs");
    assert!(
        animation_source.contains("pub use sequence::apply_sequence_to_world;"),
        "animation root should keep the public sequence application hook explicit"
    );

    let sequence_tests = include_str!("../../../../animation/sequence/tests.rs");
    for required_sequence_anchor in [
        "sequence_applies_mesh_renderer_morph_weight_track",
        "MeshRenderer.morph_weights.1",
    ] {
        assert!(
            sequence_tests.contains(required_sequence_anchor),
            "animation sequence tests should keep morph-weight property-track evidence `{required_sequence_anchor}`"
        );
    }

    let boundary_doc = include_str!("../../../../../../docs/zircon_runtime/animation/runtime.md");
    for required_anchor in [
        "Runtime Animation Module",
        "Root motion",
        "Backlog debt",
        "Morph targets",
        "asset/scene property/sequence tracks",
        "not as a dedicated animation-system morph solver",
        "`render` and `graphics` own GPU skinning and draw submission",
        "Editor authoring tools",
        "future expansion must coordinate asset, render, and graphics owners",
        "runtime_animation_backlog_boundary_requires_doc_update",
    ] {
        assert!(
            boundary_doc.contains(required_anchor),
            "animation runtime doc should record `{required_anchor}`"
        );
    }
}
