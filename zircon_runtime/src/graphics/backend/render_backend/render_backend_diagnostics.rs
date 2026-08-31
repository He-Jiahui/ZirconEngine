use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::time::{Duration, Instant};

use crate::graphics::types::GraphicsError;
use crate::rhi::{DiagnosticReadbackTerminal, SubmissionPollReceipt, SubmissionTicket};
use zr_rhi_wgpu::{
    GpuDiagnosticQueryFramePlan, GpuPassTimer, GpuPipelineStatisticsTimer,
    WgpuNativeDiagnosticQueryFrame, WgpuNativeDiagnosticQueryRecorder,
    WgpuNativeDiagnosticReadbackFrame,
};

use super::product_diagnostic_delivery_router::{
    ProductDiagnosticQueryResult, ProductDiagnosticReadbackCallback,
};
use super::render_backend::RenderBackend;

pub(super) const PRODUCT_DIAGNOSTIC_CAPTURE_TIMEOUT: Duration = Duration::from_secs(30);

pub(crate) struct ProductDiagnosticReadbackFrameScope<'a> {
    backend: &'a RenderBackend,
    active: bool,
}

impl ProductDiagnosticReadbackFrameScope<'_> {
    pub(crate) fn prepare(
        mut self,
        label: &str,
        encoder: &mut wgpu::CommandEncoder,
    ) -> Result<Option<WgpuNativeDiagnosticReadbackFrame>, GraphicsError> {
        let frame = self
            .backend
            .prepare_product_diagnostic_readback_frame(label, encoder)?;
        self.active = false;
        Ok(frame)
    }

    pub(crate) fn submit(mut self, label: &str) -> Result<Option<SubmissionTicket>, GraphicsError> {
        let submission = self
            .backend
            .submit_product_diagnostic_readback_frame(label)?;
        self.active = false;
        Ok(submission)
    }
}

impl Drop for ProductDiagnosticReadbackFrameScope<'_> {
    fn drop(&mut self) {
        if self.active {
            self.backend.abort_product_diagnostic_readback_frame();
        }
    }
}

pub(crate) struct ProductDiagnosticQueryFrameScope<'a> {
    backend: &'a RenderBackend,
    renderer_frame_generation: u64,
    query_frame_index: u64,
    plan: GpuDiagnosticQueryFramePlan,
    recorder: Option<WgpuNativeDiagnosticQueryRecorder>,
    active: bool,
}

impl ProductDiagnosticQueryFrameScope<'_> {
    pub(crate) fn attach_timers(
        &self,
        gpu_pass_timer: Option<&mut GpuPassTimer>,
        gpu_pipeline_statistics_timer: Option<&mut GpuPipelineStatisticsTimer>,
    ) {
        if let Some(timer) = gpu_pass_timer {
            if let Some(query_set) = self
                .recorder
                .as_ref()
                .and_then(WgpuNativeDiagnosticQueryRecorder::timestamp_query_set)
            {
                timer.begin_product_frame(
                    self.renderer_frame_generation,
                    self.plan.clone(),
                    query_set,
                );
            } else {
                timer.defer_frame(self.renderer_frame_generation);
            }
        }
        if let (Some(timer), Some(query_set)) = (
            gpu_pipeline_statistics_timer,
            self.recorder
                .as_ref()
                .and_then(WgpuNativeDiagnosticQueryRecorder::pipeline_statistics_query_set),
        ) {
            timer.begin_product_frame(self.renderer_frame_generation, self.plan.clone(), query_set);
        }
    }

    pub(crate) fn finish_and_prepare(
        mut self,
        encoder: &mut wgpu::CommandEncoder,
        gpu_pass_timer: Option<&mut GpuPassTimer>,
        gpu_pipeline_statistics_timer: Option<&mut GpuPipelineStatisticsTimer>,
    ) -> Result<Option<WgpuNativeDiagnosticQueryFrame>, GraphicsError> {
        if let Some(timer) = gpu_pass_timer {
            timer.finish_product_frame();
        }
        if let Some(timer) = gpu_pipeline_statistics_timer {
            timer.finish_product_frame();
        }
        let snapshot = self.plan.snapshot();
        let native_plan = snapshot.plan().clone();
        self.backend
            .register_product_diagnostic_query_plan(self.query_frame_index, snapshot)?;
        let Some(recorder) = self.recorder.take() else {
            self.active = false;
            return Ok(None);
        };
        let prepared = self
            .backend
            .render_device
            .prepare_native_diagnostic_query_frame(recorder, native_plan, encoder)
            .map_err(GraphicsError::from);
        self.active = false;
        prepared
    }
}

