#[test]
fn gpu_mesh_upload_does_not_duplicate_vertex_positions() {
    let source = include_str!("../gpu_mesh_resource_from_asset.rs");
    let duplicate_positions_declaration = ["let positions", ": Vec<Vec3>"].concat();

    assert!(!source.contains(&duplicate_positions_declaration));
}

#[test]
fn gpu_mesh_upload_hashes_vertices_during_the_single_conversion_pass() {
    let production = include_str!("../gpu_mesh_resource_from_asset.rs")
        .split_once("#[cfg(test)]")
        .expect("production source and tests must remain separated")
        .0;

    assert!(production.contains("let mut indirect_order_signature = FNV_OFFSET_BASIS;"));
    assert!(production.contains(
        "indirect_order_signature = fnv1a_mesh_vertex(indirect_order_signature, &vertex);"
    ));
    assert!(production.contains("bounds.include_position(vertex.position);"));
    assert!(!production.contains("mesh_bounds(&vertices)"));
    assert!(!production.contains("indirect_order_signature(&payload)"));
    assert!(!production.contains("fn indirect_order_signature("));
}
