use crate::asset::{AssetLoadState, Assets, Handle, ProjectAssetManager, TextureAsset};
use crate::core::resource::{ResourceId, ResourceManager};

#[test]
fn dangling_handle_queries_report_not_loaded_instead_of_panicking() {
    let handle = Handle::<TextureAsset>::new(ResourceId::from_stable_label(
        "runtime-04-dangling-texture-handle",
    ));
    let assets = Assets::<TextureAsset>::new(ResourceManager::new());
    let manager = ProjectAssetManager::default();

    assert_eq!(assets.load_state(handle), AssetLoadState::NotLoaded);
    assert_eq!(manager.load_state(handle), AssetLoadState::NotLoaded);
    assert!(!manager.is_loaded(handle));
}
