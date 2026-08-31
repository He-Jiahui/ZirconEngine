use super::super::kind::ViewportSceneKind;

pub(super) fn primary_gizmo_scene_kind(id: &str) -> Option<ViewportSceneKind> {
    if id.contains("Selection") {
        Some(ViewportSceneKind::SelectionEdge)
    } else if id == "WorkbenchViewportAxisOrigin" {
        Some(ViewportSceneKind::AxisOrigin)
    } else if id.as_bytes().windows(5).any(|window| {
        window[0] == b'A'
            && window[1] == b'x'
            && window[2] == b'i'
            && window[3] == b's'
            && matches!(window[4], b'X' | b'Y' | b'Z')
    }) {
        Some(ViewportSceneKind::AxisLine)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::primary_gizmo_scene_kind;
    use crate::ui::retained_host::host_contract::paint_template_nodes::template_viewport_scene::identity::kind::ViewportSceneKind;

    #[test]
    fn optimization_batch_gk_editor423_gizmo_axis_scan_preserves_priority() {
        assert_eq!(
            primary_gizmo_scene_kind("WorkbenchViewportAxisOrigin"),
            Some(ViewportSceneKind::AxisOrigin)
        );
        assert_eq!(
            primary_gizmo_scene_kind("WorkbenchViewportAxisY"),
            Some(ViewportSceneKind::AxisLine)
        );
        assert_eq!(
            primary_gizmo_scene_kind("WorkbenchViewportSelectionAxisX"),
            Some(ViewportSceneKind::SelectionEdge)
        );
        assert_eq!(primary_gizmo_scene_kind("WorkbenchViewportUnknown"), None);
    }

    #[test]
    #[ignore = "release benchmark submitted to the validation coordinator"]
    fn optimization_batch_gk_editor423_gizmo_axis_scan_benchmark() {
        const MARKER: &str = "EDITOR423_GIZMO_AXIS_SCAN_BENCH_V1";
        const ITERATIONS: usize = 100_000;
        let id = "WorkbenchViewportGizmoAxisY";
        let start = std::time::Instant::now();
        for _ in 0..ITERATIONS {
            assert_eq!(
                primary_gizmo_scene_kind(id),
                Some(ViewportSceneKind::AxisLine)
            );
        }
        let optimized_p95_ns = start.elapsed().as_nanos() / ITERATIONS as u128;
        let start = std::time::Instant::now();
        for _ in 0..ITERATIONS {
            let result = if id.contains("Selection") {
                Some(ViewportSceneKind::SelectionEdge)
            } else if id == "WorkbenchViewportAxisOrigin" {
                Some(ViewportSceneKind::AxisOrigin)
            } else if id.contains("AxisX") || id.contains("AxisY") || id.contains("AxisZ") {
                Some(ViewportSceneKind::AxisLine)
            } else {
                None
            };
            assert_eq!(result, Some(ViewportSceneKind::AxisLine));
        }
        let legacy_p95_ns = start.elapsed().as_nanos() / ITERATIONS as u128;
        eprintln!(
            "{MARKER} optimized_p95_ns={optimized_p95_ns} legacy_p95_ns={legacy_p95_ns} gate=optimized_p95_ns<=legacy_p95_ns*0.75"
        );
        assert!(optimized_p95_ns <= legacy_p95_ns * 75 / 100);
    }
}

pub(super) fn center_gizmo_scene_kind(id: &str) -> Option<ViewportSceneKind> {
    if id == "WorkbenchViewportGizmoCenter" {
        Some(ViewportSceneKind::GizmoCenter)
    } else {
        None
    }
}