impl Drop for ProductDiagnosticQueryFrameScope<'_> {
    fn drop(&mut self) {
        if !self.active {
            return;
        }
        let _ = self
            .backend
            .register_product_diagnostic_query_plan(self.query_frame_index, self.plan.snapshot());
        if let Some(recorder) = self.recorder.take() {
            self.backend
                .render_device
                .abort_native_diagnostic_query_recorder(
                    recorder,
                    DiagnosticReadbackTerminal::Cancelled,
                );
        }
    }
}

impl RenderBackend {
    pub(crate) fn product_diagnostic_readback_metrics(
        &self,
    ) -> zr_rhi_wgpu::WgpuDiagnosticReadbackMetricsSnapshot {
        self.render_device.diagnostic_readback_metrics()
    }

    pub(crate) fn begin_product_diagnostic_query_scope(
        &self,
        renderer_frame_generation: u64,
        timestamps_enabled: bool,
        pipeline_statistics_enabled: bool,
    ) -> Result<Option<ProductDiagnosticQueryFrameScope<'_>>, GraphicsError> {
        if !timestamps_enabled && !pipeline_statistics_enabled {
            return Ok(None);
        }
        let query_frame_index = self
            .diagnostic_delivery_router
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .reserve_query_route(renderer_frame_generation)
            .map_err(GraphicsError::BufferMap)?;
        let recorder = match self.render_device.begin_native_diagnostic_query_frame(
            query_frame_index,
            timestamps_enabled,
            pipeline_statistics_enabled,
        ) {
            Ok(recorder) => recorder,
            Err(error) => {
                self.diagnostic_delivery_router
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner())
                    .cancel_query_route(query_frame_index);
                return Err(GraphicsError::from(error));
            }
        };
        Ok(Some(ProductDiagnosticQueryFrameScope {
            backend: self,
            renderer_frame_generation,
            query_frame_index,
            plan: GpuDiagnosticQueryFramePlan::new(
                query_frame_index,
                self.device_profile().diagnostic_readback_budget().clone(),
            ),
            recorder,
            active: true,
        }))
    }

    fn register_product_diagnostic_query_plan(
        &self,
        query_frame_index: u64,
        snapshot: zr_rhi_wgpu::GpuDiagnosticQueryFramePlanSnapshot,
    ) -> Result<(), GraphicsError> {
        self.diagnostic_delivery_router
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .register_query_plan(query_frame_index, snapshot)
            .map_err(GraphicsError::BufferMap)
    }

    pub(crate) fn drain_product_diagnostic_query_results(
        &self,
    ) -> Vec<ProductDiagnosticQueryResult> {
        self.diagnostic_delivery_router
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .drain_query_results()
    }

    pub(crate) fn begin_product_diagnostic_readback_frame(
        &self,
        frame_generation: u64,
    ) -> Result<(), GraphicsError> {
        self.render_device
            .begin_diagnostic_readback_frame(frame_generation)
            .map_err(GraphicsError::from)
    }

    pub(crate) fn begin_product_diagnostic_readback_scope(
        &self,
        frame_generation: u64,
    ) -> Result<ProductDiagnosticReadbackFrameScope<'_>, GraphicsError> {
        self.begin_product_diagnostic_readback_frame(frame_generation)?;
        Ok(ProductDiagnosticReadbackFrameScope {
            backend: self,
            active: true,
        })
    }

    pub(crate) fn enqueue_product_diagnostic_texture_rgba8(
        &self,
        texture: &wgpu::Texture,
        width: u32,
        height: u32,
        callback: Box<dyn FnOnce(Result<Vec<u8>, String>) + Send + 'static>,
    ) -> Result<bool, GraphicsError> {
        let admission = self
            .render_device
            .enqueue_native_diagnostic_texture_rgba8_readback(texture, width, height)?;
        let admitted = matches!(admission, zr_rhi::DiagnosticReadbackAdmission::Admitted(_));
        let callback: ProductDiagnosticReadbackCallback =
            if admitted { callback } else { Box::new(|_| {}) };
        self.diagnostic_delivery_router
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .register(admission, callback)
            .map_err(GraphicsError::BufferMap)
    }

    pub(crate) fn enqueue_product_diagnostic_texture_rgba16float(
        &self,
        texture: &wgpu::Texture,
        mip_level: u32,
        array_layer: u32,
        width: u32,
        height: u32,
        callback: Box<dyn FnOnce(Result<Vec<u8>, String>) + Send + 'static>,
    ) -> Result<bool, GraphicsError> {
        let admission = self
            .render_device
            .enqueue_native_diagnostic_texture_rgba16float_readback(
                texture,
                mip_level,
                array_layer,
                width,
                height,
            )?;
        let admitted = matches!(admission, zr_rhi::DiagnosticReadbackAdmission::Admitted(_));
        let callback: ProductDiagnosticReadbackCallback =
            if admitted { callback } else { Box::new(|_| {}) };
        self.diagnostic_delivery_router
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .register(admission, callback)
            .map_err(GraphicsError::BufferMap)
    }

    pub(crate) fn enqueue_product_diagnostic_texture_rgba16float_mip_chain(
        &self,
        texture: &wgpu::Texture,
        array_layer: u32,
        mip_count: u32,
        callback: Box<dyn FnOnce(Result<Vec<u8>, String>) + Send + 'static>,
    ) -> Result<bool, GraphicsError> {
        let admission = self
            .render_device
            .enqueue_native_diagnostic_texture_rgba16float_mip_chain_readback(
                texture,
                array_layer,
                mip_count,
            )?;
        let admitted = matches!(admission, zr_rhi::DiagnosticReadbackAdmission::Admitted(_));
        let callback: ProductDiagnosticReadbackCallback =
            if admitted { callback } else { Box::new(|_| {}) };
        self.diagnostic_delivery_router
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .register(admission, callback)
            .map_err(GraphicsError::BufferMap)
    }

    pub(crate) fn enqueue_product_diagnostic_texture_r32_uint_texel(
        &self,
        texture: &wgpu::Texture,
        pixel: [u32; 2],
        callback: Box<dyn FnOnce(Result<u32, String>) + Send + 'static>,
    ) -> Result<bool, GraphicsError> {
        let admission = self
            .render_device
            .enqueue_native_diagnostic_texture_r32_uint_texel_readback(texture, pixel)?;
        let admitted = matches!(admission, zr_rhi::DiagnosticReadbackAdmission::Admitted(_));
        let callback: ProductDiagnosticReadbackCallback = if admitted {
            Box::new(move |result| callback(result.and_then(decode_r32_uint_texel)))
        } else {
            Box::new(|_| {})
        };
        self.diagnostic_delivery_router
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .register(admission, callback)
            .map_err(GraphicsError::BufferMap)
    }

    pub(crate) fn enqueue_product_diagnostic_texture_rgba32float_texel(
        &self,
        texture: &wgpu::Texture,
        pixel: [u32; 2],
        callback: Box<dyn FnOnce(Result<[f32; 4], String>) + Send + 'static>,
    ) -> Result<bool, GraphicsError> {
        let admission = self
            .render_device
            .enqueue_native_diagnostic_texture_rgba32float_texel_readback(texture, pixel)?;
        let admitted = matches!(admission, zr_rhi::DiagnosticReadbackAdmission::Admitted(_));
        let callback: ProductDiagnosticReadbackCallback = if admitted {
            Box::new(move |result| callback(result.and_then(decode_rgba32float_texel)))
        } else {
            Box::new(|_| {})
        };
        self.diagnostic_delivery_router
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .register(admission, callback)
            .map_err(GraphicsError::BufferMap)
    }

    pub(crate) fn enqueue_product_diagnostic_buffer(
        &self,
        buffer: &wgpu::Buffer,
        offset: u64,
        size: u64,
        callback: Box<dyn FnOnce(Result<Vec<u8>, String>) + Send + 'static>,
    ) -> Result<bool, GraphicsError> {
        let admission = self
            .render_device
            .enqueue_native_diagnostic_buffer_readback(buffer, offset, size)?;
        let admitted = matches!(admission, zr_rhi::DiagnosticReadbackAdmission::Admitted(_));
        let callback: ProductDiagnosticReadbackCallback =
            if admitted { callback } else { Box::new(|_| {}) };
        self.diagnostic_delivery_router
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .register(admission, callback)
            .map_err(GraphicsError::BufferMap)
    }

    pub(crate) fn read_product_diagnostic_texture_rgba8_blocking(
        &self,
        frame_generation: u64,
        texture: &wgpu::Texture,
        width: u32,
        height: u32,
        label: &str,
        observe_poll: &mut impl FnMut(SubmissionPollReceipt) -> Result<(), GraphicsError>,
    ) -> Result<Vec<u8>, GraphicsError> {
        let scope = self.begin_product_diagnostic_readback_scope(frame_generation)?;
        let (sender, receiver) = mpsc::sync_channel(1);
        let admitted = self.enqueue_product_diagnostic_texture_rgba8(
            texture,
            width,
            height,
            Box::new(move |result| {
                let _ = sender.try_send(result);
            }),
        )?;
        if !admitted {
            drop(scope);
            self.dispatch_product_diagnostic_deliveries();
            return Err(GraphicsError::BufferMap(
                "RGBA8 capture exceeded the product diagnostic readback budget".to_string(),
            ));
        }
        self.finish_product_diagnostic_texture_readback(scope, label, receiver, observe_poll)
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn read_product_diagnostic_texture_rgba16float_blocking(
        &self,
        frame_generation: u64,
        texture: &wgpu::Texture,
        mip_level: u32,
        array_layer: u32,
        width: u32,
        height: u32,
        label: &str,
        observe_poll: &mut impl FnMut(SubmissionPollReceipt) -> Result<(), GraphicsError>,
    ) -> Result<Vec<u8>, GraphicsError> {
        let scope = self.begin_product_diagnostic_readback_scope(frame_generation)?;
        let (sender, receiver) = mpsc::sync_channel(1);
        let admitted = self.enqueue_product_diagnostic_texture_rgba16float(
            texture,
            mip_level,
            array_layer,
            width,
            height,
            Box::new(move |result| {
                let _ = sender.try_send(result);
            }),
        )?;
        if !admitted {
            drop(scope);
            self.dispatch_product_diagnostic_deliveries();
            return Err(GraphicsError::BufferMap(
                "RGBA16Float capture exceeded the product diagnostic readback budget".to_string(),
            ));
        }
        self.finish_product_diagnostic_texture_readback(scope, label, receiver, observe_poll)
    }

    fn finish_product_diagnostic_texture_readback(
        &self,
        scope: ProductDiagnosticReadbackFrameScope<'_>,
        label: &str,
        receiver: Receiver<Result<Vec<u8>, String>>,
        observe_poll: &mut impl FnMut(SubmissionPollReceipt) -> Result<(), GraphicsError>,
    ) -> Result<Vec<u8>, GraphicsError> {
        match scope.submit(label) {
            Ok(Some(_)) => {}
            Ok(None) => {
                return Err(GraphicsError::BufferMap(format!(
                    "{label} admitted no readback"
                )));
            }
            Err(error) => {
                self.dispatch_product_diagnostic_deliveries();
                return Err(error);
            }
        }
        self.wait_for_product_diagnostic_callback(
            label,
            receiver,
            PRODUCT_DIAGNOSTIC_CAPTURE_TIMEOUT,
            observe_poll,
        )
    }

    fn wait_for_product_diagnostic_callback(
        &self,
        label: &str,
        receiver: Receiver<Result<Vec<u8>, String>>,
        timeout: Duration,
        observe_poll: &mut impl FnMut(SubmissionPollReceipt) -> Result<(), GraphicsError>,
    ) -> Result<Vec<u8>, GraphicsError> {
        let started = Instant::now();
        loop {
            let poll_receipt = self.poll_submission_completions()?;
            observe_poll(poll_receipt)?;
            match receiver.try_recv() {
                Ok(Ok(bytes)) => return Ok(bytes),
                Ok(Err(reason)) => return Err(GraphicsError::BufferMap(reason)),
                Err(TryRecvError::Disconnected) => {
                    return Err(GraphicsError::BufferMap(format!(
                        "{label} diagnostic delivery callback disconnected"
                    )));
                }
                Err(TryRecvError::Empty) => {}
            }
            if started.elapsed() >= timeout {
                return Err(GraphicsError::DiagnosticReadbackTimedOut { timeout });
            }
            std::thread::yield_now();
        }
    }

    pub(crate) fn prepare_product_diagnostic_readback_frame(
        &self,
        label: &str,
        encoder: &mut wgpu::CommandEncoder,
    ) -> Result<Option<WgpuNativeDiagnosticReadbackFrame>, GraphicsError> {
        self.render_device
            .prepare_native_diagnostic_readback_frame(label, encoder)
            .map_err(GraphicsError::from)
    }

    fn submit_product_diagnostic_readback_frame(
        &self,
        label: &str,
    ) -> Result<Option<SubmissionTicket>, GraphicsError> {
        self.render_device
            .submit_and_flush_diagnostic_readback_frame(label)
            .map(|frame| frame.map(|frame| frame.submission()))
            .map_err(GraphicsError::from)
    }

    pub(crate) fn abort_product_diagnostic_readback_frame(&self) {
        self.render_device
            .abort_diagnostic_readback_frame(DiagnosticReadbackTerminal::Cancelled);
    }

    pub(super) fn dispatch_product_diagnostic_deliveries(&self) {
        let dispatches = {
            let mut router = self
                .diagnostic_delivery_router
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            router.collect_query_results(self.render_device.as_ref());
            router.collect_dispatches(self.render_device.as_ref())
        };
        self.run_product_diagnostic_dispatches(dispatches);
    }

    fn run_product_diagnostic_dispatches(
        &self,
        dispatches: Vec<super::product_diagnostic_delivery_router::ProductDiagnosticDispatch>,
    ) {
        let callback_panics = dispatches
            .into_iter()
            .fold(0, |count, dispatch| count + usize::from(dispatch.run()));
        if callback_panics == 0 {
            return;
        }
        let mut router = self
            .diagnostic_delivery_router
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        for _ in 0..callback_panics {
            router.record_callback_panic();
        }
    }
}

