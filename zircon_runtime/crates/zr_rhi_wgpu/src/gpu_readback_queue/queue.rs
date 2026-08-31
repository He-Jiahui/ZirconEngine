use std::ops::Range;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::mpsc::{self, Receiver, TryRecvError};

use zr_rhi::DiagnosticReadbackBudget;

use super::staging_ring::{align_readback_offset, StagingCapacityPolicy, READBACK_FRAME_SLOTS};
use super::texture_readback::{
    texture_r32_uint_texel_readback_copy, texture_rgba_readback_copy, TextureReadbackCopy,
};
use super::ticket::{ReadbackCallback, ReadbackError, ReadbackTicket};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ReadbackPollStats {
    pub completed_request_count: usize,
    pub completed_bytes: u64,
    pub in_flight_count: usize,
    pub in_flight_bytes: u64,
    pub slot_reuse_rejection_count: u32,
}

pub struct GpuReadbackQueue {
    device: wgpu::Device,
    budget: DiagnosticReadbackBudget,
    slots: [StagingSlot; READBACK_FRAME_SLOTS],
    next_ticket: u64,
    pending: Vec<PendingRequest>,
    active_frame: Option<ActiveFrame>,
    active_frame_bytes: u64,
    in_flight_request_count: usize,
    in_flight_bytes: u64,
    slot_reuse_rejection_count: u32,
    last_poll_stats: ReadbackPollStats,
}

impl GpuReadbackQueue {
    pub const FRAME_SLOTS: usize = READBACK_FRAME_SLOTS;

    pub fn new(device: &wgpu::Device) -> Self {
        Self::with_budget(device, DiagnosticReadbackBudget::default())
    }

    pub fn with_budget(device: &wgpu::Device, budget: DiagnosticReadbackBudget) -> Self {
        Self {
            device: device.clone(),
            budget,
            slots: std::array::from_fn(StagingSlot::new),
            next_ticket: 1,
            pending: Vec::new(),
            active_frame: None,
            active_frame_bytes: 0,
            in_flight_request_count: 0,
            in_flight_bytes: 0,
            slot_reuse_rejection_count: 0,
            last_poll_stats: ReadbackPollStats::default(),
        }
    }

    pub fn prepare_frame(&mut self, frame_index: u64) -> Result<ReadbackPollStats, ReadbackError> {
        let mut stats = ReadbackPollStats {
            in_flight_count: self.in_flight_count(),
            in_flight_bytes: self.in_flight_bytes(),
            slot_reuse_rejection_count: self.slot_reuse_rejection_count,
            ..ReadbackPollStats::default()
        };
        if let Some(active) = self.active_frame {
            return Err(ReadbackError::FrameAlreadyActive {
                active: active.frame_index,
                requested: frame_index,
            });
        }

        let slot_index = frame_index as usize % READBACK_FRAME_SLOTS;
        if self.slots[slot_index].is_in_flight() {
            self.slot_reuse_rejection_count = self.slot_reuse_rejection_count.saturating_add(1);
            self.last_poll_stats = ReadbackPollStats {
                in_flight_count: self.in_flight_count(),
                in_flight_bytes: self.in_flight_bytes(),
                slot_reuse_rejection_count: self.slot_reuse_rejection_count,
                ..stats
            };
            return Err(ReadbackError::SlotReuseIncomplete { slot_index });
        }
        self.active_frame = Some(ActiveFrame {
            frame_index,
            slot_index,
            encoded: false,
        });
        self.active_frame_bytes = 0;
        stats.in_flight_count = self.in_flight_count();
        stats.in_flight_bytes = self.in_flight_bytes();
        stats.slot_reuse_rejection_count = self.slot_reuse_rejection_count;
        self.last_poll_stats = stats;
        Ok(stats)
    }

