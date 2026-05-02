use std::collections::{HashMap, HashSet};

use crate::virtual_geometry::{build_cluster_selections, VirtualGeometryClusterSelection};
use zircon_runtime::core::framework::render::{
    RenderVirtualGeometryExtract, RenderVirtualGeometrySelectedCluster,
    RenderVirtualGeometryVisBufferMark,
};
use zircon_runtime::core::framework::scene::EntityId;

use super::{
    VirtualGeometryPrepareCluster, VirtualGeometryPrepareClusterState,
    VirtualGeometryPrepareDrawSegment, VirtualGeometryPrepareIndirectDraw,
    VirtualGeometryPreparePage, VirtualGeometryPrepareRequest,
};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct VirtualGeometryPrepareFrame {
    pub visible_entities: Vec<EntityId>,
    pub visible_clusters: Vec<VirtualGeometryPrepareCluster>,
    pub cluster_draw_segments: Vec<VirtualGeometryPrepareDrawSegment>,
    pub resident_pages: Vec<VirtualGeometryPreparePage>,
    pub pending_page_requests: Vec<VirtualGeometryPrepareRequest>,
    pub available_slots: Vec<u32>,
    pub evictable_pages: Vec<VirtualGeometryPreparePage>,
}

impl VirtualGeometryPrepareFrame {
    pub fn unified_indirect_draws(&self) -> Vec<VirtualGeometryPrepareIndirectDraw> {
        let visible_entity_indices = self
            .visible_entities
            .iter()
            .copied()
            .enumerate()
            .map(|(visible_index, entity)| (entity, visible_index))
            .collect::<HashMap<_, _>>();
        let cluster_state = self
            .visible_clusters
            .iter()
            .map(|cluster| {
                (
                    (cluster.entity, cluster.cluster_id),
                    (cluster.page_id, cluster.resident_slot),
                )
            })
            .collect::<HashMap<_, _>>();
        let page_slot = self
            .resident_pages
            .iter()
            .chain(self.evictable_pages.iter())
            .map(|page| (page.page_id, page.slot))
            .collect::<HashMap<_, _>>();
        let request_order_by_page = self
            .pending_page_requests
            .iter()
            .map(|request| (request.page_id, request.frontier_rank))
            .collect::<HashMap<_, _>>();
        let request_submission_slot_by_page = self
            .pending_page_requests
            .iter()
            .map(|request| {
                (
                    request.page_id,
                    request.assigned_slot.or_else(|| {
                        request
                            .recycled_page_id
                            .and_then(|recycled_page_id| page_slot.get(&recycled_page_id).copied())
                    }),
                )
            })
            .collect::<HashMap<_, _>>();
        let explicit_entities = self
            .cluster_draw_segments
            .iter()
            .map(|draw_segment| draw_segment.entity)
            .collect::<HashSet<_>>();
        let mut indirect_draws = self
            .cluster_draw_segments
            .iter()
            .enumerate()
            .filter(|draw_segment| {
                !matches!(
                    draw_segment.1.state,
                    VirtualGeometryPrepareClusterState::Missing
                )
            })
            .map(|(original_index, draw_segment)| {
                let cluster_state = cluster_state
                    .get(&(draw_segment.entity, draw_segment.cluster_id))
                    .copied();
                let page_id = if draw_segment.page_id != 0 {
                    draw_segment.page_id
                } else {
                    cluster_state
                        .map(|(page_id, _resident_slot)| page_id)
                        .unwrap_or_default()
                };
                let resident_slot = draw_segment
                    .resident_slot
                    .or_else(|| cluster_state.and_then(|(_page_id, resident_slot)| resident_slot));
                let submission_slot = resident_slot.or_else(|| {
                    request_submission_slot_by_page
                        .get(&page_id)
                        .copied()
                        .flatten()
                });
                (
                    original_index,
                    VirtualGeometryPrepareIndirectDraw {
                        entity: draw_segment.entity,
                        page_id,
                        cluster_start_ordinal: draw_segment.cluster_ordinal,
                        cluster_span_count: draw_segment.cluster_span_count.max(1),
                        cluster_total_count: draw_segment.cluster_count.max(1),
                        lineage_depth: draw_segment.lineage_depth,
                        lod_level: draw_segment.lod_level,
                        frontier_rank: request_order_by_page
                            .get(&page_id)
                            .copied()
                            .unwrap_or_default(),
                        resident_slot,
                        submission_slot,
                        state: draw_segment.state,
                    },
                )
            })
            .collect::<Vec<_>>();
        indirect_draws.extend(
            fallback_unified_indirect_draws(
                self,
                &visible_entity_indices,
                &explicit_entities,
                &request_order_by_page,
                &request_submission_slot_by_page,
            )
            .into_iter()
            .enumerate()
            .map(|(fallback_index, draw)| {
                (self.cluster_draw_segments.len() + fallback_index, draw)
            }),
        );
        indirect_draws.sort_by_key(|(original_index, draw)| {
            (
                draw.submission_slot.unwrap_or(u32::MAX),
                draw.frontier_rank,
                visible_entity_indices
                    .get(&draw.entity)
                    .copied()
                    .unwrap_or(usize::MAX),
                draw.entity,
                draw.cluster_start_ordinal,
                draw.page_id,
                draw.cluster_span_count,
                draw.cluster_total_count,
                draw.lod_level,
                draw.lineage_depth,
                encode_cluster_state(draw.state),
                *original_index,
            )
        });
        indirect_draws
            .into_iter()
            .map(|(_original_index, draw)| draw)
            .collect()
    }