fn decode_r32_uint_texel(bytes: Vec<u8>) -> Result<u32, String> {
    let bytes: [u8; 4] = bytes.try_into().map_err(|bytes: Vec<u8>| {
        format!("R32Uint texel returned {} bytes; expected 4", bytes.len())
    })?;
    Ok(u32::from_le_bytes(bytes))
}

fn decode_rgba32float_texel(bytes: Vec<u8>) -> Result<[f32; 4], String> {
    let bytes: [u8; 16] = bytes.try_into().map_err(|bytes: Vec<u8>| {
        format!("RGBA32F texel returned {} bytes; expected 16", bytes.len())
    })?;
    let mut lanes = [0.0; 4];
    for (lane, chunk) in lanes.iter_mut().zip(bytes.chunks_exact(4)) {
        *lane = f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
    }
    Ok(lanes)
}

#[cfg(test)]
mod tests {
    use super::{decode_r32_uint_texel, decode_rgba32float_texel};

    #[test]
    fn pick_product_texel_decoders_require_exact_little_endian_payloads() {
        assert_eq!(decode_r32_uint_texel(7_u32.to_le_bytes().to_vec()), Ok(7));

        let expected = [1.0_f32, -2.5, 0.25, 0.75];
        let bytes = expected
            .into_iter()
            .flat_map(f32::to_le_bytes)
            .collect::<Vec<_>>();
        assert_eq!(decode_rgba32float_texel(bytes), Ok(expected));
        assert!(decode_r32_uint_texel(vec![0; 3]).is_err());
        assert!(decode_rgba32float_texel(vec![0; 12]).is_err());
    }

