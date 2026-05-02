use std::collections::HashSet;

use zircon_runtime::core::framework::render::{
    RenderVirtualGeometryCluster, RenderVirtualGeometryDebugSnapshot,
    RenderVirtualGeometryExecutionSegment, RenderVirtualGeometryExtract,
    RenderVirtualGeometrySelectedCluster, RenderVirtualGeometrySelectedClusterSource,
    RenderVirtualGeometryVisBuffer64Entry, RenderVirtualGeometryVisBuffer64Source,
    RenderVirtualGeometryVisBufferMark,
};

pub(super) fn rebuild_selected_clusters_from_execution_segments(
    snapshot: &RenderVirtualGeometryDebugSnapshot,
    virtual_geometry_extract: Option<&RenderVirtualGeometryExtract>,
    execution_segments: &[RenderVirtualGeometryExecutionSegment],
) -> Vec<RenderVirtualGeometrySelectedCluster> {
    let Some(virtual_geometry_extract) = virtual_geometry_extract else {
        return snapshot.selected_clusters.clone();
    };

    let mut selected_clusters = Vec::new();
    let mut emitted_clusters = HashSet::<(u64, u32)>::new();

    for segment in execution_segments {
        let entity_clusters =
            clusters_for_entity_in_execution_order(virtual_geometry_extract, segment.entity);
        if entity_clusters.is_empty() {
            continue;
        }

        let start = usize::try_from(segment.cluster_start_ordinal).unwrap_or(usize::MAX);
        let span = usize::try_from(segment.cluster_span_count).unwrap_or(0);
        let end = start.saturating_add(span).min(entity_clusters.len());
        if start >= end {
            continue;
        }

        for (cluster_ordinal, cluster) in entity_clusters[start..end]
            .iter()
            .enumerate()
            .map(|(index, cluster)| (start.saturating_add(index), cluster))
        {
            if !emitted_clusters.insert((cluster.entity, cluster.cluster_id)) {
                continue;
            }

            selected_clusters.push(RenderVirtualGeometrySelectedCluster {
                instance_index: segment.instance_index.or_else(|| {
                    instance_index_for_cluster(
                        snapshot,
                        virtual_geometry_extract,
                        cluster.entity,
                        cluster.cluster_id,
                    )
                }),
                entity: cluster.entity,
                cluster_id: cluster.cluster_id,
                cluster_ordinal: u32::try_from(cluster_ordinal).unwrap_or(u32::MAX),
                page_id: cluster.page_id,
                lod_level: cluster.lod_level,
                state: segment.state,
            });
        }
    }

    selected_clusters.sort_by_key(|cluster| {
        (
            cluster.instance_index.unwrap_or(u32::MAX),
            cluster.entity,
            cluster.cluster_ordinal,
            cluster.cluster_id,
            cluster.page_id,
            cluster.lod_level,
        )
    });
    selected_clusters
}

pub(super) fn resolve_selected_clusters_for_store(
    snapshot: &RenderVirtualGeometryDebugSnapshot,
    virtual_geometry_extract: Option<&RenderVirtualGeometryExtract>,
    execution_segments: &[RenderVirtualGeometryExecutionSegment],
    executed_selected_clusters: &[RenderVirtualGeometrySelectedCluster],
    selected_cluster_render_path_source: RenderVirtualGeometrySelectedClusterSource,
) -> Vec<RenderVirtualGeometrySelectedCluster> {
    if selected_cluster_render_path_source
        != RenderVirtualGeometrySelectedClusterSource::Unavailable
    {
        return executed_selected_clusters.to_vec();
    }

    rebuild_selected_clusters_from_execution_segments(
        snapshot,
        virtual_geometry_extract,
        execution_segments,
    )
}

pub(super) fn rebuild_visbuffer_debug_marks_from_selected_clusters(
    snapshot: &RenderVirtualGeometryDebugSnapshot,
    selected_clusters: &[RenderVirtualGeometrySelectedCluster],
) -> Vec<RenderVirtualGeometryVisBufferMark> {
    if !snapshot.debug.visualize_visbuffer {
        return snapshot.visbuffer_debug_marks.clone();
    }

    selected_clusters
        .iter()
        .map(|cluster| RenderVirtualGeometryVisBufferMark {
            instance_index: cluster.instance_index,
            entity: cluster.entity,
            cluster_id: cluster.cluster_id,
            page_id: cluster.page_id,
            lod_level: cluster.lod_level,
            state: cluster.state,
            color_rgba: visbuffer_mark_color(
                cluster.cluster_id,
                cluster.page_id,
                cluster.lod_level,
            ),
        })
        .collect()
}

