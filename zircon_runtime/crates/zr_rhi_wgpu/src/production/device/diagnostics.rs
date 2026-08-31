use zr_rhi::{
    BufferDesc, BufferHandle, BufferUsage, DiagnosticFrameKey, DiagnosticQueryPlan,
    DiagnosticReadbackAdmission, DiagnosticReadbackRequestId, DiagnosticReadbackTerminal,
    RenderDevice, RenderQueueClass, RhiError, SubmissionTicket, TextureCopyRegion, TextureDesc,
    TextureHandle, TextureUsage,
};

use super::super::diagnostics::{
    DiagnosticReadbackBatch, DiagnosticReadbackSource, DiagnosticTextureReadbackLayout,
    WgpuDiagnosticQueryDelivery, WgpuDiagnosticReadbackDelivery,
    WgpuDiagnosticReadbackMetricsSnapshot, WgpuNativeDiagnosticQueryFrame,
    WgpuNativeDiagnosticQueryRecorder,
};
use super::super::submission_metrics::WgpuSubmissionMetricsSnapshot;
use super::{WgpuNativeDiagnosticReadbackFrame, WgpuRenderDevice};

#[path = "diagnostics_native_texture_mip_chain.rs"]
mod native_texture_mip_chain;
use native_texture_mip_chain::{
    ensure_native_rgba16float_texture_mip_chain_readback, record_native_diagnostic_texture_copy,
};

impl WgpuRenderDevice {
    /// Returns monotonic submission facts for this WGPU device generation.
    ///
    /// This is intentionally WGPU-specific rather than a `RenderDevice` trait requirement. A
    /// profiler samples two snapshots to derive an interval without resetting another consumer.
    pub fn submission_metrics(&self) -> WgpuSubmissionMetricsSnapshot {
        self.submissions.metrics_snapshot()
    }

    /// Reserves bounded native query ranges before transitional scene passes record.
    ///
    /// Query objects are generation-qualified and carry no queue, ticket, flush, or poll access.
    /// The actual neutral query plan is supplied at the scene encoder tail.
    pub fn begin_native_diagnostic_query_frame(
        &self,
        frame_index: u64,
        timestamps_enabled: bool,
        pipeline_statistics_enabled: bool,
    ) -> Result<Option<WgpuNativeDiagnosticQueryRecorder>, RhiError> {
        self.ensure_admission()?;
        self.lock_diagnostics().begin_native_query_frame(
            &self.device,
            self.timestamp_period_ns,
            frame_index,
            timestamps_enabled,
            pipeline_statistics_enabled,
        )
    }

    /// Appends the actual query resolves and staging copies to the scene command encoder.
    ///
    /// This consumes the reservation but allocates no submission identity. The returned frame is
    /// bound only when the enclosing native scene packet is accepted.
    pub fn prepare_native_diagnostic_query_frame(
        &self,
        recorder: WgpuNativeDiagnosticQueryRecorder,
        plan: DiagnosticQueryPlan,
        encoder: &mut wgpu::CommandEncoder,
    ) -> Result<Option<WgpuNativeDiagnosticQueryFrame>, RhiError> {
        if recorder.device_id() != self.profile.device_id()
            || recorder.generation() != self.profile.generation()
        {
            let packet_device_id = recorder.device_id();
            let packet_generation = recorder.generation();
            return Err(RhiError::SubmissionPacketDeviceMismatch {
                packet_device_id,
                packet_generation,
                device_id: self.profile.device_id(),
                generation: self.profile.generation(),
            });
        }
        if let Err(error) = self.ensure_admission() {
            self.lock_diagnostics().abandon_native_query_recorder(
                recorder,
                diagnostic_terminal_status_for_error(&error),
            );
            return Err(error);
        }
        self.lock_diagnostics()
            .prepare_native_query_frame(&self.device, recorder, plan, encoder)
    }

    /// Cancels an admitted native query reservation that cannot reach the scene tail.
    pub fn abort_native_diagnostic_query_recorder(
        &self,
        recorder: WgpuNativeDiagnosticQueryRecorder,
        terminal: DiagnosticReadbackTerminal,
    ) {
        if recorder.device_id() == self.profile.device_id()
            && recorder.generation() == self.profile.generation()
        {
            self.lock_diagnostics()
                .abandon_native_query_recorder(recorder, terminal);
        }
    }

