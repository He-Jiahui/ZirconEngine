use std::collections::{BTreeMap, BTreeSet, HashSet};

use zircon_runtime::asset::ModelAsset;
use zircon_runtime::core::framework::render::{
    RenderMeshSnapshot, RenderVirtualGeometryDebugState,
};
use zircon_runtime::core::resource::ResourceId;
use zircon_runtime::graphics::{
    VirtualGeometryGpuCompletion, VirtualGeometryRuntimeExtractOutput,
    VirtualGeometryRuntimeFeedback, VirtualGeometryRuntimePrepareInput,
    VirtualGeometryRuntimePrepareOutput, VirtualGeometryRuntimeProvider,
    VirtualGeometryRuntimeState as RuntimeStateContract, VirtualGeometryRuntimeStats,
    VirtualGeometryRuntimeUpdate,
};

use crate::virtual_geometry::VirtualGeometryRuntimeState;

#[derive(Clone, Debug, Default)]
pub struct PluginVirtualGeometryRuntimeProvider;

impl VirtualGeometryRuntimeProvider for PluginVirtualGeometryRuntimeProvider {
    fn create_state(&self) -> Box<dyn RuntimeStateContract> {
        Box::<PluginVirtualGeometryRuntimeState>::default()
    }

    fn build_extract_from_admitted_sources(
        &self,
        meshes: &[RenderMeshSnapshot],
        debug: Option<RenderVirtualGeometryDebugState>,
        admit: &mut zircon_runtime::graphics::VirtualGeometrySourceLoader<'_>,
    ) -> zircon_runtime::graphics::VirtualGeometryAdmittedExtract {
        use zircon_runtime::graphics::VirtualGeometryAdmittedExtract;
        match crate::virtual_geometry::build_virtual_geometry_automatic_extract_from_admitted_sources(
            meshes, debug.unwrap_or_default(), admit,
        ) {
            Err(error) => VirtualGeometryAdmittedExtract::Failed(error),
            Ok(None) => VirtualGeometryAdmittedExtract::Ready(None),
            Ok(Some(output)) => VirtualGeometryAdmittedExtract::Ready(Some(
                VirtualGeometryRuntimeExtractOutput::new(
                    output.extract().clone(),
                    output.cpu_reference_instances().to_vec(),
                    output.bvh_visualization_instances().to_vec(),
                    output.resident_page_payloads().to_vec(),
                ).with_indexed_sources(output.indexed_sources().to_vec()),
            )),
        }
    }

    fn build_extract_from_meshes(
        &self,
        meshes: &[RenderMeshSnapshot],
        debug: Option<RenderVirtualGeometryDebugState>,
        load_model: &mut dyn FnMut(ResourceId) -> Option<ModelAsset>,
    ) -> Option<VirtualGeometryRuntimeExtractOutput> {
        let output = crate::virtual_geometry::build_virtual_geometry_automatic_extract_from_meshes_with_debug(
            meshes,
            debug.unwrap_or_default(),
            |model_id| load_model(model_id),
        )?;
        Some(VirtualGeometryRuntimeExtractOutput::new(
            output.extract().clone(),
            output.cpu_reference_instances().to_vec(),
            output.bvh_visualization_instances().to_vec(),
            output.resident_page_payloads().to_vec(),
        ))
    }
}

#[derive(Debug, Default)]
struct PluginVirtualGeometryRuntimeState {
    state: VirtualGeometryRuntimeState,
    staged_cpu_frame: Option<std::sync::Arc<zircon_runtime::core::framework::render::RenderVirtualGeometryPreparedCpuFrame>>,
    committed_previous_global: Option<zircon_runtime::core::framework::render::RenderVirtualGeometryNodeAndClusterCullGlobalStateSnapshot>,
    cpu_prepare_revision: u64,
    cpu_staging_exhausted: bool,
}

