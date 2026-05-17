use super::shared::*;

pub(super) fn descriptors() -> Vec<UiComponentDescriptor> {
    vec![
        primitive("Chip", "Chip", UiComponentCategory::Selection, "chip")
            .with_prop(text_prop())
            .with_prop(icon_prop())
            .with_prop(bool_prop("deletable", false))
            .events([
                UiComponentEventKind::Commit,
                UiComponentEventKind::SelectOption,
            ]),
        primitive("Divider", "Divider", UiComponentCategory::Visual, "divider")
            .with_prop(enum_prop("orientation", "horizontal"))
            .with_prop(text_prop()),
        primitive("Icon", "Icon", UiComponentCategory::Visual, "icon")
            .with_prop(icon_prop())
            .requires_render_capability(UiRenderCapability::Vector),
        primitive(
            "SvgIcon",
            "Svg Icon",
            UiComponentCategory::Visual,
            "svg-icon",
        )
        .with_prop(icon_prop())
        .requires_render_capability(UiRenderCapability::Vector),
        composite("List", "List", UiComponentCategory::Collection, "list")
            .with_prop(array_prop("items"))
            .slot(UiSlotSchema::new("items").multiple(true))
            .events([
                UiComponentEventKind::Hover,
                UiComponentEventKind::Press,
                UiComponentEventKind::SelectOption,
            ]),
        data_view("ListView", "List View", "list-view")
            .with_prop(int_prop("item_count", 0))
            .slot(UiSlotSchema::new("items").multiple(true))
            .event(UiComponentEventKind::SelectOption),
        data_view("VirtualList", "Virtual List", "virtual-list")
            .descriptor_kind(UiComponentDescriptorKind::Layout)
            .layout_role(UiComponentLayoutRole::VirtualList)
            .with_prop(int_prop("item_count", 0))
            .with_prop(float_prop("item_extent", 24.0))
            .with_prop(int_prop("overscan", 2))
            .event(UiComponentEventKind::SetVisibleRange)
            .requires_host_capability(UiHostCapability::VirtualizedLayout)
            .requires_render_capability(UiRenderCapability::VirtualizedLayout),
        data_view("TreeView", "Tree View", "tree-view")
            .with_prop(string_prop("query"))
            .with_prop(expanded_prop())
            .slot(UiSlotSchema::new("nodes").multiple(true))
            .events([
                UiComponentEventKind::SelectOption,
                UiComponentEventKind::ToggleExpanded,
                UiComponentEventKind::OpenPopupAt,
                UiComponentEventKind::Commit,
            ]),
        composite("Table", "Table", UiComponentCategory::Collection, "table")
            .with_prop(array_prop("rows"))
            .with_prop(array_prop("columns"))
            .slot(UiSlotSchema::new("header").multiple(true))
            .slot(UiSlotSchema::new("row").multiple(true))
            .events([
                UiComponentEventKind::Hover,
                UiComponentEventKind::Press,
                UiComponentEventKind::SelectOption,
            ]),
        primitive(
            "Typography",
            "Typography",
            UiComponentCategory::Visual,
            "typography",
        )
        .with_prop(text_prop())
        .with_prop(enum_prop("variant", "body1")),
        primitive("Avatar", "Avatar", UiComponentCategory::Visual, "avatar")
            .with_prop(text_prop())
            .with_prop(string_prop("image"))
            .requires_render_capability(UiRenderCapability::Image),
        composite(
            "AvatarGroup",
            "Avatar Group",
            UiComponentCategory::Visual,
            "avatar-group",
        )
        .with_prop(int_prop("max", 4))
        .slot(UiSlotSchema::new("avatars").multiple(true)),
        primitive("Badge", "Badge", UiComponentCategory::Feedback, "badge")
            .with_prop(text_prop())
            .with_prop(value_text_prop()),
        composite(
            "ImageList",
            "Image List",
            UiComponentCategory::Collection,
            "image-list",
        )
        .with_prop(array_prop("items"))
        .with_prop(enum_prop("variant", "standard"))
        .slot(UiSlotSchema::new("items").multiple(true))
        .requires_render_capability(UiRenderCapability::Image),
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
        ]),
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
        ]),
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
        ]),
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
        ]),
    ]
}