    /// Cancels a resolved native query frame that cannot reach its scene packet.
    pub fn abort_prepared_native_diagnostic_query_frame(
        &self,
        frame: WgpuNativeDiagnosticQueryFrame,
        terminal: DiagnosticReadbackTerminal,
    ) {
        if frame.device_id() == self.profile.device_id()
            && frame.generation() == self.profile.generation()
        {
            self.lock_diagnostics()
                .abandon_prepared_native_query_frame(frame, terminal);
        }
    }

    /// Opens a bounded diagnostics batch. Its native copy work remains queued
    /// until `submit_diagnostic_readback_frame` gives it a real submission.
    pub fn begin_diagnostic_readback_frame(&self, frame_index: u64) -> Result<(), RhiError> {
        self.ensure_admission()?;
        self.lock_diagnostics().begin_frame(frame_index)?;
        Ok(())
    }

    /// Admits a buffer request after validating the neutral handle, usage,
    /// range, and WGPU copy alignment. Quota rejection is returned as an
    /// observable terminal receipt without recording native work.
    pub fn enqueue_diagnostic_buffer_readback(
        &self,
        source: BufferHandle,
        offset: u64,
        size: u64,
    ) -> Result<DiagnosticReadbackAdmission, RhiError> {
        self.ensure_admission()?;
        let desc = self.lock_registry().buffer_desc(source)?;
        ensure_diagnostic_readback_range(source, &desc, offset, size)?;
        Ok(self.lock_diagnostics().admit_buffer(source, offset, size)?)
    }

    /// Admits one color texture mip/layer/slice region for readback. Delivery
    /// bytes are tightly row-packed even though the internal staging copy is
    /// padded to WGPU's row alignment requirement.
    pub fn enqueue_diagnostic_texture_readback(
        &self,
        source: TextureHandle,
        region: TextureCopyRegion,
    ) -> Result<DiagnosticReadbackAdmission, RhiError> {
        self.ensure_admission()?;
        let desc = self.lock_registry().texture_desc(source)?;
        let layout = ensure_diagnostic_texture_readback_region(source, &desc, region)?;
        Ok(self
            .lock_diagnostics()
            .admit_texture(source, region, layout)?)
    }

    /// Admits one transitional native buffer range owned by this product device generation.
    /// The source clone remains in the bounded batch until the scene-qualified map terminates.
    pub fn enqueue_native_diagnostic_buffer_readback(
        &self,
        source: &wgpu::Buffer,
        offset: u64,
        size: u64,
    ) -> Result<DiagnosticReadbackAdmission, RhiError> {
        self.ensure_admission()?;
        ensure_native_diagnostic_readback_range(source, offset, size)?;
        Ok(self
            .lock_diagnostics()
            .admit_native_buffer(source.clone(), offset, size)?)
    }

    /// Admits one transitional native RGBA8 texture owned by this product device generation.
    ///
    /// The texture clone is retained inside the bounded diagnostic batch until its scene-qualified
    /// submission reaches a terminal map result. This bridge is removed with the raw product data
    /// plane; new renderer code should use neutral `TextureHandle` sources.
    pub fn enqueue_native_diagnostic_texture_rgba8_readback(
        &self,
        source: &wgpu::Texture,
        width: u32,
        height: u32,
    ) -> Result<DiagnosticReadbackAdmission, RhiError> {
        self.ensure_admission()?;
        let (region, layout) = ensure_native_rgba8_texture_readback(source, width, height)?;
        Ok(self
            .lock_diagnostics()
            .admit_native_texture(source.clone(), region, layout)?)
    }

    /// Admits one transitional native RGBA16F mip/layer region.
    ///
    /// This is the product IBL artifact bridge while graph textures are still native WGPU
    /// resources. Each request remains part of the enclosing scene diagnostic batch and carries
    /// no independent submission or completion authority.
    pub fn enqueue_native_diagnostic_texture_rgba16float_readback(
        &self,
        source: &wgpu::Texture,
        mip_level: u32,
        array_layer: u32,
        width: u32,
        height: u32,
    ) -> Result<DiagnosticReadbackAdmission, RhiError> {
        self.ensure_admission()?;
        let (region, layout) = ensure_native_rgba16float_texture_readback(
            source,
            mip_level,
            array_layer,
            width,
            height,
        )?;
        Ok(self
            .lock_diagnostics()
            .admit_native_texture(source.clone(), region, layout)?)
    }

