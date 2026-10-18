use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::core::framework::render::{
    CapturedFrame, RenderParticlePreviousSpriteSnapshot, RenderPipelineHandle,
    RenderQualityProfile, RenderViewportDescriptor, RenderVirtualGeometryDebugSnapshot,
    ViewportCameraSnapshot,
};

use crate::graphics::{
    backend::ViewportSurface, runtime::ViewportFrameHistory, CompiledRenderPipeline,
    HybridGiRuntimeState, VirtualGeometryRuntimeState,
};

use super::ViewportCameraHistoryKey;

pub(in crate::graphics::runtime::render_framework) struct ViewportRecord {
    pub(super) descriptor: RenderViewportDescriptor,
    pub(super) pipeline: Option<RenderPipelineHandle>,
    pub(super) quality_profile: Option<RenderQualityProfile>,
    pub(super) generation: u64,
    pub(super) virtual_geometry_readback_slots: Arc<std::sync::atomic::AtomicUsize>,
    pub(super) last_virtual_geometry_native_probe: Mutex<
        Option<(
            ViewportCameraHistoryKey,
            crate::graphics::scene::VirtualGeometryReadyReadback,
        )>,
    >,
    pub(super) pending_virtual_geometry_readbacks:
        std::collections::VecDeque<PendingViewportVirtualGeometryReadback>,
    pub(super) last_applied_virtual_geometry_generation: HashMap<ViewportCameraHistoryKey, u64>,
    pub(super) virtual_geometry_readback_epoch:
        Option<crate::graphics::scene::RenderPassDeviceEpoch>,
    pub(super) temporal_frame_index: u64,
    pub(super) compiled_pipeline: Option<Arc<CompiledRenderPipeline>>,
    pub(super) last_capture_pipeline: Option<Arc<CompiledRenderPipeline>>,
    pub(super) capture_mailbox: Arc<Mutex<ViewportAsyncCaptureMailbox>>,
    pub(super) pending_capture_profiles:
        std::collections::BTreeMap<u64, Vec<crate::core::framework::render::RenderFrameProfile>>,
    pub(super) last_promoted_capture_generation: Option<u64>,
    pub(super) hybrid_gi_runtimes: HashMap<ViewportCameraHistoryKey, Box<dyn HybridGiRuntimeState>>,
    pub(super) virtual_geometry_runtimes:
        HashMap<ViewportCameraHistoryKey, Box<dyn VirtualGeometryRuntimeState>>,
    pub(super) light_grid_reports:
        HashMap<ViewportCameraHistoryKey, crate::graphics::scene::RenderGraphLightGridReport>,
    pub(super) virtual_geometry_debug_snapshots:
        HashMap<ViewportCameraHistoryKey, Arc<RenderVirtualGeometryDebugSnapshot>>,
    pub(super) last_capture: Option<CapturedFrame>,
    pub(super) last_visible_spatial_query:
        Option<crate::core::framework::render::RenderVisibleSpatialQuerySnapshot>,
    pub(super) camera_histories: HashMap<ViewportCameraHistoryKey, ViewportFrameHistory>,
    pub(super) motion_vector_cameras: HashMap<ViewportCameraHistoryKey, ViewportCameraSnapshot>,
    pub(super) particle_previous_sprites:
        HashMap<ViewportCameraHistoryKey, Vec<RenderParticlePreviousSpriteSnapshot>>,
    pub(super) surface: Option<ViewportSurface>,
}

pub(super) struct ViewportAsyncCaptureMailbox {
    pub(super) armed: std::collections::BTreeSet<u64>,
    pub(super) pending: std::collections::BTreeMap<u64, PendingViewportCapture>,
    pub(super) completed: std::collections::BTreeMap<u64, Result<Vec<u8>, String>>,
    pub(super) ready: Option<ReadyViewportCapture>,
}

pub(super) struct PendingViewportCapture {
    pub(super) size: crate::core::math::UVec2,
    pub(super) capture_report: crate::core::framework::render::RenderCaptureReport,
    pub(super) pipeline: Arc<CompiledRenderPipeline>,
}

