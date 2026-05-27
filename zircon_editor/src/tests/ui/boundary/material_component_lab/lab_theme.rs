use std::{collections::BTreeSet, fs};

use toml::Value;
use zircon_runtime::ui::v2::UiZuiAssetLoader;
use zircon_runtime_interface::ui::v2::UiV2NodeDefinition;

use super::support::{
    assert_component, assert_node_class, child_nodes, editor_asset, numeric_prop,
};

const TYPOGRAPHY_THEME_SELECTORS: &[&str] = &[
    ".MuiTypography-root",
    ".MuiTypography-h6",
    ".MuiTypography-alignCenter",
    ".MuiTypography-gutterBottom",
    ".MuiTypography-noWrap",
    ".MuiTypography-root.MuiTypography-h6.MuiTypography-alignCenter.MuiTypography-gutterBottom.MuiTypography-noWrap",
    ".material-typography-sample",
];

const CHIP_THEME_SELECTORS: &[&str] = &[
    ".MuiChip-root",
    ".MuiChip-filled",
    ".MuiChip-outlined",
    ".MuiChip-sizeSmall",
    ".MuiChip-sizeMedium",
    ".MuiChip-colorDefault",
    ".MuiChip-colorPrimary",
    ".MuiChip-colorSecondary",
    ".MuiChip-colorWarning",
    ".MuiChip-colorError",
    ".MuiChip-clickable",
    ".MuiChip-deletable",
    ".MuiChip-disabled",
    ".MuiChip-focusVisible",
    ".MuiChip-label",
    ".MuiChip-deleteIcon",
    ".MuiChip-icon",
    ".MuiChip-avatar",
    ".MuiChip-root.MuiChip-outlined.MuiChip-sizeSmall.MuiChip-colorWarning.MuiChip-clickable.MuiChip-deletable",
    ".material-chip-sample",
    ".material-chip-label",
    ".material-chip-delete-icon",
];

const DIVIDER_THEME_SELECTORS: &[&str] = &[
    ".MuiDivider-root",
    ".MuiDivider-middle",
    ".MuiDivider-vertical",
    ".MuiDivider-flexItem",
    ".MuiDivider-withChildren",
    ".MuiDivider-root.MuiDivider-middle.MuiDivider-vertical.MuiDivider-flexItem.MuiDivider-withChildren",
    ".MuiDivider-wrapper",
    ".MuiDivider-wrapperVertical",
    ".material-divider-sample",
    ".material-divider-wrapper",
];

const TABLE_THEME_SELECTORS: &[&str] = &[
    ".MuiTable-root",
    ".MuiTable-stickyHeader",
    ".MuiTable-root.MuiTable-stickyHeader",
    ".material-table-sample",
    ".material-table-header",
    ".material-table-row",
];

const IMAGE_LIST_THEME_SELECTORS: &[&str] = &[
    ".MuiImageList-root",
    ".MuiImageList-masonry",
    ".MuiImageList-quilted",
    ".MuiImageList-standard",
    ".MuiImageList-woven",
    ".MuiImageList-root.MuiImageList-masonry",
    ".material-image-list-sample",
    ".material-image-list-item",
];

const LIST_THEME_SELECTORS: &[&str] = &[
    ".MuiList-root",
    ".MuiList-padding",
    ".MuiList-dense",
    ".MuiList-subheader",
    ".MuiList-root.MuiList-padding.MuiList-dense.MuiList-subheader",
    ".material-list-sample",
    ".material-list-subheader",
    ".material-list-item",
];

const AVATAR_THEME_SELECTORS: &[&str] = &[
    ".MuiAvatar-root",
    ".MuiAvatar-colorDefault",
    ".MuiAvatar-circular",
    ".MuiAvatar-rounded",
    ".MuiAvatar-square",
    ".MuiAvatar-img",
    ".MuiAvatar-fallback",
    ".MuiAvatar-root.MuiAvatar-rounded.MuiAvatar-colorDefault",
    ".material-avatar-sample",
    ".material-avatar-fallback",
];

const BADGE_THEME_SELECTORS: &[&str] = &[
    ".MuiBadge-root",
    ".MuiBadge-badge",
    ".MuiBadge-standard",
    ".MuiBadge-dot",
    ".MuiBadge-invisible",
    ".MuiBadge-overlapCircular",
    ".MuiBadge-overlapRectangular",
    ".MuiBadge-anchorOriginTopRight",
    ".MuiBadge-anchorOriginBottomLeft",
    ".MuiBadge-anchorOriginBottomLeftCircular",
    ".MuiBadge-colorPrimary",
    ".MuiBadge-colorSecondary",
    ".MuiBadge-colorError",
    ".MuiBadge-colorInfo",
    ".MuiBadge-colorSuccess",
    ".MuiBadge-colorWarning",
    ".MuiBadge-badge.MuiBadge-standard.MuiBadge-anchorOriginBottomLeft.MuiBadge-anchorOriginBottomLeftCircular.MuiBadge-overlapCircular.MuiBadge-colorError",
    ".material-badge-sample",
    ".material-badge-slot",
];