    /// Admits one RGBA16F texture mip chain as a single bounded diagnostic request.
    ///
    /// Every mip is copied directly into the device-owned batch staging buffer. Delivery bytes are
    /// tightly packed in ascending mip order, while quota accounting uses the WGPU-padded staging
    /// extent of the whole chain.
    pub fn enqueue_native_diagnostic_texture_rgba16float_mip_chain_readback(
        &self,
        source: &wgpu::Texture,
        array_layer: u32,
        mip_count: u32,
    ) -> Result<DiagnosticReadbackAdmission, RhiError> {
        self.ensure_admission()?;
        let layout =
            ensure_native_rgba16float_texture_mip_chain_readback(source, array_layer, mip_count)?;
        Ok(self
            .lock_diagnostics()
            .admit_native_texture_mip_chain(source.clone(), layout)?)
    }

    /// Admits one R32Uint texel for an exact identity-product readback.
    pub fn enqueue_native_diagnostic_texture_r32_uint_texel_readback(
        &self,
        source: &wgpu::Texture,
        pixel: [u32; 2],
    ) -> Result<DiagnosticReadbackAdmission, RhiError> {
        self.ensure_admission()?;
        let (region, layout) = ensure_native_pick_texture_texel(
            source,
            pixel,
            wgpu::TextureFormat::R32Uint,
            4,
            "R32Uint",
        )?;
        Ok(self
            .lock_diagnostics()
            .admit_native_texture(source.clone(), region, layout)?)
    }

    /// Admits one RGBA32F texel for exact world-position and depth products.
    pub fn enqueue_native_diagnostic_texture_rgba32float_texel_readback(
        &self,
        source: &wgpu::Texture,
        pixel: [u32; 2],
    ) -> Result<DiagnosticReadbackAdmission, RhiError> {
        self.ensure_admission()?;
        let (region, layout) = ensure_native_pick_texture_texel(
            source,
            pixel,
            wgpu::TextureFormat::Rgba32Float,
            16,
            "RGBA32F",
        )?;
        Ok(self
            .lock_diagnostics()
            .admit_native_texture(source.clone(), region, layout)?)
    }

    /// Encodes an admitted transitional native diagnostic batch at the tail of a scene encoder.
    ///
    /// The returned opaque frame must be supplied to the scene packet submission so the copy and
    /// source writes share one `SubmissionTicket`; this method allocates no ticket and never
    /// submits, flushes, or polls native work.
    pub fn prepare_native_diagnostic_readback_frame(
        &self,
        label: &str,
        encoder: &mut wgpu::CommandEncoder,
    ) -> Result<Option<WgpuNativeDiagnosticReadbackFrame>, RhiError> {
        self.ensure_admission()?;
        if label.is_empty() {
            self.lock_diagnostics()
                .abandon_active_batch(DiagnosticReadbackTerminal::Cancelled);
            return Err(RhiError::InvalidDebugMarker {
                reason: "native diagnostic readback label must not be empty".to_string(),
            });
        }
        let batch = self.lock_diagnostics().take_active_batch()?;
        let Some(batch) = batch else {
            return Ok(None);
        };
        let staging = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(label),
            size: batch.byte_len(),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        for request in batch.requests() {
            match request.source() {
                DiagnosticReadbackSource::NativeBuffer(source_request) => {
                    encoder.copy_buffer_to_buffer(
                        source_request.source(),
                        source_request.source_offset(),
                        &staging,
                        request.staging_offset(),
                        source_request.byte_len(),
                    );
                }
                DiagnosticReadbackSource::NativeTexture(source_request) => {
                    record_native_diagnostic_texture_copy(
                        encoder,
                        &staging,
                        request.staging_offset(),
                        source_request.source(),
                        source_request.region(),
                        source_request.layout(),
                    );
                }
                DiagnosticReadbackSource::NativeTextureMipChain(source_request) => {
                    for subresource in source_request.layout().subresources() {
                        let staging_offset = request
                            .staging_offset()
                            .checked_add(subresource.staging_offset())
                            .ok_or_else(|| RhiError::InvalidCopy {
                                reason: "native diagnostic mip-chain staging offset overflowed"
                                    .to_string(),
                            })?;
                        record_native_diagnostic_texture_copy(
                            encoder,
                            &staging,
                            staging_offset,
                            source_request.source(),
                            subresource.region(),
                            subresource.layout(),
                        );
                    }
                }
                DiagnosticReadbackSource::Buffer(_) | DiagnosticReadbackSource::Texture(_) => {
                    self.lock_diagnostics()
                        .abandon_active_batch(DiagnosticReadbackTerminal::Cancelled);
                    return Err(RhiError::InvalidCopy {
                        reason: "native diagnostic frame cannot consume neutral registry sources before ticket allocation"
                            .to_string(),
                    });
                }
            }
        }
        Ok(Some(WgpuNativeDiagnosticReadbackFrame {
            device_id: self.profile.device_id(),
            generation: self.profile.generation(),
            batch,
            staging,
        }))
    }

