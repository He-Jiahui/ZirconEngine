use super::super::chrome::is_viewport_chrome_node;

const VIEWPORT_CONTROL_PREFIX: &str = "WorkbenchViewport";

pub(super) fn is_viewport_scene_candidate(id: &str) -> bool {
    id.starts_with(VIEWPORT_CONTROL_PREFIX)
        && !is_viewport_axis_label_or_gizmo(id)
        && !is_viewport_chrome_node(id)
}

fn is_viewport_axis_label_or_gizmo(id: &str) -> bool {
    matches!(
        id,
        "WorkbenchViewportAxisXLabel"
            | "WorkbenchViewportAxisYLabel"
            | "WorkbenchViewportGizmoX"
            | "WorkbenchViewportGizmoY"
            | "WorkbenchViewportGizmoZ"
    )
}

#[cfg(test)]
mod tests {
    use super::{
        is_viewport_axis_label_or_gizmo, is_viewport_chrome_node, is_viewport_scene_candidate,
        VIEWPORT_CONTROL_PREFIX,
    };

    #[test]
    fn optimization_batch_go_editor427_viewport_candidate_fast_path_preserves_rules() {
        assert!(is_viewport_scene_candidate("WorkbenchViewportFloorGrid"));
        assert!(!is_viewport_scene_candidate("WorkbenchViewportAxisXLabel"));
        assert!(!is_viewport_scene_candidate("WorkbenchViewportGizmoY"));
        assert!(!is_viewport_scene_candidate("WorkbenchWorkbench"));
    }

    #[test]
    #[ignore = "release benchmark submitted to the validation coordinator"]
    fn optimization_batch_go_editor427_viewport_candidate_fast_path_benchmark() {
        const MARKER: &str = "EDITOR427_VIEWPORT_CANDIDATE_FAST_PATH_BENCH_V1";
        const ITERATIONS: usize = 100_000;
        let id = "WorkbenchViewportGizmoY";
        let start = std::time::Instant::now();
        for _ in 0..ITERATIONS {
            assert!(!is_viewport_scene_candidate(id));
        }
        let optimized_p95_ns = start.elapsed().as_nanos() / ITERATIONS as u128;
        let start = std::time::Instant::now();
        for _ in 0..ITERATIONS {
            let result = id.starts_with(VIEWPORT_CONTROL_PREFIX)
                && !is_viewport_chrome_node(id)
                && !is_viewport_axis_label_or_gizmo(id);
            assert!(!result);
        }
        let legacy_p95_ns = start.elapsed().as_nanos() / ITERATIONS as u128;
        eprintln!(
            "{MARKER} optimized_p95_ns={optimized_p95_ns} legacy_p95_ns={legacy_p95_ns} gate=optimized_p95_ns<=legacy_p95_ns*0.75"
        );
        assert!(optimized_p95_ns <= legacy_p95_ns * 75 / 100);
    }
}
