use crate::{
    asset::{AssetEvent, Handle, SceneAsset},
    core::resource::ResourceId,
    scene::{DynamicSceneError, World},
};

use super::super::result::DynamicSceneAssetReloadResult;
use super::DynamicSceneAssetReloadReadyReport;

fn failed_result(label: &str) -> DynamicSceneAssetReloadResult {
    DynamicSceneAssetReloadResult::new(
        AssetEvent::Modified {
            handle: Handle::<SceneAsset>::new(ResourceId::from_stable_label(label)),
            locator: None,
            revision: 1,
        },
        Err(DynamicSceneError::Parse {
            reason: "bounded apply fixture".to_string(),
        }),
    )
}

#[test]
fn dynamic_scene_asset_reload_apply_bytes_are_cumulative_within_one_tick() {
    let first = failed_result("apply-budget-first");
    let second = failed_result("apply-budget-second");
    let one_result_budget = first.estimated_bytes();
    let ready = DynamicSceneAssetReloadReadyReport {
        ready: vec![first, second],
        ..DynamicSceneAssetReloadReadyReport::default()
    };

    let (report, deferred) = ready.spawn_ready_into_budgeted(
        &mut World::empty(),
        usize::MAX,
        one_result_budget,
        std::time::Duration::MAX,
    );

    assert_eq!(report.failed_count(), 1);
    assert_eq!(report.applied_bytes, one_result_budget);
    assert_eq!(deferred.len(), 1);
    assert!(report.apply_budget_exhausted);
}
