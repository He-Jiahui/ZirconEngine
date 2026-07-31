use std::collections::VecDeque;
use std::sync::mpsc::{self, Receiver, TryRecvError};

pub(crate) const DEFAULT_GPU_TIMER_MAX_PASSES: u32 = 64;
pub(crate) const GPU_TIMESTAMP_REQUIRED_FEATURES: wgpu::Features =
    wgpu::Features::TIMESTAMP_QUERY.union(wgpu::Features::TIMESTAMP_QUERY_INSIDE_ENCODERS);

const FRAMES_IN_FLIGHT: usize = 3;
const TIMESTAMPS_PER_PASS: u32 = 2;
const TIMESTAMP_SIZE_BYTES: u64 = size_of::<u64>() as u64;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct GpuPassTiming {
    pub(crate) pass_name: String,
    pub(crate) gpu_time_us: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct GpuTimerFrameResult {
    pub(crate) frame_generation: u64,
    pub(crate) pass_timings: Vec<GpuPassTiming>,
}

pub(crate) struct GpuPassTimer {
    query_set: wgpu::QuerySet,
    resolve_buffer: wgpu::Buffer,
    readback_slots: [TimerReadbackSlot; FRAMES_IN_FLIGHT],
    timestamp_period_ns: f32,
    max_timestamps: u32,
    next_slot: usize,
    active_frame: Option<ActiveTimerFrame>,
    completed_frames: VecDeque<GpuTimerFrameResult>,
}

impl GpuPassTimer {
    pub(crate) fn try_new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        max_passes: u32,
    ) -> Option<Self> {
        if !gpu_timestamp_features_supported(device.features()) || max_passes == 0 {
            return None;
        }
        let max_timestamps = max_passes.checked_mul(TIMESTAMPS_PER_PASS)?;
        let buffer_size = u64::from(max_timestamps).checked_mul(TIMESTAMP_SIZE_BYTES)?;
        let query_set = device.create_query_set(&wgpu::QuerySetDescriptor {
            label: Some("zircon-gpu-pass-timestamps"),
            ty: wgpu::QueryType::Timestamp,
            count: max_timestamps,
        });
        let resolve_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("zircon-gpu-pass-timestamp-resolve"),
            size: buffer_size,
            usage: wgpu::BufferUsages::QUERY_RESOLVE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        let readback_slots =
            std::array::from_fn(|index| TimerReadbackSlot::new(device, index, buffer_size));

        Some(Self {
            query_set,
            resolve_buffer,
            readback_slots,
            timestamp_period_ns: queue.get_timestamp_period(),
            max_timestamps,
            next_slot: 0,
            active_frame: None,
            completed_frames: VecDeque::new(),
        })
    }

    pub(crate) fn begin_frame(&mut self, device: &wgpu::Device, frame_generation: u64) {
        self.collect_ready_frames(device);
        let slot_index = self.find_available_slot();
        self.active_frame = slot_index.map(|slot_index| ActiveTimerFrame {
            frame_generation,
            slot_index,
            query_count: 0,
            pass_names: Vec::with_capacity((self.max_timestamps / TIMESTAMPS_PER_PASS) as usize),
        });
    }

    pub(crate) fn begin_pass(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        pass_name: &str,
    ) -> Option<GpuPassTimestampScope> {
        let active = self.active_frame.as_mut()?;
        let end_query_index = active.query_count.checked_add(1)?;
        if end_query_index >= self.max_timestamps {
            return None;
        }
        let begin_query_index = active.query_count;
        encoder.write_timestamp(&self.query_set, begin_query_index);
        active.query_count = active.query_count.saturating_add(TIMESTAMPS_PER_PASS);
        active.pass_names.push(pass_name.to_string());
        Some(GpuPassTimestampScope { end_query_index })
    }

    pub(crate) fn end_pass(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        scope: GpuPassTimestampScope,
    ) {
        encoder.write_timestamp(&self.query_set, scope.end_query_index);
    }

    pub(crate) fn resolve_and_copy(&mut self, encoder: &mut wgpu::CommandEncoder) {
        let Some(active) = self.active_frame.as_ref() else {
            return;
        };
        if active.query_count == 0 {
            return;
        }
        let readback = &self.readback_slots[active.slot_index].buffer;
        let resolved_bytes = u64::from(active.query_count) * TIMESTAMP_SIZE_BYTES;
        encoder.resolve_query_set(
            &self.query_set,
            0..active.query_count,
            &self.resolve_buffer,
            0,
        );
        encoder.copy_buffer_to_buffer(&self.resolve_buffer, 0, readback, 0, resolved_bytes);
    }

    pub(crate) fn after_submit(&mut self) {
        let Some(active) = self.active_frame.take() else {
            return;
        };
        if active.query_count == 0 {
            return;
        }
        let slot = &mut self.readback_slots[active.slot_index];
        let (sender, receiver) = mpsc::channel();
        slot.buffer
            .map_async(wgpu::MapMode::Read, .., move |result| {
                let _ = sender.send(result);
            });
        slot.pending = Some(PendingTimerFrame {
            receiver,
            frame_generation: active.frame_generation,
            pass_names: active.pass_names,
            query_count: active.query_count,
        });
        self.next_slot = (active.slot_index + 1) % FRAMES_IN_FLIGHT;
    }

    pub(crate) fn try_collect(&mut self, device: &wgpu::Device) -> Option<GpuTimerFrameResult> {
        self.collect_ready_frames(device);
        take_oldest_completed_frame(&mut self.completed_frames)
    }

    fn find_available_slot(&self) -> Option<usize> {
        (0..FRAMES_IN_FLIGHT)
            .map(|offset| (self.next_slot + offset) % FRAMES_IN_FLIGHT)
            .find(|slot_index| self.readback_slots[*slot_index].pending.is_none())
    }

    fn collect_ready_frames(&mut self, device: &wgpu::Device) {
        let _ = device.poll(wgpu::PollType::Poll);
        for slot in &mut self.readback_slots {
            let Some(pending) = slot.pending.as_ref() else {
                continue;
            };
            match pending.receiver.try_recv() {
                Ok(Ok(())) => {
                    let Some(pending) = slot.pending.take() else {
                        continue;
                    };
                    let result =
                        decode_timer_frame(&slot.buffer, pending, self.timestamp_period_ns);
                    slot.buffer.unmap();
                    if let Some(result) = result {
                        self.completed_frames.push_back(result);
                    }
                }
                Ok(Err(_)) | Err(TryRecvError::Disconnected) => {
                    slot.pending = None;
                    slot.buffer.unmap();
                }
                Err(TryRecvError::Empty) => {}
            }
        }
    }
}

