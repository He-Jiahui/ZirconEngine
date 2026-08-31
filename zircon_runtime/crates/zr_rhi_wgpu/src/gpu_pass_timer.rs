//! Shared WGPU timestamp-query lifecycle for scene and retained-UI render passes.

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use super::gpu_readback_queue::{GpuReadbackQueue, ReadbackCallback};
use super::{GpuDiagnosticQueryFramePlan, WgpuDiagnosticQueryDelivery};
use zr_rhi::{DiagnosticQueryPlan, DiagnosticReadbackTerminal, TimestampScope};

pub const DEFAULT_GPU_TIMER_MAX_PASSES: u32 = 64;
pub const GPU_TIMESTAMP_REQUIRED_FEATURES: wgpu::Features =
    wgpu::Features::TIMESTAMP_QUERY.union(wgpu::Features::TIMESTAMP_QUERY_INSIDE_ENCODERS);

const TIMESTAMPS_PER_PASS: u32 = 2;
const TIMESTAMP_SIZE_BYTES: u64 = size_of::<u64>() as u64;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GpuPassTiming {
    pub pass_name: String,
    pub gpu_time_us: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GpuTimerFrameResult {
    pub frame_generation: u64,
    pub pass_timings: Vec<GpuPassTiming>,
}

/// One timestamp query frame can complete later than the frame that submitted it.
/// Keep that lifecycle fact independent from framework-facing diagnostic DTOs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GpuTimerFrameStatus {
    Pending,
    Deferred,
    CapacityExhausted,
    NoPasses,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GpuTimerFrameObservation {
    pub frame_generation: u64,
    pub status: GpuTimerFrameStatus,
}

pub struct GpuPassTimer {
    legacy_query_set: Option<wgpu::QuerySet>,
    legacy_resolve_buffer: Option<wgpu::Buffer>,
    timestamp_period_ns: f32,
    max_timestamps: u32,
    active_frame: Option<ActiveTimerFrame>,
    last_frame_observation: Option<GpuTimerFrameObservation>,
    completed_frames: Arc<Mutex<VecDeque<GpuTimerFrameResult>>>,
}

impl GpuPassTimer {
    pub fn try_new(device: &wgpu::Device, queue: &wgpu::Queue, max_passes: u32) -> Option<Self> {
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
            legacy_query_set: Some(query_set),
            legacy_resolve_buffer: Some(resolve_buffer),
            timestamp_period_ns: queue.get_timestamp_period(),
            max_timestamps,
            active_frame: None,
            last_frame_observation: None,
            completed_frames: Arc::new(Mutex::new(VecDeque::new())),
        })
    }

    /// Creates a product-scene adapter without allocating a second query set or resolve buffer.
    pub fn try_new_product(
        device: &wgpu::Device,
        timestamp_period_ns: f32,
        max_passes: u32,
    ) -> Option<Self> {
        if !gpu_timestamp_features_supported(device.features()) || max_passes == 0 {
            return None;
        }
        Some(Self {
            legacy_query_set: None,
            legacy_resolve_buffer: None,
            timestamp_period_ns,
            max_timestamps: max_passes.checked_mul(TIMESTAMPS_PER_PASS)?,
            active_frame: None,
            last_frame_observation: None,
            completed_frames: Arc::new(Mutex::new(VecDeque::new())),
        })
    }

    pub fn begin_frame(&mut self, frame_generation: u64) {
        if self.legacy_query_set.is_none() || self.legacy_resolve_buffer.is_none() {
            self.defer_frame(frame_generation);
            return;
        }
        self.active_frame = Some(ActiveTimerFrame {
            frame_generation,
            query_count: 0,
            capacity_exhausted: false,
            recording: ActiveTimerRecording::Legacy {
                pass_names: Vec::with_capacity(
                    (self.max_timestamps / TIMESTAMPS_PER_PASS) as usize,
                ),
            },
        });
        self.last_frame_observation = None;
    }

    pub fn begin_product_frame(
        &mut self,
        frame_generation: u64,
        plan: GpuDiagnosticQueryFramePlan,
        query_set: &wgpu::QuerySet,
    ) {
        self.active_frame = Some(ActiveTimerFrame {
            frame_generation,
            query_count: 0,
            capacity_exhausted: false,
            recording: ActiveTimerRecording::Product {
                plan,
                query_set: query_set.clone(),
            },
        });
        self.last_frame_observation = None;
    }

    pub fn begin_pass(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        pass_name: &str,
    ) -> Option<GpuPassTimestampScope> {
        let scope = self.reserve_pass(pass_name)?;
        scope.begin(encoder);
        Some(scope)
    }

    pub fn reserve_pass(&mut self, pass_name: &str) -> Option<GpuPassTimestampScope> {
        let active = self.active_frame.as_mut()?;
        let end_query_index = active.query_count.checked_add(1)?;
        if end_query_index >= self.max_timestamps {
            active.capacity_exhausted = true;
            return None;
        }
        let (query_set, scope) = match &mut active.recording {
            ActiveTimerRecording::Legacy { pass_names } => {
                let begin_query_index = active.query_count;
                pass_names.push(pass_name.to_string());
                let scope = ProductTimestampScope::Legacy {
                    begin_query_index,
                    end_query_index,
                };
                (self.legacy_query_set.as_ref()?.clone(), scope)
            }
            ActiveTimerRecording::Product { plan, query_set } => {
                let scope = match plan.reserve_timestamp_scope(pass_name) {
                    Ok(scope) => scope,
                    Err(_) => {
                        active.capacity_exhausted = true;
                        return None;
                    }
                };
                (query_set.clone(), ProductTimestampScope::Product(scope))
            }
        };
        active.query_count = active.query_count.saturating_add(TIMESTAMPS_PER_PASS);
        Some(GpuPassTimestampScope { query_set, scope })
    }

    pub fn end_pass(&self, encoder: &mut wgpu::CommandEncoder, scope: GpuPassTimestampScope) {
        scope.end(encoder);
    }

    pub fn resolve_and_request(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        readback_queue: &mut GpuReadbackQueue,
    ) -> Option<GpuTimerFrameObservation> {
        let Some(mut active) = self.active_frame.take() else {
            return None;
        };
        let ActiveTimerRecording::Legacy { ref mut pass_names } = active.recording else {
            self.active_frame = Some(active);
            return None;
        };
        let frame_generation = active.frame_generation;
        let status = if active.query_count > 0 {
            let resolved_bytes = u64::from(active.query_count) * TIMESTAMP_SIZE_BYTES;
            let query_set = self.legacy_query_set.as_ref()?;
            let resolve_buffer = self.legacy_resolve_buffer.as_ref()?;
            encoder.resolve_query_set(query_set, 0..active.query_count, resolve_buffer, 0);

            let completed_frames = Arc::clone(&self.completed_frames);
            let timestamp_period_ns = self.timestamp_period_ns;
            let query_count = active.query_count;
            let pass_names = std::mem::take(pass_names);
            let callback: ReadbackCallback = Box::new(move |bytes| {
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
            let readback_admitted = readback_queue
                .request_readback_external(
                    "zircon-gpu-pass-timestamps",
                    resolve_buffer,
                    0..resolved_bytes,
                    callback,
                )
                .is_ok();
            timer_frame_status(
                active.query_count,
                active.capacity_exhausted,
                readback_admitted,
            )
        } else {
            GpuTimerFrameStatus::NoPasses
        };
        let observation = GpuTimerFrameObservation {
            frame_generation,
            status,
        };
        self.last_frame_observation = Some(observation);
        Some(observation)
    }

    pub fn finish_product_frame(&mut self) -> Option<GpuTimerFrameObservation> {
        let active = self.active_frame.take()?;
        if !matches!(active.recording, ActiveTimerRecording::Product { .. }) {
            self.active_frame = Some(active);
            return None;
        }
        let status = if active.query_count == 0 {
            GpuTimerFrameStatus::NoPasses
        } else if active.capacity_exhausted {
            GpuTimerFrameStatus::CapacityExhausted
        } else {
            GpuTimerFrameStatus::Pending
        };
        let observation = GpuTimerFrameObservation {
            frame_generation: active.frame_generation,
            status,
        };
        self.last_frame_observation = Some(observation);
        Some(observation)
    }

    pub fn accept_product_query_delivery(
        &mut self,
        frame_generation: u64,
        plan: &DiagnosticQueryPlan,
        pass_names: &[String],
        delivery: &WgpuDiagnosticQueryDelivery,
    ) {
        if delivery.timestamp_period_ns > 0.0 {
            self.timestamp_period_ns = delivery.timestamp_period_ns;
        }
        if delivery.terminal != DiagnosticReadbackTerminal::Succeeded {
            if self
                .last_frame_observation
                .is_some_and(|observation| observation.frame_generation == frame_generation)
            {
                self.defer_frame(frame_generation);
            }
            return;
        }
        let Some(results) = delivery.pass_results.as_ref() else {
            return;
        };
        let mut timestamp_passes = vec![false; plan.pass_count()];
        for scope in plan.timestamp_scopes() {
            timestamp_passes[scope.pass().index()] = true;
        }
        let pass_timings = results
            .iter()
            .filter(|result| timestamp_passes.get(result.pass.index()) == Some(&true))
            .filter_map(|result| {
                pass_names
                    .get(result.pass.index())
                    .map(|pass_name| GpuPassTiming {
                        pass_name: pass_name.clone(),
                        gpu_time_us: timestamp_ticks_us(
                            result.timestamp_ticks,
                            delivery.timestamp_period_ns,
                        ),
                    })
            })
            .collect();
        let mut completed = self
            .completed_frames
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        insert_completed_frame_in_order(
            &mut completed,
            GpuTimerFrameResult {
                frame_generation,
                pass_timings,
            },
        );
    }

    /// Records a non-admitted frame without allocating another ring or retrying its readback.
    pub fn defer_frame(&mut self, frame_generation: u64) -> GpuTimerFrameObservation {
        self.active_frame = None;
        let observation = GpuTimerFrameObservation {
            frame_generation,
            status: GpuTimerFrameStatus::Deferred,
        };
        self.last_frame_observation = Some(observation);
        observation
    }

    pub fn last_frame_observation(&self) -> Option<GpuTimerFrameObservation> {
        self.last_frame_observation
    }

    pub fn timestamp_period_ns(&self) -> f32 {
        self.timestamp_period_ns
    }

    pub fn try_collect(&mut self) -> Option<GpuTimerFrameResult> {
        let mut completed = self
            .completed_frames
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        take_oldest_completed_frame(&mut completed)
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
pub struct GpuPassTimestampScope {
    query_set: wgpu::QuerySet,
    scope: ProductTimestampScope,
}

impl GpuPassTimestampScope {
    pub fn begin(&self, encoder: &mut wgpu::CommandEncoder) {
        encoder.write_timestamp(&self.query_set, self.scope.begin_query_index());
    }

    pub fn end(&self, encoder: &mut wgpu::CommandEncoder) {
        encoder.write_timestamp(&self.query_set, self.scope.end_query_index());
    }
}

#[derive(Clone, Copy, Debug)]
enum ProductTimestampScope {
    Legacy {
        begin_query_index: u32,
        end_query_index: u32,
    },
    Product(TimestampScope),
}

impl ProductTimestampScope {
    const fn begin_query_index(self) -> u32 {
        match self {
            Self::Legacy {
                begin_query_index, ..
            } => begin_query_index,
            Self::Product(scope) => scope.begin_query(),
        }
    }

    const fn end_query_index(self) -> u32 {
        match self {
            Self::Legacy {
                end_query_index, ..
            } => end_query_index,
            Self::Product(scope) => scope.end_query(),
        }
    }
}

struct ActiveTimerFrame {
    frame_generation: u64,
    query_count: u32,
    capacity_exhausted: bool,
    recording: ActiveTimerRecording,
}

enum ActiveTimerRecording {
    Legacy {
        pass_names: Vec<String>,
    },
    Product {
        plan: GpuDiagnosticQueryFramePlan,
        query_set: wgpu::QuerySet,
    },
}

fn timer_frame_status(
    query_count: u32,
    capacity_exhausted: bool,
    readback_admitted: bool,
) -> GpuTimerFrameStatus {
    if query_count == 0 {
        GpuTimerFrameStatus::NoPasses
    } else if !readback_admitted {
        GpuTimerFrameStatus::Deferred
    } else if capacity_exhausted {
        GpuTimerFrameStatus::CapacityExhausted
    } else {
        GpuTimerFrameStatus::Pending
    }
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
    timestamp_ticks_us(end.saturating_sub(start), timestamp_period_ns)
}

fn timestamp_ticks_us(ticks: u64, timestamp_period_ns: f32) -> u64 {
    let elapsed_ns = ticks as f64 * f64::from(timestamp_period_ns);
    (elapsed_ns / 1_000.0).round().clamp(0.0, u64::MAX as f64) as u64
}

#[cfg(test)]
mod tests {
    use super::{
        decode_timestamp_pairs, gpu_timestamp_features_supported, insert_completed_frame_in_order,
        take_oldest_completed_frame, timer_frame_status, timestamp_delta_us, GpuTimerFrameResult,
        GpuTimerFrameStatus, GPU_TIMESTAMP_REQUIRED_FEATURES,
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

    #[test]
    fn timer_observation_distinguishes_deferred_and_capacity_limited_frames() {
        assert_eq!(
            timer_frame_status(0, false, true),
            GpuTimerFrameStatus::NoPasses
        );
        assert_eq!(
            timer_frame_status(2, false, false),
            GpuTimerFrameStatus::Deferred
        );
        assert_eq!(
            timer_frame_status(2, true, true),
            GpuTimerFrameStatus::CapacityExhausted
        );
        assert_eq!(
            timer_frame_status(2, false, true),
            GpuTimerFrameStatus::Pending
        );
    }

    #[test]
    fn timer_collector_only_drains_results_after_the_readback_owner_polls() {
        let source = include_str!("gpu_pass_timer.rs")
            .split("\n#[cfg(test)]")
            .next()
            .unwrap_or_default();

        assert!(source.contains("pub fn try_collect(&mut self)"));
        assert!(!source.contains("readback_queue.poll_completed"));
    }

    #[test]
    fn product_timer_constructor_does_not_receive_queue_authority() {
        let source = include_str!("gpu_pass_timer.rs");
        let product_constructor = source
            .split("pub fn try_new_product(")
            .nth(1)
            .and_then(|source| source.split("pub fn begin_frame").next())
            .expect("product timer constructor");

        assert!(!product_constructor.contains("wgpu::Queue"));
        assert!(!product_constructor.contains("get_timestamp_period"));
    }
}
