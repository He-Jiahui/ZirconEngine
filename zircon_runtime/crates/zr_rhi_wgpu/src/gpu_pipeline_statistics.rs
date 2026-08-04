//! Shared WGPU pipeline-statistics query lifecycle for diagnostic render passes.

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use super::gpu_readback_queue::{GpuReadbackQueue, ReadbackCallback};

pub const DEFAULT_GPU_PIPELINE_STATISTICS_MAX_SCOPES: u32 = 64;
pub const GPU_PIPELINE_STATISTICS_REQUIRED_FEATURES: wgpu::Features =
    wgpu::Features::PIPELINE_STATISTICS_QUERY;

const PIPELINE_STATISTICS_TYPES: wgpu::PipelineStatisticsTypes =
    wgpu::PipelineStatisticsTypes::VERTEX_SHADER_INVOCATIONS
        .union(wgpu::PipelineStatisticsTypes::CLIPPER_INVOCATIONS)
        .union(wgpu::PipelineStatisticsTypes::CLIPPER_PRIMITIVES_OUT)
        .union(wgpu::PipelineStatisticsTypes::FRAGMENT_SHADER_INVOCATIONS)
        .union(wgpu::PipelineStatisticsTypes::COMPUTE_SHADER_INVOCATIONS);
const PIPELINE_STATISTIC_VALUES_PER_SCOPE: u32 = 5;
const PIPELINE_STATISTIC_SIZE_BYTES: u64 = size_of::<u64>() as u64;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct GpuPipelineStatistics {
    pub vertex_shader_invocations: u64,
    pub clipper_invocations: u64,
    pub clipper_primitives_out: u64,
    pub fragment_shader_invocations: u64,
    pub compute_shader_invocations: u64,
}

