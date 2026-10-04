use std::collections::{HashMap, HashSet};

use zircon_runtime::core::framework::render::{
    render_mesh_stable_instance_key, RenderVirtualGeometryCluster,
    RenderVirtualGeometryDebugSnapshot, RenderVirtualGeometryExecutionSegment,
    RenderVirtualGeometryExtract, RenderVirtualGeometrySelectedCluster,
    RenderVirtualGeometrySelectedClusterSource, RenderVirtualGeometryVisBuffer64Entry,
    RenderVirtualGeometryVisBuffer64Source, RenderVirtualGeometryVisBufferMark,
};

pub(super) fn rebuild_selected_clusters_from_execution_segments(
    snapshot: &RenderVirtualGeometryDebugSnapshot,
    virtual_geometry_extract: Option<&RenderVirtualGeometryExtract>,
    execution_segments: &[RenderVirtualGeometryExecutionSegment],
) -> Vec<RenderVirtualGeometrySelectedCluster> {
    let Some(virtual_geometry_extract) = virtual_geometry_extract else {
        return snapshot.selected_clusters.clone();
    };

    let rebuild_index =
        SnapshotRebuildIndex::new(snapshot, virtual_geometry_extract, execution_segments);
    let indexed_cluster_count = rebuild_index
        .clusters_by_stable_instance_key
        .values()
        .fold(0_usize, |count, clusters| {
            count.saturating_add(clusters.len())
        });
    let mut selected_clusters = Vec::with_capacity(indexed_cluster_count);
    let mut emitted_clusters = HashSet::<(u64, u32)>::with_capacity(indexed_cluster_count);

    for segment in execution_segments {
        let stable_instance_key = segment.stable_instance_key_or_legacy();
        let Some(instance_clusters) = rebuild_index
            .clusters_by_stable_instance_key
            .get(&stable_instance_key)
        else {
            continue;
        };
        if instance_clusters.is_empty() {
            continue;
        }

        let start = usize::try_from(segment.cluster_start_ordinal).unwrap_or(usize::MAX);
        let span = usize::try_from(segment.cluster_span_count).unwrap_or(0);
        let end = start.saturating_add(span).min(instance_clusters.len());
        if start >= end {
            continue;
        }

        for (cluster_ordinal, cluster) in instance_clusters[start..end]
            .iter()
            .enumerate()
            .map(|(index, cluster)| (start.saturating_add(index), cluster))
        {
            if !emitted_clusters.insert((stable_instance_key, cluster.cluster_id)) {
                continue;
            }

            selected_clusters.push(RenderVirtualGeometrySelectedCluster {
                instance_index: segment.instance_index.or_else(|| {
                    rebuild_index
                        .instance_index_by_cluster
                        .get(&(stable_instance_key, cluster.cluster_id))
                        .copied()
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

// 渲染路径给出来源时直接采用其选择结果；只有来源为 Unavailable 才从执行段重建。
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

// ClearOnly 是权威空结果；其他来源先采用 pass 自有条目，缺失时才从已选簇重建。
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

struct SnapshotRebuildIndex {
    clusters_by_stable_instance_key: HashMap<u64, Vec<RenderVirtualGeometryCluster>>,
    instance_index_by_cluster: HashMap<(u64, u32), u32>,
}

impl SnapshotRebuildIndex {
    fn new(
        snapshot: &RenderVirtualGeometryDebugSnapshot,
        extract: &RenderVirtualGeometryExtract,
        execution_segments: &[RenderVirtualGeometryExecutionSegment],
    ) -> Self {
        let mut requested_stable_instance_keys = HashSet::with_capacity(execution_segments.len());
        for segment in execution_segments {
            requested_stable_instance_keys.insert(segment.stable_instance_key_or_legacy());
        }
        let mut clusters_by_stable_instance_key =
            HashMap::<u64, Vec<RenderVirtualGeometryCluster>>::with_capacity(
                requested_stable_instance_keys.len(),
            );
        let mut instance_index_by_cluster = HashMap::with_capacity(extract.clusters.len());

        if extract.instances.is_empty() {
            for &cluster in &extract.clusters {
                let stable_instance_key = render_mesh_stable_instance_key(cluster.entity, 0);
                if !requested_stable_instance_keys.contains(&stable_instance_key) {
                    continue;
                }
                clusters_by_stable_instance_key
                    .entry(stable_instance_key)
                    .or_default()
                    .push(cluster);
            }
        } else {
            for (instance_index, instance) in extract.instances.iter().enumerate() {
                let stable_instance_key = instance.stable_instance_key_or_legacy();
                if !requested_stable_instance_keys.contains(&stable_instance_key) {
                    continue;
                }
                let start = instance.cluster_offset as usize;
                let end = start.saturating_add(instance.cluster_count as usize);
                let Some(clusters) = extract.clusters.get(start..end) else {
                    continue;
                };
                clusters_by_stable_instance_key
                    .entry(stable_instance_key)
                    .or_default()
                    .extend_from_slice(clusters);

                if snapshot.instances.is_empty() {
                    continue;
                }
                let Ok(instance_index) = u32::try_from(instance_index) else {
                    continue;
                };
                for cluster in clusters {
                    instance_index_by_cluster
                        .entry((stable_instance_key, cluster.cluster_id))
                        .or_insert(instance_index);
                }
            }
        }

        for clusters in clusters_by_stable_instance_key.values_mut() {
            clusters.sort_by_key(|cluster| cluster.cluster_id);
            clusters.dedup_by_key(|cluster| cluster.cluster_id);
        }

        Self {
            clusters_by_stable_instance_key,
            instance_index_by_cluster,
        }
    }
}

#[cfg(test)]
#[path = "virtual_geometry_snapshot_rebuild/tests/performance_tests.rs"]
mod performance_tests;

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
#[path = "tests/virtual_geometry_snapshot_rebuild.rs"]
mod tests;
