use super::*;

#[test]
fn ui_asset_direct_references_include_collected_resource_dependencies() {
    let layout = UiLayoutAsset::from_toml_str(RESOURCE_REFERENCE_UI_TOML).unwrap();
    let mut references = ui_asset_references(&layout.document)
        .into_iter()
        .map(|reference| reference.locator.to_string())
        .collect::<Vec<_>>();

    references.sort();

    assert_eq!(
        references,
        vec![
            "res://fonts/inter.font.toml",
            "res://fonts/system.ttf",
            "res://textures/logo.png",
            "res://textures/root-icon.png",
            "res://textures/theme-bg.png",
            "res://ui/common/button.ui.toml",
            "res://ui/theme/editor.ui.toml",
        ]
    );
}

#[test]
fn ui_asset_direct_references_deduplicate_imported_and_resource_locators() {
    let layout = UiLayoutAsset::from_toml_str(RESOURCE_REFERENCE_UI_TOML).unwrap();
    let references = ui_asset_references(&layout.document)
        .into_iter()
        .map(|reference| reference.locator.to_string())
        .collect::<Vec<_>>();

    assert_eq!(
        references
            .iter()
            .filter(|locator| locator.as_str() == "res://textures/logo.png")
            .count(),
        1
    );
    assert_eq!(
        references
            .iter()
            .filter(|locator| locator.as_str() == "res://ui/common/button.ui.toml")
            .count(),
        1
    );
}

#[test]
fn ui_v2_asset_direct_references_include_imports_and_resources() {
    let view = UiV2ViewAsset::from_toml_str(V2_VIEW_UI_TOML).unwrap();
    let mut references = ui_v2_asset_references(&view.document)
        .into_iter()
        .map(|reference| reference.locator.to_string())
        .collect::<Vec<_>>();

    references.sort();

    assert_eq!(
        references,
        vec![
            "res://fonts/inter.font.toml",
            "res://fonts/system.ttf",
            "res://ui/common/button.v2.ui.toml",
            "res://ui/theme/editor_material.v2.ui.toml",
        ]
    );
}