impl GpuPipelineStatistics {
    fn saturating_add_assign(&mut self, other: Self) {
        self.vertex_shader_invocations = self
            .vertex_shader_invocations
            .saturating_add(other.vertex_shader_invocations);
        self.clipper_invocations = self
            .clipper_invocations
            .saturating_add(other.clipper_invocations);
        self.clipper_primitives_out = self
            .clipper_primitives_out
            .saturating_add(other.clipper_primitives_out);
        self.fragment_shader_invocations = self
            .fragment_shader_invocations
            .saturating_add(other.fragment_shader_invocations);
        self.compute_shader_invocations = self
            .compute_shader_invocations
            .saturating_add(other.compute_shader_invocations);
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GpuPassPipelineStatistics {
    pub pass_name: String,
    pub statistics: GpuPipelineStatistics,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GpuPipelineStatisticsFrameResult {
    pub frame_generation: u64,
    pub pass_statistics: Vec<GpuPassPipelineStatistics>,
}

/// Records one pipeline-statistics query per physical GPU pass. It is diagnostic-only: callers
/// must opt in after a device has negotiated `PIPELINE_STATISTICS_QUERY`.
pub struct GpuPipelineStatisticsTimer {
    query_set: wgpu::QuerySet,
    resolve_buffer: wgpu::Buffer,
    max_query_slots: u32,
    active_frame: Option<ActivePipelineStatisticsFrame>,
    completed_frames: Arc<Mutex<VecDeque<GpuPipelineStatisticsFrameResult>>>,
}

impl GpuPipelineStatisticsTimer {
    pub fn try_new(device: &wgpu::Device, max_scopes: u32) -> Option<Self> {
        if !gpu_pipeline_statistics_supported(device.features()) || max_scopes == 0 {
            return None;
        }
        let max_query_slots = max_scopes.checked_mul(PIPELINE_STATISTIC_VALUES_PER_SCOPE)?;
        let buffer_size = u64::from(max_query_slots).checked_mul(PIPELINE_STATISTIC_SIZE_BYTES)?;
        let query_set = device.create_query_set(&wgpu::QuerySetDescriptor {
            label: Some("zircon-gpu-pipeline-statistics"),
            ty: wgpu::QueryType::PipelineStatistics(PIPELINE_STATISTICS_TYPES),
            count: max_query_slots,
        });
        let resolve_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("zircon-gpu-pipeline-statistics-resolve"),
            size: buffer_size,
            usage: wgpu::BufferUsages::QUERY_RESOLVE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        Some(Self {
            query_set,
            resolve_buffer,
            max_query_slots,
            active_frame: None,
            completed_frames: Arc::new(Mutex::new(VecDeque::new())),
        })
    }

    pub fn begin_frame(&mut self, frame_generation: u64) {
        self.active_frame = Some(ActivePipelineStatisticsFrame {
            frame_generation,
            query_count: 0,
            pass_names: Vec::with_capacity(
                (self.max_query_slots / PIPELINE_STATISTIC_VALUES_PER_SCOPE) as usize,
            ),
        });
    }

    pub fn reserve_pass(&mut self, pass_name: &str) -> Option<GpuPipelineStatisticsScope> {
        let active = self.active_frame.as_mut()?;
        let next_query_count = active
            .query_count
            .checked_add(PIPELINE_STATISTIC_VALUES_PER_SCOPE)?;
        if next_query_count > self.max_query_slots {
            return None;
        }
        let query_index = active.query_count;
        active.query_count = next_query_count;
        active.pass_names.push(pass_name.to_string());
        Some(GpuPipelineStatisticsScope {
            query_set: self.query_set.clone(),
            query_index,
        })
    }

    pub fn resolve_and_request(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        readback_queue: &mut GpuReadbackQueue,
    ) {
        let Some(mut active) = self.active_frame.take() else {
            return;
        };
        if active.query_count == 0 {
            return;
        }
        let resolved_bytes = u64::from(active.query_count) * PIPELINE_STATISTIC_SIZE_BYTES;
        encoder.resolve_query_set(
            &self.query_set,
            0..active.query_count,
            &self.resolve_buffer,
            0,
        );

        let frame_generation = active.frame_generation;
        let query_count = active.query_count;
        let pass_names = std::mem::take(&mut active.pass_names);
        let completed_frames = Arc::clone(&self.completed_frames);
        let callback: ReadbackCallback = Box::new(move |bytes| {
            let Ok(bytes) = bytes else {
                return;
            };
            let Some(result) =
                decode_pipeline_statistics_frame(bytes, frame_generation, pass_names, query_count)
            else {
                return;
            };
            let mut completed = completed_frames
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            insert_completed_frame_in_order(&mut completed, result);
        });
        let _ = readback_queue.request_readback_external(
            "zircon-gpu-pipeline-statistics",
            &self.resolve_buffer,
            0..resolved_bytes,
            callback,
        );
    }

    pub fn try_collect(
        &mut self,
        device: &wgpu::Device,
        readback_queue: &mut GpuReadbackQueue,
    ) -> Option<GpuPipelineStatisticsFrameResult> {
        let _ = readback_queue.poll_completed(device);
        let mut completed = self
            .completed_frames
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        completed.pop_front()
    }
}

#[derive(Clone, Debug)]
pub struct GpuPipelineStatisticsScope {
    query_set: wgpu::QuerySet,
    query_index: u32,
}

impl GpuPipelineStatisticsScope {
    pub fn begin_compute(&self, pass: &mut wgpu::ComputePass<'_>) {
        pass.begin_pipeline_statistics_query(&self.query_set, self.query_index);
    }

    pub fn end_compute(&self, pass: &mut wgpu::ComputePass<'_>) {
        pass.end_pipeline_statistics_query();
    }

    pub fn begin_render(&self, pass: &mut wgpu::RenderPass<'_>) {
        pass.begin_pipeline_statistics_query(&self.query_set, self.query_index);
    }

    pub fn end_render(&self, pass: &mut wgpu::RenderPass<'_>) {
        pass.end_pipeline_statistics_query();
    }
}

struct ActivePipelineStatisticsFrame {
    frame_generation: u64,
    query_count: u32,
    pass_names: Vec<String>,
}

fn gpu_pipeline_statistics_supported(features: wgpu::Features) -> bool {
    features.contains(GPU_PIPELINE_STATISTICS_REQUIRED_FEATURES)
}

fn decode_pipeline_statistics_frame(
    bytes: &[u8],
    frame_generation: u64,
    pass_names: Vec<String>,
    query_count: u32,
) -> Option<GpuPipelineStatisticsFrameResult> {
    if query_count % PIPELINE_STATISTIC_VALUES_PER_SCOPE != 0
        || pass_names.len() != (query_count / PIPELINE_STATISTIC_VALUES_PER_SCOPE) as usize
    {
        return None;
    }
    let mut pass_statistics = Vec::<GpuPassPipelineStatistics>::with_capacity(pass_names.len());
    for (scope_index, pass_name) in pass_names.into_iter().enumerate() {
        let query_index = scope_index.checked_mul(PIPELINE_STATISTIC_VALUES_PER_SCOPE as usize)?;
        let statistics = GpuPipelineStatistics {
            vertex_shader_invocations: decode_query_value(bytes, query_index)?,
            clipper_invocations: decode_query_value(bytes, query_index + 1)?,
            clipper_primitives_out: decode_query_value(bytes, query_index + 2)?,
            fragment_shader_invocations: decode_query_value(bytes, query_index + 3)?,
            compute_shader_invocations: decode_query_value(bytes, query_index + 4)?,
        };
        if let Some(existing) = pass_statistics
            .iter_mut()
            .find(|existing| existing.pass_name == pass_name)
        {
            existing.statistics.saturating_add_assign(statistics);
        } else {
            pass_statistics.push(GpuPassPipelineStatistics {
                pass_name,
                statistics,
            });
        }
    }
    Some(GpuPipelineStatisticsFrameResult {
        frame_generation,
        pass_statistics,
    })
}

fn decode_query_value(bytes: &[u8], index: usize) -> Option<u64> {
    let offset = index.checked_mul(PIPELINE_STATISTIC_SIZE_BYTES as usize)?;
    Some(u64::from_le_bytes(
        bytes
            .get(offset..offset + PIPELINE_STATISTIC_SIZE_BYTES as usize)?
            .try_into()
            .ok()?,
    ))
}

fn insert_completed_frame_in_order(
    completed_frames: &mut VecDeque<GpuPipelineStatisticsFrameResult>,
    result: GpuPipelineStatisticsFrameResult,
) {
    let insertion_index = completed_frames
        .iter()
        .position(|completed| completed.frame_generation > result.frame_generation)
        .unwrap_or(completed_frames.len());
    completed_frames.insert(insertion_index, result);
}

#[cfg(test)]
mod tests {
    use super::{
        decode_pipeline_statistics_frame, gpu_pipeline_statistics_supported, GpuPipelineStatistics,
        GPU_PIPELINE_STATISTICS_REQUIRED_FEATURES,
    };

    #[test]
    fn pipeline_statistics_capability_requires_the_negotiated_wgpu_feature() {
        assert!(gpu_pipeline_statistics_supported(
            GPU_PIPELINE_STATISTICS_REQUIRED_FEATURES
        ));
        assert!(!gpu_pipeline_statistics_supported(wgpu::Features::empty()));
    }

    #[test]
    fn pipeline_statistics_decode_aggregates_multiple_physical_scopes_for_one_graph_pass() {
        let values = [
            1_u64, 2, 3, 4, 5, // HZB mip zero
            10, 20, 30, 40, 50, // HZB mip one
            7, 8, 9, 10, 11, // unrelated pass
        ];
        let mut bytes = Vec::with_capacity(values.len() * size_of::<u64>());
        for value in values {
            bytes.extend_from_slice(&value.to_le_bytes());
        }

        let result = decode_pipeline_statistics_frame(
            &bytes,
            42,
            vec![
                "hzb.build".to_string(),
                "hzb.build".to_string(),
                "ui".to_string(),
            ],
            15,
        )
        .expect("five query values per scope should decode");

        assert_eq!(result.frame_generation, 42);
        assert_eq!(result.pass_statistics.len(), 2);
        assert_eq!(result.pass_statistics[0].pass_name, "hzb.build");
        assert_eq!(
            result.pass_statistics[0].statistics,
            GpuPipelineStatistics {
                vertex_shader_invocations: 11,
                clipper_invocations: 22,
                clipper_primitives_out: 33,
                fragment_shader_invocations: 44,
                compute_shader_invocations: 55,
            }
        );
        assert_eq!(result.pass_statistics[1].pass_name, "ui");
        assert_eq!(
            result.pass_statistics[1]
                .statistics
                .compute_shader_invocations,
            11
        );
    }

    #[test]
    fn pipeline_statistics_decode_rejects_incomplete_query_slots() {
        assert!(
            decode_pipeline_statistics_frame(&[], 1, vec!["hzb.build".to_string()], 4).is_none()
        );
    }
}
