use std::sync::Arc;

use zircon_runtime_interface::ui::{
    event_ui::UiTreeId,
    surface::{UiRenderExtract, UiRenderList},
};

use super::{WorldSpaceUiMergeCache, WorldSpaceUiSurfaceSubmission};

#[test]
fn stable_world_space_generation_reuses_the_merged_allocation() {
    let mut cache = WorldSpaceUiMergeCache::default();
    let base = test_extract();
    let submissions = [test_submission()];

    let first = cache
        .resolve(Some(Arc::clone(&base)), 7, &submissions)
        .unwrap();
    let second = cache
        .resolve(Some(Arc::clone(&base)), 7, &submissions)
        .unwrap();

    assert!(!Arc::ptr_eq(&base, &first));
    assert!(Arc::ptr_eq(&first, &second));
}

#[test]
fn empty_world_space_generation_preserves_the_base_allocation() {
    let mut cache = WorldSpaceUiMergeCache::default();
    let base = test_extract();

    let merged = cache.resolve(Some(Arc::clone(&base)), 0, &[]).unwrap();

    assert!(Arc::ptr_eq(&base, &merged));
}

fn test_extract() -> Arc<UiRenderExtract> {
    Arc::new(UiRenderExtract {
        tree_id: UiTreeId::new("editor.viewport.world-space-cache-test"),
        list: UiRenderList::default(),
        raster_scale: 1.0,
    })
}

fn test_submission() -> WorldSpaceUiSurfaceSubmission {
    WorldSpaceUiSurfaceSubmission {
        viewport_width: 100.0,
        viewport_height: 40.0,
        control_id: "WorldPanel".to_string(),
        ..WorldSpaceUiSurfaceSubmission::default()
    }
}
