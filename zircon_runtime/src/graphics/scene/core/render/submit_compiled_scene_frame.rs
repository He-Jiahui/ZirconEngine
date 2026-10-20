use crate::core::framework::render::IblBakeArtifactRequest;
use crate::graphics::backend::RenderBackend;
use crate::graphics::pipeline::CompiledRenderPipeline;
use crate::graphics::scene::resources::ResourceStreamer;
#[cfg(feature = "native-host")]
use crate::graphics::scene::scene_renderer::environment::ibl_bake_runtime_writeback::{
    IblBakeRuntimeGraphWritebackQueue, PreparedIblBakeRuntimeGraphWriteback,
};
use crate::graphics::scene::scene_renderer::environment::RealtimeIblPendingSubmission;
use crate::graphics::scene::scene_renderer::graph_execution::{
    RenderGraphExecutionRecord, RenderGraphExecutionResources,
};
use crate::graphics::scene::scene_renderer::history::{
    SceneFrameHistoryTextures, SceneHistoryFrameTransaction,
};
use crate::graphics::types::GraphicsError;
#[cfg(test)]
use crate::graphics::types::ViewportRenderFrame;
use crate::graphics::EnvironmentIblBakeReservation;
use crate::render_graph::CompiledRenderGraph;
use crate::rhi::SubmissionTicket;
use zr_rhi_wgpu::{
    WgpuNativeDiagnosticQueryFrame, WgpuNativeDiagnosticReadbackFrame, WgpuNativeSurfaceFrameTarget,
};

use super::super::super::scene_renderer_core::SceneRendererCore;

#[path = "../../../core/submit_compiled_scene_frame/hzb_readback.rs"]
mod hzb_readback;
use hzb_readback::attach_hzb_occlusion_readback_stats;

#[path = "compiled_scene_graph_receipt.rs"]
mod graph_receipt;
use graph_receipt::build_graph_execution_receipt;
#[cfg(test)]
use graph_receipt::{optional_external_absence_bounds, rhi_access_range};

#[cfg(test)]
#[path = "compiled_scene_readback.rs"]
mod readback_diagnostics;
#[cfg(test)]
use readback_diagnostics::{
    attach_color_lut_readback_stats, attach_exposure_readback_stats,
    attach_scene_velocity_readback_stats,
};

pub(super) struct CompiledSceneFrameSubmissionContext<'a> {
    pub(super) backend: &'a RenderBackend,
    pub(super) pipeline: &'a CompiledRenderPipeline,
    #[cfg(test)]
    pub(super) device: &'a wgpu::Device,
    #[cfg(test)]
    pub(super) queue: &'a wgpu::Queue,
    pub(super) command_buffers: Vec<wgpu::CommandBuffer>,
    #[cfg(test)]
    pub(super) streamer: &'a ResourceStreamer,
    #[cfg(test)]
    pub(super) frame: &'a ViewportRenderFrame,
    pub(super) graph_resources: &'a mut RenderGraphExecutionResources,
    pub(super) graph_execution_record: &'a mut RenderGraphExecutionRecord,
    #[cfg(feature = "native-host")]
    pub(super) prepared_ibl_writeback: Option<PreparedIblBakeRuntimeGraphWriteback>,
    pub(super) environment_ibl_prepare_error: Option<GraphicsError>,
    pub(super) realtime_ibl_submission: Option<RealtimeIblPendingSubmission>,
    pub(super) diagnostic_frame_index: u64,
    pub(super) product_diagnostic_frame: Option<WgpuNativeDiagnosticReadbackFrame>,
    pub(super) product_diagnostic_query_frame: Option<WgpuNativeDiagnosticQueryFrame>,
    pub(super) surface_target: Option<&'a WgpuNativeSurfaceFrameTarget>,
    pub(super) history_textures: Option<&'a mut SceneFrameHistoryTextures>,
    pub(super) history_frame_transaction: SceneHistoryFrameTransaction,
    pub(super) frame_generation: u64,
    pub(super) exposure_history_reset_prepared: bool,
}