    pub fn request_readback_external(
        &mut self,
        name: impl Into<String>,
        buffer: &wgpu::Buffer,
        range: Range<u64>,
        callback: ReadbackCallback,
    ) -> Result<ReadbackTicket, ReadbackError> {
        self.ensure_request_admission_frame()?;
        validate_range(&range)?;
        let byte_len = range.end - range.start;
        self.validate_request_admission(byte_len)?;
        let ticket = ReadbackTicket::new(self.next_ticket);
        self.next_ticket = self.next_ticket.wrapping_add(1).max(1);
        self.pending.push(PendingRequest {
            ticket,
            name: name.into(),
            source: ReadbackSource::Buffer {
                buffer: buffer.clone(),
                source_offset: range.start,
            },
            byte_len,
            callback: Some(callback),
        });
        self.active_frame_bytes = self.active_frame_bytes.saturating_add(byte_len);
        self.register_in_flight_request(byte_len);
        Ok(ticket)
    }

    pub fn request_texture_rgba(
        &mut self,
        name: impl Into<String>,
        texture: &wgpu::Texture,
        width: u32,
        height: u32,
        callback: Box<dyn FnOnce(Result<Vec<u8>, ReadbackError>) + Send + 'static>,
    ) -> Result<ReadbackTicket, ReadbackError> {
        self.ensure_request_admission_frame()?;
        let copy = texture_rgba_readback_copy(width, height)?;
        self.validate_request_admission(copy.layout.staging_byte_len)?;
        let ticket = ReadbackTicket::new(self.next_ticket);
        self.next_ticket = self.next_ticket.wrapping_add(1).max(1);
        self.pending.push(PendingRequest {
            ticket,
            name: name.into(),
            source: ReadbackSource::Texture {
                texture: texture.clone(),
                copy,
            },
            byte_len: copy.layout.staging_byte_len,
            callback: Some(Box::new(move |result| {
                callback(result.and_then(|bytes| copy.layout.unpack_rgba(bytes)));
            })),
        });
        self.active_frame_bytes = self
            .active_frame_bytes
            .saturating_add(copy.layout.staging_byte_len);
        self.register_in_flight_request(copy.layout.staging_byte_len);
        Ok(ticket)
    }

    pub fn request_texture_r32_uint_texel(
        &mut self,
        name: impl Into<String>,
        texture: &wgpu::Texture,
        pixel: [u32; 2],
        callback: Box<dyn FnOnce(Result<u32, ReadbackError>) + Send + 'static>,
    ) -> Result<ReadbackTicket, ReadbackError> {
        self.ensure_request_admission_frame()?;
        let copy = texture_r32_uint_texel_readback_copy(texture, pixel)?;
        self.validate_request_admission(copy.layout.staging_byte_len)?;
        let ticket = ReadbackTicket::new(self.next_ticket);
        self.next_ticket = self.next_ticket.wrapping_add(1).max(1);
        self.pending.push(PendingRequest {
            ticket,
            name: name.into(),
            source: ReadbackSource::Texture {
                texture: texture.clone(),
                copy,
            },
            byte_len: copy.layout.staging_byte_len,
            callback: Some(Box::new(move |result| {
                callback(result.and_then(|bytes| copy.layout.unpack_r32_uint(bytes)));
            })),
        });
        self.active_frame_bytes = self
            .active_frame_bytes
            .saturating_add(copy.layout.staging_byte_len);
        self.register_in_flight_request(copy.layout.staging_byte_len);
        Ok(ticket)
    }