fn take_oldest_completed_frame(
    completed_frames: &mut VecDeque<GpuTimerFrameResult>,
) -> Option<GpuTimerFrameResult> {
    completed_frames.pop_front()
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct GpuPassTimestampScope {
    end_query_index: u32,
}

struct ActiveTimerFrame {
    frame_generation: u64,
    slot_index: usize,
    query_count: u32,
    pass_names: Vec<String>,
}

struct TimerReadbackSlot {
    buffer: wgpu::Buffer,
    pending: Option<PendingTimerFrame>,
}

impl TimerReadbackSlot {
    fn new(device: &wgpu::Device, index: usize, size: u64) -> Self {
        Self {
            buffer: device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(&format!("zircon-gpu-pass-timestamp-readback-{index}")),
                size,
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                mapped_at_creation: false,
            }),
            pending: None,
        }
    }
}

struct PendingTimerFrame {
    receiver: Receiver<Result<(), wgpu::BufferAsyncError>>,
    frame_generation: u64,
    pass_names: Vec<String>,
    query_count: u32,
}

fn gpu_timestamp_features_supported(features: wgpu::Features) -> bool {
    features.contains(GPU_TIMESTAMP_REQUIRED_FEATURES)
}

fn decode_timer_frame(
    buffer: &wgpu::Buffer,
    pending: PendingTimerFrame,
    timestamp_period_ns: f32,
) -> Option<GpuTimerFrameResult> {
    let mapped = buffer.get_mapped_range(..);
    let ticks = decode_timestamp_pairs(&mapped, pending.query_count as usize)?;
    drop(mapped);
    let pass_timings = pending
        .pass_names
        .into_iter()
        .zip(ticks)
        .map(|(pass_name, [start, end])| GpuPassTiming {
            pass_name,
            gpu_time_us: timestamp_delta_us(start, end, timestamp_period_ns),
        })
        .collect();
    Some(GpuTimerFrameResult {
        frame_generation: pending.frame_generation,
        pass_timings,
    })
}