impl RuntimeStateContract for PluginVirtualGeometryRuntimeState {
    // 每帧先用当前 extract 同步页依赖；缺少 extract 时重置流送状态，避免沿用上一场景。
    fn prepare_frame(
        &mut self,
        input: VirtualGeometryRuntimePrepareInput<'_>,
    ) -> VirtualGeometryRuntimePrepareOutput {
        let Some(extract) = input.extract() else {
            self.state = VirtualGeometryRuntimeState::default();
            self.staged_cpu_frame = None;
            self.committed_previous_global = None;
            return VirtualGeometryRuntimePrepareOutput::default().with_prepared_cpu_frame(std::sync::Arc::new(
                zircon_runtime::core::framework::render::RenderVirtualGeometryPreparedCpuFrame::unavailable()));
        };
        self.state.register_extract(Some(extract));
        if let Some(plan) = input.page_upload_plan() {
            self.state.ingest_plan(input.generation(), plan);
        }
        let prepare = self.state.build_prepare_frame_with_segments(
            input.visible_clusters(),
            input.visibility_draw_segments(),
        );
        let evictable_page_ids = prepare
            .evictable_pages
            .iter()
            .map(|page| page.page_id)
            .collect();
        let prepared_cpu_frame = self.prepare_cpu_frame(&input, &prepare);
        VirtualGeometryRuntimePrepareOutput::new(evictable_page_ids)
            .with_prepared_cpu_frame(prepared_cpu_frame)
    }

    // GPU 完成回读和可见性反馈分别推进页表与请求；无完成回读时仍消费反馈。
    fn update_after_render(
        &mut self,
        feedback: VirtualGeometryRuntimeFeedback,
    ) -> VirtualGeometryRuntimeUpdate {
        if let Some(token) = feedback.cpu_prepare_token() {
            if token.generation == feedback.generation()
                && self
                    .staged_cpu_frame
                    .as_ref()
                    .is_some_and(|frame| frame.commit_token == Some(token))
            {
                if let Some(frame) = self.staged_cpu_frame.take() {
                    self.committed_previous_global = frame.inspection.global_state.clone();
                }
            }
        }
        let previous_slot_owners = self.state.resident_slot_owners();
        let previous_pending_pages = self.state.pending_page_ids();
        let confirmed_completion = feedback.gpu_completion().map(|completion| {
            confirmed_virtual_geometry_completion(
                completion,
                previous_slot_owners.iter().copied(),
                previous_pending_pages.iter().copied(),
            )
        });
        let completed_page_count = confirmed_completion
            .as_ref()
            .map(|completion| completion.completed_page_assignments().len())
            .unwrap_or(0);
        let replaced_page_count = confirmed_completion
            .as_ref()
            .map(|completion| completion.completed_page_replacements().len())
            .unwrap_or(0);

        if let Some(feedback) = feedback.visibility_feedback() {
            self.state.refresh_hot_resident_pages(feedback);
        }
        if let Some(completion) = confirmed_completion.as_ref() {
            self.state.complete_gpu_uploads_with_replacements(
                completion.completed_page_assignments().iter().copied(),
                completion.completed_page_replacements().iter().copied(),
                feedback.evictable_page_ids(),
            );
            self.state
                .apply_gpu_page_table_entries(completion.page_table_entries());
        } else if let Some(feedback) = feedback.visibility_feedback() {
            self.state.consume_feedback(feedback);
        }
        self.state.ingest_page_requests(
            feedback.generation(),
            feedback
                .node_and_cluster_cull_page_requests()
                .iter()
                .copied(),
        );
        let snapshot = self.state.snapshot();
        VirtualGeometryRuntimeUpdate::new(VirtualGeometryRuntimeStats::new(
            snapshot.page_table_entry_count(),
            snapshot.resident_page_count(),
            snapshot.pending_request_count(),
            snapshot.page_dependency_count(),
            completed_page_count,
            replaced_page_count,
        ))
    }
}