    /// Cancels a prepared native diagnostic frame that cannot reach its scene packet.
    pub fn abort_prepared_native_diagnostic_readback_frame(
        &self,
        frame: WgpuNativeDiagnosticReadbackFrame,
    ) {
        if frame.device_id() == self.profile.device_id()
            && frame.generation() == self.profile.generation()
        {
            self.lock_diagnostics()
                .abandon_active_batch(DiagnosticReadbackTerminal::Cancelled);
        }
    }

    /// Encodes all requests admitted for the active diagnostics frame into one
    /// copy packet owned by the normal submission service.
    pub fn submit_diagnostic_readback_frame(
        &self,
        label: &str,
    ) -> Result<Option<DiagnosticFrameKey>, RhiError> {
        self.ensure_admission()?;
        if label.is_empty() {
            return Err(RhiError::InvalidDebugMarker {
                reason: "diagnostic readback batch label must not be empty".to_string(),
            });
        }
        let batch = self.lock_diagnostics().take_active_batch()?;
        let Some(batch) = batch else {
            return Ok(None);
        };
        let ticket = self.submissions.begin_packet(RenderQueueClass::Copy)?;
        let encoded = self.encode_diagnostic_readback_batch(label, ticket, &batch);
        let (command_buffer, staging) = match encoded {
            Ok(encoded) => encoded,
            Err(error) => {
                self.cancel_accepted_packet(ticket);
                self.lock_diagnostics()
                    .abandon_active_batch(DiagnosticReadbackTerminal::MapFailed);
                return Err(error);
            }
        };
        let frame_key = match self.lock_diagnostics().bind_batch(ticket, batch, staging) {
            Ok(frame_key) => frame_key,
            Err(error) => {
                self.cancel_accepted_packet(ticket);
                self.lock_diagnostics()
                    .abandon_active_batch(DiagnosticReadbackTerminal::MapFailed);
                return Err(error.into());
            }
        };
        if let Err(error) = self.submissions.commit_packet(ticket, vec![command_buffer]) {
            self.cancel_accepted_packet(ticket);
            self.lock_diagnostics()
                .terminalize_submission(ticket, DiagnosticReadbackTerminal::Cancelled);
            return Err(error);
        }
        Ok(Some(frame_key))
    }

    /// Encodes and flushes one standalone diagnostic batch through the sole device timeline.
    pub fn submit_and_flush_diagnostic_readback_frame(
        &self,
        label: &str,
    ) -> Result<Option<DiagnosticFrameKey>, RhiError> {
        let Some(frame) = self.submit_diagnostic_readback_frame(label)? else {
            return Ok(None);
        };
        if let Err(error) = self.flush_submissions() {
            let _ = self.cancel_submission(frame.submission());
            return Err(error);
        }
        Ok(Some(frame))
    }

    /// Cancels requests admitted into the active diagnostic frame before it receives a submission.
    ///
    /// This intentionally remains available after device admission has failed: callers use it on
    /// timeout and rejection paths to avoid retaining an unusable active diagnostic frame.
    pub fn abort_diagnostic_readback_frame(&self, terminal: DiagnosticReadbackTerminal) {
        self.lock_diagnostics().abandon_active_batch(terminal);
    }

