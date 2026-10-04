use super::super::super::support::{
    env_lock, BuiltinWorkbenchWindowTemplateSurfaceBridge, EditorUiBindingPayload, UiEventKind,
    UiSize,
};
use super::super::support::control_string;
use super::support::{
    assert_frame_value, FULL_WORKBENCH_HEIGHT, FULL_WORKBENCH_WIDTH, NARROW_WORKBENCH_HEIGHT,
    NARROW_WORKBENCH_WIDTH,
};
use crate::ui::binding::AssetCommand;

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

#[test]
fn full_workbench_secondary_module_commands_keep_readable_width() {
    let _guard = match env_lock().lock() {
        Ok(guard) => guard,
        Err(error) => panic!("test environment lock is poisoned: {error}"),
    };

    let bridge = match BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(
        FULL_WORKBENCH_WIDTH as f32,
        FULL_WORKBENCH_HEIGHT as f32,
    )) {
        Ok(bridge) => bridge,
        Err(error) => panic!("workbench bridge should build: {error:?}"),
    };

    let command_group = bridge
        .control_frame("WorkbenchModuleCommands")
        .expect("full toolbar should expose module commands");
    let diff = bridge
        .control_frame("WorkbenchModuleDiff")
        .expect("full toolbar should expose Diff");
    let simulate = bridge
        .control_frame("WorkbenchModuleSimulate")
        .expect("full toolbar should expose Simulate");

    assert_frame_value(
        "full module command group width",
        command_group.width,
        292.0,
    );
    assert!(
        diff.width >= 54.0,
        "Diff should keep one-line body text width, got {}",
        diff.width
    );
    assert!(
        simulate.width >= 50.0,
        "Sim should keep one-line body text width, got {}",
        simulate.width
    );
    assert_frame_value("Diff to Sim gap", simulate.x - diff.right(), 4.0);
}
