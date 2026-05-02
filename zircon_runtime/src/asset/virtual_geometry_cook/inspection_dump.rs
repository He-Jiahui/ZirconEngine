use std::collections::BTreeMap;
use std::fmt::Write as _;

use crate::asset::{
    VirtualGeometryAsset, VirtualGeometryClusterHeaderAsset, VirtualGeometryClusterPageHeaderAsset,
    VirtualGeometryHierarchyNodeAsset, VirtualGeometryPageDependencyAsset,
};

const DUMP_VERSION: u32 = 1;
const PAYLOAD_HEADER_WORD_COUNT: usize = 7;
const PAYLOAD_ITEM_WORD_COUNT: usize = 4;

pub fn format_virtual_geometry_cook_inspection_dump(asset: &VirtualGeometryAsset) -> String {
    let mut dump = String::new();
    let payload_byte_count = asset.cluster_page_data.iter().map(Vec::len).sum::<usize>();

    write_line(
        &mut dump,
        format_args!("virtual_geometry_cook_dump version={DUMP_VERSION}"),
    );
    write_line(
        &mut dump,
        format_args!(
            "debug mesh_name={} source_hint={}",
            format_optional_label(asset.debug.mesh_name.as_deref()),
            format_optional_label(asset.debug.source_hint.as_deref())
        ),
    );
    for (note_index, note) in asset.debug.notes.iter().enumerate() {
        write_line(
            &mut dump,
            format_args!("debug_note index={note_index} text={note:?}"),
        );
    }
    write_line(
        &mut dump,
        format_args!(
            "counts hierarchy_nodes={} clusters={} pages={} root_pages={} root_ranges={} page_dependencies={} payload_bytes={}",
            asset.hierarchy_buffer.len(),
            asset.cluster_headers.len(),
            asset.cluster_page_headers.len(),
            asset.root_page_table.len(),
            asset.root_cluster_ranges.len(),
            asset.page_dependencies.len(),
            payload_byte_count
        ),
    );

    write_root_pages(&mut dump, asset);
    write_root_ranges(&mut dump, asset);
    write_page_dependencies(&mut dump, asset);
    write_hierarchy(&mut dump, asset);
    write_clusters(&mut dump, asset);
    write_leaf_clusters(&mut dump, asset);
    write_mip_groups(&mut dump, asset);
    write_page_cluster_map(&mut dump, asset);
    write_pages(&mut dump, asset);

    dump
}

fn write_page_dependencies(dump: &mut String, asset: &VirtualGeometryAsset) {
    write_line(dump, format_args!("section page_dependencies"));
    for dependency in sorted_page_dependencies(asset) {
        write_line(
            dump,
            format_args!(
                "page_dependency page_id={} parent={} child_pages={}",
                dependency.page_id,
                format_optional_u32(dependency.parent_page_id),
                format_u32_list(&dependency.child_page_ids)
            ),
        );
    }
}

fn write_root_pages(dump: &mut String, asset: &VirtualGeometryAsset) {
    write_line(dump, format_args!("section root_pages"));
    for (index, page_id) in asset.root_page_table.iter().enumerate() {
        write_line(
            dump,
            format_args!("root_page index={index} page_id={page_id}"),
        );
    }
}

fn write_root_ranges(dump: &mut String, asset: &VirtualGeometryAsset) {
    write_line(dump, format_args!("section root_ranges"));
    for (index, range) in asset.root_cluster_ranges.iter().enumerate() {
        write_line(
            dump,
            format_args!(
                "root_range index={index} node_id={} cluster_start={} cluster_count={}",
                range.node_id, range.cluster_start, range.cluster_count
            ),
        );
    }
}

fn write_hierarchy(dump: &mut String, asset: &VirtualGeometryAsset) {
    write_line(dump, format_args!("section hierarchy"));
    for node in sorted_hierarchy_nodes(asset) {
        write_line(
            dump,
            format_args!(
                "node id={} parent={} mip={} page={} cluster_start={} cluster_count={} children={} bounds_center={} bounds_radius={:.6} sse={:.6}",
                node.node_id,
                format_optional_u32(node.parent_node_id),
                node.mip_level,
                node.page_id,
                node.cluster_start,
                node.cluster_count,
                format_u32_list(&node.child_node_ids),
                format_vec3(node.bounds_center),
                node.bounds_radius,
                node.screen_space_error
            ),
        );
    }
}

fn write_clusters(dump: &mut String, asset: &VirtualGeometryAsset) {
    write_line(dump, format_args!("section clusters"));
    for cluster in sorted_clusters(asset) {
        write_line(
            dump,
            format_args!(
                "cluster id={} node_id={} parent_cluster={} mip={} page={} bounds_center={} bounds_radius={:.6} sse={:.6}",
                cluster.cluster_id,
                cluster.hierarchy_node_id,
                format_optional_u32(cluster.parent_cluster_id),
                cluster.lod_level,
                cluster.page_id,
                format_vec3(cluster.bounds_center),
                cluster.bounds_radius,
                cluster.screen_space_error
            ),
        );
    }
}

fn write_leaf_clusters(dump: &mut String, asset: &VirtualGeometryAsset) {
    write_line(dump, format_args!("section leaf_clusters"));
    let child_counts = child_count_by_node_id(asset);
    for cluster in sorted_clusters(asset) {
        if child_counts
            .get(&cluster.hierarchy_node_id)
            .copied()
            .unwrap_or_default()
            == 0
        {
            write_line(
                dump,
                format_args!(
                    "leaf_cluster cluster_id={} node_id={} mip={} page={}",
                    cluster.cluster_id,
                    cluster.hierarchy_node_id,
                    cluster.lod_level,
                    cluster.page_id
                ),
            );
        }
    }
}

