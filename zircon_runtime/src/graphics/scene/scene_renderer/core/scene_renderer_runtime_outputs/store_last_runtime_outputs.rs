use crate::core::framework::render::{
    RenderVirtualGeometryCluster, RenderVirtualGeometryDebugSnapshot,
    RenderVirtualGeometryExecutionSegment, RenderVirtualGeometryExtract,
    RenderVirtualGeometryHardwareRasterizationRecord,
    RenderVirtualGeometryHardwareRasterizationSource, RenderVirtualGeometrySelectedCluster,
    RenderVirtualGeometrySubmissionEntry, RenderVirtualGeometrySubmissionRecord,
    RenderVirtualGeometryVisBuffer64Entry, RenderVirtualGeometryVisBuffer64Source,
    RenderVirtualGeometryVisBufferMark,
};
use std::sync::Arc;

use crate::graphics::scene::scene_renderer::{
    HybridGiGpuPendingReadback, VirtualGeometryGpuPendingReadback,
};
use crate::graphics::types::GraphicsError;
use std::collections::{HashMap, HashSet};
use wgpu::util::DeviceExt;

use super::super::scene_renderer::SceneRenderer;

#[allow(clippy::too_many_arguments)]
pub(in crate::graphics::scene::scene_renderer::core) fn store_last_runtime_outputs(
    renderer: &mut SceneRenderer,
    hybrid_gi_gpu_readback: Option<HybridGiGpuPendingReadback>,
    virtual_geometry_gpu_readback: Option<VirtualGeometryGpuPendingReadback>,
    virtual_geometry_debug_snapshot: Option<RenderVirtualGeometryDebugSnapshot>,
    virtual_geometry_extract: Option<&RenderVirtualGeometryExtract>,
    indirect_draw_count: u32,
    indirect_buffer_count: u32,
    indirect_segment_count: u32,
    execution_segment_count: u32,
    execution_page_count: u32,
    execution_resident_segment_count: u32,
    execution_pending_segment_count: u32,
    execution_missing_segment_count: u32,
    execution_repeated_draw_count: u32,
    execution_indirect_offsets: Vec<u64>,
    execution_segments: Vec<RenderVirtualGeometryExecutionSegment>,
    executed_selected_cluster_count: u32,
    executed_selected_cluster_buffer: Option<Arc<wgpu::Buffer>>,
    hardware_rasterization_records: Vec<RenderVirtualGeometryHardwareRasterizationRecord>,
    hardware_rasterization_render_path_source: RenderVirtualGeometryHardwareRasterizationSource,
    hardware_rasterization_record_count: u32,
    hardware_rasterization_buffer: Option<Arc<wgpu::Buffer>>,
    visbuffer64_clear_value: u64,
    visbuffer64_render_path_source: RenderVirtualGeometryVisBuffer64Source,
    visbuffer64_entry_count: u32,
    visbuffer64_buffer: Option<Arc<wgpu::Buffer>>,
    indirect_draw_submission_order: Vec<(Option<u32>, u64, u32)>,
    indirect_draw_submission_records: Vec<(u64, u32, u32, usize)>,
    indirect_draw_submission_token_records: Vec<(u64, u32, u32, u32, usize)>,
    indirect_args_buffer: Option<Arc<wgpu::Buffer>>,
    indirect_args_count: u32,
    indirect_submission_buffer: Option<Arc<wgpu::Buffer>>,
    indirect_authority_buffer: Option<Arc<wgpu::Buffer>>,
    indirect_draw_ref_buffer: Option<Arc<wgpu::Buffer>>,
    indirect_segment_buffer: Option<Arc<wgpu::Buffer>>,
    indirect_execution_submission_buffer: Option<Arc<wgpu::Buffer>>,
    indirect_execution_args_buffer: Option<Arc<wgpu::Buffer>>,
    indirect_execution_authority_buffer: Option<Arc<wgpu::Buffer>>,
) -> Result<(), GraphicsError> {
    renderer.last_hybrid_gi_gpu_readback = hybrid_gi_gpu_readback
        .map(|pending| pending.collect(&renderer.backend.device))
        .transpose()?;
    renderer.last_virtual_geometry_gpu_readback = virtual_geometry_gpu_readback
        .map(|pending| pending.collect(&renderer.backend.device))
        .transpose()?;
    let fallback_readback_visbuffer64_entries =
        if renderer.last_virtual_geometry_gpu_readback.is_some() {
            virtual_geometry_extract
                .map(|extract| {
                    rebuild_selected_clusters_from_execution_segments(
                        &RenderVirtualGeometryDebugSnapshot {
                            instances: extract.instances.clone(),
                            ..RenderVirtualGeometryDebugSnapshot::default()
                        },
                        Some(extract),
                        &execution_segments,
                    )
                })
                .map(|selected_clusters| {
                    rebuild_visbuffer64_entries_from_selected_clusters(&selected_clusters)
                })
                .unwrap_or_default()
        } else {
            Vec::new()
        };
    let fallback_readback_has_entries = !fallback_readback_visbuffer64_entries.is_empty();
    let mut virtual_geometry_debug_snapshot = virtual_geometry_debug_snapshot;
    if let Some(snapshot) = virtual_geometry_debug_snapshot.as_mut() {
        let draw_ref_index_by_original_index = indirect_draw_submission_records
            .iter()
            .map(|(_entity, _page_id, draw_ref_index, original_index)| {
                (*original_index, *draw_ref_index)
            })
            .collect::<HashMap<_, _>>();
        let instance_index_by_original_index = execution_segments
            .iter()
            .map(|segment| (segment.original_index as usize, segment.instance_index))
            .collect::<HashMap<_, _>>();
        snapshot.execution_segment_count = execution_segment_count;
        snapshot.execution_page_count = execution_page_count;
        snapshot.execution_resident_segment_count = execution_resident_segment_count;
        snapshot.execution_pending_segment_count = execution_pending_segment_count;
        snapshot.execution_missing_segment_count = execution_missing_segment_count;
        snapshot.execution_repeated_draw_count = execution_repeated_draw_count;
        snapshot.execution_indirect_offsets = execution_indirect_offsets.clone();
        let selected_clusters = rebuild_selected_clusters_from_execution_segments(
            snapshot,
            virtual_geometry_extract,
            &execution_segments,
        );
        snapshot.selected_clusters = selected_clusters.clone();
        snapshot.hardware_rasterization_records = hardware_rasterization_records.clone();
        snapshot.hardware_rasterization_source = hardware_rasterization_render_path_source;
        snapshot.visbuffer_debug_marks =
            rebuild_visbuffer_debug_marks_from_selected_clusters(snapshot, &selected_clusters);
        let visbuffer64_clear_value = RenderVirtualGeometryVisBuffer64Entry::CLEAR_VALUE;
        let visbuffer64_entries =
            rebuild_visbuffer64_entries_from_selected_clusters(&selected_clusters);
        let visbuffer64_source = resolve_visbuffer64_buffer_source(
            visbuffer64_render_path_source,
            !visbuffer64_entries.is_empty(),
            fallback_readback_has_entries,
        );
        snapshot.visbuffer64_source = visbuffer64_source;
        snapshot.visbuffer64_clear_value = visbuffer64_clear_value;
        snapshot.visbuffer64_entries = visbuffer64_entries.clone();
        if let Some(readback) = renderer.last_virtual_geometry_gpu_readback.as_mut() {
            readback.visbuffer64_clear_value = visbuffer64_clear_value;
            readback.visbuffer64_entries = visbuffer64_entries;
        }
        snapshot.execution_segments = execution_segments;
        snapshot.submission_order = indirect_draw_submission_order
            .iter()
            .map(
                |(instance_index, entity, page_id)| RenderVirtualGeometrySubmissionEntry {
                    instance_index: *instance_index,
                    entity: *entity,
                    page_id: *page_id,
                },
            )
            .collect();
        snapshot.submission_records = indirect_draw_submission_token_records
            .iter()
            .map(
                |(entity, page_id, submission_index, draw_ref_rank, original_index)| {
                    RenderVirtualGeometrySubmissionRecord {
                        instance_index: instance_index_by_original_index
                            .get(original_index)
                            .copied()
                            .flatten(),
                        entity: *entity,
                        page_id: *page_id,
                        draw_ref_index: draw_ref_index_by_original_index
                            .get(original_index)
                            .copied(),
                        submission_index: *submission_index,
                        draw_ref_rank: *draw_ref_rank,
                        original_index: *original_index as u32,
                    }
                },
            )
            .collect();
    }
    if let Some(readback) = renderer.last_virtual_geometry_gpu_readback.as_mut() {
        readback.visbuffer64_clear_value = RenderVirtualGeometryVisBuffer64Entry::CLEAR_VALUE;
        if readback.visbuffer64_entries.is_empty() {
            readback.visbuffer64_entries = fallback_readback_visbuffer64_entries;
        }
    }
    let visbuffer64_packed_words = virtual_geometry_debug_snapshot
        .as_ref()
        .map(|snapshot| {
            snapshot
                .visbuffer64_entries
                .iter()
                .map(|entry| entry.packed_value)
                .collect::<Vec<_>>()
        })
        .filter(|entries| !entries.is_empty())
        .or_else(|| {
            renderer
                .last_virtual_geometry_gpu_readback
                .as_ref()
                .map(|readback| {
                    readback
                        .visbuffer64_entries
                        .iter()
                        .map(|entry| entry.packed_value)
                        .collect::<Vec<_>>()
                })
                .filter(|entries| !entries.is_empty())
        })
        .unwrap_or_default();
    let visbuffer64_source = resolve_visbuffer64_buffer_source(
        visbuffer64_render_path_source,
        !visbuffer64_packed_words.is_empty(),
        fallback_readback_has_entries,
    );
    let selected_cluster_packed_words = virtual_geometry_debug_snapshot
        .as_ref()
        .map(|snapshot| {
            snapshot
                .selected_clusters
                .iter()
                .flat_map(RenderVirtualGeometrySelectedCluster::packed_words)
                .collect::<Vec<_>>()
        })
        .filter(|words| !words.is_empty())
        .unwrap_or_default();
    renderer.last_virtual_geometry_selected_cluster_count =
        if executed_selected_cluster_buffer.is_some() {
            executed_selected_cluster_count
        } else {
            u32::try_from(
                selected_cluster_packed_words.len()
                    / RenderVirtualGeometrySelectedCluster::GPU_WORD_COUNT,
            )
            .unwrap_or(u32::MAX)
        };
    renderer.last_virtual_geometry_selected_cluster_buffer = executed_selected_cluster_buffer
        .or_else(|| {
            create_selected_cluster_buffer(&renderer.backend.device, &selected_cluster_packed_words)
        });
    renderer.last_virtual_geometry_visbuffer64_clear_value = visbuffer64_clear_value;
    renderer.last_virtual_geometry_visbuffer64_source = visbuffer64_source;
    renderer.last_virtual_geometry_visbuffer64_entry_count = if visbuffer64_buffer.is_some() {
        visbuffer64_entry_count
    } else {
        u32::try_from(visbuffer64_packed_words.len()).unwrap_or(u32::MAX)
    };
    renderer.last_virtual_geometry_visbuffer64_buffer = visbuffer64_buffer
        .or_else(|| create_visbuffer64_buffer(&renderer.backend.device, &visbuffer64_packed_words));
    let hardware_rasterization_packed_words =
        pack_hardware_rasterization_records(&hardware_rasterization_records);
    renderer.last_virtual_geometry_hardware_rasterization_source =
        hardware_rasterization_render_path_source;
    renderer.last_virtual_geometry_hardware_rasterization_record_count =
        if hardware_rasterization_buffer.is_some() {
            hardware_rasterization_record_count
        } else {
            u32::try_from(hardware_rasterization_records.len()).unwrap_or(u32::MAX)
        };
    renderer.last_virtual_geometry_hardware_rasterization_buffer = hardware_rasterization_buffer
        .or_else(|| {
            create_hardware_rasterization_buffer(
                &renderer.backend.device,
                &hardware_rasterization_packed_words,
            )
        });
    renderer.last_virtual_geometry_debug_snapshot = virtual_geometry_debug_snapshot;
    renderer.last_virtual_geometry_indirect_draw_count = indirect_draw_count;
    renderer.last_virtual_geometry_indirect_buffer_count = indirect_buffer_count;
    renderer.last_virtual_geometry_indirect_segment_count = indirect_segment_count;
    renderer.last_virtual_geometry_execution_segment_count = execution_segment_count;
    renderer.last_virtual_geometry_execution_page_count = execution_page_count;
    renderer.last_virtual_geometry_execution_resident_segment_count =
        execution_resident_segment_count;
    renderer.last_virtual_geometry_execution_pending_segment_count =
        execution_pending_segment_count;
    renderer.last_virtual_geometry_execution_missing_segment_count =
        execution_missing_segment_count;
    renderer.last_virtual_geometry_execution_repeated_draw_count = execution_repeated_draw_count;
    renderer.last_virtual_geometry_execution_indirect_offsets = execution_indirect_offsets;
    renderer.last_virtual_geometry_mesh_draw_submission_order = indirect_draw_submission_order;
    renderer.last_virtual_geometry_mesh_draw_submission_records = indirect_draw_submission_records;
    renderer.last_virtual_geometry_mesh_draw_submission_token_records =
        indirect_draw_submission_token_records;
    renderer.last_virtual_geometry_indirect_args_buffer = indirect_args_buffer;
    renderer.last_virtual_geometry_indirect_args_count = indirect_args_count;
    renderer.last_virtual_geometry_indirect_submission_buffer = indirect_submission_buffer;
    renderer.last_virtual_geometry_indirect_authority_buffer = indirect_authority_buffer;
    renderer.last_virtual_geometry_indirect_draw_refs_buffer = indirect_draw_ref_buffer;
    renderer.last_virtual_geometry_indirect_segments_buffer = indirect_segment_buffer;
    renderer.last_virtual_geometry_indirect_execution_submission_buffer =
        indirect_execution_submission_buffer;
    renderer.last_virtual_geometry_indirect_execution_args_buffer = indirect_execution_args_buffer;
    renderer.last_virtual_geometry_indirect_execution_authority_buffer =
        indirect_execution_authority_buffer;
    Ok(())
}

