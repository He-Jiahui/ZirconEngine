use std::sync::Arc;

use super::*;

#[test]
fn tiled_plan_clones_share_prepared_mesh_buffers() {
    let plan = RecastBackend::default()
        .prepare_tiled_bake(RecastTiledBakeInput {
            mesh: RecastBakeMeshInput {
                agent_type: "humanoid".to_string(),
                vertices: vec![[-2.0, 0.0, -1.0], [2.0, 0.0, -1.0], [0.0, 0.0, 1.0]],
                indices: vec![0, 1, 2],
                triangle_areas: Vec::new(),
                default_area: 1,
            },
            tile_size: 1.0,
        })
        .unwrap();
    let cloned = plan.clone();

    assert!(Arc::ptr_eq(&plan.mesh, &cloned.mesh));
    assert!(Arc::ptr_eq(&plan.flat_vertices, &cloned.flat_vertices));
    assert!(Arc::ptr_eq(&plan.triangle_areas, &cloned.triangle_areas));
    assert!(Arc::ptr_eq(&plan.tiles, &cloned.tiles));
}