    pub fn encode_copies(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        frame_index: u64,
    ) -> Result<u64, ReadbackError> {
        let active = self
            .active_frame
            .as_mut()
            .filter(|active| active.frame_index == frame_index)
            .ok_or(ReadbackError::FrameNotActive {
                requested: frame_index,
            })?;
        if self.pending.is_empty() {
            self.slots[active.slot_index].ensure_capacity(&self.device, 0, frame_index)?;
            active.encoded = true;
            return Ok(0);
        }

        let mut cursor = 0_u64;
        let mut layouts = Vec::with_capacity(self.pending.len());
        for pending in &self.pending {
            let destination_offset =
                align_readback_offset(cursor).ok_or(ReadbackError::CapacityOverflow)?;
            let byte_len = pending.byte_len;
            cursor = destination_offset
                .checked_add(byte_len)
                .ok_or(ReadbackError::CapacityOverflow)?;
            layouts.push((destination_offset, byte_len));
        }

        let used_bytes = align_readback_offset(cursor).ok_or(ReadbackError::CapacityOverflow)?;
        let slot = &mut self.slots[active.slot_index];
        slot.ensure_capacity(&self.device, used_bytes, frame_index)?;
        let staging = slot
            .buffer
            .as_ref()
            .ok_or(ReadbackError::StagingBufferUnavailable {
                slot_index: active.slot_index,
                frame_index,
            })?;

        let mut encoded_requests = Vec::with_capacity(self.pending.len());
        for (pending, (destination_offset, byte_len)) in self.pending.drain(..).zip(layouts) {
            encoded_requests.push(EncodedRequest {
                ticket: pending.ticket,
                name: pending.name,
                destination_offset,
                byte_len,
                source: pending.source,
                callback: pending.callback,
            });
        }

        for request in &encoded_requests {
            match &request.source {
                ReadbackSource::Buffer {
                    buffer,
                    source_offset,
                } => encoder.copy_buffer_to_buffer(
                    buffer,
                    *source_offset,
                    staging,
                    request.destination_offset,
                    request.byte_len,
                ),
                ReadbackSource::Texture { texture, copy } => encoder.copy_texture_to_buffer(
                    wgpu::TexelCopyTextureInfo {
                        texture,
                        mip_level: 0,
                        origin: copy.origin,
                        aspect: wgpu::TextureAspect::All,
                    },
                    wgpu::TexelCopyBufferInfo {
                        buffer: staging,
                        layout: wgpu::TexelCopyBufferLayout {
                            offset: request.destination_offset,
                            bytes_per_row: Some(copy.layout.padded_bytes_per_row),
                            rows_per_image: Some(copy.layout.height),
                        },
                    },
                    copy.extent,
                ),
            }
        }
        slot.frame_index = Some(frame_index);
        slot.used_bytes = used_bytes;
        slot.requests = encoded_requests;
        active.encoded = true;
        Ok(used_bytes)
    }

    pub fn begin_map(&mut self, frame_index: u64) -> Result<(), ReadbackError> {
        let active = self.active_frame.ok_or(ReadbackError::FrameNotActive {
            requested: frame_index,
        })?;
        if active.frame_index != frame_index {
            return Err(ReadbackError::FrameNotActive {
                requested: frame_index,
            });
        }
        if !active.encoded {
            return Err(ReadbackError::FrameNotActive {
                requested: frame_index,
            });
        }
        let slot = &mut self.slots[active.slot_index];
        if slot.requests.is_empty() {
            self.active_frame = None;
            self.active_frame_bytes = 0;
            slot.frame_index = None;
            slot.used_bytes = 0;
            return Ok(());
        }

        let (sender, receiver) = mpsc::channel();
        let buffer = slot
            .buffer
            .as_ref()
            .ok_or(ReadbackError::StagingBufferUnavailable {
                slot_index: active.slot_index,
                frame_index,
            })?;
        buffer.map_async(wgpu::MapMode::Read, 0..slot.used_bytes, move |result| {
            let _ = sender.send(result);
        });
        slot.completion = Some(receiver);
        self.active_frame = None;
        self.active_frame_bytes = 0;
        Ok(())
    }

    pub(crate) fn collect_completed_after_device_poll(&mut self) -> ReadbackPollStats {
        let mut stats = ReadbackPollStats::default();
        for slot_index in 0..READBACK_FRAME_SLOTS {
            self.collect_slot(slot_index, &mut stats);
        }
        stats.in_flight_count = self.in_flight_count();
        stats.in_flight_bytes = self.in_flight_bytes();
        stats.slot_reuse_rejection_count = self.slot_reuse_rejection_count;
        self.last_poll_stats = stats;
        stats
    }

    #[cfg(test)]
    pub fn poll_completed(&mut self) -> ReadbackPollStats {
        let _ = self.device.poll(wgpu::PollType::Poll);
        self.collect_completed_after_device_poll()
    }

    pub fn stats(&self) -> ReadbackPollStats {
        ReadbackPollStats {
            in_flight_count: self.in_flight_count(),
            in_flight_bytes: self.in_flight_bytes(),
            slot_reuse_rejection_count: self.slot_reuse_rejection_count,
            ..self.last_poll_stats
        }
    }

