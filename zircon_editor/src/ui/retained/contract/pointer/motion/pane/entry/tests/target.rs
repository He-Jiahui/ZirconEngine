use super::retains_asset_reference_hover;
use crate::ui::retained_host::host_contract::native_pointer::routing::{
    PaneAssetReferenceList, PaneAssetSurface, PanePointerTarget,
};

#[test]
fn only_asset_reference_targets_retain_hover() {
    assert!(!retains_asset_reference_hover(
        &PanePointerTarget::AssetContent(PaneAssetSurface::Browser)
    ));
    assert!(!retains_asset_reference_hover(
        &PanePointerTarget::BrowserAssetDetails
    ));
    assert!(retains_asset_reference_hover(
        &PanePointerTarget::AssetReference(
            PaneAssetSurface::Browser,
            PaneAssetReferenceList::References,
        )
    ));
}