    /// Drains one completed diagnostic delivery. Successful deliveries include
    /// the copied bytes; every other terminal result carries no payload.
    pub fn take_diagnostic_readback_delivery(&self) -> Option<WgpuDiagnosticReadbackDelivery> {
        self.lock_diagnostics().take_delivery()
    }

    /// Moves every completed readback delivery into a caller-owned routing buffer in ticket
    /// order while acquiring the diagnostics lock once. Payload bytes are moved, not copied.
    pub fn append_diagnostic_readback_deliveries(
        &self,
        output: &mut Vec<WgpuDiagnosticReadbackDelivery>,
    ) -> usize {
        self.lock_diagnostics().append_deliveries(output)
    }

    /// Drains the oldest completed delivery only when it belongs to `request`.
    ///
    /// The diagnostic service preserves submission order, so a different oldest request remains
    /// retained for its owner instead of being accidentally reinterpreted by this caller.
    pub fn take_diagnostic_readback_delivery_for(
        &self,
        request: DiagnosticReadbackRequestId,
    ) -> Option<WgpuDiagnosticReadbackDelivery> {
        self.lock_diagnostics().take_delivery_for(request)
    }

    /// Number of oldest diagnostic deliveries evicted by the configured
    /// bounded result ring.
    pub fn dropped_diagnostic_readback_delivery_count(&self) -> u64 {
        self.lock_diagnostics().dropped_delivery_count()
    }

    /// Bytes retained by successful diagnostic deliveries that have not yet
    /// been drained. The value is bounded by the diagnostic staging budget.
    pub fn retained_diagnostic_readback_delivery_bytes(&self) -> u64 {
        self.lock_diagnostics().retained_delivery_bytes()
    }

    /// Returns monotonic diagnostic counters and current bounded-service gauges.
    ///
    /// Consumers sample two snapshots to derive an interval. Counters are never reset by a
    /// reader, so capture, telemetry, and profiling consumers cannot steal each other's window.
    pub fn diagnostic_readback_metrics(&self) -> WgpuDiagnosticReadbackMetricsSnapshot {
        self.lock_diagnostics().metrics_snapshot()
    }

    /// Drains one completed frame-qualified timestamp/statistics delivery.
    /// Labels are intentionally absent: the graph compiler maps dense pass IDs
    /// to names when it exports this data to a profiler or capture report.
    pub fn take_diagnostic_query_delivery(&self) -> Option<WgpuDiagnosticQueryDelivery> {
        self.lock_diagnostics().take_query_delivery()
    }

    /// Moves every completed typed query delivery into a caller-owned routing buffer in ticket
    /// order while acquiring the diagnostics lock once.
    pub fn append_diagnostic_query_deliveries(
        &self,
        output: &mut Vec<WgpuDiagnosticQueryDelivery>,
    ) -> usize {
        self.lock_diagnostics().append_query_deliveries(output)
    }

    /// Number of oldest query-frame deliveries evicted by the bounded result
    /// ring. Query results never grow an unbounded CPU-side completion queue.
    pub fn dropped_diagnostic_query_delivery_count(&self) -> u64 {
        self.lock_diagnostics().dropped_query_delivery_count()
    }

