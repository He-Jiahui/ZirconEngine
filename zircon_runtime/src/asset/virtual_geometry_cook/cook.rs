use crate::asset::{
    MeshVertex, VirtualGeometryAsset, VirtualGeometryClusterHeaderAsset,
    VirtualGeometryClusterPageHeaderAsset, VirtualGeometryDebugMetadataAsset,
    VirtualGeometryHierarchyNodeAsset, VirtualGeometryPageDependencyAsset,
    VirtualGeometryRootClusterRangeAsset,
};
use crate::core::math::Vec3;

const DEFAULT_CLUSTER_TRIANGLE_COUNT: usize = 64;
const DEFAULT_PAGE_CLUSTER_COUNT: usize = 16;
const TRIANGLE_INDEX_COUNT: usize = 3;
const BVH_CHILD_FANOUT: usize = 4;
const ROOT_MIP_LEVEL: u8 = 0;
const LEAF_MIP_LEVEL: u8 = 10;
const CLUSTER_PAYLOAD_MAGIC: u32 = u32::from_le_bytes(*b"ZVG0");
const PAYLOAD_VERSION: u32 = 1;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VirtualGeometryCookConfig {
    /// Maximum source triangles merged into one leaf cluster before BVH grouping.
    pub cluster_triangle_count: usize,
    /// Maximum child summaries written into one explanatory page payload record.
    pub page_cluster_count: usize,
    pub mesh_name: Option<String>,
    pub source_hint: Option<String>,
}

impl Default for VirtualGeometryCookConfig {
    fn default() -> Self {
        Self {
            cluster_triangle_count: DEFAULT_CLUSTER_TRIANGLE_COUNT,
            page_cluster_count: DEFAULT_PAGE_CLUSTER_COUNT,
            mesh_name: None,
            source_hint: None,
        }
    }
}

pub fn cook_virtual_geometry_from_mesh(
    vertices: &[MeshVertex],
    indices: &[u32],
    config: VirtualGeometryCookConfig,
) -> Option<VirtualGeometryAsset> {
    if vertices.is_empty()
        || indices.len() < TRIANGLE_INDEX_COUNT
        || indices.len() % TRIANGLE_INDEX_COUNT != 0
    {
        return None;
    }
    if indices
        .iter()
        .any(|index| usize::try_from(*index).map_or(true, |index| index >= vertices.len()))
    {
        return None;
    }

    let cluster_triangle_count = config.cluster_triangle_count.max(1);
    let page_cluster_count = config.page_cluster_count.max(1);
    let leaf_sources = build_leaf_cluster_sources(vertices, indices, cluster_triangle_count);
    let mut cooked = CookBuildState::default();
    let root = append_bvh_node(
        &leaf_sources,
        None,
        None,
        None,
        &mut cooked,
        page_cluster_count,
        leaf_sources.len(),
    );

    Some(VirtualGeometryAsset {
        hierarchy_buffer: cooked.hierarchy_buffer,
        cluster_headers: cooked.cluster_headers,
        cluster_page_headers: cooked.cluster_page_headers,
        cluster_page_data: cooked.cluster_page_data,
        root_page_table: vec![root.page_id],
        page_dependencies: cooked.page_dependencies,
        root_cluster_ranges: vec![VirtualGeometryRootClusterRangeAsset {
            node_id: root.node_id,
            cluster_start: root.cluster_start,
            cluster_count: 1,
        }],
        debug: VirtualGeometryDebugMetadataAsset {
            mesh_name: config.mesh_name,
            source_hint: config.source_hint,
            notes: vec![format!(
                "zircon-native cook: {} leaf cluster(s), {} triangle(s)/cluster, {} cluster(s)/page",
                leaf_sources.len(), cluster_triangle_count, page_cluster_count
            )],
        },
    })
}

#[derive(Clone, Debug)]
struct CookLeafClusterSource {
    triangle_start: usize,
    triangle_count: usize,
    bounds_center: Vec3,
    bounds_radius: f32,
    screen_space_error: f32,
}

#[derive(Clone, Copy, Debug)]
struct CookNodeSummary {
    node_id: u32,
    cluster_id: u32,
    cluster_start: u32,
    page_id: u32,
    triangle_start: usize,
    triangle_count: usize,
    screen_space_error: f32,
    mip_level: u8,
}