    pub fn cancel(&mut self, ticket: ReadbackTicket) -> bool {
        let mut cancelled = false;
        let mut retained = Vec::with_capacity(self.pending.len());
        let mut released_pending_count = 0_usize;
        let mut released_pending_bytes = 0_u64;
        for mut request in self.pending.drain(..) {
            if request.ticket == ticket {
                cancelled |= complete_error_callback(
                    request.callback.take(),
                    ReadbackError::Cancelled { ticket },
                );
                released_pending_count = released_pending_count.saturating_add(1);
                released_pending_bytes = released_pending_bytes.saturating_add(request.byte_len);
            } else {
                retained.push(request);
            }
        }
        self.pending = retained;
        self.active_frame_bytes = self
            .active_frame_bytes
            .saturating_sub(released_pending_bytes);
        self.release_in_flight_requests(released_pending_count, released_pending_bytes);
        for slot in &mut self.slots {
            for request in slot
                .requests
                .iter_mut()
                .filter(|request| request.ticket == ticket)
            {
                cancelled |= complete_error_callback(
                    request.callback.take(),
                    ReadbackError::Cancelled { ticket },
                );
            }
        }
        cancelled
    }

    pub fn abort_frame(&mut self, frame_index: u64) {
        let Some(active) = self.active_frame else {
            return;
        };
        if active.frame_index != frame_index {
            return;
        }
        self.active_frame = None;
        let mut released_request_count = 0_usize;
        let mut released_bytes = 0_u64;
        for mut request in self.pending.drain(..) {
            released_request_count = released_request_count.saturating_add(1);
            released_bytes = released_bytes.saturating_add(request.byte_len);
            complete_error_callback(
                request.callback.take(),
                ReadbackError::FrameAborted { frame_index },
            );
        }
        let slot = &mut self.slots[active.slot_index];
        for mut request in slot.requests.drain(..) {
            released_request_count = released_request_count.saturating_add(1);
            released_bytes = released_bytes.saturating_add(request.byte_len);
            complete_error_callback(
                request.callback.take(),
                ReadbackError::FrameAborted { frame_index },
            );
        }
        slot.frame_index = None;
        slot.used_bytes = 0;
        self.active_frame_bytes = 0;
        self.release_in_flight_requests(released_request_count, released_bytes);
    }

    fn collect_slot(&mut self, slot_index: usize, stats: &mut ReadbackPollStats) {
        let (released_request_count, released_bytes) = {
            let slot = &mut self.slots[slot_index];
            let completion = match slot.completion.as_ref().map(Receiver::try_recv) {
                Some(Ok(result)) => SlotCompletion::Map(result),
                Some(Err(TryRecvError::Disconnected)) => SlotCompletion::Disconnected,
                Some(Err(TryRecvError::Empty)) | None => return,
            };
            slot.completion = None;
            let released_request_count = slot.requests.len();
            let released_bytes = slot.requests.iter().fold(0_u64, |total, request| {
                total.saturating_add(request.byte_len)
            });

            match completion {
                SlotCompletion::Map(Ok(())) => {
                    if let Some(buffer) = slot.buffer.as_ref() {
                        let mapped = buffer.get_mapped_range(0..slot.used_bytes);
                        for mut request in slot.requests.drain(..) {
                            let mapped_bytes = usize::try_from(request.destination_offset)
                                .ok()
                                .zip(usize::try_from(request.byte_len).ok())
                                .and_then(|(start, byte_len)| {
                                    start
                                        .checked_add(byte_len)
                                        .and_then(|end| mapped.get(start..end))
                                });
                            if let Some(callback) = request.callback.take() {
                                let delivered_bytes = mapped_bytes.is_some();
                                let request_name = request.name;
                                let ticket = request.ticket;
                                let _ = catch_unwind(AssertUnwindSafe(move || {
                                    match mapped_bytes {
                                    Some(bytes) => callback(Ok(bytes)),
                                    None => callback(Err(ReadbackError::BufferMap(format!(
                                        "readback request {ticket:?} ({request_name}) exceeded its mapped staging range"
                                    )))),
                                }
                                }));
                                stats.completed_request_count =
                                    stats.completed_request_count.saturating_add(1);
                                if delivered_bytes {
                                    stats.completed_bytes =
                                        stats.completed_bytes.saturating_add(request.byte_len);
                                }
                            }
                        }
                        drop(mapped);
                        buffer.unmap();
                    } else {
                        let frame_index = slot.frame_index;
                        fail_slot_requests(
                            slot,
                            stats,
                            format!(
                                "completed readback frame {:?} has no staging buffer in slot {slot_index}",
                                frame_index
                            ),
                        );
                    }
                }
                SlotCompletion::Map(Err(error)) => {
                    fail_slot_requests(slot, stats, error.to_string());
                }
                SlotCompletion::Disconnected => {
                    fail_slot_requests(
                        slot,
                        stats,
                        "WGPU map callback disconnected before completion".to_string(),
                    );
                }
            }
            slot.frame_index = None;
            slot.used_bytes = 0;
            (released_request_count, released_bytes)
        };
        self.release_in_flight_requests(released_request_count, released_bytes);
    }

