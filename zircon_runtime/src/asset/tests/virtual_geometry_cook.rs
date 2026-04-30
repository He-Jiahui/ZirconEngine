use crate::asset::{
    cook_virtual_geometry_from_mesh, encode_virtual_geometry_cook_binary_dump,
    format_virtual_geometry_cook_bvh_graph_dump, format_virtual_geometry_cook_inspection_dump,
    AssetUri, MeshVertex, ModelAsset, ModelPrimitiveAsset, VirtualGeometryCookConfig,
};
use crate::core::math::{Vec2, Vec3};

#[test]
fn virtual_geometry_cook_builds_stable_four_ary_bvh_pages_and_payload() {
    let (vertices, indices) = five_triangle_mesh();
    let config = VirtualGeometryCookConfig {
        cluster_triangle_count: 1,
        page_cluster_count: 2,
        mesh_name: Some("cook-fixture".to_string()),
        source_hint: Some("unit-test".to_string()),
    };

    let cooked = cook_virtual_geometry_from_mesh(&vertices, &indices, config.clone())
        .expect("mesh should cook into VG data");
    let cooked_again = cook_virtual_geometry_from_mesh(&vertices, &indices, config)
        .expect("mesh should cook deterministically");

    assert_eq!(cooked, cooked_again);
    assert_eq!(cooked.cluster_headers.len(), 8);
    assert_eq!(cooked.root_cluster_ranges.len(), 1);
    assert_eq!(cooked.root_page_table.len(), 1);
    assert!(cooked
        .hierarchy_buffer
        .iter()
        .all(|node| node.child_node_ids.len() <= 4));
    assert!(cooked
        .cluster_page_headers
        .iter()
        .zip(cooked.cluster_page_data.iter())
        .all(|(header, payload)| header.payload_size_bytes == payload.len() as u64));

    let root_range = &cooked.root_cluster_ranges[0];
    let root_cluster = &cooked.cluster_headers[root_range.cluster_start as usize];
    assert_eq!(cooked.root_page_table, vec![root_cluster.page_id]);
    for child in cooked
        .cluster_headers
        .iter()
        .filter(|cluster| cluster.parent_cluster_id.is_some())
    {
        let parent_id = child.parent_cluster_id.unwrap();
        let parent = cooked
            .cluster_headers
            .iter()
            .find(|cluster| cluster.cluster_id == parent_id)
            .expect("parent cluster should exist");
        assert!(
            parent.screen_space_error >= child.screen_space_error,
            "parent cluster error must stay monotonic for automatic LOD"
        );
    }
}

#[test]
fn virtual_geometry_cook_attaches_to_model_primitive_without_dropping_base_mesh() {
    let (vertices, indices) = five_triangle_mesh();
    let cooked = cook_virtual_geometry_from_mesh(
        &vertices,
        &indices,
        VirtualGeometryCookConfig {
            cluster_triangle_count: 2,
            page_cluster_count: 2,
            mesh_name: Some("roundtrip".to_string()),
            source_hint: Some("unit-test".to_string()),
        },
    )
    .expect("mesh should cook into VG data");
    let asset = ModelAsset {
        uri: AssetUri::parse("res://models/cooked.model.toml").unwrap(),
        primitives: vec![ModelPrimitiveAsset {
            vertices: vertices.clone(),
            indices: indices.clone(),
            virtual_geometry: Some(cooked.clone()),
        }],
    };

    let loaded = ModelAsset::from_toml_str(&asset.to_toml_string().unwrap()).unwrap();

    assert_eq!(loaded.primitives[0].vertices, vertices);
    assert_eq!(loaded.primitives[0].indices, indices);
    assert_eq!(loaded.primitives[0].virtual_geometry, Some(cooked));
}

#[test]
fn virtual_geometry_cook_rejects_empty_or_invalid_triangle_input() {
    let (vertices, _) = five_triangle_mesh();

    assert!(
        cook_virtual_geometry_from_mesh(&[], &[], VirtualGeometryCookConfig::default()).is_none()
    );
    assert!(cook_virtual_geometry_from_mesh(
        &vertices,
        &[0, 1, 99],
        VirtualGeometryCookConfig::default()
    )
    .is_none());
}