const SKELETON_THEME_SELECTORS: &[&str] = &[
    ".MuiSkeleton-root",
    ".MuiSkeleton-text",
    ".MuiSkeleton-rectangular",
    ".MuiSkeleton-rounded",
    ".MuiSkeleton-circular",
    ".MuiSkeleton-pulse",
    ".MuiSkeleton-wave",
    ".MuiSkeleton-withChildren",
    ".MuiSkeleton-fitContent",
    ".MuiSkeleton-heightAuto",
    ".MuiSkeleton-root.MuiSkeleton-rounded.MuiSkeleton-wave",
    ".material-skeleton-sample",
    ".material-skeleton-child",
];

const TIMELINE_THEME_SELECTORS: &[&str] = &[
    ".MuiTimeline-root",
    ".MuiTimeline-positionAlternateReverse",
    ".MuiTimeline-root.MuiTimeline-positionAlternateReverse",
    ".MuiTimelineItem-root",
    ".MuiTimelineItem-positionAlternateReverse",
    ".MuiTimelineItem-missingOppositeContent",
    ".MuiTimelineItem-root.MuiTimelineItem-positionAlternateReverse.MuiTimelineItem-missingOppositeContent",
    ".MuiTimelineContent-root",
    ".MuiTimelineContent-positionRight",
    ".MuiTimelineContent-root.MuiTimelineContent-positionRight",
    ".MuiTimelineOppositeContent-root",
    ".MuiTimelineOppositeContent-positionLeft",
    ".MuiTimelineSeparator-root",
    ".MuiTimelineConnector-root",
    ".MuiTimelineDot-root",
    ".MuiTimelineDot-outlined",
    ".MuiTimelineDot-outlinedSecondary",
    ".MuiTimelineDot-root.MuiTimelineDot-outlined.MuiTimelineDot-outlinedSecondary",
    ".material-timeline-sample",
    ".material-timeline-item",
    ".material-timeline-content",
    ".material-timeline-separator",
    ".material-timeline-connector",
    ".material-timeline-dot",
];

const TRANSFER_LIST_THEME_SELECTORS: &[&str] = &[
    ".MuiTransferList-root",
    ".MuiTransferList-root.MuiTransferList-hasSourceItems.MuiTransferList-hasTargetItems.MuiTransferList-hasSelectedItems.MuiTransferList-hasDisabledItems.MuiTransferList-hasDisabledActions",
    ".MuiTransferList-source",
    ".MuiTransferList-source.MuiTransferList-sourcePopulated.MuiTransferList-sourceSelected",
    ".MuiTransferList-target",
    ".MuiTransferList-target.MuiTransferList-targetPopulated.MuiTransferList-targetSelected",
    ".MuiTransferList-actions",
    ".MuiTransferList-actions.MuiTransferList-actionsDisabled",
    ".material-transfer-list-sample",
    ".material-transfer-list-source",
    ".material-transfer-list-target",
    ".material-transfer-list-actions",
];

const AUTOCOMPLETE_THEME_SELECTORS: &[&str] = &[
    ".MuiAutocomplete-root",
    ".MuiAutocomplete-root.MuiAutocomplete-expanded.MuiAutocomplete-focused.MuiAutocomplete-fullWidth.MuiAutocomplete-hasClearIcon.MuiAutocomplete-hasPopupIcon",
    ".MuiAutocomplete-inputRoot",
    ".MuiAutocomplete-inputRoot.MuiAutocomplete-hasClearIcon.MuiAutocomplete-hasPopupIcon",
    ".MuiAutocomplete-input",
    ".MuiAutocomplete-input.MuiAutocomplete-inputFocused",
    ".MuiAutocomplete-tag",
    ".MuiAutocomplete-tag.MuiAutocomplete-tagSizeSmall",
    ".MuiAutocomplete-endAdornment",
    ".MuiAutocomplete-clearIndicator",
    ".MuiAutocomplete-popupIndicator",
    ".MuiAutocomplete-popupIndicator.MuiAutocomplete-popupIndicatorOpen",
    ".MuiAutocomplete-popper",
    ".MuiAutocomplete-popper.MuiAutocomplete-popperDisablePortal",
    ".MuiAutocomplete-paper",
    ".MuiAutocomplete-listbox",
    ".MuiAutocomplete-loading",
    ".MuiAutocomplete-noOptions",
    ".MuiAutocomplete-option",
    ".MuiAutocomplete-option.MuiAutocomplete-focused.MuiAutocomplete-focusVisible",
    ".MuiAutocomplete-groupLabel",
    ".MuiAutocomplete-groupUl",
    ".material-autocomplete-sample",
    ".material-autocomplete-input-root",
    ".material-autocomplete-input",
    ".material-autocomplete-tag",
    ".material-autocomplete-popup-indicator",
    ".material-autocomplete-popper",
    ".material-autocomplete-paper",
    ".material-autocomplete-listbox",
    ".material-autocomplete-option",
];

