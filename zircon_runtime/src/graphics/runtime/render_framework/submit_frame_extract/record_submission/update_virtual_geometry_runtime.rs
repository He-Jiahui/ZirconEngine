use super::super::frame_submission_context::FrameSubmissionContext;
use super::super::prepared_runtime_submission::PreparedRuntimeSubmission;
use super::super::submission_record_update::VirtualGeometryStatSnapshot;
use crate::graphics::runtime::virtual_geometry::normalized_page_table_entries;
use crate::graphics::runtime::virtual_geometry::{
    VirtualGeometryGpuCompletion, VirtualGeometryRuntimeFeedback,
};
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn update_virtual_geometry_runtime(
    context: &FrameSubmissionContext,
    prepared: &mut PreparedRuntimeSubmission,
    virtual_geometry_runtime_feedback: &VirtualGeometryRuntimeFeedback,
) -> VirtualGeometryStatSnapshot {
    let previous_slot_owners = prepared
        .virtual_geometry_runtime()
        .map(|runtime| runtime.resident_slot_owners())
        .unwrap_or_default();
    let previous_pending_pages = prepared
        .virtual_geometry_runtime()
        .map(|runtime| runtime.pending_page_ids())
        .unwrap_or_default();
    let confirmed_completion =
        virtual_geometry_runtime_feedback
            .gpu_completion()
            .map(|completion| {
                confirmed_virtual_geometry_completion(
                    completion,
                    previous_slot_owners.iter().copied(),
                    previous_pending_pages.iter().copied(),
                )
            });
    let indirect_segment_count = prepared
        .virtual_geometry_prepare()
        .map(|prepare| prepare.drawable_indirect_segment_count())
        .unwrap_or_default();
    let completed_page_count = confirmed_completion
        .as_ref()
        .map(|completion| completion.completed_page_assignments().len())
        .unwrap_or(0);
    let replaced_page_count = confirmed_completion
        .as_ref()
        .map(|completion| completion.completed_page_replacements().len())
        .unwrap_or(0);

    if let Some(runtime) = prepared.virtual_geometry_runtime_mut() {
        if let Some(feedback) = virtual_geometry_runtime_feedback.visibility_feedback() {
            runtime.refresh_hot_resident_pages(feedback);
        }
        if let Some(completion) = confirmed_completion.as_ref() {
            runtime.complete_gpu_uploads_with_replacements(
                completion.completed_page_assignments().iter().copied(),
                completion.completed_page_replacements().iter().copied(),
                virtual_geometry_runtime_feedback.evictable_page_ids(),
            );
            runtime.apply_gpu_page_table_entries(completion.page_table_entries());
        } else if let Some(feedback) = virtual_geometry_runtime_feedback.visibility_feedback() {
            runtime.consume_feedback(feedback);
        }
        runtime.ingest_page_requests(
            context.predicted_generation(),
            virtual_geometry_runtime_feedback
                .node_and_cluster_cull_page_requests()
                .iter()
                .copied(),
        );
        let snapshot = runtime.snapshot();
        VirtualGeometryStatSnapshot::new(
            snapshot.page_table_entry_count(),
            snapshot.resident_page_count(),
            snapshot.pending_request_count(),
            completed_page_count,
            replaced_page_count,
            indirect_segment_count,
        )
    } else {
        VirtualGeometryStatSnapshot::new(
            0,
            0,
            0,
            completed_page_count,
            replaced_page_count,
            indirect_segment_count,
        )
    }
}