fn decode_timestamp_pairs(bytes: &[u8], query_count: usize) -> Option<Vec<[u64; 2]>> {
    if query_count % TIMESTAMPS_PER_PASS as usize != 0 {
        return None;
    }
    let mut pairs = Vec::with_capacity(query_count / TIMESTAMPS_PER_PASS as usize);
    for offset in (0..query_count).step_by(TIMESTAMPS_PER_PASS as usize) {
        let start = decode_timestamp(bytes, offset)?;
        let end = decode_timestamp(bytes, offset + 1)?;
        pairs.push([start, end]);
    }
    Some(pairs)
}

fn decode_timestamp(bytes: &[u8], index: usize) -> Option<u64> {
    let offset = index.checked_mul(TIMESTAMP_SIZE_BYTES as usize)?;
    Some(u64::from_le_bytes(
        bytes
            .get(offset..offset + TIMESTAMP_SIZE_BYTES as usize)?
            .try_into()
            .ok()?,
    ))
}

fn timestamp_delta_us(start: u64, end: u64, timestamp_period_ns: f32) -> u64 {
    let elapsed_ns = end.saturating_sub(start) as f64 * f64::from(timestamp_period_ns);
    (elapsed_ns / 1_000.0).round().clamp(0.0, u64::MAX as f64) as u64
}

#[cfg(test)]
mod tests {
    use super::{
        decode_timestamp_pairs, gpu_timestamp_features_supported, take_oldest_completed_frame,
        timestamp_delta_us, GpuTimerFrameResult, GPU_TIMESTAMP_REQUIRED_FEATURES,
    };
    use std::collections::VecDeque;

    #[test]
    fn timestamp_gate_requires_query_and_encoder_writes_but_not_inside_passes() {
        assert!(gpu_timestamp_features_supported(
            GPU_TIMESTAMP_REQUIRED_FEATURES | wgpu::Features::TIMESTAMP_QUERY_INSIDE_PASSES
        ));
        assert!(!gpu_timestamp_features_supported(
            wgpu::Features::TIMESTAMP_QUERY
        ));
        assert!(!gpu_timestamp_features_supported(
            wgpu::Features::TIMESTAMP_QUERY_INSIDE_ENCODERS
        ));
    }

    #[test]
    fn timestamp_pairs_decode_only_the_resolved_query_range() {
        let mut bytes = Vec::new();
        for timestamp in [10_u64, 20, 30, 50, 99, 100] {
            bytes.extend_from_slice(&timestamp.to_le_bytes());
        }

        assert_eq!(
            decode_timestamp_pairs(&bytes, 4),
            Some(vec![[10, 20], [30, 50]])
        );
    }

    #[test]
    fn timestamp_delta_converts_queue_period_to_rounded_microseconds() {
        assert_eq!(timestamp_delta_us(100, 132, 2.5), 0);
        assert_eq!(timestamp_delta_us(100, 900, 2.5), 2);
        assert_eq!(timestamp_delta_us(900, 100, 2.5), 0);
    }

    #[test]
    fn completed_timer_frames_are_drained_oldest_first_without_dropping_ready_results() {
        let mut completed_frames = VecDeque::from([
            GpuTimerFrameResult {
                frame_generation: 4,
                pass_timings: Vec::new(),
            },
            GpuTimerFrameResult {
                frame_generation: 5,
                pass_timings: Vec::new(),
            },
        ]);

        assert_eq!(
            take_oldest_completed_frame(&mut completed_frames)
                .expect("first completed frame")
                .frame_generation,
            4
        );
        assert_eq!(
            take_oldest_completed_frame(&mut completed_frames)
                .expect("second completed frame")
                .frame_generation,
            5
        );
        assert!(take_oldest_completed_frame(&mut completed_frames).is_none());
    }
}