pub(super) fn rebuild_visbuffer64_entries_from_selected_clusters(
    selected_clusters: &[RenderVirtualGeometrySelectedCluster],
) -> Vec<RenderVirtualGeometryVisBuffer64Entry> {
    selected_clusters
        .iter()
        .enumerate()
        .map(|(entry_index, cluster)| {
            RenderVirtualGeometryVisBuffer64Entry::from_selected_cluster(
                u32::try_from(entry_index).unwrap_or(u32::MAX),
                cluster,
            )
        })
        .collect()
}

pub(super) fn resolve_visbuffer64_entries_for_store(
    selected_clusters: &[RenderVirtualGeometrySelectedCluster],
    pass_owned_visbuffer64_entries: &[RenderVirtualGeometryVisBuffer64Entry],
    visbuffer64_render_path_source: RenderVirtualGeometryVisBuffer64Source,
) -> Vec<RenderVirtualGeometryVisBuffer64Entry> {
    if visbuffer64_render_path_source == RenderVirtualGeometryVisBuffer64Source::RenderPathClearOnly
    {
        return Vec::new();
    }

    if !pass_owned_visbuffer64_entries.is_empty() {
        return pass_owned_visbuffer64_entries.to_vec();
    }

    rebuild_visbuffer64_entries_from_selected_clusters(selected_clusters)
}

pub(super) fn resolve_visbuffer64_buffer_source(
    render_path_source: RenderVirtualGeometryVisBuffer64Source,
    snapshot_has_entries: bool,
    readback_has_entries: bool,
) -> RenderVirtualGeometryVisBuffer64Source {
    if !matches!(
        render_path_source,
        RenderVirtualGeometryVisBuffer64Source::Unavailable
    ) {
        render_path_source
    } else if snapshot_has_entries {
        RenderVirtualGeometryVisBuffer64Source::SnapshotFallback
    } else if readback_has_entries {
        RenderVirtualGeometryVisBuffer64Source::GpuReadbackFallback
    } else {
        RenderVirtualGeometryVisBuffer64Source::Unavailable
    }
}

fn clusters_for_entity_in_execution_order(
    extract: &RenderVirtualGeometryExtract,
    entity: u64,
) -> Vec<RenderVirtualGeometryCluster> {
    let mut clusters = if extract.instances.is_empty() {
        extract
            .clusters
            .iter()
            .copied()
            .filter(|cluster| cluster.entity == entity)
            .collect::<Vec<_>>()
    } else {
        extract
            .instances
            .iter()
            .filter(|instance| instance.entity == entity)
            .flat_map(|instance| {
                let start = instance.cluster_offset as usize;
                let end = start.saturating_add(instance.cluster_count as usize);
                extract
                    .clusters
                    .get(start..end)
                    .into_iter()
                    .flatten()
                    .copied()
            })
            .collect::<Vec<_>>()
    };
    clusters.sort_by_key(|cluster| cluster.cluster_id);
    clusters.dedup_by_key(|cluster| cluster.cluster_id);
    clusters
}

fn instance_index_for_cluster(
    snapshot: &RenderVirtualGeometryDebugSnapshot,
    extract: &RenderVirtualGeometryExtract,
    entity: u64,
    cluster_id: u32,
) -> Option<u32> {
    if snapshot.instances.is_empty() {
        return None;
    }

    extract
        .instances
        .iter()
        .enumerate()
        .find(|(_, instance)| {
            if instance.entity != entity {
                return false;
            }

            let start = instance.cluster_offset as usize;
            let end = start.saturating_add(instance.cluster_count as usize);
            extract
                .clusters
                .get(start..end)
                .into_iter()
                .flatten()
                .any(|cluster| cluster.cluster_id == cluster_id)
        })
        .and_then(|(instance_index, _)| u32::try_from(instance_index).ok())
}

fn visbuffer_mark_color(cluster_id: u32, page_id: u32, lod_level: u8) -> [u8; 4] {
    let lod_level = u32::from(lod_level);
    [
        (32 + ((cluster_id * 17 + page_id * 13) % 192)) as u8,
        (32 + ((page_id * 11 + lod_level * 7) % 192)) as u8,
        (32 + ((cluster_id * 5 + lod_level * 19) % 192)) as u8,
        255,
    ]
}

#[cfg(test)]
mod tests {
    use super::{
        rebuild_selected_clusters_from_execution_segments, resolve_selected_clusters_for_store,
        resolve_visbuffer64_entries_for_store,
    };
    use zircon_runtime::core::framework::render::{
        RenderVirtualGeometryCluster, RenderVirtualGeometryDebugSnapshot,
        RenderVirtualGeometryDebugState, RenderVirtualGeometryExecutionSegment,
        RenderVirtualGeometryExecutionState, RenderVirtualGeometryExtract,
        RenderVirtualGeometryInstance, RenderVirtualGeometryPage,
        RenderVirtualGeometrySelectedCluster, RenderVirtualGeometrySelectedClusterSource,
        RenderVirtualGeometryVisBuffer64Entry, RenderVirtualGeometryVisBuffer64Source,
    };
    use zircon_runtime::core::math::{Transform, Vec3};

