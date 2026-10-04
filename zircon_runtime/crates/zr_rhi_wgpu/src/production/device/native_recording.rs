//! 过渡期原生录制只借出设备及编码器；生成的包仍须回到唯一设备时间线接受和提交。
use zr_rhi::{
    DeviceGeneration, DeviceId, DiagnosticReadbackTerminal, RenderDevice, RenderQueueClass,
    RhiError, RhiGraphExecutionReceipt, SubmissionTicket,
};

use crate::ui_surface::WgpuUiImageInFlightPins;

use super::super::diagnostics::{DiagnosticReadbackBatch, WgpuNativeDiagnosticQueryFrame};
use super::native_surface_recording::WgpuNativeSurfaceFrameTarget;
use super::WgpuRenderDevice;

/// Transitional, frame-scoped recorder for product passes not yet lowered to neutral commands.
///
/// The recorder lends the owning device only while a command-buffer callback executes. It never
/// exposes the queue, polls the device, allocates submission identity, or flushes native work.
/// Recorded buffers become observable only after `finish` and owner-qualified enqueue.
#[must_use = "native recording is dropped without submission unless it is finished and enqueued"]
pub struct WgpuNativeRecorderLease<'device> {
    device_id: DeviceId,
    generation: DeviceGeneration,
    queue_class: RenderQueueClass,
    device: &'device wgpu::Device,
    command_buffers: Vec<wgpu::CommandBuffer>,
}

impl<'device> WgpuNativeRecorderLease<'device> {
    fn new(
        device_id: DeviceId,
        generation: DeviceGeneration,
        queue_class: RenderQueueClass,
        device: &'device wgpu::Device,
    ) -> Self {
        Self {
            device_id,
            generation,
            queue_class,
            device,
            command_buffers: Vec::new(),
        }
    }

    pub const fn device_id(&self) -> DeviceId {
        self.device_id
    }

    pub const fn generation(&self) -> DeviceGeneration {
        self.generation
    }

    pub const fn queue_class(&self) -> RenderQueueClass {
        self.queue_class
    }

    pub fn command_buffer_count(&self) -> usize {
        self.command_buffers.len()
    }

    /// Records one command buffer and retains it in this lease only after the callback succeeds.
    ///
    /// `E: From<RhiError>` lets a product recorder keep its typed error while still receiving
    /// fail-closed label validation. The borrowed device must not be cloned into a retained owner;
    /// it exists only to bridge native pass recording during the neutral hard cut.
    pub fn record_command_buffer<E>(
        &mut self,
        label: &str,
        record: impl FnOnce(&wgpu::Device, &mut wgpu::CommandEncoder) -> Result<(), E>,
    ) -> Result<(), E>
    where
        E: From<RhiError>,
    {
        if label.is_empty() {
            return Err(RhiError::InvalidDebugMarker {
                reason: "native recorder command-buffer label must not be empty".to_string(),
            }
            .into());
        }
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some(label) });
        record(self.device, &mut encoder)?;
        self.command_buffers.push(encoder.finish());
        Ok(())
    }

    /// Adopts command buffers produced by existing parallel native recorders on this device.
    ///
    /// This is a migration-only escape hatch. The final product path records neutral command
    /// lists; until then, the enclosing frame owner must ensure every buffer was created from the
    /// borrowed generation. No queue or submission authority is transferred with the buffer.
    pub fn extend_recorded_command_buffers(
        &mut self,
        command_buffers: impl IntoIterator<Item = wgpu::CommandBuffer>,
    ) {
        self.command_buffers.extend(command_buffers);
    }

    pub fn finish(self) -> Result<WgpuNativeSubmissionPacket, RhiError> {
        if self.command_buffers.is_empty() {
            return Err(RhiError::EmptySubmissionPacket);
        }
        Ok(WgpuNativeSubmissionPacket {
            device_id: self.device_id,
            generation: self.generation,
            queue_class: self.queue_class,
            command_buffers: self.command_buffers,
            ui_image_pins: None,
            graph_execution_receipt: None,
        })
    }
}

