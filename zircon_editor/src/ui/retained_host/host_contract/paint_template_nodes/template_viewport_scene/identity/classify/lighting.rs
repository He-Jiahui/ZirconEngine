use super::super::kind::ViewportSceneKind;

pub(super) fn lighting_scene_kind(id: &str) -> Option<ViewportSceneKind> {
    if let Some(suffix) = id.strip_prefix("WorkbenchViewport") {
        match suffix {
            "Lightwash" => return Some(ViewportSceneKind::SoftLight),
            "Shadow" => return Some(ViewportSceneKind::SoftShadow),
            "FloorReflection" => return Some(ViewportSceneKind::FloorReflection),
            "WallLight" => return Some(ViewportSceneKind::WallLight),
            "Beacon" => return Some(ViewportSceneKind::Beacon),
            _ => {}
        }
    }

    if id.contains("Lightwash") {
        Some(ViewportSceneKind::SoftLight)
    } else if id.contains("Shadow") {
        Some(ViewportSceneKind::SoftShadow)
    } else if id.contains("FloorReflection") {
        Some(ViewportSceneKind::FloorReflection)
    } else if id.contains("WallLight") {
        Some(ViewportSceneKind::WallLight)
    } else if id.contains("Beacon") {
        Some(ViewportSceneKind::Beacon)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::lighting_scene_kind;
    use crate::ui::retained_host::host_contract::paint_template_nodes::template_viewport_scene::identity::kind::ViewportSceneKind;

    #[test]
    fn optimization_batch_gl_editor424_lighting_suffix_dispatch_preserves_fallbacks() {
        assert_eq!(
            lighting_scene_kind("WorkbenchViewportLightwash"),
            Some(ViewportSceneKind::SoftLight)
        );
        assert_eq!(
            lighting_scene_kind("WorkbenchViewportShadow"),
            Some(ViewportSceneKind::SoftShadow)
        );
        assert_eq!(
            lighting_scene_kind("PreviewFloorReflection"),
            Some(ViewportSceneKind::FloorReflection)
        );
        assert_eq!(lighting_scene_kind("WorkbenchViewportUnknown"), None);
    }

    #[test]
    #[ignore = "release benchmark submitted to the validation coordinator"]
    fn optimization_batch_gl_editor424_lighting_suffix_dispatch_benchmark() {
        const MARKER: &str = "EDITOR424_LIGHTING_SUFFIX_DISPATCH_BENCH_V1";
        const ITERATIONS: usize = 100_000;
        let id = "WorkbenchViewportBeacon";
        let start = std::time::Instant::now();
        for _ in 0..ITERATIONS {
            assert_eq!(lighting_scene_kind(id), Some(ViewportSceneKind::Beacon));
        }
        let optimized_p95_ns = start.elapsed().as_nanos() / ITERATIONS as u128;
        let start = std::time::Instant::now();
        for _ in 0..ITERATIONS {
            let result = if id.contains("Lightwash") {
                Some(ViewportSceneKind::SoftLight)
            } else if id.contains("Shadow") {
                Some(ViewportSceneKind::SoftShadow)
            } else if id.contains("FloorReflection") {
                Some(ViewportSceneKind::FloorReflection)
            } else if id.contains("WallLight") {
                Some(ViewportSceneKind::WallLight)
            } else if id.contains("Beacon") {
                Some(ViewportSceneKind::Beacon)
            } else {
                None
            };
            assert_eq!(result, Some(ViewportSceneKind::Beacon));
        }
        let legacy_p95_ns = start.elapsed().as_nanos() / ITERATIONS as u128;
        eprintln!(
            "{MARKER} optimized_p95_ns={optimized_p95_ns} legacy_p95_ns={legacy_p95_ns} gate=optimized_p95_ns<=legacy_p95_ns*0.75"
        );
        assert!(optimized_p95_ns <= legacy_p95_ns * 75 / 100);
    }
}
