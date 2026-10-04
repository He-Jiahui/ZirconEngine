use std::{collections::BTreeMap, mem::size_of};

use zircon_runtime::asset::{MeshVertex, VirtualGeometryAsset};
use zircon_runtime::core::framework::render::{
    RenderVirtualGeometryPagePayload, RenderVirtualGeometryPagePayloadClusterRange,
    RenderVirtualGeometryPagePayloadVertex,
};
use zircon_runtime::core::math::{Vec3, Vec4};

const CLUSTER_PAYLOAD_MAGIC: u32 = u32::from_le_bytes(*b"ZVG0");
const PAYLOAD_VERSION: u32 = 1;
const PAYLOAD_HEADER_WORD_COUNT: usize = 7;
const PAYLOAD_ITEM_WORD_COUNT: usize = 4;
const TRIANGLE_INDEX_COUNT: usize = 3;

#[cfg(test)]
#[path = "page_payload/tests/allocation_tests.rs"]
mod allocation_tests;

pub(super) fn render_page_payloads_for_asset(
    asset: &VirtualGeometryAsset,
    vertices: &[MeshVertex],
    indices: &[u32],
    page_remap: &BTreeMap<u32, u32>,
    cluster_remap: &BTreeMap<u32, u32>,
) -> Vec<RenderVirtualGeometryPagePayload> {
    asset
        .cluster_page_headers
        .iter()
        .enumerate()
        .filter_map(|(page_index, header)| {
            let page_id = page_remap.get(&header.page_id).copied()?;
            let payload = asset.cluster_page_data.get(page_index)?;
            let (vertices, cluster_ranges) =
                render_page_vertices(payload, vertices, indices, cluster_remap);
            (!vertices.is_empty()).then(|| {
                RenderVirtualGeometryPagePayload::new(page_id, vertices)
                    .with_cluster_ranges(cluster_ranges)
            })
        })
        .collect()
}

fn render_page_vertices(
    payload: &[u8],
    vertices: &[MeshVertex],
    indices: &[u32],
    cluster_remap: &BTreeMap<u32, u32>,
) -> (
    Vec<RenderVirtualGeometryPagePayloadVertex>,
    Vec<RenderVirtualGeometryPagePayloadClusterRange>,
) {
    let Some(item_count) = payload_item_count(payload) else {
        return (Vec::new(), Vec::new());
    };

    let mut page_vertices =
        Vec::with_capacity(page_vertex_capacity(payload, item_count, vertices, indices));
    let mut cluster_ranges = Vec::with_capacity(item_count);
    for item_index in 0..item_count {
        let Some((triangle_start, triangle_count)) = payload_triangle_range(payload, item_index)
        else {
            continue;
        };
        let vertex_start = page_vertices.len();
        let vertex_count = append_triangle_range_vertices(
            triangle_start,
            triangle_count,
            vertices,
            indices,
            &mut page_vertices,
        );
        if vertex_count > 0 {
            let item_base = PAYLOAD_HEADER_WORD_COUNT
                .saturating_add(item_index.saturating_mul(PAYLOAD_ITEM_WORD_COUNT));
            let local_cluster_id =
                payload_word(payload, item_base.saturating_add(1)).unwrap_or_default();
            let cluster_id = cluster_remap
                .get(&local_cluster_id)
                .copied()
                .unwrap_or(local_cluster_id);
            cluster_ranges.push(RenderVirtualGeometryPagePayloadClusterRange {
                cluster_id,
                vertex_start: u32::try_from(vertex_start).unwrap_or(u32::MAX),
                vertex_count: u32::try_from(vertex_count).unwrap_or(u32::MAX),
            });
        }
    }
    (page_vertices, cluster_ranges)
}

fn payload_item_count(payload: &[u8]) -> Option<usize> {
    if payload_word(payload, 0)? != CLUSTER_PAYLOAD_MAGIC
        || payload_word(payload, 1)? != PAYLOAD_VERSION
    {
        return None;
    }

    let declared_item_count = payload_word(payload, 6)? as usize;
    let available_item_count = (payload.len() / size_of::<u32>())
        .saturating_sub(PAYLOAD_HEADER_WORD_COUNT)
        / PAYLOAD_ITEM_WORD_COUNT;
    Some(declared_item_count.min(available_item_count))
}

fn payload_triangle_range(payload: &[u8], item_index: usize) -> Option<(usize, usize)> {
    let item_base =
        PAYLOAD_HEADER_WORD_COUNT.checked_add(item_index.checked_mul(PAYLOAD_ITEM_WORD_COUNT)?)?;
    Some((
        payload_word(payload, item_base.checked_add(2)?)? as usize,
        payload_word(payload, item_base.checked_add(3)?)? as usize,
    ))
}

fn payload_word(payload: &[u8], word_index: usize) -> Option<u32> {
    let byte_start = word_index.checked_mul(size_of::<u32>())?;
    let bytes = payload.get(byte_start..byte_start.checked_add(size_of::<u32>())?)?;
    Some(u32::from_le_bytes(bytes.try_into().ok()?))
}

fn page_vertex_capacity(
    payload: &[u8],
    item_count: usize,
    vertices: &[MeshVertex],
    indices: &[u32],
) -> usize {
    (0..item_count)
        .filter_map(|item_index| payload_triangle_range(payload, item_index))
        .filter_map(|(triangle_start, triangle_count)| {
            let index_start = triangle_start.checked_mul(TRIANGLE_INDEX_COUNT)?;
            let index_count = triangle_count.checked_mul(TRIANGLE_INDEX_COUNT)?;
            indices.get(index_start..index_start.checked_add(index_count)?)
        })
        .flatten()
        .filter(|source_index| vertices.get(**source_index as usize).is_some())
        .count()
}

fn append_triangle_range_vertices(
    triangle_start: usize,
    triangle_count: usize,
    vertices: &[MeshVertex],
    indices: &[u32],
    page_vertices: &mut Vec<RenderVirtualGeometryPagePayloadVertex>,
) -> usize {
    let index_start = triangle_start.saturating_mul(TRIANGLE_INDEX_COUNT);
    let index_count = triangle_count.saturating_mul(TRIANGLE_INDEX_COUNT);
    let index_end = index_start.saturating_add(index_count);
    let Some(index_slice) = indices.get(index_start..index_end) else {
        return 0;
    };

    let initial_len = page_vertices.len();
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
    page_vertices.len().saturating_sub(initial_len)
}

#[cfg(test)]
#[path = "tests/page_payload.rs"]
mod tests;
