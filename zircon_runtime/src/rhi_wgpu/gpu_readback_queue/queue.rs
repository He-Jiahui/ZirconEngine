use std::ops::Range;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::mpsc::{self, Receiver, TryRecvError};

use super::staging_ring::{align_readback_offset, StagingCapacityPolicy, READBACK_FRAME_SLOTS};
use super::ticket::{ReadbackCallback, ReadbackError, ReadbackTicket};

#[derive(Clone, Copy, Debug)]
pub(crate) struct TextureRgbaReadbackLayout {
    pub(crate) unpadded_bytes_per_row: u32,
    pub(crate) padded_bytes_per_row: u32,
    pub(crate) staging_byte_len: u64,
    height: u32,
}

impl TextureRgbaReadbackLayout {
    pub(crate) fn unpack_rgba(self, mapped: &[u8]) -> Result<Vec<u8>, ReadbackError> {
        let staging_byte_len =
            usize::try_from(self.staging_byte_len).map_err(|_| ReadbackError::CapacityOverflow)?;
        if mapped.len() < staging_byte_len {
            return Err(ReadbackError::BufferMap(
                "texture readback mapped range was smaller than its staging layout".to_string(),
            ));
        }
        let row_bytes = usize::try_from(self.unpadded_bytes_per_row)
            .map_err(|_| ReadbackError::CapacityOverflow)?;
        let padded_row_bytes = usize::try_from(self.padded_bytes_per_row)
            .map_err(|_| ReadbackError::CapacityOverflow)?;
        let total_rgba_bytes = row_bytes
            .checked_mul(self.height as usize)
            .ok_or(ReadbackError::CapacityOverflow)?;
        let mut rgba = Vec::with_capacity(total_rgba_bytes);
        for row in 0..self.height as usize {
            let start = row
                .checked_mul(padded_row_bytes)
                .ok_or(ReadbackError::CapacityOverflow)?;
            let end = start
                .checked_add(row_bytes)
                .ok_or(ReadbackError::CapacityOverflow)?;
            let Some(source_row) = mapped.get(start..end) else {
                return Err(ReadbackError::BufferMap(
                    "texture readback row exceeded its mapped staging range".to_string(),
                ));
            };
            rgba.extend_from_slice(source_row);
        }
        Ok(rgba)
    }
}

