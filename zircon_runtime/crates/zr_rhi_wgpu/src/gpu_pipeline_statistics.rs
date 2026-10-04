//! Shared WGPU pipeline-statistics query lifecycle for diagnostic render passes.

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use super::{GpuDiagnosticQueryFramePlan, WgpuDiagnosticQueryDelivery};
use zr_rhi::{DiagnosticQueryPlan, DiagnosticReadbackTerminal, PipelineStatisticsScope};

pub const DEFAULT_GPU_PIPELINE_STATISTICS_MAX_SCOPES: u32 = 64;
pub const GPU_PIPELINE_STATISTICS_REQUIRED_FEATURES: wgpu::Features =
    wgpu::Features::PIPELINE_STATISTICS_QUERY;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
/// 五类累计计数是一个查询结果的五个数据 lane，不是五个查询编号。
pub struct GpuPipelineStatistics {
    pub vertex_shader_invocations: u64,
    pub clipper_invocations: u64,
    pub clipper_primitives_out: u64,
    pub fragment_shader_invocations: u64,
    pub compute_shader_invocations: u64,
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
    max_query_count: u32,
    active_frame: Option<ActivePipelineStatisticsFrame>,
    completed_frames: Arc<Mutex<VecDeque<GpuPipelineStatisticsFrameResult>>>,
}

impl GpuPipelineStatisticsTimer {
    pub fn try_new(device: &wgpu::Device, max_scopes: u32) -> Option<Self> {
        if !gpu_pipeline_statistics_supported(device.features()) || max_scopes == 0 {
            return None;
        }
        Some(Self {
            max_query_count: max_scopes,
            active_frame: None,
            completed_frames: Arc::new(Mutex::new(VecDeque::new())),
        })
    }

    /// 借用生产所有者的同帧查询集和共享计划；scope 须在对应原生 pass 内成对开始/结束。
    pub fn begin_product_frame(
        &mut self,
        frame_generation: u64,
        plan: GpuDiagnosticQueryFramePlan,
        query_set: &wgpu::QuerySet,
    ) {
        self.active_frame = Some(ActivePipelineStatisticsFrame {
            _frame_generation: frame_generation,
            query_count: 0,
            plan,
            query_set: query_set.clone(),
        });
    }

    pub fn reserve_pass(&mut self, pass_name: &str) -> Option<GpuPipelineStatisticsScope> {
        let active = self.active_frame.as_mut()?;
        let next_query_count = active.query_count.checked_add(1)?;
        if next_query_count > self.max_query_count {
            return None;
        }
        let scope = active
            .plan
            .reserve_pipeline_statistics_scope(pass_name)
            .ok()?;
        active.query_count = next_query_count;
        Some(GpuPipelineStatisticsScope {
            query_set: active.query_set.clone(),
            scope,
        })
    }

    pub fn finish_product_frame(&mut self) {
        self.active_frame = None;
    }

    /// 完成路由传原帧快照；只接受成功终态，物理 scope 的计数按逻辑 pass 聚合。
    pub fn accept_product_query_delivery(
        &mut self,
        frame_generation: u64,
        plan: &DiagnosticQueryPlan,
        pass_names: &[String],
        delivery: &WgpuDiagnosticQueryDelivery,
    ) {
        if delivery.terminal != DiagnosticReadbackTerminal::Succeeded {
            return;
        }
        let Some(results) = delivery.pass_results.as_ref() else {
            return;
        };
        let mut statistics_passes = vec![false; plan.pass_count()];
        for scope in plan.pipeline_statistics_scopes() {
            statistics_passes[scope.pass().index()] = true;
        }
        let pass_statistics = results
            .iter()
            .filter(|result| statistics_passes.get(result.pass.index()) == Some(&true))
            .filter_map(|result| {
                pass_names
                    .get(result.pass.index())
                    .map(|pass_name| GpuPassPipelineStatistics {
                        pass_name: pass_name.clone(),
                        statistics: GpuPipelineStatistics {
                            vertex_shader_invocations: result
                                .pipeline_statistics
                                .vertex_shader_invocations,
                            clipper_invocations: result.pipeline_statistics.clipper_invocations,
                            clipper_primitives_out: result
                                .pipeline_statistics
                                .clipper_primitives_out,
                            fragment_shader_invocations: result
                                .pipeline_statistics
                                .fragment_shader_invocations,
                            compute_shader_invocations: result
                                .pipeline_statistics
                                .compute_shader_invocations,
                        },
                    })
            })
            .collect();
        let mut completed = self
            .completed_frames
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        insert_completed_frame_in_order(
            &mut completed,
            GpuPipelineStatisticsFrameResult {
                frame_generation,
                pass_statistics,
            },
        );
    }

    /// 调用者先推进唯一完成时间线，此处只消费已交付的 CPU 结果。
    pub fn try_collect(&mut self) -> Option<GpuPipelineStatisticsFrameResult> {
        let mut completed = self
            .completed_frames
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        completed.pop_front()
    }
}

/// 一个物理 pass 的查询范围；调用方须避免在同一 pass 中嵌套或重叠统计查询。
#[derive(Clone, Debug)]
pub struct GpuPipelineStatisticsScope {
    query_set: wgpu::QuerySet,
    scope: PipelineStatisticsScope,
}

impl GpuPipelineStatisticsScope {
    pub fn begin_compute(&self, pass: &mut wgpu::ComputePass<'_>) {
        pass.begin_pipeline_statistics_query(&self.query_set, self.scope.query_index());
    }

    pub fn end_compute(&self, pass: &mut wgpu::ComputePass<'_>) {
        pass.end_pipeline_statistics_query();
    }

    pub fn begin_render(&self, pass: &mut wgpu::RenderPass<'_>) {
        pass.begin_pipeline_statistics_query(&self.query_set, self.scope.query_index());
    }

    pub fn end_render(&self, pass: &mut wgpu::RenderPass<'_>) {
        pass.end_pipeline_statistics_query();
    }
}

struct ActivePipelineStatisticsFrame {
    _frame_generation: u64,
    // Native query-set indices, not the five counter values returned per query.
    query_count: u32,
    plan: GpuDiagnosticQueryFramePlan,
    query_set: wgpu::QuerySet,
}

fn gpu_pipeline_statistics_supported(features: wgpu::Features) -> bool {
    features.contains(GPU_PIPELINE_STATISTICS_REQUIRED_FEATURES)
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
#[path = "tests/gpu_pipeline_statistics.rs"]
mod tests;