#[derive(Default)]
struct CookBuildState {
    hierarchy_buffer: Vec<VirtualGeometryHierarchyNodeAsset>,
    cluster_headers: Vec<VirtualGeometryClusterHeaderAsset>,
    cluster_page_headers: Vec<VirtualGeometryClusterPageHeaderAsset>,
    cluster_page_data: Vec<Vec<u8>>,
    next_node_id: u32,
    next_cluster_id: u32,
    next_page_id: u32,
    page_dependencies: Vec<VirtualGeometryPageDependencyAsset>,
}

fn build_leaf_cluster_sources(
    vertices: &[MeshVertex],
    indices: &[u32],
    cluster_triangle_count: usize,
) -> Vec<CookLeafClusterSource> {
    indices
        .chunks(cluster_triangle_count.saturating_mul(TRIANGLE_INDEX_COUNT))
        .enumerate()
        .map(|(cluster_index, cluster_indices)| {
            let (bounds_center, bounds_radius) = cluster_bounds(vertices, cluster_indices);
            CookLeafClusterSource {
                triangle_start: cluster_index.saturating_mul(cluster_triangle_count),
                triangle_count: cluster_indices.len() / TRIANGLE_INDEX_COUNT,
                bounds_center,
                bounds_radius,
                screen_space_error: bounds_radius,
            }
        })
        .collect()
}

fn append_bvh_node(
    sources: &[CookLeafClusterSource],
    parent_node_id: Option<u32>,
    parent_cluster_id: Option<u32>,
    parent_page_id: Option<u32>,
    cooked: &mut CookBuildState,
    page_cluster_count: usize,
    leaf_cluster_count: usize,
) -> CookNodeSummary {
    let node_id = allocate_node_id(cooked);
    let cluster_id = allocate_cluster_id(cooked);
    let page_id = allocate_page_id(cooked);
    let child_summaries = if sources.len() > 1 {
        let chunk_size = sources.len().div_ceil(BVH_CHILD_FANOUT).max(1);
        sources
            .chunks(chunk_size)
            .map(|children| {
                append_bvh_node(
                    children,
                    Some(node_id),
                    Some(cluster_id),
                    Some(page_id),
                    cooked,
                    page_cluster_count,
                    leaf_cluster_count,
                )
            })
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    };
    let child_node_ids = child_summaries
        .iter()
        .map(|child| child.node_id)
        .collect::<Vec<_>>();
    let child_page_ids = child_summaries
        .iter()
        .map(|child| child.page_id)
        .collect::<Vec<_>>();
    let mip_level = if child_summaries.is_empty() {
        LEAF_MIP_LEVEL
    } else {
        child_summaries
            .iter()
            .map(|child| child.mip_level)
            .min()
            .unwrap_or(ROOT_MIP_LEVEL)
            .saturating_sub(1)
    };
    let (bounds_center, bounds_radius) = aggregate_source_bounds(sources);
    let screen_space_error = aggregate_screen_space_error(sources, &child_summaries, bounds_radius);
    let cluster_start = u32::try_from(cooked.cluster_headers.len()).unwrap_or(u32::MAX);
    let triangle_start = sources
        .iter()
        .map(|source| source.triangle_start)
        .min()
        .unwrap_or_default();
    let triangle_count = sources
        .iter()
        .map(|source| source.triangle_count)
        .sum::<usize>();

    cooked
        .hierarchy_buffer
        .push(VirtualGeometryHierarchyNodeAsset {
            node_id,
            parent_node_id,
            child_node_ids,
            cluster_start,
            cluster_count: 1,
            page_id,
            mip_level,
            bounds_center: bounds_center.to_array(),
            bounds_radius,
            screen_space_error,
        });
    cooked
        .cluster_headers
        .push(VirtualGeometryClusterHeaderAsset {
            cluster_id,
            page_id,
            hierarchy_node_id: node_id,
            lod_level: mip_level,
            parent_cluster_id,
            bounds_center: bounds_center.to_array(),
            bounds_radius,
            screen_space_error,
        });
    cooked
        .page_dependencies
        .push(VirtualGeometryPageDependencyAsset {
            page_id,
            parent_page_id,
            child_page_ids,
        });

    let summary = CookNodeSummary {
        node_id,
        cluster_id,
        cluster_start,
        page_id,
        triangle_start,
        triangle_count,
        screen_space_error,
        mip_level,
    };
    append_page_payload(
        cooked,
        summary,
        &child_summaries,
        leaf_cluster_count,
        page_cluster_count,
    );
    summary
}