impl PluginVirtualGeometryRuntimeState {
    fn prepare_cpu_frame(
        &mut self,
        input: &VirtualGeometryRuntimePrepareInput<'_>,
        prepare: &crate::virtual_geometry::VirtualGeometryPrepareFrame,
    ) -> std::sync::Arc<
        zircon_runtime::core::framework::render::RenderVirtualGeometryPreparedCpuFrame,
    > {
        use zircon_runtime::core::framework::render::{
            RenderVirtualGeometryCpuPrepareToken, RenderVirtualGeometryPreparedCpuFrame,
        };
        let unavailable =
            || std::sync::Arc::new(RenderVirtualGeometryPreparedCpuFrame::unavailable());
        if self.cpu_staging_exhausted {
            self.staged_cpu_frame = None;
            return unavailable();
        }
        let Some(camera) = input.camera() else {
            self.staged_cpu_frame = None;
            return unavailable();
        };
        let Some(revision) = self.cpu_prepare_revision.checked_add(1) else {
            self.cpu_staging_exhausted = true;
            self.staged_cpu_frame = None;
            self.committed_previous_global = None;
            return unavailable();
        };
        self.cpu_prepare_revision = revision;
        let inspection = crate::virtual_geometry::prepare_virtual_geometry_node_and_cluster_cull(
            input.extract(),
            camera,
            input.viewport_size(),
            true,
            input.cull_input(),
            self.committed_previous_global.as_ref(),
        );
        let paired_execution_segments = match paired_author_execution_segments(input) {
            Ok(rows) => rows,
            Err(_) => {
                self.staged_cpu_frame = None;
                return unavailable();
            }
        };
        let frame = std::sync::Arc::new(RenderVirtualGeometryPreparedCpuFrame {
            inspection,
            paired_execution_segments,
            execution_inputs: input.execution_inputs().cloned().unwrap_or_default(),
            author_resident_page_slots: match input
                .execution_inputs()
                .and_then(|inputs| inputs.prepare.as_ref())
            {
                Some(author) => author
                    .resident_pages
                    .iter()
                    .chain(&author.evictable_pages)
                    .map(|page| (page.page_id, page.slot))
                    .collect(),
                None => prepare
                    .resident_pages
                    .iter()
                    .map(|page| (page.page_id, page.slot))
                    .collect(),
            },
            author_pending_page_requests: match input
                .execution_inputs()
                .and_then(|inputs| inputs.prepare.as_ref())
            {
                Some(author) => author
                    .pending_page_requests
                    .iter()
                    .map(|request| {
                        (
                            request.page_id,
                            request.frontier_rank,
                            request.assigned_slot,
                            request.recycled_page_id,
                        )
                    })
                    .collect(),
                None => prepare
                    .pending_page_requests
                    .iter()
                    .map(|request| {
                        (
                            request.page_id,
                            request.frontier_rank,
                            request.assigned_slot,
                            request.recycled_page_id,
                        )
                    })
                    .collect(),
            },
            commit_token: Some(RenderVirtualGeometryCpuPrepareToken {
                generation: input.generation(),
                revision,
            }),
        });
        self.staged_cpu_frame = Some(frame.clone());
        frame
    }
}

fn confirmed_virtual_geometry_completion(
    completion: &VirtualGeometryGpuCompletion,
    previous_slot_owners: impl IntoIterator<Item = (u32, u32)>,
    previous_pending_pages: impl IntoIterator<Item = u32>,
) -> VirtualGeometryGpuCompletion {
    let page_table_entries =
        crate::virtual_geometry::normalized_page_table_entries(completion.page_table_entries());
    let page_table_slot_by_page = page_table_entries
        .iter()
        .copied()
        .collect::<BTreeMap<u32, u32>>();
    let reported_completed_page_ids = completion
        .completed_page_assignments()
        .iter()
        .map(|(page_id, _reported_slot)| *page_id)
        .collect::<HashSet<_>>();
    let previous_pending_pages = previous_pending_pages.into_iter().collect::<BTreeSet<_>>();
    let final_resident_pages = page_table_slot_by_page
        .keys()
        .copied()
        .collect::<BTreeSet<_>>();
    let previous_page_by_slot = previous_slot_owners.into_iter().collect::<BTreeMap<_, _>>();
    let completed_page_assignments = page_table_entries
        .iter()
        .filter(|(page_id, _slot)| {
            previous_pending_pages.contains(page_id)
                && reported_completed_page_ids.contains(page_id)
        })
        .copied()
        .collect::<Vec<_>>();
    let completed_page_replacements = page_table_entries
        .iter()
        .filter(|(page_id, _slot)| {
            previous_pending_pages.contains(page_id)
                && reported_completed_page_ids.contains(page_id)
        })
        .filter_map(|(page_id, _reported_slot)| {
            let confirmed_slot = page_table_slot_by_page.get(page_id).copied()?;
            let previous_page_id = previous_page_by_slot.get(&confirmed_slot).copied()?;
            (previous_page_id != *page_id && !final_resident_pages.contains(&previous_page_id))
                .then_some((*page_id, previous_page_id))
        })
        .collect::<Vec<_>>();

    VirtualGeometryGpuCompletion::new(
        page_table_entries,
        completed_page_assignments,
        completed_page_replacements,
    )
}

#[cfg(test)]
#[path = "tests/provider.rs"]
mod tests;