    fn encode_diagnostic_readback_batch(
        &self,
        label: &str,
        ticket: SubmissionTicket,
        batch: &DiagnosticReadbackBatch,
    ) -> Result<(wgpu::CommandBuffer, wgpu::Buffer), RhiError> {
        let staging = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(label),
            size: batch.byte_len(),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some(label) });
        let mut registry = self.lock_registry();
        for request in batch.requests() {
            match request.source() {
                DiagnosticReadbackSource::Buffer(source_request) => {
                    let desc = registry.buffer_desc(source_request.source())?;
                    ensure_diagnostic_readback_range(
                        source_request.source(),
                        &desc,
                        source_request.source_offset(),
                        source_request.byte_len(),
                    )?;
                    let source = registry.buffer(source_request.source())?.clone();
                    registry
                        .mark_buffer_diagnostic_readback_use(source_request.source(), ticket)?;
                    encoder.copy_buffer_to_buffer(
                        &source,
                        source_request.source_offset(),
                        &staging,
                        request.staging_offset(),
                        source_request.byte_len(),
                    );
                }
                DiagnosticReadbackSource::NativeBuffer(source_request) => {
                    encoder.copy_buffer_to_buffer(
                        source_request.source(),
                        source_request.source_offset(),
                        &staging,
                        request.staging_offset(),
                        source_request.byte_len(),
                    );
                }
                DiagnosticReadbackSource::Texture(source_request) => {
                    let desc = registry.texture_desc(source_request.source())?;
                    let layout = ensure_diagnostic_texture_readback_region(
                        source_request.source(),
                        &desc,
                        source_request.region(),
                    )?;
                    debug_assert_eq!(layout, source_request.layout());
                    let source = registry.texture(source_request.source())?.clone();
                    registry
                        .mark_texture_diagnostic_readback_use(source_request.source(), ticket)?;
                    let region = source_request.region();
                    encoder.copy_texture_to_buffer(
                        wgpu::TexelCopyTextureInfo {
                            texture: &source,
                            mip_level: region.mip_level,
                            origin: wgpu::Origin3d {
                                x: region.origin_x,
                                y: region.origin_y,
                                z: region.origin_z,
                            },
                            aspect: wgpu::TextureAspect::All,
                        },
                        wgpu::TexelCopyBufferInfo {
                            buffer: &staging,
                            layout: wgpu::TexelCopyBufferLayout {
                                offset: request.staging_offset(),
                                bytes_per_row: Some(layout.padded_bytes_per_row()),
                                rows_per_image: Some(layout.height()),
                            },
                        },
                        wgpu::Extent3d {
                            width: region.width,
                            height: region.height,
                            depth_or_array_layers: 1,
                        },
                    );
                }
                DiagnosticReadbackSource::NativeTexture(source_request) => {
                    record_native_diagnostic_texture_copy(
                        &mut encoder,
                        &staging,
                        request.staging_offset(),
                        source_request.source(),
                        source_request.region(),
                        source_request.layout(),
                    );
                }
                DiagnosticReadbackSource::NativeTextureMipChain(source_request) => {
                    for subresource in source_request.layout().subresources() {
                        let staging_offset = request
                            .staging_offset()
                            .checked_add(subresource.staging_offset())
                            .ok_or_else(|| RhiError::InvalidCopy {
                                reason: "native diagnostic mip-chain staging offset overflowed"
                                    .to_string(),
                            })?;
                        record_native_diagnostic_texture_copy(
                            &mut encoder,
                            &staging,
                            staging_offset,
                            source_request.source(),
                            subresource.region(),
                            subresource.layout(),
                        );
                    }
                }
            }
        }
        Ok((encoder.finish(), staging))
    }
}

const fn diagnostic_terminal_status_for_error(error: &RhiError) -> DiagnosticReadbackTerminal {
    match error {
        RhiError::DeviceAdmission(_) => DiagnosticReadbackTerminal::DeviceLost,
        _ => DiagnosticReadbackTerminal::Cancelled,
    }
}

fn ensure_diagnostic_texture_readback_region(
    source: TextureHandle,
    desc: &TextureDesc,
    region: TextureCopyRegion,
) -> Result<DiagnosticTextureReadbackLayout, RhiError> {
    if !desc.usage.contains(TextureUsage::COPY_SRC) {
        return Err(RhiError::InvalidCopy {
            reason: format!(
                "diagnostic texture readback source {:?} requires COPY_SRC usage",
                source.diagnostic_id()
            ),
        });
    }
    if desc.format.is_depth() {
        return Err(RhiError::InvalidCopy {
            reason: "diagnostic texture readback requires a color texture; depth/stencil conversion is not encoded by this path".to_string(),
        });
    }
    let copy = crate::texture_copy::texture_copy_layout(desc, region).ok_or_else(|| {
        RhiError::InvalidCopy {
            reason: format!(
                "diagnostic texture readback region is outside source {:?} subresource bounds",
                source.diagnostic_id()
            ),
        }
    })?;
    DiagnosticTextureReadbackLayout::new(copy.copy_row_bytes, region.height).ok_or_else(|| {
        RhiError::InvalidCopy {
            reason: "diagnostic texture readback staging layout overflowed".to_string(),
        }
    })
}