fn create_selected_cluster_buffer(
    device: &wgpu::Device,
    packed_words: &[u32],
) -> Option<Arc<wgpu::Buffer>> {
    if packed_words.is_empty() {
        return None;
    }

    Some(Arc::new(device.create_buffer_init(
        &wgpu::util::BufferInitDescriptor {
            label: Some("zircon-vg-selected-cluster-buffer"),
            contents: bytemuck::cast_slice(packed_words),
            usage: wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::STORAGE,
        },
    )))
}

fn rebuild_selected_clusters_from_execution_segments(
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

fn rebuild_visbuffer_debug_marks_from_selected_clusters(
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

fn rebuild_visbuffer64_entries_from_selected_clusters(
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

fn create_visbuffer64_buffer(
    device: &wgpu::Device,
    packed_words: &[u64],
) -> Option<Arc<wgpu::Buffer>> {
    if packed_words.is_empty() {
        return None;
    }

    Some(Arc::new(device.create_buffer_init(
        &wgpu::util::BufferInitDescriptor {
            label: Some("zircon-vg-visbuffer64-buffer"),
            contents: bytemuck::cast_slice(packed_words),
            usage: wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::STORAGE,
        },
    )))
}

fn pack_hardware_rasterization_records(
    records: &[RenderVirtualGeometryHardwareRasterizationRecord],
) -> Vec<u32> {
    records
        .iter()
        .flat_map(|record| record.packed_words())
        .collect()
}

fn create_hardware_rasterization_buffer(
    device: &wgpu::Device,
    packed_words: &[u32],
) -> Option<Arc<wgpu::Buffer>> {
    if packed_words.is_empty() {
        return None;
    }

    Some(Arc::new(device.create_buffer_init(
        &wgpu::util::BufferInitDescriptor {
            label: Some("zircon-vg-hardware-rasterization-buffer"),
            contents: bytemuck::cast_slice(packed_words),
            usage: wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::STORAGE,
        },
    )))
}

fn resolve_visbuffer64_buffer_source(
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
    use super::rebuild_selected_clusters_from_execution_segments;
    use crate::core::framework::render::{
        RenderVirtualGeometryCluster, RenderVirtualGeometryDebugSnapshot,
        RenderVirtualGeometryDebugState, RenderVirtualGeometryExecutionSegment,
        RenderVirtualGeometryExecutionState, RenderVirtualGeometryExtract,
        RenderVirtualGeometryInstance, RenderVirtualGeometryPage,
        RenderVirtualGeometrySelectedCluster,
    };
    use crate::core::math::{Transform, Vec3};

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
            pages: vec![page(200, false), page(300, true)],
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