#[test]
fn material_component_lab_chip_sample_uses_runtime_descriptor_and_theme_selectors() {
    let path = editor_asset("assets/ui/editor/material_components/material_chips.zui");
    let source = fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("{} should be readable: {error}", path.display()));
    let document = UiZuiAssetLoader::load_zui_str(&source)
        .unwrap_or_else(|error| panic!("{} should load as .zui: {error}", path.display()));
    let sample = document
        .nodes
        .get("sample")
        .unwrap_or_else(|| panic!("{} should define a sample node", path.display()));

    assert_component(&document, "sample", "Chip");
    assert_node_class(&document, "sample", "MuiChip-root");
    assert_eq!(str_prop(sample, "className"), Some("material-chip-sample"));
    assert_eq!(str_prop(sample, "component"), Some("div"));
    assert_eq!(str_prop(sample, "label"), Some("Warn"));
    assert_eq!(str_prop(sample, "variant"), Some("outlined"));
    assert_eq!(str_prop(sample, "size"), Some("small"));
    assert_eq!(str_prop(sample, "color"), Some("warning"));
    assert_eq!(bool_prop(sample, "clickable"), Some(true));
    assert_eq!(bool_prop(sample, "deletable"), Some(true));
    assert_eq!(bool_prop(sample, "onDelete"), Some(true));
    assert_eq!(bool_prop(sample, "focusVisible"), Some(true));
    assert_eq!(str_prop(sample, "deleteIcon"), Some("cancel"));
    assert_eq!(
        slot_class_name(sample, "label"),
        Some("material-chip-label")
    );
    assert_eq!(
        slot_class_name(sample, "deleteIcon"),
        Some("material-chip-delete-icon")
    );

    assert_eq!(
        child_nodes(&document, "sample"),
        vec!["chip_label", "chip_delete_icon"]
    );
    assert_component(&document, "chip_label", "Label");
    assert_node_class(&document, "chip_label", "MuiChip-label");
    assert_node_class(&document, "chip_label", "material-chip-label");
    assert_eq!(
        str_prop(node(&document, "chip_label"), "text"),
        Some("Styled Warn")
    );
    assert_non_dispatchable_child(node(&document, "chip_label"), "chip_label", "Chip");

    assert_component(&document, "chip_delete_icon", "Icon");
    assert_node_class(&document, "chip_delete_icon", "MuiChip-deleteIcon");
    assert_node_class(&document, "chip_delete_icon", "material-chip-delete-icon");
    assert_eq!(
        str_prop(node(&document, "chip_delete_icon"), "icon"),
        Some("cancel")
    );
    assert_non_dispatchable_child(
        node(&document, "chip_delete_icon"),
        "chip_delete_icon",
        "Chip",
    );

    let theme_source =
        fs::read_to_string(editor_asset("assets/ui/theme/editor_material.v2.ui.toml"))
            .expect("Editor Material theme should be readable");
    let selectors = theme_selectors(&theme_source);
    for selector in CHIP_THEME_SELECTORS {
        assert!(
            selectors.contains(*selector),
            "Editor Material theme should style Chip selector `{selector}`"
        );
    }
}

#[test]
fn material_component_lab_badge_sample_uses_runtime_descriptor_and_theme_selectors() {
    let path = editor_asset("assets/ui/editor/material_components/material_badges.zui");
    let source = fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("{} should be readable: {error}", path.display()));
    let document = UiZuiAssetLoader::load_zui_str(&source)
        .unwrap_or_else(|error| panic!("{} should load as .zui: {error}", path.display()));
    let sample = document
        .nodes
        .get("sample")
        .unwrap_or_else(|| panic!("{} should define a sample node", path.display()));

    assert_component(&document, "sample", "Badge");
    assert_node_class(&document, "sample", "MuiBadge-root");
    assert_eq!(str_prop(sample, "className"), Some("material-badge-sample"));
    assert_eq!(str_prop(sample, "component"), Some("span"));
    assert_eq!(str_prop(sample, "badgeContent"), Some("12"));
    assert_eq!(numeric_prop(sample.props.get("max")), Some(99.0));
    assert_eq!(bool_prop(sample, "showZero"), Some(false));
    assert_eq!(bool_prop(sample, "invisible"), Some(false));
    assert_eq!(str_prop(sample, "variant"), Some("standard"));
    assert_eq!(str_prop(sample, "color"), Some("error"));
    assert_eq!(str_prop(sample, "overlap"), Some("circular"));
    assert_eq!(
        table_str_prop(sample, "anchorOrigin", "vertical"),
        Some("bottom")
    );
    assert_eq!(
        table_str_prop(sample, "anchorOrigin", "horizontal"),
        Some("left")
    );
    assert_eq!(
        slot_class_name(sample, "badge"),
        Some("material-badge-slot")
    );

    assert_eq!(child_nodes(&document, "sample"), vec!["badge_slot"]);
    assert_component(&document, "badge_slot", "Label");
    assert_node_class(&document, "badge_slot", "MuiBadge-badge");
    assert_node_class(&document, "badge_slot", "material-badge-slot");
    assert_eq!(str_prop(node(&document, "badge_slot"), "text"), Some("12"));
    assert_non_dispatchable_child(node(&document, "badge_slot"), "badge_slot", "Badge");

    let theme_source =
        fs::read_to_string(editor_asset("assets/ui/theme/editor_material.v2.ui.toml"))
            .expect("Editor Material theme should be readable");
    let selectors = theme_selectors(&theme_source);
    for selector in BADGE_THEME_SELECTORS {
        assert!(
            selectors.contains(*selector),
            "Editor Material theme should style Badge selector `{selector}`"
        );
    }
}

