use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use crate::graphics::backend::RenderBackend;

const TIMESTAMP_QUERY_COUNT: u32 = 2;
const TIMESTAMP_BYTES: u64 = TIMESTAMP_QUERY_COUNT as u64 * size_of::<u64>() as u64;

#[derive(Clone, Debug, PartialEq)]
pub struct RealtimeIblGpuTimingReport {
    pub frame_number: u64,
    pub generation: u64,
    pub recipe_fingerprint: String,
    pub logical_state: u8,
    pub work_slot: String,
    pub operation_label: String,
    pub pass_count: usize,
    pub dispatch_count: usize,
    pub binding_cache_hits: usize,
    pub binding_cache_misses: usize,
    pub params_buffer_creations: usize,
    pub bind_group_creations: usize,
    pub binding_cache_resets: usize,
    pub capture_params_buffer_creations: usize,
    pub capture_bind_group_creations: usize,
    pub source_mip_params_buffer_creations: usize,
    pub source_mip_bind_group_creations: usize,
    pub scheduled_workgroups: u64,
    pub completed_workgroups: u64,
    pub terminal_reason: String,
    pub elapsed_gpu_nanoseconds: f64,
}

#[derive(Clone, Debug)]
pub(in crate::graphics) struct RealtimeIblGpuTimestampReadback {
    source: wgpu::Buffer,
}

pub(in crate::graphics) struct RealtimeIblGpuTimestampRecorder {
    query_set: wgpu::QuerySet,
    resolve_buffer: wgpu::Buffer,
}

impl RealtimeIblGpuTimestampRecorder {
    pub(in crate::graphics) fn new(device: &wgpu::Device) -> Option<Self> {
        timestamp_queries_supported(device).then(|| Self {
            query_set: device.create_query_set(&wgpu::QuerySetDescriptor {
                label: Some("zircon-realtime-ibl-timestamps"),
                ty: wgpu::QueryType::Timestamp,
                count: TIMESTAMP_QUERY_COUNT,
            }),
            resolve_buffer: device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("zircon-realtime-ibl-timestamp-resolve"),
                size: TIMESTAMP_BYTES,
                usage: wgpu::BufferUsages::QUERY_RESOLVE | wgpu::BufferUsages::COPY_SRC,
                mapped_at_creation: false,
            }),
        })
    }

    pub(in crate::graphics) fn write_start(&self, encoder: &mut wgpu::CommandEncoder) {
        encoder.write_timestamp(&self.query_set, 0);
    }

    pub(in crate::graphics) fn write_end_and_resolve(
        &self,
        encoder: &mut wgpu::CommandEncoder,
    ) -> RealtimeIblGpuTimestampReadback {
        encoder.write_timestamp(&self.query_set, 1);
        encoder.resolve_query_set(
            &self.query_set,
            0..TIMESTAMP_QUERY_COUNT,
            &self.resolve_buffer,
            0,
        );
        RealtimeIblGpuTimestampReadback {
            source: self.resolve_buffer.clone(),
        }
    }
}

pub(in crate::graphics) struct RealtimeIblGpuTimestampCollector {
    supported: bool,
    completed: Arc<Mutex<VecDeque<RealtimeIblGpuTimingReport>>>,
}

