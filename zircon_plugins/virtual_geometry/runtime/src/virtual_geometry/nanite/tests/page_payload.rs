use super::*;
use zircon_runtime::asset::{cook_virtual_geometry_from_mesh, VirtualGeometryCookConfig};
use zircon_runtime::core::math::Vec2;

#[test]
fn render_page_payloads_decode_cooked_triangle_vertices_with_global_page_ids() {
    let vertices = vec![
        MeshVertex::new(Vec3::new(1.0, 0.0, 0.0), Vec3::Y, Vec2::ZERO)
            .with_tangent([1.0, 0.0, 0.0, 1.0]),
        MeshVertex::new(Vec3::new(0.0, 2.0, 0.0), Vec3::Z, Vec2::ZERO)
            .with_tangent([0.0, 1.0, 0.0, -1.0]),
        MeshVertex::new(Vec3::new(0.0, 0.0, 3.0), Vec3::X, Vec2::ZERO)
            .with_tangent([0.0, 0.0, 1.0, 1.0]),
    ];
    let indices = vec![2, 0, 1];
    let asset = cook_virtual_geometry_from_mesh(
        &vertices,
        &indices,
        VirtualGeometryCookConfig {
            cluster_triangle_count: 1,
            page_cluster_count: 1,
            mesh_name: Some("payload-test".to_string()),
            source_hint: Some("unit-test".to_string()),
        },
    )
    .expect("single triangle should cook");
    let local_page_id = asset.cluster_page_headers[0].page_id;
    let page_remap = BTreeMap::from([(local_page_id, 77)]);
    let cluster_remap = BTreeMap::from([(asset.cluster_headers[0].cluster_id, 99)]);

    let payloads =
        render_page_payloads_for_asset(&asset, &vertices, &indices, &page_remap, &cluster_remap);

    assert_eq!(payloads.len(), 1);
    assert_eq!(payloads[0].page_id, 77);
    assert_eq!(payloads[0].vertices.len(), 3);
    assert_eq!(payloads[0].vertices[0].position, Vec3::new(0.0, 0.0, 3.0));
    assert_eq!(payloads[0].vertices[1].position, Vec3::new(1.0, 0.0, 0.0));
    assert_eq!(payloads[0].vertices[2].position, Vec3::new(0.0, 2.0, 0.0));
    assert_eq!(payloads[0].vertices[2].normal, Vec3::Z);
    assert_eq!(
        payloads[0].cluster_ranges,
        vec![RenderVirtualGeometryPagePayloadClusterRange {
            cluster_id: 99,
            vertex_start: 0,
            vertex_count: 3,
        }]
    );
    assert_eq!(
        payloads[0].vertices[1].tangent,
        Vec4::new(1.0, 0.0, 0.0, 1.0)
    );
}