#[test]
fn material_component_lab_skeleton_sample_uses_runtime_descriptor_and_theme_selectors() {
    let path = editor_asset("assets/ui/editor/material_components/material_skeleton.zui");
    let source = fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("{} should be readable: {error}", path.display()));
    let document = UiZuiAssetLoader::load_zui_str(&source)
        .unwrap_or_else(|error| panic!("{} should load as .zui: {error}", path.display()));
    let sample = document
        .nodes
        .get("sample")
        .unwrap_or_else(|| panic!("{} should define a sample node", path.display()));

    assert_component(&document, "sample", "Skeleton");
    assert_node_class(&document, "sample", "MuiSkeleton-root");
    assert_eq!(
        str_prop(sample, "className"),
        Some("material-skeleton-sample")
    );
    assert_eq!(str_prop(sample, "component"), Some("span"));
    assert_eq!(str_prop(sample, "variant"), Some("rounded"));
    assert_eq!(str_prop(sample, "animation"), Some("wave"));
    assert_eq!(numeric_prop(sample.props.get("width")), Some(144.0));
    assert_eq!(numeric_prop(sample.props.get("height")), Some(20.0));

    assert_eq!(child_nodes(&document, "sample"), vec!["skeleton_child"]);
    assert_component(&document, "skeleton_child", "Label");
    assert_node_class(&document, "skeleton_child", "material-skeleton-child");
    assert_eq!(
        str_prop(node(&document, "skeleton_child"), "text"),
        Some("Loading")
    );
    assert_non_dispatchable_child(
        node(&document, "skeleton_child"),
        "skeleton_child",
        "Skeleton",
    );

    let theme_source =
        fs::read_to_string(editor_asset("assets/ui/theme/editor_material.v2.ui.toml"))
            .expect("Editor Material theme should be readable");
    let selectors = theme_selectors(&theme_source);
    for selector in SKELETON_THEME_SELECTORS {
        assert!(
            selectors.contains(*selector),
            "Editor Material theme should style Skeleton selector `{selector}`"
        );
    }
}

#[test]
fn material_component_lab_avatar_sample_uses_runtime_descriptor_and_theme_selectors() {
    let path = editor_asset("assets/ui/editor/material_components/material_avatars.zui");
    let source = fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("{} should be readable: {error}", path.display()));
    let document = UiZuiAssetLoader::load_zui_str(&source)
        .unwrap_or_else(|error| panic!("{} should load as .zui: {error}", path.display()));
    let sample = document
        .nodes
        .get("sample")
        .unwrap_or_else(|| panic!("{} should define a sample node", path.display()));

    assert_component(&document, "sample", "Avatar");
    assert_node_class(&document, "sample", "MuiAvatar-root");
    assert_eq!(
        str_prop(sample, "className"),
        Some("material-avatar-sample")
    );
    assert_eq!(str_prop(sample, "component"), Some("div"));
    assert_eq!(str_prop(sample, "variant"), Some("rounded"));
    assert_eq!(str_prop(sample, "text"), Some("ZR"));
    assert_eq!(str_prop(sample, "alt"), Some("Zircon renderer"));
    assert_eq!(str_prop(sample, "src"), Some(""));
    assert_eq!(str_prop(sample, "srcSet"), Some(""));
    assert_eq!(
        slot_class_name(sample, "fallback"),
        Some("material-avatar-fallback")
    );

    let theme_source =
        fs::read_to_string(editor_asset("assets/ui/theme/editor_material.v2.ui.toml"))
            .expect("Editor Material theme should be readable");
    let selectors = theme_selectors(&theme_source);
    for selector in AVATAR_THEME_SELECTORS {
        assert!(
            selectors.contains(*selector),
            "Editor Material theme should style Avatar selector `{selector}`"
        );
    }
}

#[test]
fn material_component_lab_list_sample_uses_runtime_descriptor_and_theme_selectors() {
    let path = editor_asset("assets/ui/editor/material_components/material_lists.zui");
    let source = fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("{} should be readable: {error}", path.display()));
    let document = UiZuiAssetLoader::load_zui_str(&source)
        .unwrap_or_else(|error| panic!("{} should load as .zui: {error}", path.display()));
    let sample = document
        .nodes
        .get("sample")
        .unwrap_or_else(|| panic!("{} should define a sample node", path.display()));

    assert_component(&document, "sample", "List");
    assert_node_class(&document, "sample", "MuiList-root");
    assert_eq!(str_prop(sample, "className"), Some("material-list-sample"));
    assert_eq!(str_prop(sample, "component"), Some("ul"));
    assert_eq!(str_prop(sample, "subheader"), Some("Scene Layers"));
    assert_eq!(bool_prop(sample, "dense"), Some(true));
    assert_eq!(bool_prop(sample, "disablePadding"), Some(false));
    assert_eq!(array_len(sample, "items"), Some(3));
    assert_eq!(
        slot_class_name(sample, "subheader"),
        Some("material-list-subheader")
    );
    assert_eq!(slot_class_name(sample, "items"), Some("material-list-item"));

    let theme_source =
        fs::read_to_string(editor_asset("assets/ui/theme/editor_material.v2.ui.toml"))
            .expect("Editor Material theme should be readable");
    let selectors = theme_selectors(&theme_source);
    for selector in LIST_THEME_SELECTORS {
        assert!(
            selectors.contains(*selector),
            "Editor Material theme should style List selector `{selector}`"
        );
    }
}

