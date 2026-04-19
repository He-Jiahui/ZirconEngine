use std::collections::{HashMap, HashSet};

use zircon_framework::scene::EntityId;

use super::{
    VirtualGeometryPrepareCluster, VirtualGeometryPrepareClusterState,
    VirtualGeometryPrepareDrawSegment, VirtualGeometryPrepareIndirectDraw,
    VirtualGeometryPreparePage, VirtualGeometryPrepareRequest,
};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct VirtualGeometryPrepareFrame {
    pub(crate) visible_entities: Vec<EntityId>,
    pub(crate) visible_clusters: Vec<VirtualGeometryPrepareCluster>,
    pub(crate) cluster_draw_segments: Vec<VirtualGeometryPrepareDrawSegment>,
    pub(crate) resident_pages: Vec<VirtualGeometryPreparePage>,
    pub(crate) pending_page_requests: Vec<VirtualGeometryPrepareRequest>,
    pub(crate) available_slots: Vec<u32>,
    pub(crate) evictable_pages: Vec<VirtualGeometryPreparePage>,
}

impl VirtualGeometryPrepareFrame {
    pub(crate) fn unified_indirect_draws(&self) -> Vec<VirtualGeometryPrepareIndirectDraw> {
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
                (
                    self.cluster_draw_segments.len() + fallback_index,
                    draw,
                )
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
        *entity_cluster_total_count.entry(cluster.entity).or_default() += 1;
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
            clusters.into_iter().map(move |cluster| {
                FallbackIndirectDraw {
                    entity,
                    visible_index,
                    cluster_ordinal: cluster.entity_cluster_ordinal,
                    cluster_total_count: cluster.entity_cluster_total_count.max(cluster_total_count),
                    page_id: cluster.page_id,
                    frontier_rank: cluster.frontier_rank,
                    resident_slot: cluster.resident_slot,
                    submission_slot: cluster.submission_slot,
                    lod_level: cluster.lod_level,
                    state: cluster.state,
                }
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

