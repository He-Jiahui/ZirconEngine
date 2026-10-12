use zircon_runtime_interface::ui::{binding::UiEventKind, component::UiValue, layout::UiSize};

use super::*;

#[test]
fn folder_asset_and_import_actions_share_one_projection() {
    let mut bridge = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(900.0, 620.0))
        .expect("workbench bridge should build");

    assert!(bridge.control_bool("WorkbenchAssetsForestRow", "selected"));
    assert!(bridge.control_bool("WorkbenchAssetsTableRow01", "selected"));
    bridge
        .dispatch_control_state("WorkbenchAssetsMaterialRow", UiEventKind::Click)
        .expect("materials folder should dispatch")
        .expect("materials folder should bind");
    assert!(bridge.control_bool("WorkbenchAssetsMaterialRow", "selected"));
    assert!(bridge.control_bool("WorkbenchAssetsTableRow01", "selected"));
    assert_eq!(
        Some("Content/Materials".to_string()),
        bridge.control_string("WorkbenchAssetsCenterTitle", "text")
    );
    assert_eq!(
        Some("Material Instance".to_string()),
        bridge.control_string("WorkbenchAssetsTypeField", "value")
    );

    bridge
        .dispatch_control_state("WorkbenchAssetsTableRow02", UiEventKind::Click)
        .expect("second material should dispatch")
        .expect("second material should bind");
    assert_eq!(
        Some("/Game/Materials/M_Metal_Brushed".to_string()),
        bridge.control_string("WorkbenchAssetsPathField", "value")
    );
    for (control_id, value) in [
        ("WorkbenchAssetsTypeField", "Custom Material"),
        ("WorkbenchAssetsPathField", "/Game/Custom/M_Custom"),
        ("WorkbenchAssetsOwnerField", "Custom Importer"),
    ] {
        bridge
            .mutate_control_property(control_id, "value", UiValue::String(value.to_string()))
            .expect("asset metadata should edit");
    }
    bridge
        .dispatch_control_state("WorkbenchAssetsImportButton", UiEventKind::Click)
        .expect("asset import should dispatch")
        .expect("asset import should bind");
    assert_eq!(
        Some("Import: M_Custom   Custom Material / Custom Importer".to_string()),
        bridge.control_string("WorkbenchAssetsOutputRow", "text")
    );
}