#[test]
fn material_component_lab_image_list_sample_uses_runtime_descriptor_and_theme_selectors() {
    let path = editor_asset("assets/ui/editor/material_components/material_image_list.zui");
    let source = fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("{} should be readable: {error}", path.display()));
    let document = UiZuiAssetLoader::load_zui_str(&source)
        .unwrap_or_else(|error| panic!("{} should load as .zui: {error}", path.display()));
    let sample = document
        .nodes
        .get("sample")
        .unwrap_or_else(|| panic!("{} should define a sample node", path.display()));

    assert_component(&document, "sample", "ImageList");
    assert_node_class(&document, "sample", "MuiImageList-root");
    assert_eq!(
        str_prop(sample, "className"),
        Some("material-image-list-sample")
    );
    assert_eq!(str_prop(sample, "component"), Some("ul"));
    assert_eq!(str_prop(sample, "variant"), Some("masonry"));
    assert_eq!(str_prop(sample, "rowHeight"), Some("auto"));
    assert_eq!(numeric_prop(sample.props.get("cols")), Some(3.0));
    assert_eq!(numeric_prop(sample.props.get("gap")), Some(6.0));
    assert_eq!(array_len(sample, "items"), Some(3));
    assert_eq!(
        slot_class_name(sample, "items"),
        Some("material-image-list-item")
    );

    let theme_source =
        fs::read_to_string(editor_asset("assets/ui/theme/editor_material.v2.ui.toml"))
            .expect("Editor Material theme should be readable");
    let selectors = theme_selectors(&theme_source);
    for selector in IMAGE_LIST_THEME_SELECTORS {
        assert!(
            selectors.contains(*selector),
            "Editor Material theme should style ImageList selector `{selector}`"
        );
    }
}

#[test]
fn material_component_lab_table_sample_uses_runtime_descriptor_and_theme_selectors() {
    let path = editor_asset("assets/ui/editor/material_components/material_table.zui");
    let source = fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("{} should be readable: {error}", path.display()));
    let document = UiZuiAssetLoader::load_zui_str(&source)
        .unwrap_or_else(|error| panic!("{} should load as .zui: {error}", path.display()));
    let sample = document
        .nodes
        .get("sample")
        .unwrap_or_else(|| panic!("{} should define a sample node", path.display()));

    assert_component(&document, "sample", "Table");
    assert_node_class(&document, "sample", "MuiTable-root");
    assert_eq!(str_prop(sample, "className"), Some("material-table-sample"));
    assert_eq!(str_prop(sample, "component"), Some("table"));
    assert_eq!(str_prop(sample, "padding"), Some("checkbox"));
    assert_eq!(str_prop(sample, "size"), Some("small"));
    assert_eq!(bool_prop(sample, "stickyHeader"), Some(true));
    assert_eq!(array_len(sample, "rows"), Some(2));
    assert_eq!(array_len(sample, "columns"), Some(2));
    assert_eq!(
        slot_class_name(sample, "header"),
        Some("material-table-header")
    );
    assert_eq!(slot_class_name(sample, "row"), Some("material-table-row"));

    let theme_source =
        fs::read_to_string(editor_asset("assets/ui/theme/editor_material.v2.ui.toml"))
            .expect("Editor Material theme should be readable");
    let selectors = theme_selectors(&theme_source);
    for selector in TABLE_THEME_SELECTORS {
        assert!(
            selectors.contains(*selector),
            "Editor Material theme should style Table selector `{selector}`"
        );
    }
}

#[test]
fn material_component_lab_divider_sample_uses_runtime_descriptor_and_theme_selectors() {
    let path = editor_asset("assets/ui/editor/material_components/material_dividers.zui");
    let source = fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("{} should be readable: {error}", path.display()));
    let document = UiZuiAssetLoader::load_zui_str(&source)
        .unwrap_or_else(|error| panic!("{} should load as .zui: {error}", path.display()));
    let sample = document
        .nodes
        .get("sample")
        .unwrap_or_else(|| panic!("{} should define a sample node", path.display()));

    assert_component(&document, "sample", "Divider");
    assert_node_class(&document, "sample", "MuiDivider-root");
    assert_eq!(
        str_prop(sample, "className"),
        Some("material-divider-sample")
    );
    assert_eq!(str_prop(sample, "component"), Some("div"));
    assert_eq!(str_prop(sample, "orientation"), Some("vertical"));
    assert_eq!(str_prop(sample, "variant"), Some("middle"));
    assert_eq!(str_prop(sample, "textAlign"), Some("right"));
    assert_eq!(bool_prop(sample, "flexItem"), Some(true));
    assert_eq!(
        slot_class_name(sample, "wrapper"),
        Some("material-divider-wrapper")
    );

    let theme_source =
        fs::read_to_string(editor_asset("assets/ui/theme/editor_material.v2.ui.toml"))
            .expect("Editor Material theme should be readable");
    let selectors = theme_selectors(&theme_source);
    for selector in DIVIDER_THEME_SELECTORS {
        assert!(
            selectors.contains(*selector),
            "Editor Material theme should style Divider selector `{selector}`"
        );
    }
}

