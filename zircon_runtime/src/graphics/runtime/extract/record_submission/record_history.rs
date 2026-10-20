//! 历史登记把本次图句柄与相机键一起发布；旧句柄由成功提交后的调用链释放。
use crate::core::framework::render::{
    FrameHistoryHandle, FrameHistoryStatus, RenderFrameworkError,
};

use crate::graphics::runtime::ViewportFrameHistory;

use super::super::super::viewport_record::ViewportRecord;
use super::super::frame_submission_context::FrameSubmissionContext;

pub(in crate::graphics::runtime::render_framework::submit_frame_extract) fn record_history(
    record: &mut ViewportRecord,
    context: &FrameSubmissionContext,
    generation: u64,
    allocated_history: Option<FrameHistoryHandle>,
) -> Result<
    (
        Option<FrameHistoryHandle>,
        FrameHistoryHandle,
        FrameHistoryStatus,
    ),
    RenderFrameworkError,
> {
    let camera_history_key = context.camera_history_key();
    let previous_handle = record
        .history(camera_history_key)
        .map(|history| history.handle());
    let history_handle = match (record.history_mut(camera_history_key), allocated_history) {
        (Some(history), None) => {
            history.update(
                generation,
                context.compiled_pipeline().history_bindings.clone(),
                context.visibility_context().history_snapshot.clone(),
                context.visibility_context().static_index().clone(),
                context.visibility_context().dynamic_index().clone(),
                context.history_validation_key_shared(),
            );
            history.handle()
        }
        (_, Some(handle)) => {
            record.replace_history(
                camera_history_key.clone(),
                ViewportFrameHistory::new(
                    handle,
                    context.size(),
                    context.render_size(),
                    context.pipeline_handle(),
                    generation,
                    context.compiled_pipeline().history_bindings.clone(),
                    context.visibility_context().history_snapshot.clone(),
                    context.visibility_context().static_index().clone(),
                    context.visibility_context().dynamic_index().clone(),
                    context.history_validation_key_shared(),
                ),
            );
            handle
        }
        (None, None) => {
            return Err(RenderFrameworkError::InvalidSubmissionState {
                state: "history rotation is missing for the submitted camera",
            });
        }
    };

    record.accept_virtual_geometry_readbacks(
        context.virtual_geometry_readback_scope().as_ref(),
        camera_history_key,
        generation,
    );

    let previous_available = previous_handle.is_some()
        && allocated_history.is_none()
        && context.history_invalidation_reason().is_none();
    let invalidation_reason = if previous_available {
        None
    } else {
        context.history_invalidation_reason()
    };
    let history_status = FrameHistoryStatus::new(
        Some(history_handle),
        previous_handle,
        previous_available,
        invalidation_reason,
        context.size(),
        context.render_size(),
    );

    Ok((previous_handle, history_handle, history_status))
}

#[cfg(test)]
#[path = "../../framework/extract/tests/record_history.rs"]
mod tests;

