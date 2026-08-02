//! Shared WGPU timestamp-query lifecycle for scene and retained-UI render passes.

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use super::gpu_readback_queue::GpuReadbackQueue;

pub(crate) const DEFAULT_GPU_TIMER_MAX_PASSES: u32 = 64;
pub(crate) const GPU_TIMESTAMP_REQUIRED_FEATURES: wgpu::Features =
    wgpu::Features::TIMESTAMP_QUERY.union(wgpu::Features::TIMESTAMP_QUERY_INSIDE_ENCODERS);

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
    timestamp_period_ns: f32,
    max_timestamps: u32,
    active_frame: Option<ActiveTimerFrame>,
    completed_frames: Arc<Mutex<VecDeque<GpuTimerFrameResult>>>,
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
        Some(Self {
            query_set,
            resolve_buffer,
            timestamp_period_ns: queue.get_timestamp_period(),
            max_timestamps,
            active_frame: None,
            completed_frames: Arc::new(Mutex::new(VecDeque::new())),
        })
    }

    pub(crate) fn begin_frame(&mut self, frame_generation: u64) {
        self.active_frame = Some(ActiveTimerFrame {
            frame_generation,
            query_count: 0,
            pass_names: Vec::with_capacity((self.max_timestamps / TIMESTAMPS_PER_PASS) as usize),
        });
    }

    pub(crate) fn begin_pass(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        pass_name: &str,
    ) -> Option<GpuPassTimestampScope> {
        let scope = self.reserve_pass(pass_name)?;
        scope.begin(encoder);
        Some(scope)
    }

    pub(crate) fn reserve_pass(&mut self, pass_name: &str) -> Option<GpuPassTimestampScope> {
        let active = self.active_frame.as_mut()?;
        let end_query_index = active.query_count.checked_add(1)?;
        if end_query_index >= self.max_timestamps {
            return None;
        }
        let begin_query_index = active.query_count;
        active.query_count = active.query_count.saturating_add(TIMESTAMPS_PER_PASS);
        active.pass_names.push(pass_name.to_string());
        Some(GpuPassTimestampScope {
            query_set: self.query_set.clone(),
            begin_query_index,
            end_query_index,
        })
    }

    pub(crate) fn end_pass(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        scope: GpuPassTimestampScope,
    ) {
        scope.end(encoder);
    }

    pub(crate) fn resolve_and_request(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        readback_queue: &mut GpuReadbackQueue,
    ) {
        let Some(mut active) = self.active_frame.take() else {
            return;
        };
        let frame_generation = active.frame_generation;
        if active.query_count > 0 {
            let resolved_bytes = u64::from(active.query_count) * TIMESTAMP_SIZE_BYTES;
            encoder.resolve_query_set(
                &self.query_set,
                0..active.query_count,
                &self.resolve_buffer,
                0,
            );

            let completed_frames = Arc::clone(&self.completed_frames);
            let timestamp_period_ns = self.timestamp_period_ns;
            let query_count = active.query_count;
            let pass_names = std::mem::take(&mut active.pass_names);
            let callback = Box::new(move |bytes| {
                let Ok(bytes) = bytes else {
                    return;
                };
                let Some(result) = decode_timer_frame(
                    bytes,
                    frame_generation,
                    pass_names,
                    query_count,
                    timestamp_period_ns,
                ) else {
                    return;
                };
                let mut completed = completed_frames
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
                insert_completed_frame_in_order(&mut completed, result);
            });
            let _ = readback_queue.request_readback_external(
                "zircon-gpu-pass-timestamps",
                &self.resolve_buffer,
                0..resolved_bytes,
                callback,
            );
        }
    }

    pub(crate) fn try_collect(
        &mut self,
        device: &wgpu::Device,
        readback_queue: &mut GpuReadbackQueue,
    ) -> Option<GpuTimerFrameResult> {
        self.collect_ready_frames(device, readback_queue);
        let mut completed = self
            .completed_frames
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        take_oldest_completed_frame(&mut completed)
    }

    fn collect_ready_frames(
        &mut self,
        device: &wgpu::Device,
        readback_queue: &mut GpuReadbackQueue,
    ) {
        let _ = readback_queue.poll_completed(device);
    }
}

fn insert_completed_frame_in_order(
    completed_frames: &mut VecDeque<GpuTimerFrameResult>,
    result: GpuTimerFrameResult,
) {
    let insertion_index = completed_frames
        .iter()
        .position(|completed| completed.frame_generation > result.frame_generation)
        .unwrap_or(completed_frames.len());
    completed_frames.insert(insertion_index, result);
}

fn take_oldest_completed_frame(
    completed_frames: &mut VecDeque<GpuTimerFrameResult>,
) -> Option<GpuTimerFrameResult> {
    completed_frames.pop_front()
}

#[derive(Clone, Debug)]
pub(crate) struct GpuPassTimestampScope {
    query_set: wgpu::QuerySet,
    begin_query_index: u32,
    end_query_index: u32,
}

impl GpuPassTimestampScope {
    pub(crate) fn begin(&self, encoder: &mut wgpu::CommandEncoder) {
        encoder.write_timestamp(&self.query_set, self.begin_query_index);
    }

    pub(crate) fn end(&self, encoder: &mut wgpu::CommandEncoder) {
        encoder.write_timestamp(&self.query_set, self.end_query_index);
    }
}

struct ActiveTimerFrame {
    frame_generation: u64,
    query_count: u32,
    pass_names: Vec<String>,
}

fn gpu_timestamp_features_supported(features: wgpu::Features) -> bool {
    features.contains(GPU_TIMESTAMP_REQUIRED_FEATURES)
}

fn decode_timer_frame(
    bytes: &[u8],
    frame_generation: u64,
    pass_names: Vec<String>,
    query_count: u32,
    timestamp_period_ns: f32,
) -> Option<GpuTimerFrameResult> {
    let ticks = decode_timestamp_pairs(bytes, query_count as usize)?;
    let pass_timings = pass_names
        .into_iter()
        .zip(ticks)
        .map(|(pass_name, [start, end])| GpuPassTiming {
            pass_name,
            gpu_time_us: timestamp_delta_us(start, end, timestamp_period_ns),
        })
        .collect();
    Some(GpuTimerFrameResult {
        frame_generation,
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
        decode_timestamp_pairs, gpu_timestamp_features_supported, insert_completed_frame_in_order,
        take_oldest_completed_frame, timestamp_delta_us, GpuTimerFrameResult,
        GPU_TIMESTAMP_REQUIRED_FEATURES,
    };
    use std::collections::VecDeque;

    #[test]
    fn render_perf_gpu_timer_capability_gate() {
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
        let mut completed_frames = VecDeque::new();
        for frame_generation in [4, 2, 3] {
            insert_completed_frame_in_order(
                &mut completed_frames,
                GpuTimerFrameResult {
                    frame_generation,
                    pass_timings: Vec::new(),
                },
            );
        }

        let drained_generations = std::iter::from_fn(|| {
            take_oldest_completed_frame(&mut completed_frames).map(|frame| frame.frame_generation)
        })
        .collect::<Vec<_>>();

        assert_eq!(drained_generations, vec![2, 3, 4]);
        assert!(take_oldest_completed_frame(&mut completed_frames).is_none());
    }
}
