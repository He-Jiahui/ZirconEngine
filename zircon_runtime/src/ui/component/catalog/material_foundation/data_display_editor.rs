use super::shared::*;

pub(super) fn descriptors() -> Vec<UiComponentDescriptor> {
    vec![
        folder_tree(),
        asset_grid(),
        asset_list(),
        categorized_list(),
    ]
}

fn folder_tree() -> UiComponentDescriptor {
    editor_panel_component(
        "FolderTree",
        "Folder Tree",
        UiComponentCategory::Collection,
        "folder-tree",
    )
    .with_prop(string_prop("root_path"))
    .with_prop(string_prop("query"))
    .slot(UiSlotSchema::new("nodes").multiple(true))
    .events([
        UiComponentEventKind::SelectOption,
        UiComponentEventKind::ToggleExpanded,
        UiComponentEventKind::OpenPopupAt,
    ])
}

fn asset_grid() -> UiComponentDescriptor {
    editor_panel_component(
        "AssetGrid",
        "Asset Grid",
        UiComponentCategory::Collection,
        "asset-grid",
    )
    .with_prop(int_prop("item_count", 0))
    .with_prop(string_prop("query"))
    .slot(UiSlotSchema::new("items").multiple(true))
    .events([
        UiComponentEventKind::SelectOption,
        UiComponentEventKind::OpenReference,
        UiComponentEventKind::LocateReference,
        UiComponentEventKind::OpenPopupAt,
    ])
}

fn asset_list() -> UiComponentDescriptor {
    editor_panel_component(
        "AssetList",
        "Asset List",
        UiComponentCategory::Collection,
        "asset-list",
    )
    .with_prop(int_prop("item_count", 0))
    .with_prop(string_prop("query"))
    .slot(UiSlotSchema::new("items").multiple(true))
    .events([
        UiComponentEventKind::SelectOption,
        UiComponentEventKind::OpenReference,
        UiComponentEventKind::LocateReference,
        UiComponentEventKind::OpenPopupAt,
    ])
}

fn categorized_list() -> UiComponentDescriptor {
    editor_panel_component(
        "CategorizedList",
        "Categorized List",
        UiComponentCategory::Collection,
        "categorized-list",
    )
    .with_prop(string_prop("query"))
    .slot(UiSlotSchema::new("categories").multiple(true))
    .slot(UiSlotSchema::new("items").multiple(true))
    .events([
        UiComponentEventKind::SelectOption,
        UiComponentEventKind::ToggleExpanded,
    ])
}
