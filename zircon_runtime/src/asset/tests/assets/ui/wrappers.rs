use super::*;

#[test]
fn ui_asset_wrappers_parse_and_validate_kind() {
    let layout = UiLayoutAsset::from_toml_str(LAYOUT_UI_TOML).unwrap();
    let widget = UiWidgetAsset::from_toml_str(WIDGET_UI_TOML).unwrap();
    let style = UiStyleAsset::from_toml_str(STYLE_UI_TOML).unwrap();

    assert_eq!(layout.document.asset.kind, UiAssetKind::Layout);
    assert_eq!(widget.document.asset.kind, UiAssetKind::Widget);
    assert_eq!(style.document.asset.kind, UiAssetKind::Style);
    assert!(UiLayoutAsset::from_toml_str(WIDGET_UI_TOML).is_err());
}

#[test]
fn ui_theme_asset_round_trips_toml_and_registers_facade_label() {
    let theme = UiThemeAsset::from_toml_str(THEME_UI_TOML).unwrap();

    assert_eq!(theme.document.id, "zircon.test.dark");
    assert_eq!(
        theme.document.palette.accent,
        UiRgbaColor::new(0.1, 0.2, 0.3, 1.0)
    );
    assert_eq!(
        theme.document.palette.surface[2],
        UiRgbaColor::from_u8(27, 31, 35, 255)
    );
    assert_eq!(<UiThemeAsset as crate::asset::Asset>::LABEL, "ui_theme");
    assert_eq!(
        <<UiThemeAsset as crate::asset::Asset>::Marker as ResourceMarker>::KIND,
        ResourceKind::UiStyle
    );

    let round_trip = UiThemeAsset::from_toml_str(&theme.to_toml_string().unwrap()).unwrap();
    assert_eq!(round_trip, theme);
}

#[test]
fn ui_icon_asset_round_trips_toml_and_registers_facade_label() {
    let icon = UiIconAsset::from_toml_str(ICON_UI_TOML).unwrap();

    assert_eq!(icon.semantic_id, "icons/run");
    assert_eq!(icon.default_size, 18.0);
    assert_eq!(
        icon.source,
        UiIconSource {
            kind: UiIconSourceKind::SvgAsset,
            text: None,
            uri: Some("res://ui/icons/run.svg".to_string())
        }
    );
    assert_eq!(
        icon.direct_references()
            .iter()
            .map(|reference| reference.locator.to_string())
            .collect::<Vec<_>>(),
        vec!["res://ui/icons/run.svg"]
    );
    assert_eq!(<UiIconAsset as crate::asset::Asset>::LABEL, "ui_icon");
    assert_eq!(
        <<UiIconAsset as crate::asset::Asset>::Marker as ResourceMarker>::KIND,
        ResourceKind::Texture
    );

    let round_trip = UiIconAsset::from_toml_str(&icon.to_toml_string().unwrap()).unwrap();
    assert_eq!(round_trip, icon);
}

#[test]
fn ui_v2_asset_wrappers_parse_and_validate_kind() {
    let view = UiV2ViewAsset::from_toml_str(V2_VIEW_UI_TOML).unwrap();
    let component = UiV2ComponentAsset::from_toml_str(V2_COMPONENT_UI_TOML).unwrap();
    let style = UiV2StyleAsset::from_toml_str(V2_STYLE_UI_TOML).unwrap();

    assert_eq!(view.document.asset.kind, UiV2AssetKind::View);
    assert_eq!(component.document.asset.kind, UiV2AssetKind::Component);
    assert_eq!(style.document.asset.kind, UiV2AssetKind::Style);
    assert!(UiV2ViewAsset::from_toml_str(V2_COMPONENT_UI_TOML).is_err());
}