    fn in_flight_count(&self) -> usize {
        self.in_flight_request_count
    }

    fn in_flight_bytes(&self) -> u64 {
        self.in_flight_bytes
    }

    fn ensure_request_admission_frame(&self) -> Result<(), ReadbackError> {
        let Some(active) = self.active_frame else {
            return Err(ReadbackError::NoActiveFrame);
        };
        if active.encoded {
            return Err(ReadbackError::FrameRequestsSealed {
                frame_index: active.frame_index,
            });
        }
        Ok(())
    }

    fn validate_request_admission(&self, byte_len: u64) -> Result<(), ReadbackError> {
        if byte_len > self.budget.max_request_bytes() {
            return Err(ReadbackError::RequestBytesExceeded {
                requested_bytes: byte_len,
                limit_bytes: self.budget.max_request_bytes(),
            });
        }
        if self.pending.len() >= self.budget.max_requests_per_frame() {
            return Err(ReadbackError::FrameRequestLimitExceeded {
                current_requests: self.pending.len(),
                limit: self.budget.max_requests_per_frame(),
            });
        }
        if byte_len
            > self
                .budget
                .max_frame_bytes()
                .saturating_sub(self.active_frame_bytes)
        {
            return Err(ReadbackError::FrameBytesExceeded {
                current_bytes: self.active_frame_bytes,
                requested_bytes: byte_len,
                limit_bytes: self.budget.max_frame_bytes(),
            });
        }
        if self.in_flight_request_count >= self.budget.max_pending_requests() {
            return Err(ReadbackError::PendingRequestLimitExceeded {
                current_requests: self.in_flight_request_count,
                limit: self.budget.max_pending_requests(),
            });
        }
        if byte_len
            > self
                .budget
                .max_pending_bytes()
                .saturating_sub(self.in_flight_bytes)
        {
            return Err(ReadbackError::PendingBytesExceeded {
                current_bytes: self.in_flight_bytes,
                requested_bytes: byte_len,
                limit_bytes: self.budget.max_pending_bytes(),
            });
        }
        Ok(())
    }

    fn register_in_flight_request(&mut self, byte_len: u64) {
        self.in_flight_request_count = self.in_flight_request_count.saturating_add(1);
        self.in_flight_bytes = self.in_flight_bytes.saturating_add(byte_len);
    }

    fn release_in_flight_requests(&mut self, request_count: usize, byte_len: u64) {
        self.in_flight_request_count = self.in_flight_request_count.saturating_sub(request_count);
        self.in_flight_bytes = self.in_flight_bytes.saturating_sub(byte_len);
    }
}

impl Drop for GpuReadbackQueue {
    fn drop(&mut self) {
        for mut request in self.pending.drain(..) {
            complete_error_callback(request.callback.take(), ReadbackError::Shutdown);
        }
        for slot in &mut self.slots {
            for mut request in slot.requests.drain(..) {
                complete_error_callback(request.callback.take(), ReadbackError::Shutdown);
            }
        }
    }
}

fn fail_slot_requests(slot: &mut StagingSlot, stats: &mut ReadbackPollStats, message: String) {
    for mut request in slot.requests.drain(..) {
        if let Some(callback) = request.callback.take() {
            let request_message = format!(
                "readback request {:?} ({}) failed: {message}",
                request.ticket, request.name
            );
            let _ = catch_unwind(AssertUnwindSafe(move || {
                callback(Err(ReadbackError::BufferMap(request_message)));
            }));
            stats.completed_request_count = stats.completed_request_count.saturating_add(1);
        }
    }
    if let Some(buffer) = slot.buffer.as_ref() {
        buffer.unmap();
    }
}