pub(crate) fn texture_rgba_readback_layout(
    width: u32,
    height: u32,
) -> Result<TextureRgbaReadbackLayout, ReadbackError> {
    if width == 0 || height == 0 {
        return Err(ReadbackError::InvalidTextureExtent { width, height });
    }
    let unpadded_bytes_per_row = width
        .checked_mul(4)
        .ok_or(ReadbackError::InvalidTextureExtent { width, height })?;
    let padded_bytes_per_row = unpadded_bytes_per_row
        .div_ceil(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT)
        .checked_mul(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT)
        .ok_or(ReadbackError::InvalidTextureExtent { width, height })?;
    let staging_byte_len = u64::from(padded_bytes_per_row)
        .checked_mul(u64::from(height))
        .ok_or(ReadbackError::InvalidTextureExtent { width, height })?;
    Ok(TextureRgbaReadbackLayout {
        unpadded_bytes_per_row,
        padded_bytes_per_row,
        staging_byte_len,
        height,
    })
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct ReadbackPollStats {
    pub(crate) completed_request_count: usize,
    pub(crate) completed_bytes: u64,
    pub(crate) in_flight_count: usize,
    pub(crate) in_flight_bytes: u64,
    pub(crate) slot_reuse_rejection_count: u32,
}

pub(crate) struct GpuReadbackQueue {
    device: wgpu::Device,
    slots: [StagingSlot; READBACK_FRAME_SLOTS],
    next_ticket: u64,
    pending: Vec<PendingRequest>,
    active_frame: Option<ActiveFrame>,
    slot_reuse_rejection_count: u32,
    last_poll_stats: ReadbackPollStats,
}

impl GpuReadbackQueue {
    pub(crate) const FRAME_SLOTS: usize = READBACK_FRAME_SLOTS;

    pub(crate) fn new(device: &wgpu::Device) -> Self {
        Self {
            device: device.clone(),
            slots: std::array::from_fn(StagingSlot::new),
            next_ticket: 1,
            pending: Vec::new(),
            active_frame: None,
            slot_reuse_rejection_count: 0,
            last_poll_stats: ReadbackPollStats::default(),
        }
    }

    pub(crate) fn prepare_frame(
        &mut self,
        device: &wgpu::Device,
        frame_index: u64,
    ) -> Result<ReadbackPollStats, ReadbackError> {
        let mut stats = self.poll_completed(device);
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
        stats.in_flight_count = self.in_flight_count();
        stats.in_flight_bytes = self.in_flight_bytes();
        stats.slot_reuse_rejection_count = self.slot_reuse_rejection_count;
        self.last_poll_stats = stats;
        Ok(stats)
    }

    pub(crate) fn request_readback_external(
        &mut self,
        name: impl Into<String>,
        buffer: &wgpu::Buffer,
        range: Range<u64>,
        callback: ReadbackCallback,
    ) -> Result<ReadbackTicket, ReadbackError> {
        if self.active_frame.is_none() {
            return Err(ReadbackError::NoActiveFrame);
        }
        validate_range(&range)?;
        let ticket = ReadbackTicket::new(self.next_ticket);
        self.next_ticket = self.next_ticket.wrapping_add(1).max(1);
        let byte_len = range.end - range.start;
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
        Ok(ticket)
    }

    pub(crate) fn request_texture_rgba(
        &mut self,
        name: impl Into<String>,
        texture: &wgpu::Texture,
        width: u32,
        height: u32,
        callback: Box<dyn FnOnce(Result<Vec<u8>, ReadbackError>) + Send + 'static>,
    ) -> Result<ReadbackTicket, ReadbackError> {
        if self.active_frame.is_none() {
            return Err(ReadbackError::NoActiveFrame);
        }
        let layout = texture_rgba_readback_layout(width, height)?;
        let ticket = ReadbackTicket::new(self.next_ticket);
        self.next_ticket = self.next_ticket.wrapping_add(1).max(1);
        self.pending.push(PendingRequest {
            ticket,
            name: name.into(),
            source: ReadbackSource::TextureRgba {
                texture: texture.clone(),
                width,
                layout,
            },
            byte_len: layout.staging_byte_len,
            callback: Some(Box::new(move |result| {
                callback(result.and_then(|bytes| layout.unpack_rgba(bytes)));
            })),
        });
        Ok(ticket)
    }

    pub(crate) fn encode_copies(
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
                ReadbackSource::TextureRgba {
                    texture,
                    width,
                    layout,
                } => encoder.copy_texture_to_buffer(
                    texture.as_image_copy(),
                    wgpu::TexelCopyBufferInfo {
                        buffer: staging,
                        layout: wgpu::TexelCopyBufferLayout {
                            offset: request.destination_offset,
                            bytes_per_row: Some(layout.padded_bytes_per_row),
                            rows_per_image: Some(layout.height),
                        },
                    },
                    wgpu::Extent3d {
                        width: *width,
                        height: layout.height,
                        depth_or_array_layers: 1,
                    },
                ),
            }
        }
        slot.frame_index = Some(frame_index);
        slot.used_bytes = used_bytes;
        slot.requests = encoded_requests;
        active.encoded = true;
        Ok(used_bytes)
    }

    pub(crate) fn begin_map(&mut self, frame_index: u64) -> Result<(), ReadbackError> {
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
        Ok(())
    }

    pub(crate) fn poll_completed(&mut self, device: &wgpu::Device) -> ReadbackPollStats {
        let _ = device.poll(wgpu::PollType::Poll);
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

    pub(crate) fn stats(&self) -> ReadbackPollStats {
        ReadbackPollStats {
            in_flight_count: self.in_flight_count(),
            in_flight_bytes: self.in_flight_bytes(),
            slot_reuse_rejection_count: self.slot_reuse_rejection_count,
            ..self.last_poll_stats
        }
    }

    pub(crate) fn cancel(&mut self, ticket: ReadbackTicket) {
        self.pending.retain(|request| request.ticket != ticket);
        for slot in &mut self.slots {
            if let Some(request) = slot
                .requests
                .iter_mut()
                .find(|request| request.ticket == ticket)
            {
                request.callback = None;
            }
        }
    }

    pub(crate) fn abort_frame(&mut self, frame_index: u64) {
        let Some(active) = self.active_frame else {
            return;
        };
        if active.frame_index != frame_index {
            return;
        }
        self.active_frame = None;
        for mut request in self.pending.drain(..) {
            if let Some(callback) = request.callback.take() {
                let _ = catch_unwind(AssertUnwindSafe(|| {
                    callback(Err(ReadbackError::FrameAborted { frame_index }));
                }));
            }
        }
        let slot = &mut self.slots[active.slot_index];
        for mut request in slot.requests.drain(..) {
            if let Some(callback) = request.callback.take() {
                let _ = catch_unwind(AssertUnwindSafe(|| {
                    callback(Err(ReadbackError::FrameAborted { frame_index }));
                }));
            }
        }
        slot.frame_index = None;
        slot.used_bytes = 0;
    }

    fn collect_slot(&mut self, slot_index: usize, stats: &mut ReadbackPollStats) {
        let slot = &mut self.slots[slot_index];
        let completion = match slot.completion.as_ref().map(Receiver::try_recv) {
            Some(Ok(result)) => SlotCompletion::Map(result),
            Some(Err(TryRecvError::Disconnected)) => SlotCompletion::Disconnected,
            Some(Err(TryRecvError::Empty)) | None => return,
        };
        slot.completion = None;

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
    }

    fn in_flight_count(&self) -> usize {
        self.slots
            .iter()
            .map(|slot| slot.requests.len())
            .sum::<usize>()
            .saturating_add(self.pending.len())
    }

    fn in_flight_bytes(&self) -> u64 {
        self.slots
            .iter()
            .map(|slot| {
                slot.requests
                    .iter()
                    .map(|request| request.byte_len)
                    .sum::<u64>()
            })
            .sum::<u64>()
            .saturating_add(self.pending.iter().map(|request| request.byte_len).sum())
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
    TextureRgba {
        texture: wgpu::Texture,
        width: u32,
        layout: TextureRgbaReadbackLayout,
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
