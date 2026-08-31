use super::*;

#[test]
fn material_component_lab_chip_sample_uses_runtime_descriptor_and_theme_selectors() {
    let path = editor_asset("assets/ui/editor/material_components/data_display/material_chips.zui");
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

    let selectors = editor_material_theme_selectors();
    for selector in CHIP_THEME_SELECTORS {
        assert!(
            selectors.contains(*selector),
            "Editor Material theme should style Chip selector `{selector}`"
        );
    }
}

#[test]
fn material_component_lab_badge_sample_uses_runtime_descriptor_and_theme_selectors() {
    let path =
        editor_asset("assets/ui/editor/material_components/data_display/material_badges.zui");
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

    let selectors = editor_material_theme_selectors();
    for selector in BADGE_THEME_SELECTORS {
        assert!(
            selectors.contains(*selector),
            "Editor Material theme should style Badge selector `{selector}`"
        );
    }
}

#[test]
fn material_component_lab_skeleton_sample_uses_runtime_descriptor_and_theme_selectors() {
    let path = editor_asset("assets/ui/editor/material_components/feedback/material_skeleton.zui");
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

    let selectors = editor_material_theme_selectors();
    for selector in SKELETON_THEME_SELECTORS {
        assert!(
            selectors.contains(*selector),
            "Editor Material theme should style Skeleton selector `{selector}`"
        );
    }
}

#[test]
fn material_component_lab_avatar_sample_uses_runtime_descriptor_and_theme_selectors() {
    let path =
        editor_asset("assets/ui/editor/material_components/data_display/material_avatars.zui");
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

    let selectors = editor_material_theme_selectors();
    for selector in AVATAR_THEME_SELECTORS {
        assert!(
            selectors.contains(*selector),
            "Editor Material theme should style Avatar selector `{selector}`"
        );
    }
}

#[test]
fn material_component_lab_list_sample_uses_runtime_descriptor_and_theme_selectors() {
    let path = editor_asset("assets/ui/editor/material_components/data_display/material_lists.zui");
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

    let selectors = editor_material_theme_selectors();
    for selector in LIST_THEME_SELECTORS {
        assert!(
            selectors.contains(*selector),
            "Editor Material theme should style List selector `{selector}`"
        );
    }
}

#[test]
fn material_component_lab_image_list_sample_uses_runtime_descriptor_and_theme_selectors() {
    let path =
        editor_asset("assets/ui/editor/material_components/data_display/material_image_list.zui");
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

    let selectors = editor_material_theme_selectors();
    for selector in IMAGE_LIST_THEME_SELECTORS {
        assert!(
            selectors.contains(*selector),
            "Editor Material theme should style ImageList selector `{selector}`"
        );
    }
}

#[test]
fn material_component_lab_table_sample_uses_runtime_descriptor_and_theme_selectors() {
    let path = editor_asset("assets/ui/editor/material_components/data_display/material_table.zui");
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

    let selectors = editor_material_theme_selectors();
    for selector in TABLE_THEME_SELECTORS {
        assert!(
            selectors.contains(*selector),
            "Editor Material theme should style Table selector `{selector}`"
        );
    }
}

#[test]
fn material_component_lab_divider_sample_uses_runtime_descriptor_and_theme_selectors() {
    let path =
        editor_asset("assets/ui/editor/material_components/data_display/material_dividers.zui");
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

    let selectors = editor_material_theme_selectors();
    for selector in DIVIDER_THEME_SELECTORS {
        assert!(
            selectors.contains(*selector),
            "Editor Material theme should style Divider selector `{selector}`"
        );
    }
}
