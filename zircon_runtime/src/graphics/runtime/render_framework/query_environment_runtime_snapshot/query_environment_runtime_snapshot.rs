use crate::core::framework::render::{EnvironmentRuntimeSnapshot, RenderFrameworkError};

use super::super::wgpu_render_framework::WgpuRenderFramework;

pub(in crate::graphics::runtime::render_framework) fn query_environment_runtime_snapshot(
    framework: &WgpuRenderFramework,
) -> Result<EnvironmentRuntimeSnapshot, RenderFrameworkError> {
    framework.finish_submission()?;
    let _operation_guard = framework.lock_operation();
    let state = framework.lock_state();

    Ok(EnvironmentRuntimeSnapshot::try_from_current_reports(
        state.stats.last_generation,
        &state.stats.last_frame_profile,
        state.stats.last_scene_submission_completion_report,
        state.stats.last_reflection_probe_workload,
        state.renderer.realtime_ibl_status_report(),
    )?)
}

impl WgpuRenderFramework {
    pub fn query_environment_runtime_snapshot(
        &self,
    ) -> Result<EnvironmentRuntimeSnapshot, RenderFrameworkError> {
        query_environment_runtime_snapshot(self)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn snapshot_finishes_submission_then_projects_under_one_lock_pair() {
        let source = include_str!("query_environment_runtime_snapshot.rs")
            .split_once("#[cfg(test)]")
            .map(|(production, _)| production)
            .expect("snapshot query production source");
        let finish = source
            .find("framework.finish_submission()?;")
            .expect("pending submission must finish first");
        let operation = source
            .find("framework.lock_operation()")
            .expect("query must serialize renderer access");
        let state = source
            .find("framework.lock_state()")
            .expect("query must take the state lock once");
        let projection = source
            .find("EnvironmentRuntimeSnapshot::try_from_current_reports")
            .expect("query must use the core contract projection");

        assert!(finish < operation && operation < state && state < projection);
        assert_eq!(source.matches("framework.lock_operation()").count(), 1);
        assert_eq!(source.matches("framework.lock_state()").count(), 1);
        assert!(!source.contains("query_stats("));
        assert!(!source.contains("take_realtime_ibl"));
        assert!(!source.contains("take_completed_gpu_timing_report"));
    }
}
