use super::*;

#[test]
fn streamed_half_resolution_owner_selection_preserves_cardinality_contract() {
    let mut empty = Vec::new();
    assert!(!replace_with_half_resolution_transparent_mesh_pass(&mut empty).unwrap());

    let mut unique = vec![transparent_owner("unique")];
    assert!(replace_with_half_resolution_transparent_mesh_pass(&mut unique).unwrap());
    assert_eq!(
        unique[0].stage_passes[0].executor_id.as_str(),
        HALF_RES_TRANSPARENCY_MESH_EXECUTOR_ID
    );

    let mut duplicate = vec![transparent_owner("first"), transparent_owner("second")];
    let duplicate_error =
        replace_with_half_resolution_transparent_mesh_pass(&mut duplicate).unwrap_err();
    assert!(duplicate_error.contains("exactly one"));
}

fn transparent_owner(name: &str) -> RenderFeatureDescriptor {
    RenderFeatureDescriptor::new(
        name,
        Vec::new(),
        Vec::new(),
        vec![RenderFeaturePassDescriptor::new(
            RenderPassStage::Transparent3d,
            "transparent-mesh",
            QueueLane::Graphics,
        )
        .with_executor_id("mesh.transparent")
        .write_texture_with_ops(
            PostProcessGraphResourceNames::SCENE_COLOR,
            RenderGraphAttachmentOps::load_store(),
        )
        .read_texture(PostProcessGraphResourceNames::SCENE_DEPTH)],
    )
}
