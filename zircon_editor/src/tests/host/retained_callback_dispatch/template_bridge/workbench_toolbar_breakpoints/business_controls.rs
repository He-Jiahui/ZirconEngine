use super::*;
use crate::ui::binding::{AssetCommand, EditorUiBindingPayload};
use zircon_runtime_interface::ui::binding::UiEventKind;

#[test]
fn toolbar_assets_control_opens_the_real_asset_browser() {
    let _guard = match env_lock().lock() {
        Ok(guard) => guard,
        Err(error) => panic!("test environment lock is poisoned: {error}"),
    };
    let bridge = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(
        NARROW_WORKBENCH_WIDTH as f32,
        NARROW_WORKBENCH_HEIGHT as f32,
    ))
    .expect("narrow workbench should build");

    assert!(bridge.has_control("WorkbenchToolbarAssets"));
    assert!(!bridge.has_control("WorkbenchToolbarNew"));
    assert_eq!(
        control_string(&bridge, "WorkbenchToolbarAssets", "label").as_deref(),
        Some("Assets")
    );
    assert_eq!(
        control_string(&bridge, "WorkbenchToolbarAssets", "icon").as_deref(),
        Some("zircon_editor_shell/toolbar/package.svg")
    );
    let binding = bridge
        .binding_for_control("WorkbenchToolbarAssets", UiEventKind::Click)
        .expect("Assets should expose the canonical asset-browser binding");
    assert_eq!(
        binding.payload(),
        &EditorUiBindingPayload::asset_command(AssetCommand::OpenAssetBrowser)
    );
}