fn ensure_native_rgba8_texture_readback(
    source: &wgpu::Texture,
    width: u32,
    height: u32,
) -> Result<(TextureCopyRegion, DiagnosticTextureReadbackLayout), RhiError> {
    let source_size = source.size();
    if width == 0
        || height == 0
        || width > source_size.width
        || height > source_size.height
        || source_size.depth_or_array_layers == 0
    {
        return Err(RhiError::InvalidCopy {
            reason: format!(
                "native RGBA8 diagnostic extent {width}x{height} exceeds source {}x{}x{}",
                source_size.width, source_size.height, source_size.depth_or_array_layers
            ),
        });
    }
    if source.dimension() != wgpu::TextureDimension::D2 || source.sample_count() != 1 {
        return Err(RhiError::InvalidCopy {
            reason: "native RGBA8 diagnostic readback requires a single-sample 2D texture"
                .to_string(),
        });
    }
    if !matches!(
        source.format(),
        wgpu::TextureFormat::Rgba8Unorm | wgpu::TextureFormat::Rgba8UnormSrgb
    ) {
        return Err(RhiError::InvalidCopy {
            reason: "native viewport diagnostic readback currently supports only RGBA8 textures"
                .to_string(),
        });
    }
    if !source.usage().contains(wgpu::TextureUsages::COPY_SRC) {
        return Err(RhiError::InvalidCopy {
            reason: "native viewport diagnostic readback source requires COPY_SRC usage"
                .to_string(),
        });
    }
    let copy_row_bytes = u64::from(width)
        .checked_mul(4)
        .ok_or_else(|| RhiError::InvalidCopy {
            reason: "native viewport diagnostic row byte count overflowed".to_string(),
        })?;
    let layout = DiagnosticTextureReadbackLayout::new(copy_row_bytes, height).ok_or_else(|| {
        RhiError::InvalidCopy {
            reason: "native viewport diagnostic staging layout overflowed".to_string(),
        }
    })?;
    Ok((TextureCopyRegion::new(width, height), layout))
}

fn ensure_native_rgba16float_texture_readback(
    source: &wgpu::Texture,
    mip_level: u32,
    array_layer: u32,
    width: u32,
    height: u32,
) -> Result<(TextureCopyRegion, DiagnosticTextureReadbackLayout), RhiError> {
    if source.dimension() != wgpu::TextureDimension::D2 || source.sample_count() != 1 {
        return Err(RhiError::InvalidCopy {
            reason: "native RGBA16F diagnostic readback requires a single-sample 2D texture"
                .to_string(),
        });
    }
    if source.format() != wgpu::TextureFormat::Rgba16Float {
        return Err(RhiError::InvalidCopy {
            reason: "native IBL diagnostic readback requires an RGBA16F texture".to_string(),
        });
    }
    if !source.usage().contains(wgpu::TextureUsages::COPY_SRC) {
        return Err(RhiError::InvalidCopy {
            reason: "native IBL diagnostic readback source requires COPY_SRC usage".to_string(),
        });
    }
    let source_size = source.size();
    if mip_level >= source.mip_level_count() || array_layer >= source_size.depth_or_array_layers {
        return Err(RhiError::InvalidCopy {
            reason: format!(
                "native RGBA16F diagnostic subresource mip {mip_level} layer {array_layer} exceeds {} mips and {} layers",
                source.mip_level_count(),
                source_size.depth_or_array_layers
            ),
        });
    }
    let mip_width = source_size.width.checked_shr(mip_level).unwrap_or(0).max(1);
    let mip_height = source_size
        .height
        .checked_shr(mip_level)
        .unwrap_or(0)
        .max(1);
    if width == 0 || height == 0 || width > mip_width || height > mip_height {
        return Err(RhiError::InvalidCopy {
            reason: format!(
                "native RGBA16F diagnostic extent {width}x{height} exceeds mip {mip_level} extent {mip_width}x{mip_height}"
            ),
        });
    }
    let copy_row_bytes = u64::from(width)
        .checked_mul(8)
        .ok_or_else(|| RhiError::InvalidCopy {
            reason: "native RGBA16F diagnostic row byte count overflowed".to_string(),
        })?;
    let layout = DiagnosticTextureReadbackLayout::new(copy_row_bytes, height).ok_or_else(|| {
        RhiError::InvalidCopy {
            reason: "native RGBA16F diagnostic staging layout overflowed".to_string(),
        }
    })?;
    Ok((
        TextureCopyRegion::new(width, height)
            .with_mip_level(mip_level)
            .with_origin(0, 0, array_layer),
        layout,
    ))
}