#[test]
fn material_component_lab_typography_sample_uses_runtime_descriptor_and_theme_selectors() {
    let path = editor_asset("assets/ui/editor/material_components/material_typography.zui");
    let source = fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("{} should be readable: {error}", path.display()));
    let document = UiZuiAssetLoader::load_zui_str(&source)
        .unwrap_or_else(|error| panic!("{} should load as .zui: {error}", path.display()));
    let sample = document
        .nodes
        .get("sample")
        .unwrap_or_else(|| panic!("{} should define a sample node", path.display()));

    assert_component(&document, "sample", "Typography");
    assert_node_class(&document, "sample", "MuiTypography-root");
    assert_eq!(
        str_prop(sample, "className"),
        Some("material-typography-sample")
    );
    assert_eq!(str_prop(sample, "component"), Some("h6"));
    assert_eq!(str_prop(sample, "variant"), Some("h6"));
    assert_eq!(str_prop(sample, "align"), Some("center"));
    assert_eq!(bool_prop(sample, "gutterBottom"), Some(true));
    assert_eq!(bool_prop(sample, "noWrap"), Some(true));
    assert_eq!(table_str_prop(sample, "variantMapping", "h6"), Some("h2"));
    assert_eq!(table_str_prop(sample, "variantMapping", "body2"), Some("p"));

    let theme_source =
        fs::read_to_string(editor_asset("assets/ui/theme/editor_material.v2.ui.toml"))
            .expect("Editor Material theme should be readable");
    let selectors = theme_selectors(&theme_source);
    for selector in TYPOGRAPHY_THEME_SELECTORS {
        assert!(
            selectors.contains(*selector),
            "Editor Material theme should style Typography selector `{selector}`"
        );
    }
}

#[test]
fn material_component_lab_timeline_sample_uses_runtime_descriptor_and_theme_selectors() {
    let path = editor_asset("assets/ui/editor/material_components/material_timeline.zui");
    let source = fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("{} should be readable: {error}", path.display()));
    let document = UiZuiAssetLoader::load_zui_str(&source)
        .unwrap_or_else(|error| panic!("{} should load as .zui: {error}", path.display()));
    let sample = document
        .nodes
        .get("sample")
        .unwrap_or_else(|| panic!("{} should define a sample node", path.display()));

    assert_component(&document, "sample", "Timeline");
    assert_node_class(&document, "sample", "MuiTimeline-root");
    assert_eq!(
        str_prop(sample, "className"),
        Some("material-timeline-sample")
    );
    assert_eq!(str_prop(sample, "component"), Some("ul"));
    assert_eq!(str_prop(sample, "position"), Some("alternate-reverse"));
    assert_eq!(numeric_prop(sample.props.get("time")), Some(12.0));
    assert_eq!(numeric_prop(sample.props.get("duration")), Some(48.0));
    for (slot, expected_class) in [
        ("items", "material-timeline-item"),
        ("content", "material-timeline-content"),
        ("separator", "material-timeline-separator"),
        ("connector", "material-timeline-connector"),
        ("dot", "material-timeline-dot"),
    ] {
        assert_eq!(
            slot_class_name(sample, slot),
            Some(expected_class),
            "Timeline sample slotProps.{slot}.className should stay theme-addressable"
        );
    }

    assert_eq!(child_nodes(&document, "sample"), vec!["timeline_item"]);
    assert_component(&document, "timeline_item", "TimelineItem");
    assert_node_class(&document, "timeline_item", "MuiTimelineItem-root");
    let item = node(&document, "timeline_item");
    assert_eq!(str_prop(item, "position"), Some("alternate-reverse"));
    assert_eq!(bool_prop(item, "hasOppositeContent"), Some(false));
    assert_eq!(
        child_nodes(&document, "timeline_item"),
        vec!["timeline_separator", "timeline_content"]
    );

    assert_component(&document, "timeline_separator", "TimelineSeparator");
    assert_node_class(&document, "timeline_separator", "MuiTimelineSeparator-root");
    assert_eq!(
        child_nodes(&document, "timeline_separator"),
        vec!["timeline_dot", "timeline_connector"]
    );

    assert_component(&document, "timeline_dot", "TimelineDot");
    assert_node_class(&document, "timeline_dot", "MuiTimelineDot-root");
    let dot = node(&document, "timeline_dot");
    assert_eq!(str_prop(dot, "variant"), Some("outlined"));
    assert_eq!(str_prop(dot, "color"), Some("secondary"));
    assert_eq!(str_prop(dot, "className"), Some("material-timeline-dot"));

    assert_component(&document, "timeline_connector", "TimelineConnector");
    assert_node_class(&document, "timeline_connector", "MuiTimelineConnector-root");

    assert_component(&document, "timeline_content", "TimelineContent");
    assert_node_class(&document, "timeline_content", "MuiTimelineContent-root");
    let content = node(&document, "timeline_content");
    assert_eq!(str_prop(content, "position"), Some("right"));
    assert_eq!(
        str_prop(content, "className"),
        Some("material-timeline-content")
    );

    let theme_source =
        fs::read_to_string(editor_asset("assets/ui/theme/editor_material.v2.ui.toml"))
            .expect("Editor Material theme should be readable");
    let selectors = theme_selectors(&theme_source);
    for selector in TIMELINE_THEME_SELECTORS {
        assert!(
            selectors.contains(*selector),
            "Editor Material theme should style Timeline selector `{selector}`"
        );
    }
}