fn write_mip_groups(dump: &mut String, asset: &VirtualGeometryAsset) {
    write_line(dump, format_args!("section mip_groups"));
    let mut groups = BTreeMap::<u8, Vec<u32>>::new();
    for cluster in &asset.cluster_headers {
        groups
            .entry(cluster.lod_level)
            .or_default()
            .push(cluster.cluster_id);
    }
    for (mip_level, cluster_ids) in groups {
        write_line(
            dump,
            format_args!(
                "mip level={mip_level} cluster_ids={}",
                format_u32_list(&sorted_u32s(cluster_ids))
            ),
        );
    }
}

fn write_page_cluster_map(dump: &mut String, asset: &VirtualGeometryAsset) {
    write_line(dump, format_args!("section page_cluster_map"));
    let mut groups = BTreeMap::<u32, Vec<u32>>::new();
    for cluster in &asset.cluster_headers {
        groups
            .entry(cluster.page_id)
            .or_default()
            .push(cluster.cluster_id);
    }
    for (page_id, cluster_ids) in groups {
        write_line(
            dump,
            format_args!(
                "page_cluster page_id={page_id} cluster_ids={}",
                format_u32_list(&sorted_u32s(cluster_ids))
            ),
        );
    }
}

fn write_pages(dump: &mut String, asset: &VirtualGeometryAsset) {
    write_line(dump, format_args!("section pages"));
    let mut pages = asset
        .cluster_page_headers
        .iter()
        .enumerate()
        .map(|(index, header)| (header, asset.cluster_page_data.get(index)))
        .collect::<Vec<_>>();
    pages.sort_by_key(|(header, _)| header.page_id);

    for (header, payload) in pages {
        write_page(dump, header, payload.map(Vec::as_slice));
    }
}

fn write_page(
    dump: &mut String,
    header: &VirtualGeometryClusterPageHeaderAsset,
    payload: Option<&[u8]>,
) {
    let data_size = payload.map_or(0, <[u8]>::len);
    write_line(
        dump,
        format_args!(
            "page id={} offset={} header_bytes={} data_bytes={}",
            header.page_id, header.start_offset, header.payload_size_bytes, data_size
        ),
    );

    let Some(payload) = payload else {
        write_line(
            dump,
            format_args!("payload page_id={} status=missing", header.page_id),
        );
        return;
    };
    let words = payload_u32_words(payload);
    if words.len() < PAYLOAD_HEADER_WORD_COUNT {
        write_line(
            dump,
            format_args!(
                "payload page_id={} status=short word_count={} trailing_bytes={}",
                header.page_id,
                words.len(),
                payload.len() % 4
            ),
        );
        return;
    }

    let item_count = words[6] as usize;
    write_line(
        dump,
        format_args!(
            "payload page_id={} magic=0x{:08X} version={} payload_page_id={} payload_cluster_id={} leaf_cluster_count={} page_cluster_count={} item_count={} trailing_bytes={}",
            header.page_id,
            words[0],
            words[1],
            words[2],
            words[3],
            words[4],
            words[5],
            item_count,
            payload.len() % 4
        ),
    );
    for item_index in 0..item_count {
        let base = PAYLOAD_HEADER_WORD_COUNT + item_index * PAYLOAD_ITEM_WORD_COUNT;
        let item = words.get(base..base + PAYLOAD_ITEM_WORD_COUNT);
        if let Some(item) = item {
            write_line(
                dump,
                format_args!(
                    "payload_item page_id={} item_index={} node_id={} cluster_id={} triangle_start={} triangle_count={}",
                    header.page_id, item_index, item[0], item[1], item[2], item[3]
                ),
            );
        } else {
            write_line(
                dump,
                format_args!(
                    "payload_item page_id={} item_index={} status=short",
                    header.page_id, item_index
                ),
            );
        }
    }
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

fn sorted_page_dependencies(
    asset: &VirtualGeometryAsset,
) -> Vec<&VirtualGeometryPageDependencyAsset> {
    let mut dependencies = asset.page_dependencies.iter().collect::<Vec<_>>();
    dependencies.sort_by_key(|dependency| dependency.page_id);
    dependencies
}

fn child_count_by_node_id(asset: &VirtualGeometryAsset) -> BTreeMap<u32, usize> {
    asset
        .hierarchy_buffer
        .iter()
        .map(|node| (node.node_id, node.child_node_ids.len()))
        .collect()
}

fn sorted_u32s(mut values: Vec<u32>) -> Vec<u32> {
    values.sort_unstable();
    values
}

fn payload_u32_words(payload: &[u8]) -> Vec<u32> {
    payload
        .chunks_exact(4)
        .map(|chunk| u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
        .collect()
}

fn format_optional_label(value: Option<&str>) -> String {
    value.map_or_else(|| "-".to_string(), |value| format!("{value:?}"))
}

fn format_optional_u32(value: Option<u32>) -> String {
    value.map_or_else(|| "-".to_string(), |value| value.to_string())
}

fn format_u32_list(values: &[u32]) -> String {
    let mut formatted = String::from("[");
    for (index, value) in values.iter().enumerate() {
        if index > 0 {
            formatted.push(',');
        }
        write!(formatted, "{value}").expect("writing to String cannot fail");
    }
    formatted.push(']');
    formatted
}

fn format_vec3(value: [f32; 3]) -> String {
    format!("[{:.6},{:.6},{:.6}]", value[0], value[1], value[2])
}

fn write_line(dump: &mut String, args: std::fmt::Arguments<'_>) {
    writeln!(dump, "{args}").expect("writing to String cannot fail");
}