    #[test]
    fn rebuild_selected_clusters_from_execution_segments_drops_visibility_only_superset() {
        let entity = 77_u64;
        let extract = RenderVirtualGeometryExtract {
            cluster_budget: 2,
            page_budget: 0,
            clusters: vec![
                cluster(entity, 20, 200, 0, Vec3::ZERO, 9.0),
                cluster(entity, 30, 300, 0, Vec3::new(0.1, 0.0, 0.0), 8.0),
            ],
            hierarchy_nodes: Vec::new(),
            hierarchy_child_ids: Vec::new(),
            pages: vec![page(200, false), page(300, true)],
            page_dependencies: Vec::new(),
            instances: vec![RenderVirtualGeometryInstance {
                entity,
                source_model: None,
                transform: Transform::default(),
                cluster_offset: 0,
                cluster_count: 2,
                page_offset: 0,
                page_count: 2,
                mesh_name: Some("StoreOutputsUnitTestMesh".to_string()),
                source_hint: Some("unit-test".to_string()),
            }],
            debug: RenderVirtualGeometryDebugState::default(),
        };
        let snapshot = RenderVirtualGeometryDebugSnapshot {
            instances: extract.instances.clone(),
            debug: extract.debug,
            selected_clusters: vec![
                RenderVirtualGeometrySelectedCluster {
                    instance_index: Some(0),
                    entity,
                    cluster_id: 20,
                    cluster_ordinal: 0,
                    page_id: 200,
                    lod_level: 0,
                    state: RenderVirtualGeometryExecutionState::Missing,
                },
                RenderVirtualGeometrySelectedCluster {
                    instance_index: Some(0),
                    entity,
                    cluster_id: 30,
                    cluster_ordinal: 1,
                    page_id: 300,
                    lod_level: 0,
                    state: RenderVirtualGeometryExecutionState::Resident,
                },
            ],
            ..RenderVirtualGeometryDebugSnapshot::default()
        };
        let execution_segments = vec![RenderVirtualGeometryExecutionSegment {
            original_index: 0,
            instance_index: Some(0),
            entity,
            page_id: 300,
            draw_ref_index: 0,
            submission_index: Some(0),
            draw_ref_rank: Some(0),
            cluster_start_ordinal: 1,
            cluster_span_count: 1,
            cluster_total_count: 2,
            submission_slot: Some(0),
            state: RenderVirtualGeometryExecutionState::Resident,
            lineage_depth: 0,
            lod_level: 0,
            frontier_rank: 0,
        }];

        assert_eq!(
            rebuild_selected_clusters_from_execution_segments(
                &snapshot,
                Some(&extract),
                &execution_segments,
            ),
            vec![RenderVirtualGeometrySelectedCluster {
                instance_index: Some(0),
                entity,
                cluster_id: 30,
                cluster_ordinal: 1,
                page_id: 300,
                lod_level: 0,
                state: RenderVirtualGeometryExecutionState::Resident,
            }],
            "expected the post-render authoritative selection to shrink from the submission-build visibility superset down to the real execution-backed cluster subset"
        );
    }