    pub fn drawable_indirect_segment_count(&self) -> usize {
        self.cluster_draw_segments
            .iter()
            .filter(|segment| !matches!(segment.state, VirtualGeometryPrepareClusterState::Missing))
            .count()
    }

    pub fn selected_clusters(
        &self,
        extract: &RenderVirtualGeometryExtract,
    ) -> Vec<RenderVirtualGeometrySelectedCluster> {
        self.cluster_selections(extract)
            .into_iter()
            .map(VirtualGeometryClusterSelection::to_selected_cluster)
            .collect()
    }

    pub fn cluster_selections(
        &self,
        extract: &RenderVirtualGeometryExtract,
    ) -> Vec<VirtualGeometryClusterSelection> {
        build_cluster_selections(self, extract)
    }

    pub fn same_frame_visbuffer_debug_marks(
        &self,
        extract: &RenderVirtualGeometryExtract,
    ) -> Vec<RenderVirtualGeometryVisBufferMark> {
        if !extract.debug.visualize_visbuffer {
            return Vec::new();
        }

        self.cluster_selections(extract)
            .into_iter()
            .map(VirtualGeometryClusterSelection::to_visbuffer_debug_mark)
            .collect()
    }
}

#[derive(Clone, Copy)]
struct FallbackIndirectCluster {
    entity_cluster_ordinal: usize,
    entity_cluster_total_count: usize,
    page_id: u32,
    frontier_rank: u32,
    resident_slot: Option<u32>,
    submission_slot: Option<u32>,
    lod_level: u8,
    state: VirtualGeometryPrepareClusterState,
}

#[derive(Clone, Copy)]
struct FallbackIndirectDraw {
    entity: EntityId,
    visible_index: usize,
    cluster_ordinal: usize,
    cluster_total_count: usize,
    page_id: u32,
    frontier_rank: u32,
    resident_slot: Option<u32>,
    submission_slot: Option<u32>,
    lod_level: u8,
    state: VirtualGeometryPrepareClusterState,
}