pub(super) struct ReadyViewportCapture {
    pub(super) capture: CapturedFrame,
    pub(super) pipeline: Arc<CompiledRenderPipeline>,
}

pub(super) struct PendingViewportVirtualGeometryReadback {
    pub(super) history: ViewportCameraHistoryKey,
    pub(super) packet: crate::graphics::scene::VirtualGeometryTicketedReadback,
    pub(super) ready: Option<crate::graphics::scene::VirtualGeometryReadyReadback>,
}

impl ViewportRecord {
    pub(in crate::graphics::runtime::render_framework) fn native_probe_scope_id(
        &self,
        epoch: crate::graphics::scene::RenderPassDeviceEpoch,
    ) -> Option<u64> {
        let mut handle = self
            .last_virtual_geometry_native_probe
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let valid = handle.as_ref().is_some_and(|(history, ready)| {
            ready.native_probe_matches(self.generation, epoch)
                && ready.native_probe_scope().is_some_and(|scope| {
                    self.last_applied_virtual_geometry_generation.get(history)
                        == Some(&scope.frame_generation)
                })
        });
        if !valid {
            drop(handle.take());
        }
        handle
            .as_ref()?
            .1
            .native_probe_scope()
            .map(|scope| scope.scope_id)
    }
    pub(in crate::graphics::runtime::render_framework) fn native_probe_handle(
        &self,
        scope: u64,
        epoch: crate::graphics::scene::RenderPassDeviceEpoch,
    ) -> Result<crate::graphics::scene::VirtualGeometryReadyReadback, String> {
        if self.native_probe_scope_id(epoch) != Some(scope) {
            return Err("native probe replaced or retired".into());
        }
        self.last_virtual_geometry_native_probe
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .as_ref()
            .and_then(|(_, r)| r.clone_native_probe_handle())
            .ok_or_else(|| "native probe unavailable".into())
    }
    #[cfg(feature = "test-support")]
    pub(in crate::graphics::runtime::render_framework) fn virtual_geometry_pending_readback_count(
        &self,
    ) -> usize {
        self.pending_virtual_geometry_readbacks.len()
    }

    pub(in crate::graphics::runtime::render_framework) fn virtual_geometry_readback_scope(
        &self,
        budget: &Arc<crate::graphics::scene::VirtualGeometryReadbackBudget>,
        viewport: crate::core::framework::render::RenderViewportHandle,
        generation: u64,
    ) -> Option<crate::graphics::scene::VirtualGeometryReadbackScope> {
        if let Some(epoch) = self.virtual_geometry_readback_epoch {
            let _ = self.native_probe_scope_id(epoch);
        }
        budget.scope(
            viewport,
            self.generation,
            generation,
            Arc::clone(&self.virtual_geometry_readback_slots),
        )
    }

    pub(in crate::graphics::runtime::render_framework) fn admit_virtual_geometry_readbacks(
        &mut self,
        renderer: &mut crate::graphics::SceneRenderer,
        viewport: crate::core::framework::render::RenderViewportHandle,
        scope: Option<&crate::graphics::scene::VirtualGeometryReadbackScope>,
        history: &ViewportCameraHistoryKey,
        generation: u64,
    ) {
        let epoch = renderer.virtual_geometry_device_epoch();
        if self
            .virtual_geometry_readback_epoch
            .is_some_and(|previous| previous != epoch)
        {
            self.pending_virtual_geometry_readbacks.clear();
            drop(
                self.last_virtual_geometry_native_probe
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .take(),
            );
            self.last_applied_virtual_geometry_generation.clear();
            self.virtual_geometry_runtimes.clear();
        }
        self.virtual_geometry_readback_epoch = Some(epoch);
        for packet in renderer.take_ticketed_virtual_geometry_readbacks() {
            if packet.scope.viewport != viewport
                || packet.scope.viewport_generation != self.generation
                || packet.scope.frame_generation != generation
                || scope.is_none_or(|expected| expected.scope_id != packet.scope.scope_id)
            {
                continue;
            }
            if self
                .virtual_geometry_readback_epoch
                .is_some_and(|epoch| epoch != packet.epoch)
            {
                self.pending_virtual_geometry_readbacks.clear();
                drop(
                    self.last_virtual_geometry_native_probe
                        .lock()
                        .unwrap_or_else(|e| e.into_inner())
                        .take(),
                );
                self.last_applied_virtual_geometry_generation.clear();
                self.virtual_geometry_runtimes.clear();
            }
            self.virtual_geometry_readback_epoch = Some(packet.epoch);
            self.pending_virtual_geometry_readbacks.push_back(
                PendingViewportVirtualGeometryReadback {
                    history: history.clone(),
                    packet,
                    ready: None,
                },
            );
        }
        self.observe_virtual_geometry_readbacks(renderer);
    }

