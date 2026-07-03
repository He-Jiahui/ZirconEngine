use std::collections::BTreeMap;

use zircon_runtime::asset::{MeshVertex, VirtualGeometryAsset};
use zircon_runtime::core::framework::render::{
    RenderVirtualGeometryPagePayload, RenderVirtualGeometryPagePayloadVertex,
};
use zircon_runtime::core::math::{Vec3, Vec4};

const CLUSTER_PAYLOAD_MAGIC: u32 = u32::from_le_bytes(*b"ZVG0");
const PAYLOAD_VERSION: u32 = 1;
const PAYLOAD_HEADER_WORD_COUNT: usize = 7;
const PAYLOAD_ITEM_WORD_COUNT: usize = 4;
const TRIANGLE_INDEX_COUNT: usize = 3;

pub(super) fn render_page_payloads_for_asset(
    asset: &VirtualGeometryAsset,
    vertices: &[MeshVertex],
    indices: &[u32],
    page_remap: &BTreeMap<u32, u32>,
) -> Vec<RenderVirtualGeometryPagePayload> {
    asset
        .cluster_page_headers
        .iter()
        .enumerate()
        .filter_map(|(page_index, header)| {
            let page_id = page_remap.get(&header.page_id).copied()?;
            let payload = asset.cluster_page_data.get(page_index)?;
            let vertices = render_page_vertices(payload, vertices, indices);
            (!vertices.is_empty()).then(|| RenderVirtualGeometryPagePayload::new(page_id, vertices))
        })
        .collect()
}

fn render_page_vertices(
    payload: &[u8],
    vertices: &[MeshVertex],
    indices: &[u32],
) -> Vec<RenderVirtualGeometryPagePayloadVertex> {
    let words = payload_u32_words(payload);
    if !payload_header_is_supported(&words) {
        return Vec::new();
    }

    let item_count = words[6] as usize;
    let mut page_vertices = Vec::new();
    for item_index in 0..item_count {
        let item_base = PAYLOAD_HEADER_WORD_COUNT + item_index * PAYLOAD_ITEM_WORD_COUNT;
        let Some(item) = words.get(item_base..item_base + PAYLOAD_ITEM_WORD_COUNT) else {
            continue;
        };
        append_triangle_range_vertices(
            item[2] as usize,
            item[3] as usize,
            vertices,
            indices,
            &mut page_vertices,
        );
    }
    page_vertices
}

fn payload_header_is_supported(words: &[u32]) -> bool {
    words.len() >= PAYLOAD_HEADER_WORD_COUNT
        && words[0] == CLUSTER_PAYLOAD_MAGIC
        && words[1] == PAYLOAD_VERSION
}

fn append_triangle_range_vertices(
    triangle_start: usize,
    triangle_count: usize,
    vertices: &[MeshVertex],
    indices: &[u32],
    page_vertices: &mut Vec<RenderVirtualGeometryPagePayloadVertex>,
) {
    let index_start = triangle_start.saturating_mul(TRIANGLE_INDEX_COUNT);
    let index_count = triangle_count.saturating_mul(TRIANGLE_INDEX_COUNT);
    let index_end = index_start.saturating_add(index_count);
    let Some(index_slice) = indices.get(index_start..index_end) else {
        return;
    };

    for source_index in index_slice {
        let Some(vertex) = vertices.get(*source_index as usize).copied() else {
            continue;
        };
        page_vertices.push(RenderVirtualGeometryPagePayloadVertex {
            position: Vec3::from_array(vertex.position),
            normal: Vec3::from_array(vertex.normal),
            tangent: Vec4::from_array(vertex.tangent),
        });
    }
}

fn payload_u32_words(payload: &[u8]) -> Vec<u32> {
    payload
        .chunks_exact(4)
        .map(|chunk| u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
        .collect()
}

#[cfg(test)]
mod tests {
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

        let payloads = render_page_payloads_for_asset(&asset, &vertices, &indices, &page_remap);

        assert_eq!(payloads.len(), 1);
        assert_eq!(payloads[0].page_id, 77);
        assert_eq!(payloads[0].vertices.len(), 3);
        assert_eq!(payloads[0].vertices[0].position, Vec3::new(0.0, 0.0, 3.0));
        assert_eq!(payloads[0].vertices[1].position, Vec3::new(1.0, 0.0, 0.0));
        assert_eq!(payloads[0].vertices[2].position, Vec3::new(0.0, 2.0, 0.0));
        assert_eq!(payloads[0].vertices[2].normal, Vec3::Z);
        assert_eq!(
            payloads[0].vertices[1].tangent,
            Vec4::new(1.0, 0.0, 0.0, 1.0)
        );
    }
}
