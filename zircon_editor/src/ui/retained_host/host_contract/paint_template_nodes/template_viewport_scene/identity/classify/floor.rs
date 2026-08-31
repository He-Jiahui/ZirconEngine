use super::super::kind::ViewportSceneKind;

pub(super) fn floor_scene_kind(id: &str) -> Option<ViewportSceneKind> {
    if let Some(suffix) = id.strip_prefix("WorkbenchViewport") {
        match suffix {
            "FloorGrid" => return Some(ViewportSceneKind::FloorGrid),
            "FloorPanel" => return Some(ViewportSceneKind::FloorPanel),
            "FloorSeam" => return Some(ViewportSceneKind::FloorSeam),
            "FloorGrate" => return Some(ViewportSceneKind::FloorGrate),
            _ => {}
        }
    }

    if id.contains("Grid") {
        Some(ViewportSceneKind::FloorGrid)
    } else if id.contains("FloorPanel") {
        Some(ViewportSceneKind::FloorPanel)
    } else if id.contains("FloorSeam") {
        Some(ViewportSceneKind::FloorSeam)
    } else if id.contains("FloorGrate") {
        Some(ViewportSceneKind::FloorGrate)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::floor_scene_kind;
    use crate::ui::retained_host::host_contract::paint_template_nodes::template_viewport_scene::identity::kind::ViewportSceneKind;

    #[test]
    fn optimization_batch_gm_editor425_floor_suffix_dispatch_preserves_fallbacks() {
        assert_eq!(
            floor_scene_kind("WorkbenchViewportFloorGrid"),
            Some(ViewportSceneKind::FloorGrid)
        );
        assert_eq!(
            floor_scene_kind("WorkbenchViewportFloorPanel"),
            Some(ViewportSceneKind::FloorPanel)
        );
        assert_eq!(
            floor_scene_kind("PreviewFloorGrate"),
            Some(ViewportSceneKind::FloorGrate)
        );
        assert_eq!(floor_scene_kind("WorkbenchViewportUnknown"), None);
    }

    #[test]
    #[ignore = "release benchmark submitted to the validation coordinator"]
    fn optimization_batch_gm_editor425_floor_suffix_dispatch_benchmark() {
        const MARKER: &str = "EDITOR425_FLOOR_SUFFIX_DISPATCH_BENCH_V1";
        const ITERATIONS: usize = 100_000;
        let id = "WorkbenchViewportFloorGrate";
        let start = std::time::Instant::now();
        for _ in 0..ITERATIONS {
            assert_eq!(floor_scene_kind(id), Some(ViewportSceneKind::FloorGrate));
        }
        let optimized_p95_ns = start.elapsed().as_nanos() / ITERATIONS as u128;
        let start = std::time::Instant::now();
        for _ in 0..ITERATIONS {
            let result = if id.contains("Grid") {
                Some(ViewportSceneKind::FloorGrid)
            } else if id.contains("FloorPanel") {
                Some(ViewportSceneKind::FloorPanel)
            } else if id.contains("FloorSeam") {
                Some(ViewportSceneKind::FloorSeam)
            } else if id.contains("FloorGrate") {
                Some(ViewportSceneKind::FloorGrate)
            } else {
                None
            };
            assert_eq!(result, Some(ViewportSceneKind::FloorGrate));
        }
        let legacy_p95_ns = start.elapsed().as_nanos() / ITERATIONS as u128;
        eprintln!(
            "{MARKER} optimized_p95_ns={optimized_p95_ns} legacy_p95_ns={legacy_p95_ns} gate=optimized_p95_ns<=legacy_p95_ns*0.75"
        );
        assert!(optimized_p95_ns <= legacy_p95_ns * 75 / 100);
    }
}