#[test]
fn virtual_geometry_cook_dump_exposes_teaching_maps_deterministically() {
    let (vertices, indices) = five_triangle_mesh();
    let config = VirtualGeometryCookConfig {
        cluster_triangle_count: 1,
        page_cluster_count: 2,
        mesh_name: Some("dump-fixture".to_string()),
        source_hint: Some("unit-test".to_string()),
    };
    let cooked = cook_virtual_geometry_from_mesh(&vertices, &indices, config)
        .expect("mesh should cook into VG data");

    let dump = format_virtual_geometry_cook_inspection_dump(&cooked);
    let dump_again = format_virtual_geometry_cook_inspection_dump(&cooked);

    assert_eq!(dump, dump_again);
    assert!(dump.starts_with("virtual_geometry_cook_dump version=1\n"));
    assert!(dump.contains("debug mesh_name=\"dump-fixture\" source_hint=\"unit-test\"\n"));
    assert!(dump.contains(
        "counts hierarchy_nodes=8 clusters=8 pages=8 root_pages=1 root_ranges=1 payload_bytes=400\n"
    ));
    assert!(dump.contains("section hierarchy\n"));
    assert!(dump.contains("node id=0 parent=- mip=8 page=1 cluster_start=7 cluster_count=1"));
    assert!(dump.contains("children=[1,4,7]"));
    assert!(dump.contains("section leaf_clusters\n"));
    assert!(dump.contains("leaf_cluster cluster_id=8 node_id=7 mip=10 page=8\n"));
    assert!(dump.contains("section mip_groups\n"));
    assert!(dump.contains("mip level=8 cluster_ids=[1]\n"));
    assert!(dump.contains("mip level=9 cluster_ids=[2,5]\n"));
    assert!(dump.contains("mip level=10 cluster_ids=[3,4,6,7,8]\n"));
    assert!(dump.contains("section page_cluster_map\n"));
    assert!(dump.contains("page_cluster page_id=1 cluster_ids=[1]\n"));
    assert!(dump.contains("section pages\n"));
    assert!(dump.contains(
        "payload page_id=1 magic=0x3047565A version=1 payload_page_id=1 payload_cluster_id=1 leaf_cluster_count=5 page_cluster_count=2 item_count=2 trailing_bytes=0\n"
    ));
    assert!(dump.contains(
        "payload_item page_id=8 item_index=0 node_id=7 cluster_id=8 triangle_start=4 triangle_count=1\n"
    ));
}

#[test]
fn virtual_geometry_cook_bvh_graph_dump_exports_stable_dot_shape() {
    let cooked = five_triangle_cook("graph-fixture");

    let graph = format_virtual_geometry_cook_bvh_graph_dump(&cooked);
    let graph_again = format_virtual_geometry_cook_bvh_graph_dump(&cooked);

    assert_eq!(graph, graph_again);
    assert!(graph.starts_with("digraph virtual_geometry_bvh {\n"));
    assert!(graph.contains("  graph [rankdir=TB];\n"));
    assert!(
        graph.contains("  node_0 [shape=box label=\"node 0\\nmip 8\\npage 1\\nclusters [1]\\nsse ")
    );
    assert!(graph.contains(
        "  node_2 [shape=ellipse label=\"node 2\\nmip 10\\npage 3\\nclusters [3]\\nsse "
    ));
    assert!(graph.contains("  node_0 -> node_1;\n"));
    assert!(graph.contains("  node_1 -> node_2;\n"));
    assert!(graph.ends_with("}\n"));
}