#[test]
fn material_component_lab_transfer_list_sample_uses_runtime_descriptor_and_theme_selectors() {
    let path = editor_asset("assets/ui/editor/material_components/material_transfer_list.zui");
    let source = fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("{} should be readable: {error}", path.display()));
    let document = UiZuiAssetLoader::load_zui_str(&source)
        .unwrap_or_else(|error| panic!("{} should load as .zui: {error}", path.display()));
    let sample = document
        .nodes
        .get("sample")
        .unwrap_or_else(|| panic!("{} should define a sample node", path.display()));

    assert_component(&document, "sample", "TransferList");
    assert_node_class(&document, "sample", "MuiTransferList-root");
    assert_eq!(
        str_prop(sample, "className"),
        Some("material-transfer-list-sample")
    );
    assert_eq!(
        string_array_prop(sample, "source_items"),
        vec!["Scene", "Materials", "Lighting"]
    );
    assert_eq!(string_array_prop(sample, "target_items"), vec!["Export"]);
    assert_eq!(
        string_array_prop(sample, "source_selected_items"),
        vec!["Materials"]
    );
    assert_eq!(
        string_array_prop(sample, "target_selected_items"),
        vec!["Export"]
    );
    assert_eq!(
        string_array_prop(sample, "disabled_items"),
        vec!["Lighting"]
    );
    assert_eq!(
        string_array_prop(sample, "disabled_actions"),
        vec!["move_all_left"]
    );
    for (slot, expected_class) in [
        ("source", "material-transfer-list-source"),
        ("target", "material-transfer-list-target"),
        ("actions", "material-transfer-list-actions"),
    ] {
        assert_eq!(
            slot_class_name(sample, slot),
            Some(expected_class),
            "TransferList sample slotProps.{slot}.className should stay theme-addressable"
        );
    }

    assert_eq!(
        child_nodes(&document, "sample"),
        vec!["transfer_source", "transfer_actions", "transfer_target"]
    );
    assert_component(&document, "transfer_source", "List");
    assert_node_class(
        &document,
        "transfer_source",
        "material-transfer-list-source",
    );
    assert_component(&document, "transfer_actions", "Button");
    assert_node_class(
        &document,
        "transfer_actions",
        "material-transfer-list-actions",
    );
    assert_component(&document, "transfer_target", "List");
    assert_node_class(
        &document,
        "transfer_target",
        "material-transfer-list-target",
    );
    for child_id in ["transfer_source", "transfer_actions", "transfer_target"] {
        assert_non_dispatchable_child(node(&document, child_id), child_id, "TransferList");
    }

    let theme_source =
        fs::read_to_string(editor_asset("assets/ui/theme/editor_material.v2.ui.toml"))
            .expect("Editor Material theme should be readable");
    let selectors = theme_selectors(&theme_source);
    for selector in TRANSFER_LIST_THEME_SELECTORS {
        assert!(
            selectors.contains(*selector),
            "Editor Material theme should style TransferList selector `{selector}`"
        );
    }
}