fn fallback_unified_indirect_draws(
    frame: &VirtualGeometryPrepareFrame,
    visible_entity_indices: &HashMap<EntityId, usize>,
    explicit_entities: &HashSet<EntityId>,
    request_order_by_page: &HashMap<u32, u32>,
    request_submission_slot_by_page: &HashMap<u32, Option<u32>>,
) -> Vec<VirtualGeometryPrepareIndirectDraw> {
    let mut clusters_by_entity = HashMap::<EntityId, Vec<FallbackIndirectCluster>>::new();
    let mut entity_cluster_total_count = HashMap::<EntityId, usize>::new();
    for cluster in &frame.visible_clusters {
        *entity_cluster_total_count
            .entry(cluster.entity)
            .or_default() += 1;
    }
    let mut entity_cluster_ordinal = HashMap::<EntityId, usize>::new();
    let clusters_present_by_entity = frame
        .visible_clusters
        .iter()
        .map(|cluster| cluster.entity)
        .collect::<HashSet<_>>();
    for cluster in &frame.visible_clusters {
        let next_cluster_ordinal = entity_cluster_ordinal.entry(cluster.entity).or_default();
        let cluster_ordinal = *next_cluster_ordinal;
        *next_cluster_ordinal += 1;
        if explicit_entities.contains(&cluster.entity)
            || !visible_entity_indices.contains_key(&cluster.entity)
            || matches!(cluster.state, VirtualGeometryPrepareClusterState::Missing)
        {
            continue;
        }
        clusters_by_entity
            .entry(cluster.entity)
            .or_default()
            .push(FallbackIndirectCluster {
                entity_cluster_ordinal: cluster_ordinal,
                entity_cluster_total_count: entity_cluster_total_count
                    .get(&cluster.entity)
                    .copied()
                    .unwrap_or(1),
                page_id: cluster.page_id,
                frontier_rank: request_order_by_page
                    .get(&cluster.page_id)
                    .copied()
                    .unwrap_or_default(),
                resident_slot: cluster.resident_slot,
                submission_slot: cluster.resident_slot.or_else(|| {
                    request_submission_slot_by_page
                        .get(&cluster.page_id)
                        .copied()
                        .flatten()
                }),
                lod_level: cluster.lod_level,
                state: cluster.state,
            });
    }
    for entity in &frame.visible_entities {
        if explicit_entities.contains(entity) {
            continue;
        }
        if clusters_present_by_entity.contains(entity) {
            let Some(clusters) = clusters_by_entity.get_mut(entity) else {
                // Visibility already emitted cluster truth for this entity, but every cluster
                // collapsed to Missing. That is authoritative no-draw, so we must not resurrect
                // a placeholder full-mesh fallback slice here.
                continue;
            };
            clusters.sort_by_key(fallback_cluster_authority_key);
            continue;
        }
        let clusters = clusters_by_entity.entry(*entity).or_insert_with(|| {
            vec![FallbackIndirectCluster {
                entity_cluster_ordinal: 0,
                entity_cluster_total_count: 1,
                page_id: 0,
                frontier_rank: 0,
                resident_slot: None,
                submission_slot: None,
                lod_level: 0,
                state: VirtualGeometryPrepareClusterState::Resident,
            }]
        });
        clusters.sort_by_key(fallback_cluster_authority_key);
    }

    let mut fallback_draws = clusters_by_entity
        .into_iter()
        .flat_map(|(entity, clusters)| {
            let visible_index = visible_entity_indices
                .get(&entity)
                .copied()
                .unwrap_or(usize::MAX);
            let cluster_total_count = clusters.len();
            clusters
                .into_iter()
                .map(move |cluster| FallbackIndirectDraw {
                    entity,
                    visible_index,
                    cluster_ordinal: cluster.entity_cluster_ordinal,
                    cluster_total_count: cluster
                        .entity_cluster_total_count
                        .max(cluster_total_count),
                    page_id: cluster.page_id,
                    frontier_rank: cluster.frontier_rank,
                    resident_slot: cluster.resident_slot,
                    submission_slot: cluster.submission_slot,
                    lod_level: cluster.lod_level,
                    state: cluster.state,
                })
        })
        .collect::<Vec<_>>();
    fallback_draws.sort_by_key(|draw| {
        (
            draw.submission_slot.unwrap_or(u32::MAX),
            draw.frontier_rank,
            draw.visible_index,
            draw.entity,
            draw.cluster_ordinal,
            draw.page_id,
            draw.lod_level,
            encode_cluster_state(draw.state),
        )
    });
    fallback_draws
        .into_iter()
        .map(|draw| VirtualGeometryPrepareIndirectDraw {
            entity: draw.entity,
            page_id: draw.page_id,
            cluster_start_ordinal: draw.cluster_ordinal as u32,
            cluster_span_count: 1,
            cluster_total_count: draw.cluster_total_count.max(1) as u32,
            lineage_depth: 0,
            lod_level: draw.lod_level,
            frontier_rank: draw.frontier_rank,
            resident_slot: draw.resident_slot,
            submission_slot: draw.submission_slot,
            state: draw.state,
        })
        .collect()
}

