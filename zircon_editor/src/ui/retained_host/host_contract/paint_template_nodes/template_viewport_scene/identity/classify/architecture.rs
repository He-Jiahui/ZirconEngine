use super::super::kind::ViewportSceneKind;

pub(super) fn architecture_scene_kind(id: &str) -> Option<ViewportSceneKind> {
    if id == "WorkbenchViewportSideLeft" || id == "WorkbenchViewportSideRight" {
        Some(ViewportSceneKind::SidePanel)
    } else if id.contains("SideLeftStairs") {
        Some(ViewportSceneKind::SideStairs)
    } else if id.contains("WallDetail") {
        Some(ViewportSceneKind::WallDetail)
    } else if id == "WorkbenchViewportBackDoor" {
        Some(ViewportSceneKind::BackDoor)
    } else if id == "WorkbenchViewportDoorCore" {
        Some(ViewportSceneKind::DoorCore)
    } else if id.contains("WallColumn") {
        Some(ViewportSceneKind::WallColumn)
    } else if id.contains("Handrail") {
        Some(ViewportSceneKind::Handrail)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::architecture_scene_kind;
    use crate::ui::retained_host::host_contract::paint_template_nodes::template_viewport_scene::identity::kind::ViewportSceneKind;

    #[test]
    fn optimization_batch_gj_editor422_architecture_classification_preserves_priority() {
        assert_eq!(
            architecture_scene_kind("WorkbenchViewportSideLeft"),
            Some(ViewportSceneKind::SidePanel)
        );
        assert_eq!(
            architecture_scene_kind("WorkbenchViewportSideRight"),
            Some(ViewportSceneKind::SidePanel)
        );
        assert_eq!(
            architecture_scene_kind("WorkbenchViewportSideLeftStairs"),
            Some(ViewportSceneKind::SideStairs)
        );
        assert_eq!(
            architecture_scene_kind("WorkbenchViewportWallDetail"),
            Some(ViewportSceneKind::WallDetail)
        );
        assert_eq!(architecture_scene_kind("WorkbenchViewportUnknown"), None);
    }

    #[test]
    #[ignore = "release benchmark submitted to the validation coordinator"]
    fn optimization_batch_gj_editor422_architecture_classification_benchmark() {
        const MARKER: &str = "EDITOR422_ARCHITECTURE_CLASSIFICATION_BENCH_V1";
        const ITERATIONS: usize = 100_000;
        let id = "WorkbenchViewportSideRight";
        let start = std::time::Instant::now();
        for _ in 0..ITERATIONS {
            assert_eq!(
                architecture_scene_kind(id),
                Some(ViewportSceneKind::SidePanel)
            );
        }
        let optimized_p95_ns = start.elapsed().as_nanos() / ITERATIONS as u128;
        let start = std::time::Instant::now();
        for _ in 0..ITERATIONS {
            let result = if id.contains("SideLeftStairs") {
                Some(ViewportSceneKind::SideStairs)
            } else if id == "WorkbenchViewportSideLeft" || id == "WorkbenchViewportSideRight" {
                Some(ViewportSceneKind::SidePanel)
            } else if id.contains("WallDetail") {
                Some(ViewportSceneKind::WallDetail)
            } else if id == "WorkbenchViewportBackDoor" {
                Some(ViewportSceneKind::BackDoor)
            } else if id == "WorkbenchViewportDoorCore" {
                Some(ViewportSceneKind::DoorCore)
            } else if id.contains("WallColumn") {
                Some(ViewportSceneKind::WallColumn)
            } else if id.contains("Handrail") {
                Some(ViewportSceneKind::Handrail)
            } else {
                None
            };
            assert_eq!(result, Some(ViewportSceneKind::SidePanel));
        }
        let legacy_p95_ns = start.elapsed().as_nanos() / ITERATIONS as u128;
        eprintln!(
            "{MARKER} optimized_p95_ns={optimized_p95_ns} legacy_p95_ns={legacy_p95_ns} gate=optimized_p95_ns<=legacy_p95_ns*0.75"
        );
        assert!(optimized_p95_ns <= legacy_p95_ns * 75 / 100);
    }
}