fn confirmed_virtual_geometry_completion(
    completion: &VirtualGeometryGpuCompletion,
    previous_slot_owners: impl IntoIterator<Item = (u32, u32)>,
    previous_pending_pages: impl IntoIterator<Item = u32>,
) -> VirtualGeometryGpuCompletion {
    let page_table_entries = normalized_page_table_entries(completion.page_table_entries());
    let page_table_slot_by_page = page_table_entries
        .iter()
        .copied()
        .collect::<BTreeMap<u32, u32>>();
    let previous_pending_pages = previous_pending_pages.into_iter().collect::<BTreeSet<_>>();
    let final_resident_pages = page_table_slot_by_page
        .keys()
        .copied()
        .collect::<BTreeSet<_>>();
    let previous_page_by_slot = previous_slot_owners.into_iter().collect::<BTreeMap<_, _>>();
    let completed_page_assignments = page_table_entries
        .iter()
        .filter(|(page_id, _slot)| previous_pending_pages.contains(page_id))
        .copied()
        .collect::<Vec<_>>();
    let completed_page_replacements = page_table_entries
        .iter()
        .filter(|(page_id, _slot)| previous_pending_pages.contains(page_id))
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
mod tests {
    use crate::core::framework::render::{
        RenderPipelineHandle, RenderVirtualGeometryExtract, RenderVirtualGeometryPage,
    };
    use crate::core::math::UVec2;
    use crate::scene::world::World;

    use crate::{
        runtime::render_framework::submit_frame_extract::frame_submission_context::UiSubmissionStats,
        runtime::virtual_geometry::{VirtualGeometryGpuCompletion, VirtualGeometryRuntimeFeedback},
        runtime::HybridGiSceneInputs,
        types::VirtualGeometryPrepareRequest,
        RenderPipelineAsset, RenderPipelineCompileOptions, VisibilityContext,
        VisibilityVirtualGeometryFeedback, VisibilityVirtualGeometryPageUploadPlan,
    };

    use super::*;

    #[test]
    fn gpu_completion_path_still_refreshes_virtual_geometry_hot_frontier_feedback_for_next_prepare()
    {
        let context = frame_submission_context(VisibilityVirtualGeometryFeedback {
            visible_cluster_ids: Vec::new(),
            requested_pages: vec![700],
            evictable_pages: vec![200, 300],
            hot_resident_pages: vec![200],
        });
        let mut prepared = prepared_runtime_submission();

        update_virtual_geometry_runtime(
            &context,
            &mut prepared,
            &VirtualGeometryRuntimeFeedback::new(
                Some(VirtualGeometryGpuCompletion::new(
                    vec![(200, 0), (300, 1)],
                    Vec::new(),
                    Vec::new(),
                )),
                Vec::new(),
                context.virtual_geometry_feedback().cloned(),
            ),
        );

        let runtime = prepared
            .virtual_geometry_runtime_mut()
            .expect("expected virtual geometry runtime");
        runtime.ingest_plan(
            2,
            &VisibilityVirtualGeometryPageUploadPlan {
                resident_pages: vec![200, 300],
                requested_pages: vec![700],
                dirty_requested_pages: Vec::new(),
                evictable_pages: vec![200, 300],
            },
        );

        let prepare = runtime.build_prepare_frame(&[]);
        assert_eq!(
            prepare.pending_page_requests,
            vec![VirtualGeometryPrepareRequest {
                page_id: 700,
                size_bytes: 4096,
                generation: 1,
                frontier_rank: 0,
                assigned_slot: Some(1),
                recycled_page_id: Some(300),
            }],
            "expected the GPU-completion branch to refresh hot-frontier feedback before applying page-table truth so the next prepare recycle plan still preserves the recently-hot frontier page"
        );
    }

    #[test]
    fn gpu_completion_path_keeps_request_pending_when_page_table_truth_rejects_completed_assignment(
    ) {
        let context = frame_submission_context(VisibilityVirtualGeometryFeedback {
            visible_cluster_ids: Vec::new(),
            requested_pages: vec![700],
            evictable_pages: vec![200, 300],
            hot_resident_pages: Vec::new(),
        });
        let mut prepared = prepared_runtime_submission();

        let stats = update_virtual_geometry_runtime(
            &context,
            &mut prepared,
            &VirtualGeometryRuntimeFeedback::new(
                Some(VirtualGeometryGpuCompletion::new(
                    vec![(200, 0), (300, 1)],
                    vec![(700, 1)],
                    vec![(700, 300)],
                )),
                Vec::new(),
                context.virtual_geometry_feedback().cloned(),
            ),
        );

        let runtime = prepared
            .virtual_geometry_runtime_mut()
            .expect("expected virtual geometry runtime");
        runtime.ingest_plan(
            2,
            &VisibilityVirtualGeometryPageUploadPlan {
                resident_pages: vec![200, 300],
                requested_pages: vec![700],
                dirty_requested_pages: Vec::new(),
                evictable_pages: vec![200, 300],
            },
        );

        let prepare = runtime.build_prepare_frame(&[]);
        assert_eq!(
            prepare.pending_page_requests,
            vec![VirtualGeometryPrepareRequest {
                page_id: 700,
                size_bytes: 4096,
                generation: 1,
                frontier_rank: 0,
                assigned_slot: Some(0),
                recycled_page_id: Some(200),
            }],
            "expected page-table truth to preserve the preexisting pending request when the reported completed assignment never becomes resident in the final GPU page table, instead of silently requiring a new dirty request to recreate it"
        );
        assert_eq!(
            stats.completed_page_count(), 0,
            "expected runtime stats to follow confirmed page-table truth instead of counting a rejected completed assignment as successful GPU completion"
        );
        assert_eq!(
            stats.replaced_page_count(), 0,
            "expected rejected completed assignments to stop contributing replacement pressure once the final GPU page table does not retain them"
        );
    }

    #[test]
    fn node_and_cluster_cull_page_requests_become_runtime_pending_upload_requests() {
        let context = frame_submission_context(VisibilityVirtualGeometryFeedback {
            visible_cluster_ids: Vec::new(),
            requested_pages: Vec::new(),
            evictable_pages: vec![200, 300],
            hot_resident_pages: Vec::new(),
        });
        let mut runtime = crate::graphics::runtime::VirtualGeometryRuntimeState::default();
        runtime.register_extract(Some(&RenderVirtualGeometryExtract {
            cluster_budget: 3,
            page_budget: 2,
            clusters: Vec::new(),
            hierarchy_nodes: Vec::new(),
            hierarchy_child_ids: Vec::new(),
            pages: vec![
                page(200, true, 2048),
                page(300, true, 2048),
                page(700, false, 4096),
            ],
            instances: Vec::new(),
            debug: Default::default(),
        }));
        runtime.ingest_plan(
            1,
            &VisibilityVirtualGeometryPageUploadPlan {
                resident_pages: vec![200, 300],
                requested_pages: Vec::new(),
                dirty_requested_pages: Vec::new(),
                evictable_pages: vec![200, 300],
            },
        );
        let mut prepared = PreparedRuntimeSubmission::new(
            None,
            None,
            None,
            None,
            Vec::new(),
            Some(runtime),
            None,
            vec![200, 300],
        );

        update_virtual_geometry_runtime(
            &context,
            &mut prepared,
            &VirtualGeometryRuntimeFeedback::new(
                None,
                vec![700],
                context.virtual_geometry_feedback().cloned(),
            ),
        );

        let prepare = prepared
            .virtual_geometry_runtime()
            .expect("expected virtual geometry runtime")
            .build_prepare_frame(&[]);
        assert_eq!(
            prepare
                .pending_page_requests
                .iter()
                .map(|request| (
                    request.page_id,
                    request.size_bytes,
                    request.generation,
                    request.frontier_rank
                ))
                .collect::<Vec<_>>(),
            vec![(700, 4096, 2, 0)],
            "expected NodeAndClusterCull page-request feedback collected after rendering to become the next runtime pending upload request with the current predicted frame generation and request order"
        );
    }

    #[test]
    fn gpu_completion_path_ignores_reported_replacement_when_previous_slot_owner_stays_resident() {
        let context = frame_submission_context(VisibilityVirtualGeometryFeedback {
            visible_cluster_ids: Vec::new(),
            requested_pages: vec![700],
            evictable_pages: vec![200, 300],
            hot_resident_pages: Vec::new(),
        });
        let mut runtime = crate::graphics::runtime::VirtualGeometryRuntimeState::default();
        runtime.register_extract(Some(&RenderVirtualGeometryExtract {
            cluster_budget: 3,
            page_budget: 3,
            clusters: Vec::new(),
            hierarchy_nodes: Vec::new(),
            hierarchy_child_ids: Vec::new(),
            pages: vec![
                page(200, true, 2048),
                page(300, true, 2048),
                page(700, false, 4096),
            ],
            instances: Vec::new(),
            debug: Default::default(),
        }));
        runtime.ingest_plan(
            1,
            &VisibilityVirtualGeometryPageUploadPlan {
                resident_pages: vec![200, 300],
                requested_pages: vec![700],
                dirty_requested_pages: vec![700],
                evictable_pages: vec![200, 300],
            },
        );
        let mut prepared = PreparedRuntimeSubmission::new(
            None,
            None,
            None,
            None,
            Vec::new(),
            Some(runtime),
            None,
            vec![200, 300],
        );

        let stats = update_virtual_geometry_runtime(
            &context,
            &mut prepared,
            &VirtualGeometryRuntimeFeedback::new(
                Some(VirtualGeometryGpuCompletion::new(
                    vec![(200, 0), (700, 1), (300, 2)],
                    vec![(700, 1)],
                    vec![(700, 300)],
                )),
                Vec::new(),
                context.virtual_geometry_feedback().cloned(),
            ),
        );

        let runtime = prepared
            .virtual_geometry_runtime()
            .expect("expected virtual geometry runtime");
        assert_eq!(
            runtime.page_slot(300),
            Some(2),
            "expected final GPU page-table truth to keep the previous slot owner resident in its reassigned slot"
        );
        assert_eq!(
            stats.completed_page_count(),
            1,
            "expected page-table-confirmed completion to keep counting the finished page upload"
        );
        assert_eq!(
            stats.replaced_page_count(), 0,
            "expected replacement pressure to ignore stale reported recycled-page ids when the previous slot owner still remains resident in the final GPU page table"
        );
    }

    #[test]
    fn confirmed_virtual_geometry_completion_uses_previous_slot_owner_when_reported_replacement_is_stale(
    ) {
        let confirmed = confirmed_virtual_geometry_completion(
            &VirtualGeometryGpuCompletion::new(
                vec![(200, 0), (700, 1)],
                vec![(700, 1)],
                vec![(700, 200)],
            ),
            [(0, 200), (1, 300)],
            [700],
        );

        assert_eq!(
            confirmed.completed_page_replacements(),
            vec![(700, 300)],
            "expected confirmed replacement truth to follow the previous owner of the final resident slot instead of trusting a stale GPU replacement id from another slot"
        );
    }

    #[test]
    fn gpu_completion_path_infers_confirmed_completion_from_final_page_table_when_raw_completion_is_missing(
    ) {
        let context = frame_submission_context(VisibilityVirtualGeometryFeedback {
            visible_cluster_ids: Vec::new(),
            requested_pages: vec![700],
            evictable_pages: vec![200, 300],
            hot_resident_pages: Vec::new(),
        });
        let mut prepared = prepared_runtime_submission();

        let stats = update_virtual_geometry_runtime(
            &context,
            &mut prepared,
            &VirtualGeometryRuntimeFeedback::new(
                Some(VirtualGeometryGpuCompletion::new(
                    vec![(200, 0), (700, 1)],
                    Vec::new(),
                    Vec::new(),
                )),
                Vec::new(),
                context.virtual_geometry_feedback().cloned(),
            ),
        );

        let runtime = prepared
            .virtual_geometry_runtime()
            .expect("expected virtual geometry runtime");
        assert_eq!(
            runtime.page_slot(700),
            Some(1),
            "expected final GPU page-table truth to promote the previously pending page into its confirmed resident slot even when raw completed-assignment readback is missing"
        );
        assert_eq!(
            runtime.page_slot(300),
            None,
            "expected the previous slot owner to be evicted once final page-table truth proves the pending page took over that slot"
        );
        assert_eq!(
            runtime.pending_requests().len(),
            0,
            "expected page-table-confirmed completion to clear the pending request even without a raw completed-assignment record"
        );
        assert_eq!(
            stats.completed_page_count(), 1,
            "expected runtime stats to infer confirmed completion from final page-table truth when the pending page is now resident"
        );
        assert_eq!(
            stats.replaced_page_count(), 1,
            "expected replacement pressure to be reconstructed from the confirmed slot owner that disappeared from the final page table, even without raw replacement readback"
        );
    }

    #[test]
    fn confirmed_virtual_geometry_completion_normalizes_reassigned_page_table_truth_before_runtime_apply(
    ) {
        let confirmed = confirmed_virtual_geometry_completion(
            &VirtualGeometryGpuCompletion::new(
                vec![(200, 0), (300, 1), (700, 1), (300, 2)],
                vec![(700, 1)],
                Vec::new(),
            ),
            [(0, 200), (1, 300)],
            [700],
        );

        assert_eq!(
            confirmed.page_table_entries(),
            vec![(200, 0), (700, 1), (300, 2)],
            "expected confirmed completion to normalize raw page-table readback into the final last-writer slot ownership so runtime apply does not lose a resident page that moved to a new slot in the same GPU snapshot"
        );
        assert_eq!(
            confirmed.completed_page_assignments(),
            vec![(700, 1)],
            "expected normalized page-table truth to keep the pending page completion while ignoring stale intermediate ownership for the page that was later reassigned"
        );
    }

    #[test]
    fn confirmed_virtual_geometry_completion_deduplicates_replacement_truth_after_page_table_normalization(
    ) {
        let confirmed = confirmed_virtual_geometry_completion(
            &VirtualGeometryGpuCompletion::new(
                vec![(200, 0), (700, 1), (700, 2)],
                vec![(700, 1), (700, 2)],
                vec![(700, 300), (700, 400)],
            ),
            [(0, 200), (1, 300), (2, 400)],
            [700],
        );

        assert_eq!(
            confirmed.completed_page_replacements(),
            vec![(700, 400)],
            "expected normalized page-table truth to count replacement pressure once for the final surviving slot owner instead of duplicating replacement stats for stale intermediate entries of the same pending page"
        );
    }

    fn frame_submission_context(
        feedback: VisibilityVirtualGeometryFeedback,
    ) -> FrameSubmissionContext {
        let mut extract = World::new().to_render_frame_extract();
        extract.apply_viewport_size(UVec2::new(32, 32));
        let compiled_pipeline = RenderPipelineAsset::default_forward_plus()
            .compile_with_options(&extract, &RenderPipelineCompileOptions::default())
            .expect("expected test pipeline to compile");

        FrameSubmissionContext::new(
            UVec2::new(32, 32),
            RenderPipelineHandle::new(1),
            None,
            compiled_pipeline,
            VisibilityContext::default(),
            UiSubmissionStats::default(),
            None,
            None,
            false,
            true,
            None,
            HybridGiSceneInputs::default(),
            None,
            None,
            None,
            Vec::new(),
            Vec::new(),
            None,
            Some(feedback),
            2,
        )
    }

    fn prepared_runtime_submission() -> PreparedRuntimeSubmission {
        let mut runtime = crate::graphics::runtime::VirtualGeometryRuntimeState::default();
        runtime.register_extract(Some(&RenderVirtualGeometryExtract {
            cluster_budget: 3,
            page_budget: 2,
            clusters: Vec::new(),
            hierarchy_nodes: Vec::new(),
            hierarchy_child_ids: Vec::new(),
            pages: vec![
                page(200, true, 2048),
                page(300, true, 2048),
                page(700, false, 4096),
            ],
            instances: Vec::new(),
            debug: Default::default(),
        }));
        runtime.ingest_plan(
            1,
            &VisibilityVirtualGeometryPageUploadPlan {
                resident_pages: vec![200, 300],
                requested_pages: vec![700],
                dirty_requested_pages: vec![700],
                evictable_pages: vec![200, 300],
            },
        );

        PreparedRuntimeSubmission::new(
            None,
            None,
            None,
            None,
            Vec::new(),
            Some(runtime),
            None,
            vec![200, 300],
        )
    }

    fn page(page_id: u32, resident: bool, size_bytes: u64) -> RenderVirtualGeometryPage {
        RenderVirtualGeometryPage {
            page_id,
            resident,
            size_bytes,
        }
    }
}