fn fallback_cluster_authority_key(
    cluster: &FallbackIndirectCluster,
) -> (u32, u32, usize, u32, u8, u32) {
    (
        cluster.submission_slot.unwrap_or(u32::MAX),
        cluster.frontier_rank,
        cluster.entity_cluster_ordinal,
        cluster.page_id,
        cluster.lod_level,
        encode_cluster_state(cluster.state),
    )
}

fn encode_cluster_state(state: VirtualGeometryPrepareClusterState) -> u32 {
    match state {
        VirtualGeometryPrepareClusterState::Resident => 0,
        VirtualGeometryPrepareClusterState::PendingUpload => 1,
        VirtualGeometryPrepareClusterState::Missing => 2,
    }
}

#[cfg(test)]
mod tests {
    use super::VirtualGeometryPrepareFrame;
    use crate::virtual_geometry::{
        VirtualGeometryClusterSelection, VirtualGeometryPrepareClusterState,
        VirtualGeometryPrepareDrawSegment, VirtualGeometryPreparePage,
        VirtualGeometryPrepareRequest,
    };
    use zircon_runtime::core::framework::render::{
        RenderVirtualGeometryCluster, RenderVirtualGeometryDebugState,
        RenderVirtualGeometryExecutionState, RenderVirtualGeometryExtract,
        RenderVirtualGeometryInstance, RenderVirtualGeometrySelectedCluster,
        RenderVirtualGeometryVisBufferMark,
    };
    use zircon_runtime::core::math::{Transform, Vec3};

    #[test]
    fn prepare_frame_exposes_same_frame_visbuffer_marks_from_unified_draw_truth() {
        let entity = 101_u64;
        let extract = RenderVirtualGeometryExtract {
            cluster_budget: 2,
            page_budget: 1,
            clusters: vec![
                RenderVirtualGeometryCluster {
                    entity,
                    cluster_id: 1,
                    hierarchy_node_id: None,
                    page_id: 10,
                    lod_level: 10,
                    parent_cluster_id: None,
                    bounds_center: Vec3::ZERO,
                    bounds_radius: 0.5,
                    screen_space_error: 0.25,
                },
                RenderVirtualGeometryCluster {
                    entity,
                    cluster_id: 2,
                    hierarchy_node_id: None,
                    page_id: 20,
                    lod_level: 10,
                    parent_cluster_id: Some(1),
                    bounds_center: Vec3::X,
                    bounds_radius: 0.5,
                    screen_space_error: 0.2,
                },
            ],
            hierarchy_nodes: Vec::new(),
            hierarchy_child_ids: Vec::new(),
            pages: Vec::new(),
            instances: vec![RenderVirtualGeometryInstance {
                entity,
                source_model: None,
                transform: Transform::default(),
                cluster_offset: 0,
                cluster_count: 2,
                page_offset: 0,
                page_count: 0,
                mesh_name: Some("PrepareVisBufferUnitTest".to_string()),
                source_hint: Some("unit-test".to_string()),
            }],
            debug: RenderVirtualGeometryDebugState {
                visualize_visbuffer: true,
                ..RenderVirtualGeometryDebugState::default()
            },
        };
        let frame = VirtualGeometryPrepareFrame {
            visible_entities: vec![entity],
            visible_clusters: Vec::new(),
            cluster_draw_segments: Vec::new(),
            resident_pages: vec![VirtualGeometryPreparePage {
                page_id: 10,
                slot: 0,
                size_bytes: 4096,
            }],
            pending_page_requests: Vec::new(),
            available_slots: Vec::new(),
            evictable_pages: Vec::new(),
        };

        assert_eq!(
            frame.same_frame_visbuffer_debug_marks(&extract),
            vec![RenderVirtualGeometryVisBufferMark {
                instance_index: Some(0),
                entity,
                cluster_id: 1,
                page_id: 10,
                lod_level: 10,
                state:
                    zircon_runtime::core::framework::render::RenderVirtualGeometryExecutionState::Resident,
                color_rgba: [179, 212, 35, 255],
            }],
            "expected prepare-owned same-frame visbuffer marks to follow unified indirect draw truth before the stored renderer snapshot is backfilled from execution"
        );
    }

