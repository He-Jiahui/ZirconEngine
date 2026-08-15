use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, SyncSender, sync_channel};

use crate::scene::viewport::{
    CapturedFrame, RenderFrameExtract, RenderFramework, RenderFrameworkError, RenderPipelineHandle,
    RenderQualityProfile, RenderStats, RenderViewportDescriptor, RenderViewportHandle,
    RenderViewportProduct,
};
use zircon_runtime_interface::math::UVec2;
use zircon_runtime_interface::ui::surface::UiRenderExtract;

#[derive(Default)]
pub(super) struct FakeRenderFramework {
    pub(super) state: Mutex<FakeRenderFrameworkState>,
    submit_gate: Mutex<Option<SubmitGate>>,
    destroy_notifier: Mutex<Option<SyncSender<RenderViewportHandle>>>,
    submit_in_flight: AtomicBool,
}

struct SubmitGate {
    started: SyncSender<()>,
    release: Receiver<()>,
}

#[derive(Default)]
pub(super) struct FakeRenderFrameworkState {
    pub(super) next_viewport_id: u64,
    pub(super) created_viewports: Vec<RenderViewportDescriptor>,
    pub(super) viewport_sizes: HashMap<RenderViewportHandle, UVec2>,
    pub(super) destroyed_viewports: Vec<RenderViewportHandle>,
    pub(super) submitted_viewports: Vec<RenderViewportHandle>,
    pub(super) submitted_aspect_ratios: Vec<f32>,
    pub(super) submitted_ui_command_counts: Vec<usize>,
    pub(super) submitted_ui_texts: Vec<Vec<String>>,
    pub(super) quality_profiles: Vec<(RenderViewportHandle, RenderQualityProfile)>,
    pub(super) submitted_hybrid_gi_settings:
        Vec<Option<crate::scene::viewport::RenderHybridGiExtract>>,
    pub(super) capture_requests: usize,
    pub(super) capture_error: Option<String>,
    pub(super) captures: HashMap<RenderViewportHandle, CapturedFrame>,
    pub(super) products: HashMap<RenderViewportHandle, RenderViewportProduct>,
}

impl FakeRenderFramework {
    pub(super) fn block_next_submit(&self) -> (Receiver<()>, SyncSender<()>) {
        let (started_sender, started_receiver) = sync_channel(1);
        let (release_sender, release_receiver) = sync_channel(1);
        *self.submit_gate.lock().unwrap() = Some(SubmitGate {
            started: started_sender,
            release: release_receiver,
        });
        (started_receiver, release_sender)
    }

    pub(super) fn notify_next_destroy(&self) -> Receiver<RenderViewportHandle> {
        let (sender, receiver) = sync_channel(1);
        *self.destroy_notifier.lock().unwrap() = Some(sender);
        receiver
    }

    fn wait_for_submit_gate(&self) {
        let Some(gate) = self.submit_gate.lock().unwrap().take() else {
            return;
        };
        self.submit_in_flight.store(true, Ordering::Release);
        let _ = gate.started.send(());
        let _ = gate.release.recv();
        self.submit_in_flight.store(false, Ordering::Release);
    }

    fn notify_destroy(&self, viewport: RenderViewportHandle) {
        if let Some(notifier) = self.destroy_notifier.lock().unwrap().take() {
            let _ = notifier.send(viewport);
        }
    }
}

impl RenderFramework for FakeRenderFramework {
    fn create_viewport(
        &self,
        descriptor: RenderViewportDescriptor,
    ) -> Result<RenderViewportHandle, RenderFrameworkError> {
        let mut state = self.state.lock().unwrap();
        state.next_viewport_id += 1;
        let handle = RenderViewportHandle::new(state.next_viewport_id);
        state.viewport_sizes.insert(handle, descriptor.size);
        state.created_viewports.push(descriptor);
        Ok(handle)
    }

    fn destroy_viewport(&self, viewport: RenderViewportHandle) -> Result<(), RenderFrameworkError> {
        self.notify_destroy(viewport);
        if self.submit_in_flight.load(Ordering::Acquire) {
            return Err(RenderFrameworkError::Backend(
                "test framework rejected viewport destruction during submit".to_string(),
            ));
        }
        let mut state = self.state.lock().unwrap();
        state.destroyed_viewports.push(viewport);
        state.viewport_sizes.remove(&viewport);
        state.captures.remove(&viewport);
        state.products.remove(&viewport);
        Ok(())
    }