fn ensure_native_pick_texture_texel(
    source: &wgpu::Texture,
    pixel: [u32; 2],
    expected_format: wgpu::TextureFormat,
    copy_row_bytes: u64,
    format_label: &str,
) -> Result<(TextureCopyRegion, DiagnosticTextureReadbackLayout), RhiError> {
    let source_size = source.size();
    if source.dimension() != wgpu::TextureDimension::D2
        || source.sample_count() != 1
        || source_size.depth_or_array_layers != 1
    {
        return Err(RhiError::InvalidCopy {
            reason: format!(
                "native {format_label} pick readback requires a single-sample, single-layer 2D texture"
            ),
        });
    }
    if source.format() != expected_format {
        return Err(RhiError::InvalidCopy {
            reason: format!("native pick readback requires a {format_label} texture"),
        });
    }
    if !source.usage().contains(wgpu::TextureUsages::COPY_SRC) {
        return Err(RhiError::InvalidCopy {
            reason: "native pick readback source requires COPY_SRC usage".to_string(),
        });
    }
    if pixel[0] >= source_size.width || pixel[1] >= source_size.height {
        return Err(RhiError::InvalidCopy {
            reason: format!(
                "native pick texel {},{} exceeds source {}x{}",
                pixel[0], pixel[1], source_size.width, source_size.height
            ),
        });
    }
    let layout = DiagnosticTextureReadbackLayout::new(copy_row_bytes, 1).ok_or_else(|| {
        RhiError::InvalidCopy {
            reason: "native pick diagnostic staging layout overflowed".to_string(),
        }
    })?;
    Ok((
        TextureCopyRegion::new(1, 1).with_origin(pixel[0], pixel[1], 0),
        layout,
    ))
}

fn ensure_diagnostic_readback_range(
    handle: BufferHandle,
    desc: &BufferDesc,
    offset: u64,
    size: u64,
) -> Result<(), RhiError> {
    if !desc.usage.contains(BufferUsage::COPY_SRC) {
        return Err(RhiError::InvalidBufferUsage {
            buffer: handle.diagnostic_id(),
            required: BufferUsage::COPY_SRC,
            actual: desc.usage,
        });
    }
    let copy_alignment = u64::from(wgpu::COPY_BUFFER_ALIGNMENT);
    if size == 0 || offset % copy_alignment != 0 || size % copy_alignment != 0 {
        return Err(RhiError::InvalidCopy {
            reason:
                "diagnostic buffer readback requires non-zero WGPU copy-aligned offset and size"
                    .to_string(),
        });
    }
    if offset.saturating_add(size) > desc.size_bytes {
        return Err(RhiError::ReadbackOutOfRange {
            buffer: handle.diagnostic_id(),
            offset,
            size,
        });
    }
    Ok(())
}

fn ensure_native_diagnostic_readback_range(
    source: &wgpu::Buffer,
    offset: u64,
    size: u64,
) -> Result<(), RhiError> {
    if !source.usage().contains(wgpu::BufferUsages::COPY_SRC) {
        return Err(RhiError::InvalidCopy {
            reason: "native diagnostic buffer source requires COPY_SRC usage".to_string(),
        });
    }
    let copy_alignment = u64::from(wgpu::COPY_BUFFER_ALIGNMENT);
    if size == 0 || offset % copy_alignment != 0 || size % copy_alignment != 0 {
        return Err(RhiError::InvalidCopy {
            reason: "native diagnostic buffer readback requires non-zero WGPU copy-aligned offset and size"
                .to_string(),
        });
    }
    if offset.saturating_add(size) > source.size() {
        return Err(RhiError::InvalidCopy {
            reason: format!(
                "native diagnostic buffer range {offset}..{} exceeds source size {}",
                offset.saturating_add(size),
                source.size()
            ),
        });
    }
    Ok(())
}