#[cfg(all(test, feature = "native-host", feature = "test-support"))]
mod accepted_scope_tests {
    use super::record_history;
    use crate::core::framework::render::{
        FrameHistoryHandle, RenderFrameHistoryInput, RenderFramework, RenderFrameworkError,
    };
    use crate::graphics::runtime::render_framework::{
        render_framework_state::RenderFrameworkState,
        submit_frame_extract::{
            build_frame_submission_context::build_frame_submission_context_from_runtime_frame_extract,
            prepare_runtime_submission::prepare_runtime_submission,
        },
        wgpu_render_framework::WgpuRenderFrameworkAccess,
    };
    use crate::graphics::{ViewportRenderFrame, WgpuRenderFramework};
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    };

    fn real_completed_packet_then_history_commit(mode: u64, commit: u8) {
        let updates = Arc::new(AtomicUsize::new(0));
        let (framework, viewport) =
            WgpuRenderFramework::terminal_readback_test_fixture(Arc::clone(&updates));
        let extract = Arc::new(WgpuRenderFramework::terminal_readback_test_extract(mode));
        let mut context = build_frame_submission_context_from_runtime_frame_extract(
            &framework, viewport, &extract, None,
        )
        .unwrap();
        let mut state = framework.lock_state();
        let generation = state.renderer.next_frame_generation();
        assert_eq!(generation, context.predicted_generation());
        context.set_virtual_geometry_readback_scope(Some(
            state
                .viewports
                .get(&viewport)
                .unwrap()
                .virtual_geometry_readback_scope(
                    framework.virtual_geometry_readback_budget(),
                    viewport,
                    generation,
                )
                .unwrap(),
        ));
        let prepared = prepare_runtime_submission(&mut state, viewport, &context).unwrap();
        let history_handle = FrameHistoryHandle::new(state.next_history_id);
        state.next_history_id += 1;
        let mut frame =
            ViewportRenderFrame::from_shared_extract(context.submission_extract(), context.size())
                .with_prepared_runtime_sidebands(prepared.into_prepared_runtime_sidebands());
        frame.virtual_geometry_readback_scope = context.virtual_geometry_readback_scope();
        frame.virtual_geometry_execution_inputs = context.virtual_geometry_execution_inputs();
        let submitted = state.renderer
            .render_frame_with_pipeline_async_capture_task_pool_with_environment_ibl_bake_reservation(
                &frame, context.compiled_pipeline(), context.capabilities(),
                RenderFrameHistoryInput::new(Some(history_handle), false, context.history_invalidation_reason()),
                framework.compute_task_pool(), None, None, false,
            ).unwrap();
        assert_eq!(submitted.generation(), generation);
        let RenderFrameworkState {
            renderer,
            viewports,
            ..
        } = &mut *state;
        let record = viewports.get_mut(&viewport).unwrap();
        record.admit_virtual_geometry_readbacks(
            renderer,
            viewport,
            context.virtual_geometry_readback_scope().as_ref(),
            context.camera_history_key(),
            generation,
        );
        assert_eq!(record.virtual_geometry_pending_readback_count(), 1);
        // This is a real GPU wait, deliberately without record observation or history acceptance.
        renderer
            .finish_virtual_geometry_native_readbacks_for_test()
            .unwrap();
        assert!(renderer
            .test_support_last_virtual_geometry_readback_outputs()
            .is_none());
        assert_eq!(updates.load(Ordering::SeqCst), 0);
        if commit == 2 {
            // Same viewport/generations/history, different actual scope reservation.
            context.set_virtual_geometry_readback_scope(Some(
                record
                    .virtual_geometry_readback_scope(
                        framework.virtual_geometry_readback_budget(),
                        viewport,
                        generation,
                    )
                    .unwrap(),
            ));
        }
        let result = record_history(
            record,
            &context,
            generation,
            (commit != 0).then_some(history_handle),
        );
        if commit == 0 {
            assert!(matches!(
                result,
                Err(RenderFrameworkError::InvalidSubmissionState {
                    state: "history rotation is missing for the submitted camera",
                })
            ));
            assert!(record.history(context.camera_history_key()).is_none());
        } else {
            result.unwrap();
            assert_eq!(
                record
                    .history(context.camera_history_key())
                    .unwrap()
                    .handle(),
                history_handle
            );
        }
        drop(state);
        framework.finish_submission().unwrap();
        let state = framework.lock_state();
        let record = state.viewports.get(&viewport).unwrap();
        assert_eq!(record.virtual_geometry_pending_readback_count(), 0);
        assert_eq!(updates.load(Ordering::SeqCst), usize::from(commit == 1));
        assert_eq!(
            state
                .renderer
                .test_support_last_virtual_geometry_readback_outputs()
                .is_some(),
            commit == 1,
        );
        assert_eq!(
            state
                .renderer
                .test_support_last_virtual_geometry_native_inspection()
                .is_some(),
            false,
        );
        assert_eq!(
            record
                .native_probe_scope_id(state.renderer.virtual_geometry_device_epoch())
                .is_some(),
            commit == 1,
            "the same actual probe is published only by the exact accepted scope",
        );
        drop(state);
        framework.destroy_viewport(viewport).unwrap();
        // Rejected rotation never transferred this real renderer history into the viewport.
        if commit == 0 {
            framework
                .lock_state()
                .renderer
                .release_history(history_handle);
        }
    }

    #[test]
    fn completed_native_packet_is_discarded_after_real_history_commit_rejection() {
        real_completed_packet_then_history_commit(0, 0);
        real_completed_packet_then_history_commit(1, 0);
    }

    #[test]
    fn completed_native_packet_is_published_only_after_exact_history_commit() {
        real_completed_packet_then_history_commit(0, 1);
    }

    #[test]
    fn completed_native_packet_rejects_another_scope_with_equal_generations() {
        real_completed_packet_then_history_commit(0, 2);
    }
}