fn complete_error_callback(callback: Option<ReadbackCallback>, error: ReadbackError) -> bool {
    let Some(callback) = callback else {
        return false;
    };
    let _ = catch_unwind(AssertUnwindSafe(|| {
        callback(Err(error));
    }));
    true
}

enum SlotCompletion {
    Map(Result<(), wgpu::BufferAsyncError>),
    Disconnected,
}

struct PendingRequest {
    ticket: ReadbackTicket,
    name: String,
    source: ReadbackSource,
    byte_len: u64,
    callback: Option<ReadbackCallback>,
}

struct EncodedRequest {
    ticket: ReadbackTicket,
    name: String,
    destination_offset: u64,
    byte_len: u64,
    source: ReadbackSource,
    callback: Option<ReadbackCallback>,
}

enum ReadbackSource {
    Buffer {
        buffer: wgpu::Buffer,
        source_offset: u64,
    },
    Texture {
        texture: wgpu::Texture,
        copy: TextureReadbackCopy,
    },
}

#[derive(Clone, Copy)]
struct ActiveFrame {
    frame_index: u64,
    slot_index: usize,
    encoded: bool,
}

struct StagingSlot {
    index: usize,
    buffer: Option<wgpu::Buffer>,
    capacity_policy: StagingCapacityPolicy,
    last_frame_index: Option<u64>,
    frame_index: Option<u64>,
    used_bytes: u64,
    requests: Vec<EncodedRequest>,
    completion: Option<Receiver<Result<(), wgpu::BufferAsyncError>>>,
}

impl StagingSlot {
    fn new(index: usize) -> Self {
        Self {
            index,
            buffer: None,
            capacity_policy: StagingCapacityPolicy::default(),
            last_frame_index: None,
            frame_index: None,
            used_bytes: 0,
            requests: Vec::new(),
            completion: None,
        }
    }

    fn ensure_capacity(
        &mut self,
        device: &wgpu::Device,
        used_bytes: u64,
        frame_index: u64,
    ) -> Result<(), ReadbackError> {
        let elapsed_frames = self
            .last_frame_index
            .and_then(|previous| frame_index.checked_sub(previous))
            .unwrap_or(1)
            .clamp(1, u64::from(u16::MAX)) as u16;
        self.last_frame_index = Some(frame_index);
        let resized_capacity = self
            .capacity_policy
            .capacity_for_elapsed_frames(used_bytes, elapsed_frames);
        if used_bytes > self.capacity_policy.capacity() {
            return Err(ReadbackError::CapacityOverflow);
        }
        if let Some(capacity) = resized_capacity {
            self.buffer = Some(device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(&format!("zircon-gpu-readback-slot-{}", self.index)),
                size: capacity,
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                mapped_at_creation: false,
            }));
        }
        Ok(())
    }

    fn is_in_flight(&self) -> bool {
        self.completion.is_some()
    }
}

fn validate_range(range: &Range<u64>) -> Result<(), ReadbackError> {
    if range.start >= range.end {
        return Err(ReadbackError::EmptyRange {
            range: range.clone(),
        });
    }
    let byte_len = range.end - range.start;
    let alignment = u64::from(wgpu::COPY_BUFFER_ALIGNMENT);
    if range.start % alignment != 0 {
        return Err(ReadbackError::UnalignedSourceOffset {
            source_offset: range.start,
            alignment,
        });
    }
    if byte_len % alignment != 0 {
        return Err(ReadbackError::UnalignedCopySize {
            byte_len,
            alignment,
        });
    }
    Ok(())
}

#[cfg(test)]
impl GpuReadbackQueue {
    pub(super) fn inject_in_flight_slot_for_tests(
        &mut self,
        frame_index: u64,
    ) -> mpsc::Sender<Result<(), wgpu::BufferAsyncError>> {
        let slot_index = frame_index as usize % READBACK_FRAME_SLOTS;
        let (sender, receiver) = mpsc::channel();
        let slot = &mut self.slots[slot_index];
        slot.frame_index = Some(frame_index);
        slot.completion = Some(receiver);
        sender
    }
}
