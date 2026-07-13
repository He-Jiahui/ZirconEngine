use std::collections::VecDeque;
use std::sync::mpsc::{self, Receiver, TryRecvError};

const TIMESTAMP_QUERY_COUNT: u32 = 2;
const TIMESTAMP_BYTES: u64 = TIMESTAMP_QUERY_COUNT as u64 * size_of::<u64>() as u64;

#[derive(Clone, Debug, PartialEq)]
pub struct RealtimeIblGpuTimingReport {
    pub frame_number: u64,
    pub logical_state: u8,
    pub full_update: bool,
    pub operation_label: String,
    pub pass_count: usize,
    pub dispatch_count: usize,
    pub elapsed_gpu_nanoseconds: f64,
}

#[derive(Clone, Debug)]
pub(in crate::graphics) struct RealtimeIblGpuTimestampReadback {
    buffer: wgpu::Buffer,
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
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
    ) -> RealtimeIblGpuTimestampReadback {
        encoder.write_timestamp(&self.query_set, 1);
        encoder.resolve_query_set(
            &self.query_set,
            0..TIMESTAMP_QUERY_COUNT,
            &self.resolve_buffer,
            0,
        );
        let readback = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("zircon-realtime-ibl-timestamp-readback"),
            size: TIMESTAMP_BYTES,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        encoder.copy_buffer_to_buffer(&self.resolve_buffer, 0, &readback, 0, TIMESTAMP_BYTES);
        RealtimeIblGpuTimestampReadback { buffer: readback }
    }
}

pub(in crate::graphics) struct RealtimeIblGpuTimestampCollector {
    supported: bool,
    pending: VecDeque<PendingTimestampReadback>,
    completed: VecDeque<RealtimeIblGpuTimingReport>,
}

impl RealtimeIblGpuTimestampCollector {
    pub(in crate::graphics) fn new(device: &wgpu::Device) -> Self {
        Self {
            supported: timestamp_queries_supported(device),
            pending: VecDeque::new(),
            completed: VecDeque::new(),
        }
    }

    pub(in crate::graphics) fn is_supported(&self) -> bool {
        self.supported
    }

    pub(in crate::graphics) fn begin_readback(
        &mut self,
        readback: RealtimeIblGpuTimestampReadback,
        metadata: RealtimeIblGpuTimingMetadata,
        timestamp_period_nanoseconds: f32,
    ) {
        let (sender, receiver) = mpsc::channel();
        readback
            .buffer
            .map_async(wgpu::MapMode::Read, .., move |result| {
                let _ = sender.send(result);
            });
        self.pending.push_back(PendingTimestampReadback {
            readback,
            receiver,
            metadata,
            timestamp_period_nanoseconds,
        });
    }

    pub(in crate::graphics) fn poll(&mut self, device: &wgpu::Device, wait: bool) {
        let poll_type = if wait {
            wgpu::PollType::wait_indefinitely()
        } else {
            wgpu::PollType::Poll
        };
        let _ = device.poll(poll_type);
        self.collect_ready();
    }

    pub(in crate::graphics) fn take_completed(&mut self) -> Vec<RealtimeIblGpuTimingReport> {
        self.completed.drain(..).collect()
    }

    fn collect_ready(&mut self) {
        let mut still_pending = VecDeque::with_capacity(self.pending.len());
        while let Some(pending) = self.pending.pop_front() {
            match pending.receiver.try_recv() {
                Ok(Ok(())) => {
                    if let Some(report) = pending.finish() {
                        self.completed.push_back(report);
                    }
                }
                Ok(Err(_)) | Err(TryRecvError::Disconnected) => {}
                Err(TryRecvError::Empty) => still_pending.push_back(pending),
            }
        }
        self.pending = still_pending;
    }
}

#[derive(Clone, Debug)]
pub(in crate::graphics) struct RealtimeIblGpuTimingMetadata {
    pub frame_number: u64,
    pub logical_state: u8,
    pub full_update: bool,
    pub operation_label: String,
    pub pass_count: usize,
    pub dispatch_count: usize,
}

struct PendingTimestampReadback {
    readback: RealtimeIblGpuTimestampReadback,
    receiver: Receiver<Result<(), wgpu::BufferAsyncError>>,
    metadata: RealtimeIblGpuTimingMetadata,
    timestamp_period_nanoseconds: f32,
}

impl PendingTimestampReadback {
    fn finish(self) -> Option<RealtimeIblGpuTimingReport> {
        let mapped = self.readback.buffer.get_mapped_range(..);
        let timestamps = decode_timestamp_pair(&mapped);
        drop(mapped);
        self.readback.buffer.unmap();
        let timestamps = timestamps?;
        Some(
            self.metadata
                .into_report(timestamps, self.timestamp_period_nanoseconds),
        )
    }
}

impl RealtimeIblGpuTimingMetadata {
    fn into_report(
        self,
        timestamps: [u64; 2],
        timestamp_period_nanoseconds: f32,
    ) -> RealtimeIblGpuTimingReport {
        RealtimeIblGpuTimingReport {
            frame_number: self.frame_number,
            logical_state: self.logical_state,
            full_update: self.full_update,
            operation_label: self.operation_label,
            pass_count: self.pass_count,
            dispatch_count: self.dispatch_count,
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
}