fn paired_prepare_from_neutral(
    input: &zircon_runtime::core::framework::render::RenderVirtualGeometryPrepareFrameInput,
) -> crate::virtual_geometry::VirtualGeometryPrepareFrame {
    use crate::virtual_geometry::{
        VirtualGeometryPrepareCluster, VirtualGeometryPrepareClusterState,
        VirtualGeometryPrepareDrawSegment, VirtualGeometryPrepareFrame, VirtualGeometryPreparePage,
        VirtualGeometryPrepareRequest,
    };
    use zircon_runtime::core::framework::render::RenderVirtualGeometryExecutionState as State;
    let state = |value| match value {
        State::Resident => VirtualGeometryPrepareClusterState::Resident,
        State::PendingUpload => VirtualGeometryPrepareClusterState::PendingUpload,
        State::Missing => VirtualGeometryPrepareClusterState::Missing,
    };
    let page =
        |value: &zircon_runtime::core::framework::render::RenderVirtualGeometryPreparePageInput| {
            VirtualGeometryPreparePage {
                page_id: value.page_id,
                slot: value.slot,
                size_bytes: value.size_bytes,
            }
        };
    VirtualGeometryPrepareFrame {
        visible_entities: input.visible_entities.clone(),
        visible_clusters: input
            .visible_clusters
            .iter()
            .map(|row| VirtualGeometryPrepareCluster {
                entity: row.entity,
                cluster_id: row.cluster_id,
                page_id: row.page_id,
                lod_level: row.lod_level,
                resident_slot: row.resident_slot,
                state: state(row.state),
            })
            .collect(),
        cluster_draw_segments: input
            .cluster_draw_segments
            .iter()
            .map(|row| VirtualGeometryPrepareDrawSegment {
                entity: row.entity,
                cluster_id: row.cluster_id,
                page_id: row.page_id,
                resident_slot: row.resident_slot,
                cluster_ordinal: row.cluster_ordinal,
                cluster_span_count: row.cluster_span_count,
                cluster_count: row.cluster_count,
                lineage_depth: row.lineage_depth,
                lod_level: row.lod_level,
                state: state(row.state),
            })
            .collect(),
        resident_pages: input.resident_pages.iter().map(page).collect(),
        pending_page_requests: input
            .pending_page_requests
            .iter()
            .map(|row| VirtualGeometryPrepareRequest {
                page_id: row.page_id,
                size_bytes: row.size_bytes,
                generation: row.generation,
                frontier_rank: row.frontier_rank,
                assigned_slot: row.assigned_slot,
                recycled_page_id: row.recycled_page_id,
            })
            .collect(),
        available_slots: input.available_slots.clone(),
        evictable_pages: input.evictable_pages.iter().map(page).collect(),
    }
}
fn paired_author_execution_segments(
    input: &VirtualGeometryRuntimePrepareInput<'_>,
) -> Result<
    Option<Vec<zircon_runtime::core::framework::render::RenderVirtualGeometryExecutionSegment>>,
    String,