    #[test]
    fn product_texel_decoder_has_no_production_lane_expect() {
        let source = include_str!("render_backend_diagnostics.rs");
        let production = source
            .split_once("#[cfg(test)]")
            .map(|(production, _)| production)
            .expect("diagnostics source must retain its test-module boundary");
        assert!(!production.contains("fixed RGBA32F lane"));
    }

    #[test]
    fn explicit_texture_capture_uses_the_product_diagnostic_owner() {
        let diagnostics = include_str!("render_backend_diagnostics.rs")
            .split("\n#[cfg(test)]")
            .next()
            .unwrap_or_default();
        let scene_target = include_str!(
            "../../scene/scene_renderer/core/scene_renderer_target/finish_viewport_frame.rs"
        );
        let renderer = include_str!(
            "../../scene/scene_renderer/core/scene_renderer_render_with_pipeline/render_frame_with_pipeline/readback.rs"
        );
        let backend_root = include_str!("mod.rs");
        let graphics_backend_root = include_str!("../mod.rs");
        let standalone = diagnostics
            .split("fn finish_product_diagnostic_texture_readback(")
            .nth(1)
            .and_then(|source| {
                source
                    .split("fn wait_for_product_diagnostic_callback(")
                    .next()
            })
            .expect("standalone product diagnostic submission owner");

        assert!(diagnostics.contains("fn read_product_diagnostic_texture_rgba8_blocking("));
        assert!(diagnostics.contains("fn read_product_diagnostic_texture_rgba16float_blocking("));
        assert!(diagnostics.contains("begin_product_diagnostic_readback_scope(frame_generation)"));
        assert!(standalone.contains("scope.submit(label)"));
        assert!(!standalone.contains("create_command_encoder"));
        assert!(!standalone.contains("submit_graphics_command_buffers_with_diagnostics("));
        assert!(diagnostics.contains("wait_for_product_diagnostic_callback("));
        assert!(diagnostics.contains("PRODUCT_DIAGNOSTIC_CAPTURE_TIMEOUT"));
        assert!(diagnostics.contains("self.poll_submission_completions()?"));
        assert!(diagnostics.contains("observe_poll(poll_receipt)?"));
        assert!(diagnostics.contains("receiver.try_recv()"));
        assert!(!diagnostics.contains("receiver.recv()"));
        assert!(!diagnostics.contains("wait_indefinitely"));
        assert!(scene_target.contains("backend.read_product_diagnostic_texture_rgba8_blocking("));
        assert!(renderer.contains(".read_product_diagnostic_texture_rgba16float_blocking("));
        assert!(backend_root.contains("#[cfg(test)]\nmod read_texture_rgba;"));
        assert!(backend_root.contains("#[cfg(test)]\nmod read_texture_rgba16float_region;"));
        assert!(
            graphics_backend_root
                .contains("#[cfg(test)]\npub(crate) use render_backend::read_texture_rgba;")
        );
        assert!(graphics_backend_root.contains(
            "#[cfg(test)]\npub(crate) use render_backend::{\n    read_texture_rgba16float_cube_mip_chain, read_texture_rgba16float_region,"
        ));
    }
}