    fn submit_frame_extract(
        &self,
        viewport: RenderViewportHandle,
        extract: RenderFrameExtract,
    ) -> Result<(), RenderFrameworkError> {
        {
            let mut state = self.state.lock().unwrap();
            state.submitted_viewports.push(viewport);
            state
                .submitted_hybrid_gi_settings
                .push(extract.lighting.hybrid_global_illumination.clone());
            let size = state
                .viewport_sizes
                .get(&viewport)
                .copied()
                .unwrap_or(UVec2::new(1, 1));
            state
                .submitted_aspect_ratios
                .push(size.x as f32 / size.y as f32);
            state.captures.insert(
                viewport,
                CapturedFrame::new(1, 1, vec![viewport.raw() as u8, 0, 0, 255], viewport.raw()),
            );
        }
        self.wait_for_submit_gate();
        Ok(())
    }

    fn submit_frame_extract_with_ui(
        &self,
        viewport: RenderViewportHandle,
        extract: RenderFrameExtract,
        ui: Option<UiRenderExtract>,
    ) -> Result<(), RenderFrameworkError> {
        {
            let mut state = self.state.lock().unwrap();
            state.submitted_viewports.push(viewport);
            state
                .submitted_hybrid_gi_settings
                .push(extract.lighting.hybrid_global_illumination.clone());
            let size = state
                .viewport_sizes
                .get(&viewport)
                .copied()
                .unwrap_or(UVec2::new(1, 1));
            state
                .submitted_aspect_ratios
                .push(size.x as f32 / size.y as f32);
            state.submitted_ui_command_counts.push(
                ui.as_ref()
                    .map(|extract| extract.list.commands.len())
                    .unwrap_or(0),
            );
            state.submitted_ui_texts.push(
                ui.as_ref()
                    .map(|extract| {
                        extract
                            .list
                            .commands
                            .iter()
                            .filter_map(|command| command.text.clone())
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default(),
            );
            state.captures.insert(
                viewport,
                CapturedFrame::new(1, 1, vec![viewport.raw() as u8, 0, 0, 255], viewport.raw()),
            );
        }
        self.wait_for_submit_gate();
        Ok(())
    }

    fn set_pipeline_asset(
        &self,
        _viewport: RenderViewportHandle,
        _pipeline: RenderPipelineHandle,
    ) -> Result<(), RenderFrameworkError> {
        Ok(())
    }

    fn reload_pipeline(&self, _pipeline: RenderPipelineHandle) -> Result<(), RenderFrameworkError> {
        Ok(())
    }

    fn query_stats(&self) -> Result<RenderStats, RenderFrameworkError> {
        Ok(RenderStats::default())
    }

    fn query_virtual_geometry_debug_snapshot(
        &self,
    ) -> Result<
        Option<zircon_runtime::core::framework::render::RenderVirtualGeometryDebugSnapshot>,
        RenderFrameworkError,
    > {
        Ok(None)
    }

    fn capture_frame(
        &self,
        viewport: RenderViewportHandle,
    ) -> Result<Option<CapturedFrame>, RenderFrameworkError> {
        let mut state = self.state.lock().unwrap();
        state.capture_requests += 1;
        if let Some(error) = &state.capture_error {
            return Err(RenderFrameworkError::Backend(error.clone()));
        }
        Ok(state.captures.get(&viewport).cloned())
    }

    fn poll_captured_frame_if_newer(
        &self,
        viewport: RenderViewportHandle,
        last_generation: Option<u64>,
    ) -> Result<Option<CapturedFrame>, RenderFrameworkError> {
        let mut state = self.state.lock().unwrap();
        state.capture_requests += 1;
        if let Some(error) = &state.capture_error {
            return Err(RenderFrameworkError::Backend(error.clone()));
        }
        Ok(state
            .captures
            .get(&viewport)
            .filter(|frame| last_generation.is_none_or(|generation| frame.generation > generation))
            .cloned())
    }

    fn poll_viewport_product_if_newer(
        &self,
        viewport: RenderViewportHandle,
        last_generation: Option<u64>,
    ) -> Result<Option<RenderViewportProduct>, RenderFrameworkError> {
        Ok(self
            .state
            .lock()
            .unwrap()
            .products
            .get(&viewport)
            .filter(|product| Some(product.generation()) != last_generation)
            .cloned())
    }

    fn set_quality_profile(
        &self,
        viewport: RenderViewportHandle,
        profile: RenderQualityProfile,
    ) -> Result<(), RenderFrameworkError> {
        self.state
            .lock()
            .unwrap()
            .quality_profiles
            .push((viewport, profile));
        Ok(())
    }
}