/// Opaque generation-qualified native packet accepted only by `WgpuRenderDevice`.
#[must_use = "a finished native packet has no GPU effect until it is enqueued"]
pub struct WgpuNativeSubmissionPacket {
    device_id: DeviceId,
    generation: DeviceGeneration,
    queue_class: RenderQueueClass,
    command_buffers: Vec<wgpu::CommandBuffer>,
    ui_image_pins: Option<WgpuUiImageInFlightPins>,
    graph_execution_receipt: Option<RhiGraphExecutionReceipt>,
}

impl WgpuNativeSubmissionPacket {
    pub const fn device_id(&self) -> DeviceId {
        self.device_id
    }

    pub const fn generation(&self) -> DeviceGeneration {
        self.generation
    }

    pub const fn queue_class(&self) -> RenderQueueClass {
        self.queue_class
    }

    pub fn command_buffer_count(&self) -> usize {
        self.command_buffers.len()
    }

    /// Attaches the compiled graph execution proof before native queue admission.
    pub fn with_graph_execution_receipt(
        mut self,
        receipt: RhiGraphExecutionReceipt,
    ) -> Result<Self, RhiError> {
        if receipt.device_id() != self.device_id || receipt.generation() != self.generation {
            return Err(RhiError::SubmissionPacketDeviceMismatch {
                packet_device_id: receipt.device_id(),
                packet_generation: receipt.generation(),
                device_id: self.device_id,
                generation: self.generation,
            });
        }
        if receipt.submission_queue() != self.queue_class {
            return Err(RhiError::SubmissionPacketQueueMismatch {
                packet_queue: self.queue_class,
                command_queue: receipt.submission_queue(),
            });
        }
        self.graph_execution_receipt = Some(receipt);
        Ok(self)
    }

    pub(crate) fn retain_ui_image_pins(&mut self, pins: WgpuUiImageInFlightPins) {
        debug_assert!(self.ui_image_pins.is_none());
        self.ui_image_pins = Some(pins);
    }

    fn into_submission_parts(
        self,
    ) -> (
        Vec<wgpu::CommandBuffer>,
        Option<WgpuUiImageInFlightPins>,
        Option<RhiGraphExecutionReceipt>,
    ) {
        (
            self.command_buffers,
            self.ui_image_pins,
            self.graph_execution_receipt,
        )
    }
}

/// Opaque staging frame encoded at the tail of a transitional native scene packet.
#[must_use = "a prepared diagnostic frame must be submitted or explicitly aborted"]
pub struct WgpuNativeDiagnosticReadbackFrame {
    pub(super) device_id: DeviceId,
    pub(super) generation: DeviceGeneration,
    pub(super) batch: DiagnosticReadbackBatch,
    pub(super) staging: wgpu::Buffer,
}

impl WgpuNativeDiagnosticReadbackFrame {
    pub const fn device_id(&self) -> DeviceId {
        self.device_id
    }

    pub const fn generation(&self) -> DeviceGeneration {
        self.generation
    }
}