impl SceneRendererCore {
    pub(super) fn submit_compiled_scene_frame(
        &mut self,
        ctx: CompiledSceneFrameSubmissionContext<'_>,
    ) -> Result<SubmissionTicket, GraphicsError> {
        let CompiledSceneFrameSubmissionContext {
            backend,
            pipeline,
            #[cfg(test)]
            device,
            #[cfg(test)]
            queue,
            command_buffers,
            #[cfg(test)]
            streamer,
            #[cfg(test)]
            frame,
            graph_resources,
            graph_execution_record,
            #[cfg(feature = "native-host")]
            prepared_ibl_writeback,
            environment_ibl_prepare_error,
            realtime_ibl_submission,
            diagnostic_frame_index,
            product_diagnostic_frame,
            product_diagnostic_query_frame,
            surface_target,
            history_textures,
            history_frame_transaction,
            frame_generation,
            exposure_history_reset_prepared,
        } = ctx;

        debug_assert!(!command_buffers.is_empty());
        let graph_execution_receipt =
            build_graph_execution_receipt(backend, pipeline, graph_resources, frame_generation)?;
        let submission_ticket = backend
            .submit_graphics_command_buffers_with_graph_receipt_and_frame_diagnostics_and_surface(
                command_buffers,
                graph_execution_receipt,
                product_diagnostic_frame,
                product_diagnostic_query_frame,
                surface_target,
            )?;
        let history_domains_report = if let Some(history) = history_textures {
            if exposure_history_reset_prepared {
                let committed = history.commit_exposure_history_reset();
                debug_assert!(committed);
            }
            history.commit_history_frame(history_frame_transaction, frame_generation)
        } else {
            Default::default()
        };
        graph_execution_record.set_history_domains_report(history_domains_report);
        self.mesh_pipelines
            .bind_recorded_pipeline_usage_to_submission(submission_ticket);
        self.commit_scene_environment_frame();
        self.mesh_pipelines
            .reflection_probes
            .commit_pending_uploads();
        #[cfg(feature = "native-host")]
        if let Some(prepared) = prepared_ibl_writeback {
            self.ibl_bake_runtime_writebacks.commit_submitted(prepared);
        }
        if let Some(submission) = realtime_ibl_submission {
            self.realtime_ibl.complete_submission(submission, true);
        }
        #[cfg(test)]
        attach_scene_velocity_readback_stats(
            device,
            queue,
            graph_resources,
            graph_execution_record,
        );
        #[cfg(test)]
        let exposure_readback_report =
            attach_exposure_readback_stats(device, queue, graph_resources, graph_execution_record);
        #[cfg(test)]
        attach_color_lut_readback_stats(
            device,
            queue,
            streamer,
            frame,
            graph_resources,
            exposure_readback_report,
            graph_execution_record,
        );
        if let Some(hzb_occlusion_culler) = self.hzb_occlusion_culler.as_ref() {
            attach_hzb_occlusion_readback_stats(
                hzb_occlusion_culler,
                Some(diagnostic_frame_index),
                graph_execution_record,
            );
        }
        graph_resources.retire_transient_backings_after_submission(
            &mut self.transient_resource_pool,
            submission_ticket,
        );
        graph_execution_record.set_graph_submission(submission_ticket);
        self.transient_resource_pool.end_frame();
        graph_execution_record.set_resource_report(
            graph_execution_record
                .resource_report()
                .with_transient_pool_report(self.transient_resource_pool.last_frame_report()),
        );
        let environment_ibl_error = environment_ibl_prepare_error.map(|error| error.to_string());
        if environment_ibl_error.is_some() {
            return Err(GraphicsError::FrameFailedAfterSceneSubmission {
                scene_submission: submission_ticket,
                source: Box::new(GraphicsError::SceneSubmissionFinalization {
                    readback: None,
                    environment_ibl: environment_ibl_error,
                }),
            });
        }
        Ok(submission_ticket)
    }
}

#[cfg(feature = "native-host")]
pub(super) fn prepare_environment_ibl_runtime_cache_writeback(
    writebacks: &IblBakeRuntimeGraphWritebackQueue,
    backend: &RenderBackend,
    streamer: &ResourceStreamer,
    request: Option<IblBakeArtifactRequest>,
    reservation: Option<EnvironmentIblBakeReservation>,
    graph_resources: &RenderGraphExecutionResources,
    graph: &CompiledRenderGraph,
) -> Result<Option<PreparedIblBakeRuntimeGraphWriteback>, GraphicsError> {
    let Some(reservation) = reservation else {
        return Ok(None);
    };
    let Some(request) = request else {
        return Ok(None);
    };
    let Some(store) = streamer.asset_manager()?.ibl_bake_artifact_cache_store() else {
        return Ok(None);
    };
    writebacks
        .prepare(backend, store, request, reservation, graph_resources, graph)
        .map_err(|error| GraphicsError::Asset(error.to_string()))
}

#[cfg(test)]
#[path = "../../renderer/core/render/render/tests/cases.rs"]
mod submission_order_tests;
