use std::fs;

use toml::Value;
use zircon_runtime::ui::v2::UiZuiAssetLoader;
use zircon_runtime_interface::ui::v2::UiV2NodeDefinition;

use super::support::{
    assert_component, assert_node_class, child_nodes, editor_asset,
    editor_material_theme_selectors, numeric_prop,
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

mod core_components;
mod extended_components;

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