impl RealtimeIblGpuTimestampCollector {
    pub(in crate::graphics) fn new(device: &wgpu::Device) -> Self {
        Self {
            supported: timestamp_queries_supported(device),
            completed: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    pub(in crate::graphics) fn is_supported(&self) -> bool {
        self.supported
    }

    pub(in crate::graphics) fn request_product_readback(
        &mut self,
        readback: &RealtimeIblGpuTimestampReadback,
        metadata: RealtimeIblGpuTimingMetadata,
        timestamp_period_nanoseconds: f32,
        backend: &RenderBackend,
    ) -> bool {
        let completed = Arc::clone(&self.completed);
        backend
            .enqueue_product_diagnostic_buffer(
                &readback.source,
                0,
                TIMESTAMP_BYTES,
                Box::new(move |result| {
                    let Ok(bytes) = result else {
                        return;
                    };
                    let Some(timestamps) = decode_timestamp_pair(&bytes) else {
                        return;
                    };
                    let report = metadata.into_report(timestamps, timestamp_period_nanoseconds);
                    completed
                        .lock()
                        .unwrap_or_else(|poisoned| poisoned.into_inner())
                        .push_back(report);
                }),
            )
            .unwrap_or(false)
    }

    pub(in crate::graphics) fn take_completed(&mut self) -> Vec<RealtimeIblGpuTimingReport> {
        self.completed
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .drain(..)
            .collect()
    }
}

#[derive(Clone, Debug)]
pub(in crate::graphics) struct RealtimeIblGpuTimingMetadata {
    pub frame_number: u64,
    pub generation: u64,
    pub recipe_fingerprint: String,
    pub logical_state: u8,
    pub work_slot: String,
    pub operation_label: String,
    pub pass_count: usize,
    pub dispatch_count: usize,
    pub binding_cache_hits: usize,
    pub binding_cache_misses: usize,
    pub params_buffer_creations: usize,
    pub bind_group_creations: usize,
    pub binding_cache_resets: usize,
    pub capture_params_buffer_creations: usize,
    pub capture_bind_group_creations: usize,
    pub source_mip_params_buffer_creations: usize,
    pub source_mip_bind_group_creations: usize,
    pub scheduled_workgroups: u64,
    pub completed_workgroups: u64,
    pub terminal_reason: String,
}

impl RealtimeIblGpuTimingMetadata {
    fn into_report(
        self,
        timestamps: [u64; 2],
        timestamp_period_nanoseconds: f32,
    ) -> RealtimeIblGpuTimingReport {
        RealtimeIblGpuTimingReport {
            frame_number: self.frame_number,
            generation: self.generation,
            recipe_fingerprint: self.recipe_fingerprint,
            logical_state: self.logical_state,
            work_slot: self.work_slot,
            operation_label: self.operation_label,
            pass_count: self.pass_count,
            dispatch_count: self.dispatch_count,
            binding_cache_hits: self.binding_cache_hits,
            binding_cache_misses: self.binding_cache_misses,
            params_buffer_creations: self.params_buffer_creations,
            bind_group_creations: self.bind_group_creations,
            binding_cache_resets: self.binding_cache_resets,
            capture_params_buffer_creations: self.capture_params_buffer_creations,
            capture_bind_group_creations: self.capture_bind_group_creations,
            source_mip_params_buffer_creations: self.source_mip_params_buffer_creations,
            source_mip_bind_group_creations: self.source_mip_bind_group_creations,
            scheduled_workgroups: self.scheduled_workgroups,
            completed_workgroups: self.completed_workgroups,
            terminal_reason: self.terminal_reason,
            elapsed_gpu_nanoseconds: elapsed_gpu_nanoseconds(
                timestamps,
                timestamp_period_nanoseconds,
            ),
        }
    }
}

fn timestamp_queries_supported(device: &wgpu::Device) -> bool {
    let required =
        wgpu::Features::TIMESTAMP_QUERY | wgpu::Features::TIMESTAMP_QUERY_INSIDE_ENCODERS;
    device.features().contains(required)
}

fn decode_timestamp_pair(bytes: &[u8]) -> Option<[u64; 2]> {
    let first = u64::from_le_bytes(bytes.get(0..8)?.try_into().ok()?);
    let second = u64::from_le_bytes(bytes.get(8..16)?.try_into().ok()?);
    Some([first, second])
}

fn elapsed_gpu_nanoseconds(timestamps: [u64; 2], timestamp_period_nanoseconds: f32) -> f64 {
    timestamps[1].saturating_sub(timestamps[0]) as f64 * timestamp_period_nanoseconds as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timestamp_pair_decodes_little_endian_words() {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&40_u64.to_le_bytes());
        bytes.extend_from_slice(&58_u64.to_le_bytes());

        assert_eq!(decode_timestamp_pair(&bytes), Some([40, 58]));
    }

    #[test]
    fn timestamp_delta_uses_queue_period_in_nanoseconds() {
        assert_eq!(elapsed_gpu_nanoseconds([100, 132], 2.5), 80.0);
    }

    #[test]
    fn timestamp_delta_saturates_invalid_reverse_order() {
        assert_eq!(elapsed_gpu_nanoseconds([132, 100], 2.5), 0.0);
    }

    #[test]
    fn gpu_timing_contract_excludes_cpu_recording_windows() {
        let source = include_str!("realtime_ibl_gpu_timestamps.rs");
        let gpu_report = source
            .split("pub struct RealtimeIblGpuTimingReport {")
            .nth(1)
            .and_then(|definition| definition.split("\n}\n\n#[derive(Clone, Debug)]").next())
            .expect("GPU timing report definition");
        let gpu_metadata = source
            .split("pub(in crate::graphics) struct RealtimeIblGpuTimingMetadata {")
            .nth(1)
            .and_then(|definition| {
                definition
                    .split("\n}\n\nimpl RealtimeIblGpuTimingMetadata")
                    .next()
            })
            .expect("GPU timing metadata definition");

        for cpu_window in [
            "command_plan_creation_micros",
            "pipeline_ensure_micros",
            "binding_creation_micros",
            "capture_binding_creation_micros",
            "source_mip_binding_creation_micros",
        ] {
            assert!(
                !gpu_report.contains(cpu_window),
                "GPU timing report must not expose CPU recording window {cpu_window}"
            );
            assert!(
                !gpu_metadata.contains(cpu_window),
                "GPU timing metadata must not transport CPU recording window {cpu_window}"
            );
        }
    }
}