#[test]
fn virtual_geometry_cook_binary_dump_exports_stable_inspection_bytes() {
    let cooked = five_triangle_cook("binary-fixture");

    let dump = encode_virtual_geometry_cook_binary_dump(&cooked);
    let dump_again = encode_virtual_geometry_cook_binary_dump(&cooked);

    assert_eq!(dump, dump_again);
    assert_eq!(&dump[0..4], b"ZVGB");

    let mut cursor = BinaryDumpCursor::new(&dump[4..]);
    assert_eq!(cursor.read_u32(), 1);
    assert_eq!(cursor.read_u32(), cooked.hierarchy_buffer.len() as u32);
    assert_eq!(cursor.read_u32(), cooked.cluster_headers.len() as u32);
    assert_eq!(cursor.read_u32(), cooked.cluster_page_headers.len() as u32);
    assert_eq!(cursor.read_u32(), cooked.root_page_table.len() as u32);
    assert_eq!(cursor.read_u32(), cooked.root_cluster_ranges.len() as u32);
    assert_eq!(cursor.read_u32(), cooked.cluster_page_data.len() as u32);
    assert_eq!(cursor.read_u64(), 400);
    assert_eq!(cursor.read_string(), "binary-fixture");
    assert_eq!(cursor.read_string(), "unit-test");
    assert_eq!(cursor.read_u32(), 1);
    assert_eq!(
        cursor.read_string(),
        "zircon-native cook: 5 leaf cluster(s), 1 triangle(s)/cluster, 2 cluster(s)/page"
    );

    // First hierarchy row is the sorted root node, proving the binary dump is
    // independent of any later consumer-specific traversal order.
    assert_eq!(cursor.read_u32(), 0);
    assert_eq!(cursor.read_u32(), u32::MAX);
    assert_eq!(cursor.read_u32(), 3);
    assert_eq!(cursor.read_u32(), 7);
    assert_eq!(cursor.read_u32(), 1);
    assert_eq!(cursor.read_u32(), 1);
    assert_eq!(cursor.read_u32(), 8);
}

fn five_triangle_mesh() -> (Vec<MeshVertex>, Vec<u32>) {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    for triangle_index in 0..5_u32 {
        let x = triangle_index as f32 * 2.0;
        let base = vertices.len() as u32;
        vertices.push(MeshVertex::new(Vec3::new(x, 0.0, 0.0), Vec3::Y, Vec2::ZERO));
        vertices.push(MeshVertex::new(
            Vec3::new(x + 0.5, 0.0, 0.0),
            Vec3::Y,
            Vec2::X,
        ));
        vertices.push(MeshVertex::new(Vec3::new(x, 0.5, 0.0), Vec3::Y, Vec2::Y));
        indices.extend([base, base + 1, base + 2]);
    }

    (vertices, indices)
}

fn five_triangle_cook(mesh_name: &str) -> crate::asset::VirtualGeometryAsset {
    let (vertices, indices) = five_triangle_mesh();
    cook_virtual_geometry_from_mesh(
        &vertices,
        &indices,
        VirtualGeometryCookConfig {
            cluster_triangle_count: 1,
            page_cluster_count: 2,
            mesh_name: Some(mesh_name.to_string()),
            source_hint: Some("unit-test".to_string()),
        },
    )
    .expect("mesh should cook into VG data")
}

struct BinaryDumpCursor<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> BinaryDumpCursor<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, offset: 0 }
    }

    fn read_u32(&mut self) -> u32 {
        let value = u32::from_le_bytes(
            self.bytes[self.offset..self.offset + 4]
                .try_into()
                .expect("u32 bytes should be present"),
        );
        self.offset += 4;
        value
    }

    fn read_u64(&mut self) -> u64 {
        let value = u64::from_le_bytes(
            self.bytes[self.offset..self.offset + 8]
                .try_into()
                .expect("u64 bytes should be present"),
        );
        self.offset += 8;
        value
    }

    fn read_string(&mut self) -> &str {
        let len = self.read_u32() as usize;
        let value = std::str::from_utf8(&self.bytes[self.offset..self.offset + len])
            .expect("dump strings should be utf8");
        self.offset += len;
        value
    }
}