impl WgpuRenderDevice {
    /// Starts one frame-scoped native recording lease for the transitional product renderer.
    pub fn begin_native_recording(
        &self,
        queue_class: RenderQueueClass,
    ) -> Result<WgpuNativeRecorderLease<'_>, RhiError> {
        self.ensure_admission()?;
        if !self.caps.supports_queue(queue_class) {
            return Err(RhiError::UnsupportedQueue(queue_class));
        }
        Ok(WgpuNativeRecorderLease::new(
            self.profile.device_id(),
            self.profile.generation(),
            queue_class,
            &self.device,
        ))
    }

    /// Enqueues a finished native packet without flushing it.
    ///
    /// The outer frame transaction remains the only owner allowed to flush. On commit failure the
    /// accepted ticket is cancelled immediately, so dropping or rejecting a packet cannot leak
    /// unresolved submission identity into a later frame.
    pub fn enqueue_native_recording_packet(
        &self,
        packet: WgpuNativeSubmissionPacket,
    ) -> Result<SubmissionTicket, RhiError> {
        self.enqueue_native_recording_packet_with_frame_diagnostics(packet, None, None)
    }

    /// Enqueues one native packet and binds its tail diagnostic copies to the same ticket.
    pub fn enqueue_native_recording_packet_with_diagnostics(
        &self,
        packet: WgpuNativeSubmissionPacket,
        diagnostic_frame: Option<WgpuNativeDiagnosticReadbackFrame>,
    ) -> Result<SubmissionTicket, RhiError> {
        self.enqueue_native_recording_packet_with_frame_diagnostics(packet, diagnostic_frame, None)
    }

    /// Enqueues one scene packet with every diagnostic tail bound to its sole ticket.
    /// 同一个场景票据同时约束命令缓冲、复制诊断和查询诊断；任一绑定失败须取消已接受包。
    pub fn enqueue_native_recording_packet_with_frame_diagnostics(
        &self,
        packet: WgpuNativeSubmissionPacket,
        diagnostic_frame: Option<WgpuNativeDiagnosticReadbackFrame>,
        query_frame: Option<WgpuNativeDiagnosticQueryFrame>,
    ) -> Result<SubmissionTicket, RhiError> {
        if let Err(error) = self.ensure_admission() {
            self.abandon_unbound_native_diagnostics(diagnostic_frame, query_frame);
            return Err(error);
        }
        if packet.device_id() != self.profile.device_id()
            || packet.generation() != self.profile.generation()
        {
            self.abandon_unbound_native_diagnostics(diagnostic_frame, query_frame);
            return Err(RhiError::SubmissionPacketDeviceMismatch {
                packet_device_id: packet.device_id(),
                packet_generation: packet.generation(),
                device_id: self.profile.device_id(),
                generation: self.profile.generation(),
            });
        }
        if !self.caps.supports_queue(packet.queue_class()) {
            self.abandon_unbound_native_diagnostics(diagnostic_frame, query_frame);
            return Err(RhiError::UnsupportedQueue(packet.queue_class()));
        }
        if let Some(frame) = diagnostic_frame.as_ref() {
            if frame.device_id() != self.profile.device_id()
                || frame.generation() != self.profile.generation()
            {
                let packet_device_id = frame.device_id();
                let packet_generation = frame.generation();
                self.abandon_unbound_native_diagnostics(diagnostic_frame, query_frame);
                return Err(RhiError::SubmissionPacketDeviceMismatch {
                    packet_device_id,
                    packet_generation,
                    device_id: self.profile.device_id(),
                    generation: self.profile.generation(),
                });
            }
        }
        if let Some(frame) = query_frame.as_ref() {
            if frame.device_id() != self.profile.device_id()
                || frame.generation() != self.profile.generation()
            {
                let packet_device_id = frame.device_id();
                let packet_generation = frame.generation();
                self.abandon_unbound_native_diagnostics(diagnostic_frame, query_frame);
                return Err(RhiError::SubmissionPacketDeviceMismatch {
                    packet_device_id,
                    packet_generation,
                    device_id: self.profile.device_id(),
                    generation: self.profile.generation(),
                });
            }
        }
        let queue_class = packet.queue_class();
        let (command_buffers, ui_image_pins, graph_execution_receipt) =
            packet.into_submission_parts();
        if command_buffers.is_empty() {
            self.abandon_unbound_native_diagnostics(diagnostic_frame, query_frame);
            return Err(RhiError::EmptySubmissionPacket);
        }
        let ticket = match self.submissions.begin_packet(queue_class) {
            Ok(ticket) => ticket,
            Err(error) => {
                self.abandon_unbound_native_diagnostics(diagnostic_frame, query_frame);
                return Err(error);
            }
        };
        let mut diagnostic_frame = diagnostic_frame;
        let mut query_frame = query_frame;
        if let Some(frame) = diagnostic_frame.take() {
            if let Err(error) =
                self.lock_diagnostics()
                    .bind_batch(ticket, frame.batch, frame.staging)
            {
                self.cancel_accepted_packet(ticket);
                self.lock_diagnostics()
                    .abandon_active_batch(DiagnosticReadbackTerminal::Cancelled);
                self.abandon_unbound_native_diagnostics(None, query_frame);
                return Err(error.into());
            }
        }
        if let Some(frame) = query_frame.take() {
            if let Err(error) = self
                .lock_diagnostics()
                .bind_native_query_frame(ticket, frame)
            {
                self.cancel_accepted_packet(ticket);
                return Err(error);
            }
        }
        let commit_result = match graph_execution_receipt {
            Some(receipt) => self
                .submissions
                .commit_packet_with_ui_image_pins_and_graph_receipt(
                    ticket,
                    command_buffers,
                    ui_image_pins,
                    receipt,
                ),
            None => self.submissions.commit_packet_with_ui_image_pins(
                ticket,
                command_buffers,
                ui_image_pins,
            ),
        };
        if let Err(error) = commit_result {
            self.cancel_accepted_packet(ticket);
            return Err(error);
        }
        Ok(ticket)
    }

    /// Enqueues and flushes one transitional native packet through the sole device timeline.
    pub fn submit_native_recording_packet(
        &self,
        packet: WgpuNativeSubmissionPacket,
    ) -> Result<SubmissionTicket, RhiError> {
        let ticket = self.enqueue_native_recording_packet(packet)?;
        if let Err(error) = self.flush_submissions() {
            let _ = self.cancel_submission(ticket);
            return Err(error);
        }
        Ok(ticket)
    }

    /// Submits a scene packet whose diagnostic tail shares the exact scene ticket.
    pub fn submit_native_recording_packet_with_diagnostics(
        &self,
        packet: WgpuNativeSubmissionPacket,
        diagnostic_frame: Option<WgpuNativeDiagnosticReadbackFrame>,
    ) -> Result<SubmissionTicket, RhiError> {
        let ticket = self.enqueue_native_recording_packet_with_frame_diagnostics(
            packet,
            diagnostic_frame,
            None,
        )?;
        if let Err(error) = self.flush_submissions() {
            let _ = self.cancel_submission(ticket);
            return Err(error);
        }
        Ok(ticket)
    }

    /// Submits one scene packet with copy and typed-query diagnostics on its sole ticket.
    pub fn submit_native_recording_packet_with_frame_diagnostics(
        &self,
        packet: WgpuNativeSubmissionPacket,
        diagnostic_frame: Option<WgpuNativeDiagnosticReadbackFrame>,
        query_frame: Option<WgpuNativeDiagnosticQueryFrame>,
    ) -> Result<SubmissionTicket, RhiError> {
        self.submit_native_recording_packet_with_frame_diagnostics_and_surface(
            packet,
            diagnostic_frame,
            query_frame,
            None,
        )
    }

    /// Submits one scene packet and binds an acquired surface target to its sole ticket.
    /// 先把表面帧目标登记到场景票据，再刷新设备提交队列，保证呈现能验证该票据。
    pub fn submit_native_recording_packet_with_frame_diagnostics_and_surface(
        &self,
        packet: WgpuNativeSubmissionPacket,
        diagnostic_frame: Option<WgpuNativeDiagnosticReadbackFrame>,
        query_frame: Option<WgpuNativeDiagnosticQueryFrame>,
        surface_target: Option<&WgpuNativeSurfaceFrameTarget>,
    ) -> Result<SubmissionTicket, RhiError> {
        if let Some(surface_target) = surface_target {
            if let Err(error) = surface_target.validate_owner(self) {
                self.abandon_unbound_native_diagnostics(diagnostic_frame, query_frame);
                return Err(error);
            }
        }
        let ticket = self.enqueue_native_recording_packet_with_frame_diagnostics(
            packet,
            diagnostic_frame,
            query_frame,
        )?;
        if let Some(surface_target) = surface_target {
            if let Err(error) =
                self.register_native_surface_frame_use(surface_target.frame_lease(), ticket)
            {
                self.cancel_accepted_packet(ticket);
                return Err(error);
            }
        }
        if let Err(error) = self.flush_submissions() {
            self.cancel_accepted_packet(ticket);
            return Err(error);
        }
        Ok(ticket)
    }

    fn abandon_unbound_native_diagnostics(
        &self,
        diagnostic_frame: Option<WgpuNativeDiagnosticReadbackFrame>,
        query_frame: Option<WgpuNativeDiagnosticQueryFrame>,
    ) {
        if diagnostic_frame.as_ref().is_some_and(|frame| {
            frame.device_id() == self.profile.device_id()
                && frame.generation() == self.profile.generation()
        }) {
            self.lock_diagnostics()
                .abandon_active_batch(DiagnosticReadbackTerminal::Cancelled);
        }
        if let Some(frame) = query_frame.filter(|frame| {
            frame.device_id() == self.profile.device_id()
                && frame.generation() == self.profile.generation()
        }) {
            self.lock_diagnostics()
                .abandon_prepared_native_query_frame(frame, DiagnosticReadbackTerminal::Cancelled);
        }
    }
}

#[cfg(test)]
#[path = "tests/native_recording.rs"]
mod tests;
