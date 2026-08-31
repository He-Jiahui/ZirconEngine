use super::super::kind::ViewportSceneKind;

pub(super) fn prop_scene_kind(id: &str) -> Option<ViewportSceneKind> {
    if id == "WorkbenchViewportPropBody" {
        Some(ViewportSceneKind::PropBody)
    } else if id == "WorkbenchViewportPropTop" {
        Some(ViewportSceneKind::PropTop)
    } else if id.contains("Cargo") {
        if id.contains("Inner") {
            Some(ViewportSceneKind::CargoInner)
        } else {
            Some(ViewportSceneKind::Cargo)
        }
    } else if id.contains("Rack") {
        Some(ViewportSceneKind::Rack)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::prop_scene_kind;
    use crate::ui::retained_host::host_contract::paint_template_nodes::template_viewport_scene::identity::kind::ViewportSceneKind;

    #[test]
    fn optimization_batch_gh_editor421_prop_scene_classification_preserves_priority() {
        assert_eq!(
            prop_scene_kind("WorkbenchViewportPropBody"),
            Some(ViewportSceneKind::PropBody)
        );
        assert_eq!(
            prop_scene_kind("WorkbenchViewportCargoInner"),
            Some(ViewportSceneKind::CargoInner)
        );
        assert_eq!(
            prop_scene_kind("WorkbenchViewportCargo"),
            Some(ViewportSceneKind::Cargo)
        );
        assert_eq!(
            prop_scene_kind("WorkbenchViewportRack"),
            Some(ViewportSceneKind::Rack)
        );
        assert_eq!(prop_scene_kind("WorkbenchViewportUnknown"), None);
    }

    #[test]
    #[ignore = "release benchmark submitted to the validation coordinator"]
    fn optimization_batch_gh_editor421_prop_scene_classification_benchmark() {
        const MARKER: &str = "EDITOR421_PROP_SCENE_CLASSIFICATION_BENCH_V1";
        const ITERATIONS: usize = 100_000;
        let id = "WorkbenchViewportCargo";
        let start = std::time::Instant::now();
        for _ in 0..ITERATIONS {
            assert_eq!(prop_scene_kind(id), Some(ViewportSceneKind::Cargo));
        }
        let optimized_p95_ns = start.elapsed().as_nanos() / ITERATIONS as u128;
        let start = std::time::Instant::now();
        for _ in 0..ITERATIONS {
            let result = if id == "WorkbenchViewportPropBody" {
                Some(ViewportSceneKind::PropBody)
            } else if id == "WorkbenchViewportPropTop" {
                Some(ViewportSceneKind::PropTop)
            } else if id.contains("Cargo") && id.contains("Inner") {
                Some(ViewportSceneKind::CargoInner)
            } else if id.contains("Cargo") {
                Some(ViewportSceneKind::Cargo)
            } else if id.contains("Rack") {
                Some(ViewportSceneKind::Rack)
            } else {
                None
            };
            assert_eq!(result, Some(ViewportSceneKind::Cargo));
        }
        let legacy_p95_ns = start.elapsed().as_nanos() / ITERATIONS as u128;
        eprintln!(
            "{MARKER} optimized_p95_ns={optimized_p95_ns} legacy_p95_ns={legacy_p95_ns} gate=optimized_p95_ns<=legacy_p95_ns*0.75"
        );
        assert!(optimized_p95_ns <= legacy_p95_ns * 75 / 100);
    }
}
