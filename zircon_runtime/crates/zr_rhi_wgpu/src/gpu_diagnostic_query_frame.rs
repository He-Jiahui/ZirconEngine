//! Transitional product-side query planning shared by timestamp and statistics adapters.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use zr_rhi::{
    DiagnosticQueryPlan, DiagnosticQueryPlanError, DiagnosticReadbackBudget, PassDiagnosticId,
    PipelineStatisticsScope, TimestampScope,
};

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
mod tests {
    use super::GpuDiagnosticQueryFramePlan;
    use zr_rhi::DiagnosticReadbackBudget;

    #[test]
    fn timestamp_and_statistics_scopes_share_dense_logical_pass_ids() {
        let frame = GpuDiagnosticQueryFramePlan::new(41, DiagnosticReadbackBudget::default());
        let timestamp_a = frame.reserve_timestamp_scope("hzb.build").unwrap();
        let timestamp_b = frame.reserve_timestamp_scope("hzb.build").unwrap();
        let statistics = frame
            .reserve_pipeline_statistics_scope("hzb.build")
            .unwrap();
        let ui = frame.reserve_timestamp_scope("ui").unwrap();
        let snapshot = frame.snapshot();

        assert_eq!(timestamp_a.pass(), timestamp_b.pass());
        assert_eq!(timestamp_a.pass(), statistics.pass());
        assert_ne!(timestamp_a.pass(), ui.pass());
        assert_eq!(snapshot.pass_names(), ["hzb.build", "ui"]);
        assert_eq!(snapshot.plan().pass_count(), 2);
        assert_eq!(snapshot.plan().timestamp_query_count(), 6);
        assert_eq!(snapshot.plan().pipeline_statistics_query_count(), 1);
    }
}