#[test]
fn material_component_lab_autocomplete_sample_uses_runtime_descriptor_and_theme_selectors() {
    let path = editor_asset("assets/ui/editor/material_components/material_autocomplete.zui");
    let source = fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("{} should be readable: {error}", path.display()));
    let document = UiZuiAssetLoader::load_zui_str(&source)
        .unwrap_or_else(|error| panic!("{} should load as .zui: {error}", path.display()));
    let sample = document
        .nodes
        .get("sample")
        .unwrap_or_else(|| panic!("{} should define a sample node", path.display()));

    assert_component(&document, "sample", "Autocomplete");
    assert_node_class(&document, "sample", "MuiAutocomplete-root");
    assert_eq!(
        str_prop(sample, "className"),
        Some("material-autocomplete-sample")
    );
    assert_eq!(str_prop(sample, "query"), Some("at"));
    assert_eq!(str_prop(sample, "inputValue"), Some("at"));
    assert_eq!(str_prop(sample, "value"), Some("atlas"));
    assert_eq!(bool_prop(sample, "multiple"), Some(true));
    assert_eq!(bool_prop(sample, "popup_open"), Some(true));
    assert_eq!(bool_prop(sample, "popupOpen"), Some(true));
    assert_eq!(bool_prop(sample, "fullWidth"), Some(true));
    assert_eq!(bool_prop(sample, "disablePortal"), Some(true));
    assert_eq!(bool_prop(sample, "inputFocused"), Some(true));
    assert_eq!(string_array_prop(sample, "selected_options"), vec!["atlas"]);
    assert_eq!(
        string_array_prop(sample, "filtered_options"),
        vec!["atlas", "asset"]
    );
    assert_eq!(
        string_array_prop(sample, "matched_options"),
        vec!["atlas", "asset"]
    );
    for (slot, expected_class) in [
        ("inputRoot", "material-autocomplete-input-root"),
        ("input", "material-autocomplete-input"),
        ("tag", "material-autocomplete-tag"),
        ("popupIndicator", "material-autocomplete-popup-indicator"),
        ("popper", "material-autocomplete-popper"),
        ("paper", "material-autocomplete-paper"),
        ("listbox", "material-autocomplete-listbox"),
        ("option", "material-autocomplete-option"),
    ] {
        assert_eq!(
            slot_class_name(sample, slot),
            Some(expected_class),
            "Autocomplete sample slotProps.{slot}.className should stay theme-addressable"
        );
    }

    assert_eq!(
        child_nodes(&document, "sample"),
        vec![
            "autocomplete_input_root",
            "autocomplete_input",
            "autocomplete_tag",
            "autocomplete_popup_indicator",
            "autocomplete_popper",
            "autocomplete_paper",
            "autocomplete_listbox",
            "autocomplete_option",
        ]
    );
    for (child_id, expected_component, expected_slot) in [
        ("autocomplete_input_root", "HorizontalBox", "inputRoot"),
        ("autocomplete_input", "Label", "input"),
        ("autocomplete_tag", "Label", "tag"),
        ("autocomplete_popup_indicator", "Label", "popupIndicator"),
        ("autocomplete_popper", "Label", "popper"),
        ("autocomplete_paper", "Label", "paper"),
        ("autocomplete_listbox", "Label", "listbox"),
        ("autocomplete_option", "Label", "option"),
    ] {
        assert_component(&document, child_id, expected_component);
        assert_eq!(
            child_slot_name(sample, child_id),
            Some(expected_slot),
            "Autocomplete child `{child_id}` should mount the expected slot"
        );
        assert_non_dispatchable_child(node(&document, child_id), child_id, "Autocomplete");
    }

    let theme_source =
        fs::read_to_string(editor_asset("assets/ui/theme/editor_material.v2.ui.toml"))
            .expect("Editor Material theme should be readable");
    let selectors = theme_selectors(&theme_source);
    for selector in AUTOCOMPLETE_THEME_SELECTORS {
        assert!(
            selectors.contains(*selector),
            "Editor Material theme should style Autocomplete selector `{selector}`"
        );
    }
}

fn node<'a>(
    document: &'a zircon_runtime_interface::ui::v2::UiV2AssetDocument,
    node_id: &str,
) -> &'a UiV2NodeDefinition {
    document
        .nodes
        .get(node_id)
        .unwrap_or_else(|| panic!("Material Lab should contain node `{node_id}`"))
}

fn str_prop<'a>(node: &'a UiV2NodeDefinition, name: &str) -> Option<&'a str> {
    node.props.get(name).and_then(Value::as_str)
}

fn bool_prop(node: &UiV2NodeDefinition, name: &str) -> Option<bool> {
    node.props.get(name).and_then(Value::as_bool)
}

fn array_len(node: &UiV2NodeDefinition, name: &str) -> Option<usize> {
    node.props.get(name).and_then(Value::as_array).map(Vec::len)
}

fn slot_class_name<'a>(node: &'a UiV2NodeDefinition, slot: &str) -> Option<&'a str> {
    node.props
        .get("slotProps")
        .and_then(Value::as_table)
        .and_then(|slot_props| slot_props.get(slot))
        .and_then(Value::as_table)
        .and_then(|props| props.get("className"))
        .and_then(Value::as_str)
}

fn table_str_prop<'a>(
    node: &'a UiV2NodeDefinition,
    table_name: &str,
    key: &str,
) -> Option<&'a str> {
    node.props
        .get(table_name)
        .and_then(Value::as_table)
        .and_then(|table| table.get(key))
        .and_then(Value::as_str)
}

fn string_array_prop<'a>(node: &'a UiV2NodeDefinition, name: &str) -> Vec<&'a str> {
    node.props
        .get(name)
        .and_then(Value::as_array)
        .map(|values| values.iter().filter_map(Value::as_str).collect())
        .unwrap_or_default()
}

fn child_slot_name<'a>(node: &'a UiV2NodeDefinition, child_id: &str) -> Option<&'a str> {
    node.children
        .iter()
        .find(|child| child.node == child_id)
        .and_then(|child| child.slot.get("name"))
        .and_then(Value::as_str)
}

fn assert_non_dispatchable_child(node: &UiV2NodeDefinition, node_id: &str, component: &str) {
    for prop in [
        "input_interactive",
        "input_clickable",
        "input_hoverable",
        "input_focusable",
    ] {
        assert_eq!(
            node.props.get(prop).and_then(Value::as_bool),
            Some(false),
            "{component} child `{node_id}` should leave dispatchability on the visible sample"
        );
    }
}

fn theme_selectors(source: &str) -> BTreeSet<String> {
    toml::from_str::<Value>(source)
        .expect("Editor Material theme should parse as TOML")
        .get("stylesheets")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .flat_map(|stylesheet| {
            stylesheet
                .get("rules")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
        })
        .filter_map(|rule| rule.get("selector").and_then(Value::as_str))
        .map(ToOwned::to_owned)
        .collect()
}
