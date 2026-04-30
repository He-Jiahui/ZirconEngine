use crate::asset::{
    VirtualGeometryAsset, VirtualGeometryClusterHeaderAsset, VirtualGeometryClusterPageHeaderAsset,
    VirtualGeometryHierarchyNodeAsset,
};

const BINARY_DUMP_MAGIC: &[u8; 4] = b"ZVGB";
const BINARY_DUMP_VERSION: u32 = 1;
const NONE_U32: u32 = u32::MAX;
const MISSING_PAYLOAD_LEN: u64 = u64::MAX;

pub fn encode_virtual_geometry_cook_binary_dump(asset: &VirtualGeometryAsset) -> Vec<u8> {
    let mut dump = Vec::new();
    dump.extend(BINARY_DUMP_MAGIC);
    append_u32(&mut dump, BINARY_DUMP_VERSION);
    append_u32(&mut dump, asset.hierarchy_buffer.len());
    append_u32(&mut dump, asset.cluster_headers.len());
    append_u32(&mut dump, asset.cluster_page_headers.len());
    append_u32(&mut dump, asset.root_page_table.len());
    append_u32(&mut dump, asset.root_cluster_ranges.len());
    append_u32(&mut dump, asset.cluster_page_data.len());
    append_u64(&mut dump, payload_byte_count(asset));

    append_optional_string(&mut dump, asset.debug.mesh_name.as_deref());
    append_optional_string(&mut dump, asset.debug.source_hint.as_deref());
    append_u32(&mut dump, asset.debug.notes.len());
    for note in &asset.debug.notes {
        append_string(&mut dump, note);
    }

    for node in sorted_hierarchy_nodes(asset) {
        append_hierarchy_node(&mut dump, node);
    }
    for cluster in sorted_clusters(asset) {
        append_cluster_header(&mut dump, cluster);
    }
    for (header, _) in sorted_pages(asset) {
        append_page_header(&mut dump, header);
    }
    for page_id in sorted_u32s(asset.root_page_table.clone()) {
        append_u32(&mut dump, page_id);
    }
    let mut root_ranges = asset.root_cluster_ranges.iter().collect::<Vec<_>>();
    root_ranges.sort_by_key(|range| (range.node_id, range.cluster_start));
    for range in root_ranges {
        append_u32(&mut dump, range.node_id);
        append_u32(&mut dump, range.cluster_start);
        append_u32(&mut dump, range.cluster_count);
    }
    for (header, payload) in sorted_pages(asset) {
        append_u32(&mut dump, header.page_id);
        if let Some(payload) = payload {
            append_u64(&mut dump, u64::try_from(payload.len()).unwrap_or(u64::MAX));
            dump.extend(payload);
        } else {
            append_u64(&mut dump, MISSING_PAYLOAD_LEN);
        }
    }

    dump
}

fn append_hierarchy_node(dump: &mut Vec<u8>, node: &VirtualGeometryHierarchyNodeAsset) {
    append_u32(dump, node.node_id);
    append_optional_u32(dump, node.parent_node_id);
    append_u32(dump, node.child_node_ids.len());
    append_u32(dump, node.cluster_start);
    append_u32(dump, node.cluster_count);
    append_u32(dump, node.page_id);
    append_u32(dump, node.mip_level);
    append_f32_array(dump, node.bounds_center);
    append_f32(dump, node.bounds_radius);
    append_f32(dump, node.screen_space_error);
    for child_node_id in &node.child_node_ids {
        append_u32(dump, *child_node_id);
    }
}

fn append_cluster_header(dump: &mut Vec<u8>, cluster: &VirtualGeometryClusterHeaderAsset) {
    append_u32(dump, cluster.cluster_id);
    append_u32(dump, cluster.page_id);
    append_u32(dump, cluster.hierarchy_node_id);
    append_u32(dump, cluster.lod_level);
    append_optional_u32(dump, cluster.parent_cluster_id);
    append_f32_array(dump, cluster.bounds_center);
    append_f32(dump, cluster.bounds_radius);
    append_f32(dump, cluster.screen_space_error);
}

fn append_page_header(dump: &mut Vec<u8>, header: &VirtualGeometryClusterPageHeaderAsset) {
    append_u32(dump, header.page_id);
    append_u32(dump, header.start_offset);
    append_u64(dump, header.payload_size_bytes);
}

fn sorted_hierarchy_nodes(asset: &VirtualGeometryAsset) -> Vec<&VirtualGeometryHierarchyNodeAsset> {
    let mut nodes = asset.hierarchy_buffer.iter().collect::<Vec<_>>();
    nodes.sort_by_key(|node| node.node_id);
    nodes
}

fn sorted_clusters(asset: &VirtualGeometryAsset) -> Vec<&VirtualGeometryClusterHeaderAsset> {
    let mut clusters = asset.cluster_headers.iter().collect::<Vec<_>>();
    clusters.sort_by_key(|cluster| cluster.cluster_id);
    clusters
}

fn sorted_pages(
    asset: &VirtualGeometryAsset,
) -> Vec<(&VirtualGeometryClusterPageHeaderAsset, Option<&[u8]>)> {
    let mut pages = asset
        .cluster_page_headers
        .iter()
        .enumerate()
        .map(|(index, header)| {
            (
                header,
                asset.cluster_page_data.get(index).map(Vec::as_slice),
            )
        })
        .collect::<Vec<_>>();
    pages.sort_by_key(|(header, _)| header.page_id);
    pages
}

fn sorted_u32s(mut values: Vec<u32>) -> Vec<u32> {
    values.sort_unstable();
    values
}

fn payload_byte_count(asset: &VirtualGeometryAsset) -> u64 {
    asset
        .cluster_page_data
        .iter()
        .map(|payload| u64::try_from(payload.len()).unwrap_or(u64::MAX))
        .sum()
}

fn append_optional_string(dump: &mut Vec<u8>, value: Option<&str>) {
    if let Some(value) = value {
        append_string(dump, value);
    } else {
        append_u32_raw(dump, NONE_U32);
    }
}

fn append_string(dump: &mut Vec<u8>, value: &str) {
    append_u32(dump, value.len());
    dump.extend(value.as_bytes());
}

fn append_optional_u32(dump: &mut Vec<u8>, value: Option<u32>) {
    append_u32_raw(dump, value.unwrap_or(NONE_U32));
}

fn append_u32<T>(dump: &mut Vec<u8>, value: T)
where
    T: TryInto<u32>,
{
    append_u32_raw(dump, value.try_into().unwrap_or(u32::MAX));
}

fn append_u32_raw(dump: &mut Vec<u8>, value: u32) {
    dump.extend(value.to_le_bytes());
}

fn append_u64(dump: &mut Vec<u8>, value: u64) {
    dump.extend(value.to_le_bytes());
}

fn append_f32_array(dump: &mut Vec<u8>, value: [f32; 3]) {
    for component in value {
        append_f32(dump, component);
    }
}

fn append_f32(dump: &mut Vec<u8>, value: f32) {
    dump.extend(value.to_le_bytes());
}