    #[test]
    fn prepare_frame_exposes_cluster_selection_from_unified_draw_truth() {
        let entity = 101_u64;
        let extract = RenderVirtualGeometryExtract {
            cluster_budget: 2,
            page_budget: 1,
            clusters: vec![
                RenderVirtualGeometryCluster {
                    entity,
                    cluster_id: 1,
                    hierarchy_node_id: None,
                    page_id: 10,
                    lod_level: 10,
                    parent_cluster_id: None,
                    bounds_center: Vec3::ZERO,
                    bounds_radius: 0.5,
                    screen_space_error: 0.25,
                },
                RenderVirtualGeometryCluster {
                    entity,
                    cluster_id: 2,
                    hierarchy_node_id: None,
                    page_id: 20,
                    lod_level: 10,
                    parent_cluster_id: Some(1),
                    bounds_center: Vec3::X,
                    bounds_radius: 0.5,
                    screen_space_error: 0.2,
                },
            ],
            hierarchy_nodes: Vec::new(),
            hierarchy_child_ids: Vec::new(),
            pages: Vec::new(),
            instances: vec![RenderVirtualGeometryInstance {
                entity,
                source_model: None,
                transform: Transform::default(),
                cluster_offset: 0,
                cluster_count: 2,
                page_offset: 0,
                page_count: 0,
                mesh_name: Some("PrepareClusterSelectionUnitTest".to_string()),
                source_hint: Some("unit-test".to_string()),
            }],
            debug: RenderVirtualGeometryDebugState {
                visualize_visbuffer: true,
                ..RenderVirtualGeometryDebugState::default()
            },
        };
        let frame = VirtualGeometryPrepareFrame {
            visible_entities: vec![entity],
            visible_clusters: Vec::new(),
            cluster_draw_segments: Vec::new(),
            resident_pages: vec![VirtualGeometryPreparePage {
                page_id: 10,
                slot: 0,
                size_bytes: 4096,
            }],
            pending_page_requests: Vec::new(),
            available_slots: Vec::new(),
            evictable_pages: Vec::new(),
        };

        assert_eq!(
            frame.selected_clusters(&extract),
            vec![RenderVirtualGeometrySelectedCluster {
                instance_index: Some(0),
                entity,
                cluster_id: 1,
                cluster_ordinal: 0,
                page_id: 10,
                lod_level: 10,
                state: RenderVirtualGeometryExecutionState::Resident,
            }],
            "expected prepare-owned cluster selection to expose the authoritative current-frame cluster worklist derived from unified indirect draw truth"
        );
    }

