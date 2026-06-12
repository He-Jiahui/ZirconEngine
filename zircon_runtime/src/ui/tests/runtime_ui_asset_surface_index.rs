use crate::asset::watch::{AssetChange, AssetChangeKind};
use crate::asset::AssetUri;
use crate::core::math::UVec2;
use crate::ui::template::{UiAssetDependencyIndex, UiAssetHotReloadPlan};
use crate::ui::{RuntimeUiFixture, RuntimeUiManager};
use zircon_runtime_interface::ui::event_ui::UiTreeId;

#[test]
fn runtime_ui_manager_registers_loaded_fixture_in_asset_surface_index() {
    let mut manager = RuntimeUiManager::new(UVec2::new(960, 540));
    manager
        .load_builtin_fixture(RuntimeUiFixture::QuestLogDialog)
        .expect("quest log runtime fixture should load");

    let tree_id = UiTreeId::new(RuntimeUiFixture::QuestLogDialog.asset_id());
    let index = manager.asset_surface_index();

    assert_eq!(index.surface_count(), 1);
    assert_eq!(
        index.assets_for_surface(&tree_id),
        &[
            RuntimeUiFixture::QuestLogDialog.asset_id().to_string(),
            RuntimeUiFixture::QuestLogDialog.asset_uri().to_string(),
        ]
    );
    assert_eq!(
        index
            .surfaces_for_asset(RuntimeUiFixture::QuestLogDialog.asset_uri())
            .cloned()
            .collect::<Vec<_>>(),
        vec![tree_id]
    );
}

#[test]
fn runtime_ui_asset_surface_index_targets_loaded_fixture_for_template_reload() {
    let mut manager = RuntimeUiManager::new(UVec2::new(960, 540));
    manager
        .load_builtin_fixture(RuntimeUiFixture::QuestLogDialog)
        .expect("quest log runtime fixture should load");

    let mut dependency_index = UiAssetDependencyIndex::new();
    let report = dependency_index.apply_watch_changes(&[AssetChange::new(
        AssetChangeKind::Modified,
        uri(RuntimeUiFixture::QuestLogDialog.asset_uri()),
        None,
    )]);

    let plan = UiAssetHotReloadPlan::from_watch_report(&report);
    let targets = manager
        .asset_surface_index()
        .target_surfaces_for_plan(&plan);

    assert_eq!(
        targets.template_rebuild_surfaces,
        vec![UiTreeId::new(RuntimeUiFixture::QuestLogDialog.asset_id())]
    );
    assert!(targets.rebuild_required);
    assert!(targets.dirty.layout);
    assert!(targets.dirty.hit_test);
    assert!(targets.dirty.render);
}

fn uri(value: &str) -> AssetUri {
    AssetUri::parse(value).unwrap()
}