    pub(in crate::graphics::runtime::render_framework) fn observe_virtual_geometry_readbacks(
        &mut self,
        renderer: &mut crate::graphics::SceneRenderer,
    ) {
        let _ = self.native_probe_scope_id(renderer.virtual_geometry_device_epoch());
        self.pending_virtual_geometry_readbacks
            .retain_mut(|pending| {
                if pending.packet.scope.viewport_generation != self.generation
                    || self.virtual_geometry_readback_epoch != Some(pending.packet.epoch)
                {
                    return false;
                }
                if pending.ready.is_some() {
                    return true;
                }
                renderer.observe_virtual_geometry_readback(&mut pending.packet);
                match pending.packet.take_ready() {
                    Some(Ok(ready)) => {
                        #[cfg(feature = "test-support")]
                        renderer.record_completed_virtual_geometry_native_readback_for_test(
                            &ready.outputs,
                            ready.inspection.as_ref(),
                        );
                        pending.ready = Some(ready);
                        true
                    }
                    Some(Err(_)) => false,
                    None => true,
                }
            });
    }

    /// Called only after the outer product frame and history validation succeeded.
    pub(in crate::graphics::runtime::render_framework) fn apply_ready_virtual_geometry_readbacks(
        &mut self,
    ) {
        let mut ordered = std::mem::take(&mut self.pending_virtual_geometry_readbacks)
            .into_iter()
            .collect::<Vec<_>>();
        ordered.sort_by_key(|entry| entry.packet.scope.frame_generation);
        let mut blocked_histories = std::collections::HashSet::new();
        for mut pending in ordered {
            if blocked_histories.contains(&pending.history) {
                self.pending_virtual_geometry_readbacks.push_back(pending);
                continue;
            }
            if self
                .last_applied_virtual_geometry_generation
                .get(&pending.history)
                .is_some_and(|last| *last >= pending.packet.scope.frame_generation)
            {
                continue;
            }
            let Some(mut ready) = pending.ready.take() else {
                blocked_histories.insert(pending.history.clone());
                self.pending_virtual_geometry_readbacks.push_back(pending);
                continue;
            };
            let Some(runtime) = self.virtual_geometry_runtimes.get_mut(&pending.history) else {
                continue;
            };
            drop(ready.inspection.take());
            let mut outputs = std::mem::take(&mut ready.outputs);
            let requests = outputs.take_node_and_cluster_cull_page_request_ids();
            // A completed clear-only payload is still a real completion envelope.
            let completion = crate::graphics::VirtualGeometryGpuCompletion::from_readback_outputs(
                outputs.clone(),
            );
            let feedback = crate::graphics::VirtualGeometryRuntimeFeedback::new(
                completion,
                requests,
                None,
                pending.packet.scope.frame_generation,
            )
            .with_native_readback_outputs(outputs);
            runtime.update_after_render(feedback);
            self.last_applied_virtual_geometry_generation.insert(
                pending.history.clone(),
                pending.packet.scope.frame_generation,
            );
            // Consume feedback before retaining only its opaque native source handle.
            *self
                .last_virtual_geometry_native_probe
                .lock()
                .unwrap_or_else(|e| e.into_inner()) = ready
                .into_native_probe_handle()
                .map(|handle| (pending.history, handle));
        }
    }
}