    #[test]
    fn resolve_selected_clusters_for_store_prefers_pass_owned_selected_clusters() {
        let entity = 77_u64;
        let extract = RenderVirtualGeometryExtract {
            cluster_budget: 2,
            page_budget: 0,
            clusters: vec![
                cluster(entity, 20, 200, 0, Vec3::ZERO, 9.0),
                cluster(entity, 30, 300, 0, Vec3::new(0.1, 0.0, 0.0), 8.0),
            ],
            hierarchy_nodes: Vec::new(),
            hierarchy_child_ids: Vec::new(),
            pages: vec![page(200, true), page(300, true)],
            page_dependencies: Vec::new(),
            instances: vec![RenderVirtualGeometryInstance {
                entity,
                source_model: None,
                transform: Transform::default(),
                cluster_offset: 0,
                cluster_count: 2,
                page_offset: 0,
                page_count: 2,
                mesh_name: Some("StoreOutputsExplicitSelectionUnitTestMesh".to_string()),
                source_hint: Some("unit-test".to_string()),
            }],
            debug: RenderVirtualGeometryDebugState::default(),
        };
        let snapshot = RenderVirtualGeometryDebugSnapshot {
            instances: extract.instances.clone(),
            debug: extract.debug,
            selected_clusters: vec![RenderVirtualGeometrySelectedCluster {
                instance_index: Some(0),
                entity,
                cluster_id: 20,
                cluster_ordinal: 0,
                page_id: 200,
                lod_level: 0,
                state: RenderVirtualGeometryExecutionState::Resident,
            }],
            ..RenderVirtualGeometryDebugSnapshot::default()
        };
        let execution_segments = vec![RenderVirtualGeometryExecutionSegment {
            original_index: 0,
            instance_index: Some(0),
            entity,
            page_id: 300,
            draw_ref_index: 0,
            submission_index: Some(0),
            draw_ref_rank: Some(0),
            cluster_start_ordinal: 1,
            cluster_span_count: 1,
            cluster_total_count: 2,
            submission_slot: Some(0),
            state: RenderVirtualGeometryExecutionState::Resident,
            lineage_depth: 0,
            lod_level: 0,
            frontier_rank: 0,
        }];
        let explicit_selected_clusters = vec![RenderVirtualGeometrySelectedCluster {
            instance_index: Some(0),
            entity,
            cluster_id: 20,
            cluster_ordinal: 0,
            page_id: 200,
            lod_level: 0,
            state: RenderVirtualGeometryExecutionState::Resident,
        }];

        assert_eq!(
            resolve_selected_clusters_for_store(
                &snapshot,
                Some(&extract),
                &execution_segments,
                &explicit_selected_clusters,
                RenderVirtualGeometrySelectedClusterSource::RenderPathExecutionSelections,
            ),
            explicit_selected_clusters,
            "expected store_last_runtime_outputs to trust the executed selected-cluster pass output directly once that seam produced an authoritative render-path selection instead of re-deriving a second cluster list from execution segments"
        );
    }

    #[test]
    fn resolve_visbuffer64_entries_for_store_prefers_pass_owned_entries() {
        let selected_clusters = vec![RenderVirtualGeometrySelectedCluster {
            instance_index: Some(0),
            entity: 77,
            cluster_id: 30,
            cluster_ordinal: 1,
            page_id: 300,
            lod_level: 0,
            state: RenderVirtualGeometryExecutionState::Resident,
        }];
        let pass_owned_entries = vec![RenderVirtualGeometryVisBuffer64Entry {
            entry_index: 0,
            packed_value: RenderVirtualGeometryVisBuffer64Entry::packed_value_for(
                Some(9),
                20,
                200,
                1,
                RenderVirtualGeometryExecutionState::PendingUpload,
            ),
            instance_index: Some(9),
            entity: 999,
            cluster_id: 20,
            page_id: 200,
            lod_level: 1,
            state: RenderVirtualGeometryExecutionState::PendingUpload,
        }];

        assert_eq!(
            resolve_visbuffer64_entries_for_store(
                &selected_clusters,
                &pass_owned_entries,
                RenderVirtualGeometryVisBuffer64Source::RenderPathExecutionSelections,
            ),
            pass_owned_entries,
            "expected store_last_runtime_outputs to trust the render-path VisBuffer64 pass output directly once that seam exists instead of rebuilding a second logical entry stream from selected clusters"
        );
    }

    #[test]
    fn resolve_visbuffer64_entries_for_store_rebuilds_when_pass_entries_are_missing() {
        let selected_clusters = vec![RenderVirtualGeometrySelectedCluster {
            instance_index: Some(0),
            entity: 77,
            cluster_id: 30,
            cluster_ordinal: 1,
            page_id: 300,
            lod_level: 0,
            state: RenderVirtualGeometryExecutionState::Resident,
        }];

        assert_eq!(
            resolve_visbuffer64_entries_for_store(
                &selected_clusters,
                &[],
                RenderVirtualGeometryVisBuffer64Source::RenderPathExecutionSelections,
            ),
            vec![RenderVirtualGeometryVisBuffer64Entry::from_selected_cluster(
                0,
                &selected_clusters[0],
            )],
            "expected store_last_runtime_outputs to fall back to execution-backed selected-cluster rebuild when the render path claims execution ownership but the pass-owned VisBuffer64 entry stream is still empty"
        );
    }

    fn cluster(
        entity: u64,
        cluster_id: u32,
        page_id: u32,
        lod_level: u8,
        bounds_center: Vec3,
        screen_space_error: f32,
    ) -> RenderVirtualGeometryCluster {
        RenderVirtualGeometryCluster {
            entity,
            cluster_id,
            hierarchy_node_id: None,
            page_id,
            lod_level,
            parent_cluster_id: None,
            bounds_center,
            bounds_radius: 0.5,
            screen_space_error,
        }
    }

    fn page(page_id: u32, resident: bool) -> RenderVirtualGeometryPage {
        RenderVirtualGeometryPage {
            page_id,
            resident,
            size_bytes: 4096,
        }
    }
}