> {
    use zircon_runtime::core::framework::render::{
        RenderVirtualGeometryExecutionSegment as Segment,
        RenderVirtualGeometryExecutionState as State,
    };
    let Some(inputs) = input.execution_inputs() else {
        return Ok(None);
    };
    if inputs.selections.is_none() && inputs.prepare.is_none() {
        return Ok(None);
    }
    let extract = input
        .extract()
        .ok_or("paired author source extract is absent")?;
    let resolve = |entity: u64, explicit: Option<u32>| -> Result<(Option<u32>, u64), String> {
        if extract.instances.is_empty() {
            if explicit.is_some() {
                return Err("paired author declares an unavailable instance".into());
            }
            return Ok((
                None,
                zircon_runtime::core::framework::render::render_mesh_stable_instance_key(entity, 0),
            ));
        }
        if let Some(index) = explicit {
            let instance = extract
                .instances
                .get(usize::try_from(index).map_err(|_| "paired instance overflow")?)
                .ok_or("paired source instance is absent")?;
            if instance.entity != entity {
                return Err("paired author entity differs from its instance".into());
            }
            return Ok((Some(index), instance.stable_instance_key_or_legacy()));
        }
        let instances = extract
            .instances
            .iter()
            .enumerate()
            .filter(|(_, instance)| instance.entity == entity)
            .collect::<Vec<_>>();
        if instances.len() != 1 {
            return Err(
                "paired prepare does not uniquely identify an actual source instance".into(),
            );
        }
        Ok((
            Some(u32::try_from(instances[0].0).map_err(|_| "paired instance index overflow")?),
            instances[0].1.stable_instance_key_or_legacy(),
        ))
    };
    let mut result = Vec::<Segment>::new();
    let mut ranks = std::collections::BTreeMap::<u64, u32>::new();
    if let Some(selections) = inputs.selections.as_ref() {
        let mut groups = std::collections::BTreeMap::<(u64, u32, Option<u32>), usize>::new();
        for row in selections {
            let key = (row.entity, row.submission_index, row.instance_index);
            if let Some(&index) = groups.get(&key) {
                let previous = &result[index];
                if previous.page_id != row.submission_page_id
                    || previous.cluster_start_ordinal
                        != u32::try_from(row.entity_cluster_start_ordinal)
                            .map_err(|_| "paired start overflow")?
                    || previous.cluster_span_count
                        != u32::try_from(row.entity_cluster_span_count)
                            .map_err(|_| "paired span overflow")?
                    || previous.cluster_total_count
                        != u32::try_from(row.entity_cluster_total_count)
                            .map_err(|_| "paired total overflow")?
                    || previous.submission_slot != row.submission_slot
                    || previous.state != row.state
                    || previous.lineage_depth != row.lineage_depth
                    || previous.lod_level != row.submission_lod_level
                    || previous.frontier_rank != row.frontier_rank
                {
                    return Err("paired author rows disagree on one declared submission".into());
                }
                continue;
            }
            let (instance_index, stable_instance_key) = resolve(row.entity, row.instance_index)?;
            let index =
                u32::try_from(result.len()).map_err(|_| "paired draw-ref index overflow")?;
            let rank = ranks.entry(row.entity).or_default();
            let current_rank = *rank;
            *rank = rank.checked_add(1).ok_or("paired draw-ref rank overflow")?;
            groups.insert(key, result.len());
            result.push(Segment {
                original_index: index,
                draw_ref_index: index,
                instance_index,
                entity: row.entity,
                stable_instance_key,
                page_id: row.submission_page_id,
                submission_index: Some(row.submission_index),
                draw_ref_rank: Some(current_rank),
                cluster_start_ordinal: u32::try_from(row.entity_cluster_start_ordinal)
                    .map_err(|_| "paired start overflow")?,
                cluster_span_count: u32::try_from(row.entity_cluster_span_count)
                    .map_err(|_| "paired span overflow")?,
                cluster_total_count: u32::try_from(row.entity_cluster_total_count)
                    .map_err(|_| "paired total overflow")?,
                submission_slot: row.submission_slot,
                state: row.state,
                lineage_depth: row.lineage_depth,
                lod_level: row.submission_lod_level,
                frontier_rank: row.frontier_rank,
            });
        }
    } else if let Some(prepare) = inputs.prepare.as_ref() {
        let actual = paired_prepare_from_neutral(prepare);
        // Reuse the original prepare producer's fallback, slot/frontier ordering,
        // and span policy. This does not replace the raw author candidate catalog.
        for (index, draw) in actual.unified_indirect_draws().into_iter().enumerate() {
            let (instance_index, stable_instance_key) = resolve(draw.entity, None)?;
            let index = u32::try_from(index).map_err(|_| "paired submission index overflow")?;
            let rank = ranks.entry(draw.entity).or_default();
            let current_rank = *rank;
            *rank = rank.checked_add(1).ok_or("paired draw-ref rank overflow")?;
            let state = match draw.state {
                crate::virtual_geometry::VirtualGeometryPrepareClusterState::Resident => {
                    State::Resident
                }
                crate::virtual_geometry::VirtualGeometryPrepareClusterState::PendingUpload => {
                    State::PendingUpload
                }
                crate::virtual_geometry::VirtualGeometryPrepareClusterState::Missing => {
                    State::Missing
                }
            };
            result.push(Segment {
                original_index: index,
                draw_ref_index: index,
                instance_index,
                entity: draw.entity,
                stable_instance_key,
                page_id: draw.page_id,
                submission_index: Some(index),
                draw_ref_rank: Some(current_rank),
                cluster_start_ordinal: draw.cluster_start_ordinal,
                cluster_span_count: draw.cluster_span_count,
                cluster_total_count: draw.cluster_total_count,
                submission_slot: draw.submission_slot,
                state,
                lineage_depth: draw.lineage_depth,
                lod_level: draw.lod_level,
                frontier_rank: draw.frontier_rank,
            });
        }
    }
    Ok(Some(result))
}
