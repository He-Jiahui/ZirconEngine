//! Transitional product-side query planning shared by timestamp and statistics adapters.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use zr_rhi::{
    DiagnosticQueryPlan, DiagnosticQueryPlanError, DiagnosticReadbackBudget, PassDiagnosticId,
    PipelineStatisticsScope, TimestampScope,
};

/// 时间戳与统计共用一帧计划；克隆共享预留状态，两类查询使用同一逻辑 pass 编号空间。
/// 同名 pass 聚合为同一逻辑身份，但每次预留仍对应独立物理查询 scope。
#[derive(Clone)]
pub struct GpuDiagnosticQueryFramePlan {
    inner: Arc<Mutex<GpuDiagnosticQueryFramePlanState>>,
}

impl GpuDiagnosticQueryFramePlan {
    pub fn new(query_frame_index: u64, budget: DiagnosticReadbackBudget) -> Self {
        Self {
            inner: Arc::new(Mutex::new(GpuDiagnosticQueryFramePlanState {
                plan: DiagnosticQueryPlan::for_frame(query_frame_index, budget),
                pass_ids: HashMap::new(),
                pass_names: Vec::new(),
            })),
        }
    }

    pub fn reserve_timestamp_scope(
        &self,
        pass_name: &str,
    ) -> Result<TimestampScope, DiagnosticQueryPlanError> {
        let mut state = self.lock();
        let pass = state.pass_id(pass_name)?;
        state.plan.reserve_timestamp_scope(pass)
    }

    pub fn reserve_pipeline_statistics_scope(
        &self,
        pass_name: &str,
    ) -> Result<PipelineStatisticsScope, DiagnosticQueryPlanError> {
        let mut state = self.lock();
        let pass = state.pass_id(pass_name)?;
        state.plan.reserve_pipeline_statistics_scope(pass)
    }

    /// 结束录制后取原子快照，将计划和名称表一起交给提交与结果路由，避免索引错配。
    pub fn snapshot(&self) -> GpuDiagnosticQueryFramePlanSnapshot {
        let state = self.lock();
        GpuDiagnosticQueryFramePlanSnapshot {
            plan: state.plan.clone(),
            pass_names: state.pass_names.clone(),
        }
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, GpuDiagnosticQueryFramePlanState> {
        self.inner
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

/// 一次锁内取得的计划/名称对；结果消费者应携带此对，不重新读取正在录制的名称表。
pub struct GpuDiagnosticQueryFramePlanSnapshot {
    plan: DiagnosticQueryPlan,
    pass_names: Vec<String>,
}

impl GpuDiagnosticQueryFramePlanSnapshot {
    pub fn plan(&self) -> &DiagnosticQueryPlan {
        &self.plan
    }

    pub fn pass_names(&self) -> &[String] {
        &self.pass_names
    }

    pub fn into_parts(self) -> (DiagnosticQueryPlan, Vec<String>) {
        (self.plan, self.pass_names)
    }
}

struct GpuDiagnosticQueryFramePlanState {
    plan: DiagnosticQueryPlan,
    pass_ids: HashMap<String, PassDiagnosticId>,
    pass_names: Vec<String>,
}

impl GpuDiagnosticQueryFramePlanState {
    fn pass_id(&mut self, pass_name: &str) -> Result<PassDiagnosticId, DiagnosticQueryPlanError> {
        if let Some(pass) = self.pass_ids.get(pass_name) {
            return Ok(*pass);
        }
        let pass = self.plan.register_pass()?;
        self.pass_ids.insert(pass_name.to_owned(), pass);
        self.pass_names.push(pass_name.to_owned());
        debug_assert_eq!(pass.index(), self.pass_names.len().saturating_sub(1));
        Ok(pass)
    }
}

#[cfg(test)]
#[path = "tests/gpu_diagnostic_query_frame.rs"]
mod tests;
