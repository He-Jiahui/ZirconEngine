use super::*;

#[test]
fn mui_data_display_utility_classes_match_local_mui_selectors() {
    let style = UiAssetLoader::load_toml_str(MUI_WEB_STYLE_TOML).unwrap();
    let layout = UiAssetLoader::load_toml_str(MUI_WEB_DATA_DISPLAY_UTILITY_LAYOUT_TOML).unwrap();
    let mut compiler = UiDocumentCompiler::default();
    compiler
        .register_style_import("asset://ui/tests/mui_web_style.ui", style)
        .unwrap();

    let compiled = compiler.compile(&layout).unwrap();
    let root = &compiled.template_instance().root;
    let typography = &root.children[0];
    let divider = &root.children[1];
    let avatar = &root.children[2];
    let chip = &root.children[3];
    let badge = &root.children[4];
    let list = &root.children[5];
    let image_list = &root.children[6];
    let table = &root.children[7];

    assert_eq!(str_attr(typography, "text_tone"), Some("info"));
    assert_classes(
        typography,
        &[
            "MuiTypography-root",
            "MuiTypography-h6",
            "MuiTypography-alignCenter",
            "MuiTypography-gutterBottom",
            "MuiTypography-noWrap",
        ],
    );
    assert_no_classes(
        typography,
        &["MuiTypography-colorPrimary", "MuiTypography-sizeMedium"],
    );

    assert_eq!(str_attr(divider, "surface_variant"), Some("divider"));
    assert_classes(
        divider,
        &[
            "MuiDivider-root",
            "MuiDivider-middle",
            "MuiDivider-vertical",
            "MuiDivider-flexItem",
            "MuiDivider-withChildren",
        ],
    );
    let divider_wrapper = &divider.children[0];
    assert_eq!(str_attr(divider_wrapper, "text_tone"), Some("muted"));
    assert_classes(
        divider_wrapper,
        &["MuiDivider-wrapper", "MuiDivider-wrapperVertical"],
    );

    assert_eq!(str_attr(avatar, "surface_variant"), Some("avatar"));
    assert_classes(
        avatar,
        &[
            "MuiAvatar-root",
            "MuiAvatar-rounded",
            "MuiAvatar-colorDefault",
        ],
    );
    assert_no_classes(avatar, &["MuiAvatar-colorPrimary", "MuiAvatar-sizeMedium"]);

    assert_eq!(str_attr(chip, "validation_level"), Some("warning"));
    assert_classes(
        chip,
        &[
            "MuiChip-root",
            "MuiChip-outlined",
            "MuiChip-sizeSmall",
            "MuiChip-colorWarning",
            "MuiChip-clickable",
            "MuiChip-deletable",
        ],
    );
    let chip_label = &chip.children[0];
    assert_eq!(str_attr(chip_label, "text"), Some("Styled Warn"));
    assert_eq!(str_attr(chip_label, "text_tone"), Some("info"));
    assert_classes(
        chip_label,
        &["MuiChip-label", "chip-label-extra", "MuiLabel-root"],
    );
    assert!(
        str_attr(chip, "component_variant").is_some_and(|value| {
            value.contains("hasDeleteIcon") && value.contains("colorWarning")
        }),
        "Chip root should carry retained painter metadata"
    );
    let chip_delete_icon = &chip.children[1];
    assert!(
        str_attr(chip_delete_icon, "component_variant").is_some_and(|value| {
            value.contains("muiChipSlot") && value.contains("chipSlotDeleteIcon")
        }),
        "Chip deleteIcon slot should carry retained painter metadata"
    );
    assert_classes(chip_delete_icon, &["MuiChip-deleteIcon", "MuiIcon-root"]);

    assert_classes(badge, &["MuiBadge-root"]);
    assert_no_classes(badge, &["MuiBadge-dot", "MuiBadge-colorError"]);
    let badge_slot = &badge.children[0];
    assert_eq!(str_attr(badge_slot, "validation_level"), Some("error"));
    assert!(
        str_attr(badge_slot, "component_variant")
            .is_some_and(|value| value.contains("muiBadgeSlot") && value.contains("invisible")),
        "Badge slot should carry retained painter metadata"
    );
    assert_classes(
        badge_slot,
        &[
            "MuiBadge-badge",
            "MuiBadge-dot",
            "MuiBadge-invisible",
            "MuiBadge-anchorOriginBottomLeft",
            "MuiBadge-anchorOriginBottomLeftCircular",
            "MuiBadge-overlapCircular",
            "MuiBadge-colorError",
        ],
    );

    assert_eq!(str_attr(list, "surface_variant"), Some("list"));
    assert_classes(
        list,
        &[
            "MuiList-root",
            "MuiList-padding",
            "MuiList-dense",
            "MuiList-subheader",
        ],
    );
    assert_no_classes(list, &["MuiList-colorPrimary", "MuiList-sizeMedium"]);

    assert_eq!(str_attr(image_list, "overflow"), Some("scroll"));
    assert_classes(image_list, &["MuiImageList-root", "MuiImageList-masonry"]);
    assert_no_classes(
        image_list,
        &["MuiImageList-colorPrimary", "MuiImageList-sizeMedium"],
    );

    assert_eq!(int_attr(table, "z_index"), Some(2));
    assert_classes(table, &["MuiTable-root", "MuiTable-stickyHeader"]);
    assert_no_classes(table, &["MuiTable-colorPrimary", "MuiTable-sizeMedium"]);
}