    #[test]
    fn prepare_frame_cluster_selections_include_submission_metadata_from_unified_draw_truth() {
        let entity = 101_u64;
        let extract = RenderVirtualGeometryExtract {
            cluster_budget: 3,
            page_budget: 2,
            clusters: vec![
                RenderVirtualGeometryCluster {
                    entity,
                    cluster_id: 1,
                    hierarchy_node_id: None,
                    page_id: 10,
                    lod_level: 10,
                    parent_cluster_id: None,
                    bounds_center: Vec3::ZERO,
                    bounds_radius: 0.5,
                    screen_space_error: 0.25,
                },
                RenderVirtualGeometryCluster {
                    entity,
                    cluster_id: 2,
                    hierarchy_node_id: None,
                    page_id: 20,
                    lod_level: 11,
                    parent_cluster_id: Some(1),
                    bounds_center: Vec3::X,
                    bounds_radius: 0.5,
                    screen_space_error: 0.2,
                },
                RenderVirtualGeometryCluster {
                    entity,
                    cluster_id: 3,
                    hierarchy_node_id: None,
                    page_id: 20,
                    lod_level: 11,
                    parent_cluster_id: Some(1),
                    bounds_center: Vec3::new(2.0, 0.0, 0.0),
                    bounds_radius: 0.5,
                    screen_space_error: 0.15,
                },
            ],
            hierarchy_nodes: Vec::new(),
            hierarchy_child_ids: Vec::new(),
            pages: Vec::new(),
            instances: vec![RenderVirtualGeometryInstance {
                entity,
                source_model: None,
                transform: Transform::default(),
                cluster_offset: 0,
                cluster_count: 3,
                page_offset: 0,
                page_count: 0,
                mesh_name: Some("PrepareClusterSelectionMetadataUnitTest".to_string()),
                source_hint: Some("unit-test".to_string()),
            }],
            debug: RenderVirtualGeometryDebugState::default(),
        };
        let frame = VirtualGeometryPrepareFrame {
            visible_entities: vec![entity],
            visible_clusters: vec![
                crate::virtual_geometry::VirtualGeometryPrepareCluster {
                    entity,
                    cluster_id: 1,
                    page_id: 10,
                    lod_level: 10,
                    resident_slot: Some(5),
                    state: VirtualGeometryPrepareClusterState::Resident,
                },
                crate::virtual_geometry::VirtualGeometryPrepareCluster {
                    entity,
                    cluster_id: 2,
                    page_id: 20,
                    lod_level: 11,
                    resident_slot: None,
                    state: VirtualGeometryPrepareClusterState::PendingUpload,
                },
                crate::virtual_geometry::VirtualGeometryPrepareCluster {
                    entity,
                    cluster_id: 3,
                    page_id: 20,
                    lod_level: 11,
                    resident_slot: None,
                    state: VirtualGeometryPrepareClusterState::PendingUpload,
                },
            ],
            cluster_draw_segments: vec![VirtualGeometryPrepareDrawSegment {
                entity,
                cluster_id: 2,
                page_id: 20,
                resident_slot: None,
                cluster_ordinal: 1,
                cluster_span_count: 2,
                cluster_count: 3,
                lineage_depth: 4,
                lod_level: 11,
                state: VirtualGeometryPrepareClusterState::PendingUpload,
            }],
            resident_pages: vec![VirtualGeometryPreparePage {
                page_id: 10,
                slot: 5,
                size_bytes: 4096,
            }],
            pending_page_requests: vec![VirtualGeometryPrepareRequest {
                page_id: 20,
                size_bytes: 8192,
                generation: 2,
                frontier_rank: 7,
                assigned_slot: Some(9),
                recycled_page_id: None,
            }],
            available_slots: vec![11],
            evictable_pages: Vec::new(),
        };

        assert_eq!(
            frame.cluster_selections(&extract),
            vec![
                VirtualGeometryClusterSelection {
                    submission_index: 0,
                    instance_index: Some(0),
                    entity,
                    cluster_id: 2,
                    cluster_ordinal: 1,
                    page_id: 20,
                    lod_level: 11,
                    submission_page_id: 20,
                    submission_lod_level: 11,
                    entity_cluster_start_ordinal: 1,
                    entity_cluster_span_count: 2,
                    entity_cluster_total_count: 3,
                    lineage_depth: 4,
                    frontier_rank: 7,
                    resident_slot: None,
                    submission_slot: Some(9),
                    state: VirtualGeometryPrepareClusterState::PendingUpload,
                },
                VirtualGeometryClusterSelection {
                    submission_index: 0,
                    instance_index: Some(0),
                    entity,
                    cluster_id: 3,
                    cluster_ordinal: 2,
                    page_id: 20,
                    lod_level: 11,
                    submission_page_id: 20,
                    submission_lod_level: 11,
                    entity_cluster_start_ordinal: 1,
                    entity_cluster_span_count: 2,
                    entity_cluster_total_count: 3,
                    lineage_depth: 4,
                    frontier_rank: 7,
                    resident_slot: None,
                    submission_slot: Some(9),
                    state: VirtualGeometryPrepareClusterState::PendingUpload,
                },
            ],
            "expected prepare-owned cluster selections to keep both cluster identity and raster submission metadata so the same internal worklist can project public selected-cluster debug state and fallback raster submissions"
        );
    }
}