fn aggregate_source_bounds(sources: &[CookLeafClusterSource]) -> (Vec3, f32) {
    let mut weighted_center = Vec3::ZERO;
    let mut total_triangles = 0usize;
    for source in sources {
        let triangle_count = source.triangle_count.max(1);
        weighted_center += source.bounds_center * triangle_count as f32;
        total_triangles = total_triangles.saturating_add(triangle_count);
    }
    let bounds_center = weighted_center / total_triangles.max(1) as f32;
    let bounds_radius = sources
        .iter()
        .map(|source| bounds_center.distance(source.bounds_center) + source.bounds_radius)
        .fold(0.0_f32, f32::max);
    (bounds_center, bounds_radius)
}

fn aggregate_screen_space_error(
    sources: &[CookLeafClusterSource],
    children: &[CookNodeSummary],
    bounds_radius: f32,
) -> f32 {
    sources
        .iter()
        .map(|source| source.screen_space_error)
        .chain(children.iter().map(|child| child.screen_space_error))
        .fold(bounds_radius, f32::max)
}

fn cluster_bounds(vertices: &[MeshVertex], indices: &[u32]) -> (Vec3, f32) {
    let mut bounds_min = Vec3::splat(f32::INFINITY);
    let mut bounds_max = Vec3::splat(f32::NEG_INFINITY);
    for index in indices {
        let vertex = vertices[*index as usize];
        let position = Vec3::from_array(vertex.position);
        bounds_min = bounds_min.min(position);
        bounds_max = bounds_max.max(position);
    }

    let center = (bounds_min + bounds_max) * 0.5;
    let radius = indices
        .iter()
        .map(|index| Vec3::from_array(vertices[*index as usize].position).distance(center))
        .fold(0.0_f32, f32::max);
    (center, radius)
}

fn append_page_payload(
    cooked: &mut CookBuildState,
    summary: CookNodeSummary,
    children: &[CookNodeSummary],
    leaf_cluster_count: usize,
    page_cluster_count: usize,
) {
    // The first cook payload is intentionally inspection-friendly: each page
    // stores fixed metadata plus child or leaf summaries in little-endian words.
    let payload_item_count = if children.is_empty() {
        1
    } else {
        children.len().min(page_cluster_count)
    };
    let mut payload = Vec::new();
    append_u32(&mut payload, CLUSTER_PAYLOAD_MAGIC);
    append_u32(&mut payload, PAYLOAD_VERSION);
    append_u32(&mut payload, summary.page_id);
    append_u32(&mut payload, summary.cluster_id);
    append_u32(
        &mut payload,
        u32::try_from(leaf_cluster_count).unwrap_or(u32::MAX),
    );
    append_u32(
        &mut payload,
        u32::try_from(page_cluster_count).unwrap_or(u32::MAX),
    );
    append_u32(
        &mut payload,
        u32::try_from(payload_item_count).unwrap_or(u32::MAX),
    );

    if children.is_empty() {
        append_payload_item(&mut payload, summary);
    } else {
        for child in children.iter().take(page_cluster_count) {
            append_payload_item(&mut payload, *child);
        }
    }

    let start_offset = cooked
        .cluster_page_headers
        .iter()
        .map(|page| page.payload_size_bytes)
        .sum::<u64>();
    cooked
        .cluster_page_headers
        .push(VirtualGeometryClusterPageHeaderAsset {
            page_id: summary.page_id,
            start_offset: u32::try_from(start_offset).unwrap_or(u32::MAX),
            payload_size_bytes: u64::try_from(payload.len()).unwrap_or(u64::MAX),
        });
    cooked.cluster_page_data.push(payload);
}

fn append_payload_item(payload: &mut Vec<u8>, summary: CookNodeSummary) {
    append_u32(payload, summary.node_id);
    append_u32(payload, summary.cluster_id);
    append_u32(
        payload,
        u32::try_from(summary.triangle_start).unwrap_or(u32::MAX),
    );
    append_u32(
        payload,
        u32::try_from(summary.triangle_count).unwrap_or(u32::MAX),
    );
}

fn allocate_node_id(cooked: &mut CookBuildState) -> u32 {
    let node_id = cooked.next_node_id;
    cooked.next_node_id = cooked.next_node_id.saturating_add(1);
    node_id
}

fn allocate_cluster_id(cooked: &mut CookBuildState) -> u32 {
    cooked.next_cluster_id = cooked.next_cluster_id.saturating_add(1);
    cooked.next_cluster_id
}

fn allocate_page_id(cooked: &mut CookBuildState) -> u32 {
    cooked.next_page_id = cooked.next_page_id.saturating_add(1);
    cooked.next_page_id
}

fn append_u32(payload: &mut Vec<u8>, value: u32) {
    payload.extend(value.to_le_bytes());
}
