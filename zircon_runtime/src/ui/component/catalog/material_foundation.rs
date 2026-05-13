use std::sync::OnceLock;

use crate::ui::component::UiComponentDescriptorRegistry;
use zircon_runtime_interface::ui::component::{
    UiComponentCategory, UiComponentDescriptor, UiComponentDescriptorKind, UiComponentEventKind,
    UiComponentLayoutRole, UiHostCapability, UiPropSchema, UiRenderCapability, UiSlotSchema,
    UiValue, UiValueKind,
};
use zircon_runtime_interface::ui::skin::{
    FYROX_PANEL_PRESET_ID, JETBRAINS_SHELL_PRESET_ID, MATERIAL_DARK_SKIN_ID,
    UNREAL_WINDOW_MODEL_PRESET_ID,
};

static MATERIAL_EDITOR_FOUNDATION_REGISTRY: OnceLock<UiComponentDescriptorRegistry> =
    OnceLock::new();

impl UiComponentDescriptorRegistry {
    /// Builds the component catalog for the Material Dark editor foundation.
    pub fn material_editor_foundation() -> Self {
        MATERIAL_EDITOR_FOUNDATION_REGISTRY
            .get_or_init(build_material_editor_foundation_registry)
            .clone()
    }
}

fn build_material_editor_foundation_registry() -> UiComponentDescriptorRegistry {
    let mut registry = UiComponentDescriptorRegistry::new();
    for descriptor in material_editor_foundation_descriptors() {
        registry
            .register(descriptor)
            .expect("Material editor foundation descriptors must validate");
    }
    registry
}

fn material_editor_foundation_descriptors() -> Vec<UiComponentDescriptor> {
    vec![
        primitive("Button", "Button", UiComponentCategory::Input, "button")
            .with_prop(text_prop())
            .default_prop("text", UiValue::String("Button".to_string()))
            .event(UiComponentEventKind::Commit),
        primitive(
            "IconButton",
            "Icon Button",
            UiComponentCategory::Input,
            "icon-button",
        )
        .with_prop(text_prop())
        .with_prop(icon_prop())
        .event(UiComponentEventKind::Commit)
        .requires_render_capability(UiRenderCapability::Vector),
        primitive(
            "TextField",
            "Text Field",
            UiComponentCategory::Input,
            "text-field",
        )
        .with_prop(text_prop())
        .event(UiComponentEventKind::ValueChanged)
        .requires_host_capability(UiHostCapability::TextInput),
        primitive(
            "Checkbox",
            "Checkbox",
            UiComponentCategory::Input,
            "checkbox",
        )
        .with_prop(checked_prop())
        .event(UiComponentEventKind::ValueChanged),
        primitive("Switch", "Switch", UiComponentCategory::Input, "switch")
            .with_prop(checked_prop())
            .event(UiComponentEventKind::ValueChanged),
        primitive(
            "Dropdown",
            "Dropdown",
            UiComponentCategory::Selection,
            "dropdown",
        )
        .with_prop(options_prop())
        .with_prop(value_text_prop())
        .event(UiComponentEventKind::ValueChanged),
        primitive("Slider", "Slider", UiComponentCategory::Numeric, "slider")
            .with_prop(number_value_prop())
            .event(UiComponentEventKind::ValueChanged),
        composite("Tabs", "Tabs", UiComponentCategory::Container, "tabs")
            .slot(UiSlotSchema::new("tabs").multiple(true))
            .slot(UiSlotSchema::new("panels").multiple(true))
            .event(UiComponentEventKind::ValueChanged),
        composite("Menu", "Menu", UiComponentCategory::Input, "menu")
            .slot(UiSlotSchema::new("items").multiple(true))
            .event(UiComponentEventKind::Commit),
        composite(
            "Tooltip",
            "Tooltip",
            UiComponentCategory::Feedback,
            "tooltip",
        )
        .slot(UiSlotSchema::new("anchor").required(true))
        .slot(UiSlotSchema::new("content").required(true)),
        primitive(
            "Scrollbar",
            "Scrollbar",
            UiComponentCategory::Numeric,
            "scrollbar",
        )
        .with_prop(number_value_prop())
        .event(UiComponentEventKind::ValueChanged)
        .requires_render_capability(UiRenderCapability::Scroll),
        layout(
            "Splitter",
            "Splitter",
            UiComponentLayoutRole::Size,
            "splitter",
        )
        .with_prop(number_value_prop())
        .event(UiComponentEventKind::ValueChanged),
        composite("Panel", "Panel", UiComponentCategory::Container, "panel")
            .slot(UiSlotSchema::new("header"))
            .slot(UiSlotSchema::new("content").multiple(true)),
        composite("Modal", "Modal", UiComponentCategory::Container, "modal")
            .with_prop(bool_prop("open", false))
            .slot(UiSlotSchema::new("content").multiple(true))
            .event(UiComponentEventKind::ClosePopup),
        composite("Slot", "Slot", UiComponentCategory::Container, "slot")
            .with_prop(
                UiPropSchema::new("name", UiValueKind::String)
                    .default_value(UiValue::String("content".to_string())),
            )
            .slot(UiSlotSchema::new("content").multiple(true)),
        composite(
            "Composite",
            "Composite",
            UiComponentCategory::Container,
            "composite",
        )
        .slot(UiSlotSchema::new("content").multiple(true)),
        layout(
            "FlexGroup",
            "Flex Group",
            UiComponentLayoutRole::Flex,
            "flex-group",
        ),
        layout(
            "HorizontalGroup",
            "Horizontal Group",
            UiComponentLayoutRole::Flex,
            "horizontal-group",
        ),
        layout(
            "VerticalGroup",
            "Vertical Group",
            UiComponentLayoutRole::Flex,
            "vertical-group",
        ),
        layout(
            "GridGroup",
            "Grid Group",
            UiComponentLayoutRole::Grid,
            "grid-group",
        ),
        layout(
            "Overlay",
            "Overlay",
            UiComponentLayoutRole::Overlay,
            "overlay",
        ),
        layout(
            "ScrollView",
            "Scroll View",
            UiComponentLayoutRole::Flex,
            "scroll-view",
        )
        .requires_render_capability(UiRenderCapability::Scroll),
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
        composite(
            "PropertyGrid",
            "Property Grid",
            UiComponentCategory::Container,
            "property-grid",
        )
        .with_prop(string_prop("selection_summary"))
        .slot(UiSlotSchema::new("sections").multiple(true))
        .event(UiComponentEventKind::ValueChanged)
        .requires_host_capability(UiHostCapability::Editor),
        composite(
            "InspectorSection",
            "Inspector Section",
            UiComponentCategory::Container,
            "inspector-section",
        )
        .with_prop(text_prop())
        .with_prop(expanded_prop())
        .slot(UiSlotSchema::new("fields").multiple(true))
        .event(UiComponentEventKind::ToggleExpanded)
        .requires_host_capability(UiHostCapability::Editor),
        editor_panel_component(
            "SearchField",
            "Search Field",
            UiComponentCategory::Input,
            "search-field",
        )
        .with_prop(string_prop("query"))
        .events([
            UiComponentEventKind::ValueChanged,
            UiComponentEventKind::Commit,
        ])
        .requires_host_capability(UiHostCapability::TextInput),
        editor_panel_component(
            "ContextMenu",
            "Context Menu",
            UiComponentCategory::Input,
            "context-menu",
        )
        .slot(UiSlotSchema::new("items").multiple(true))
        .events([
            UiComponentEventKind::OpenPopupAt,
            UiComponentEventKind::SelectOption,
            UiComponentEventKind::ClosePopup,
            UiComponentEventKind::Commit,
        ]),
        editor_panel_component(
            "FieldEditor",
            "Field Editor",
            UiComponentCategory::Container,
            "field-editor",
        )
        .with_prop(text_prop())
        .with_prop(value_text_prop())
        .slot(UiSlotSchema::new("field").required(true))
        .events([
            UiComponentEventKind::ValueChanged,
            UiComponentEventKind::Commit,
        ]),
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
            "PreviewPane",
            "Preview Pane",
            UiComponentCategory::Container,
            "preview-pane",
        )
        .with_prop(string_prop("asset_id"))
        .slot(UiSlotSchema::new("content").multiple(true))
        .requires_host_capability(UiHostCapability::ImageRender)
        .requires_render_capability(UiRenderCapability::Image),
        editor_panel_component(
            "MetadataPane",
            "Metadata Pane",
            UiComponentCategory::Container,
            "metadata-pane",
        )
        .with_prop(string_prop("asset_id"))
        .slot(UiSlotSchema::new("fields").multiple(true))
        .event(UiComponentEventKind::ValueChanged),
        editor_panel_layout(
            "ViewportHost",
            "Viewport Host",
            UiComponentLayoutRole::Canvas,
            "viewport-host",
        )
        .with_prop(string_prop("camera_target"))
        .slot(UiSlotSchema::new("overlays").multiple(true))
        .events([
            UiComponentEventKind::SetWorldSurface,
            UiComponentEventKind::SetWorldTransform,
        ])
        .requires_host_capability(UiHostCapability::CanvasRender)
        .requires_render_capability(UiRenderCapability::Canvas),
        editor_panel_component(
            "PaneToolbar",
            "Pane Toolbar",
            UiComponentCategory::Container,
            "pane-toolbar",
        )
        .slot(UiSlotSchema::new("actions").multiple(true))
        .event(UiComponentEventKind::Commit),
        editor_panel_component(
            "GizmoControls",
            "Gizmo Controls",
            UiComponentCategory::Input,
            "gizmo-controls",
        )
        .with_prop(enum_prop("mode", "translate"))
        .events([
            UiComponentEventKind::BeginDrag,
            UiComponentEventKind::DragDelta,
            UiComponentEventKind::EndDrag,
            UiComponentEventKind::SetWorldTransform,
        ]),
        editor_panel_component(
            "FilterBar",
            "Filter Bar",
            UiComponentCategory::Input,
            "filter-bar",
        )
        .with_prop(string_prop("query"))
        .with_prop(enum_prop("severity", "all"))
        .slot(UiSlotSchema::new("filters").multiple(true))
        .events([
            UiComponentEventKind::ValueChanged,
            UiComponentEventKind::SelectOption,
        ]),
        editor_panel_component(
            "SeverityChips",
            "Severity Chips",
            UiComponentCategory::Selection,
            "severity-chips",
        )
        .with_prop(enum_prop("selected_severity", "all"))
        .event(UiComponentEventKind::SelectOption),
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
        editor_panel_component(
            "StatusActionControls",
            "Status Action Controls",
            UiComponentCategory::Feedback,
            "status-action-controls",
        )
        .with_prop(string_prop("status"))
        .slot(UiSlotSchema::new("actions").multiple(true))
        .events([
            UiComponentEventKind::Commit,
            UiComponentEventKind::ValueChanged,
        ]),
        editor_panel_layout(
            "GraphCanvas",
            "Graph Canvas",
            UiComponentLayoutRole::Canvas,
            "graph-canvas",
        )
        .slot(UiSlotSchema::new("nodes").multiple(true))
        .slot(UiSlotSchema::new("edges").multiple(true))
        .events([
            UiComponentEventKind::SelectOption,
            UiComponentEventKind::BeginDrag,
            UiComponentEventKind::EndDrag,
            UiComponentEventKind::DropHover,
        ])
        .requires_host_capability(UiHostCapability::CanvasRender)
        .requires_render_capability(UiRenderCapability::Canvas),
        editor_panel_component(
            "SourceEditor",
            "Source Editor",
            UiComponentCategory::Input,
            "source-editor",
        )
        .with_prop(text_prop())
        .events([
            UiComponentEventKind::ValueChanged,
            UiComponentEventKind::Commit,
        ])
        .requires_host_capability(UiHostCapability::TextInput),
        editor_panel_component(
            "Timeline",
            "Timeline",
            UiComponentCategory::Numeric,
            "timeline",
        )
        .with_prop(float_prop("time", 0.0))
        .with_prop(float_prop("duration", 0.0))
        .events([
            UiComponentEventKind::ValueChanged,
            UiComponentEventKind::DragDelta,
        ]),
        editor_panel_layout(
            "VisualDesigner",
            "Visual Designer",
            UiComponentLayoutRole::Canvas,
            "visual-designer",
        )
        .slot(UiSlotSchema::new("content").multiple(true))
        .slot(UiSlotSchema::new("overlays").multiple(true))
        .events([
            UiComponentEventKind::SelectOption,
            UiComponentEventKind::BeginDrag,
            UiComponentEventKind::EndDrag,
            UiComponentEventKind::DropHover,
        ])
        .requires_host_capability(UiHostCapability::CanvasRender)
        .requires_render_capability(UiRenderCapability::Canvas),
        shell("Drawer", "Drawer", "drawer")
            .with_prop(enum_prop("slot", "left_top"))
            .with_prop(enum_prop("mode", "pinned"))
            .with_prop(string_prop("active_view"))
            .slot(UiSlotSchema::new("tabs").multiple(true))
            .slot(UiSlotSchema::new("content").multiple(true))
            .event(UiComponentEventKind::SelectOption),
        shell("View", "View", "view")
            .with_prop(required_string_prop("view_id"))
            .with_prop(text_prop())
            .with_prop(bool_prop("dirty", false))
            .slot(UiSlotSchema::new("content").multiple(true))
            .event(UiComponentEventKind::Focus),
        shell("ViewTab", "View Tab", "view-tab")
            .with_prop(required_string_prop("view_id"))
            .with_prop(text_prop())
            .events([
                UiComponentEventKind::Commit,
                UiComponentEventKind::BeginDrag,
                UiComponentEventKind::EndDrag,
            ]),
        shell("Window", "Window", "window")
            .with_prop(required_string_prop("window_id"))
            .with_prop(text_prop())
            .with_prop(enum_prop("dock_policy", "main_workbench"))
            .with_prop(bool_prop("floating", false))
            .slot(UiSlotSchema::new("views").multiple(true))
            .events([
                UiComponentEventKind::Focus,
                UiComponentEventKind::BeginDrag,
                UiComponentEventKind::EndDrag,
            ]),
        shell("WindowFrame", "Window Frame", "window-frame")
            .with_prop(text_prop())
            .slot(UiSlotSchema::new("chrome"))
            .slot(UiSlotSchema::new("content").multiple(true))
            .event(UiComponentEventKind::ClosePopup),
        shell("DocumentNode", "Document Node", "document-node")
            .descriptor_kind(UiComponentDescriptorKind::Layout)
            .layout_role(UiComponentLayoutRole::EditorDock)
            .with_prop(enum_prop("node_kind", "tabs"))
            .slot(UiSlotSchema::new("content").multiple(true)),
        shell("TabStack", "Tab Stack", "tab-stack")
            .with_prop(string_prop("active_tab"))
            .slot(UiSlotSchema::new("tabs").multiple(true))
            .slot(UiSlotSchema::new("content").multiple(true))
            .event(UiComponentEventKind::SelectOption),
        shell("FloatingWindow", "Floating Window", "floating-window")
            .with_prop(required_string_prop("window_id"))
            .with_prop(string_prop("focused_view"))
            .slot(UiSlotSchema::new("content").multiple(true))
            .events([
                UiComponentEventKind::Focus,
                UiComponentEventKind::BeginDrag,
                UiComponentEventKind::EndDrag,
            ]),
        shell("DockHost", "Dock Host", "dock-host")
            .descriptor_kind(UiComponentDescriptorKind::Layout)
            .layout_role(UiComponentLayoutRole::EditorDock)
            .with_prop(string_prop("active_window"))
            .slot(UiSlotSchema::new("windows").multiple(true))
            .events([UiComponentEventKind::Focus, UiComponentEventKind::DropHover]),
        shell("WorkbenchShell", "Workbench Shell", "workbench-shell")
            .with_prop(default_string_prop("skin_id", MATERIAL_DARK_SKIN_ID))
            .with_prop(default_string_prop(
                "panel_preset_id",
                FYROX_PANEL_PRESET_ID,
            ))
            .with_prop(default_string_prop(
                "shell_preset_id",
                JETBRAINS_SHELL_PRESET_ID,
            ))
            .with_prop(default_string_prop(
                "window_model_preset_id",
                UNREAL_WINDOW_MODEL_PRESET_ID,
            ))
            .with_prop(string_prop("active_window"))
            .slot(UiSlotSchema::new("drawers").multiple(true))
            .slot(UiSlotSchema::new("documents").multiple(true))
            .slot(UiSlotSchema::new("status"))
            .slot(UiSlotSchema::new("menu"))
            .event(UiComponentEventKind::Commit),
    ]
}

fn primitive(
    id: &str,
    display_name: &str,
    category: UiComponentCategory,
    role: &str,
) -> UiComponentDescriptor {
    with_material_defaults(UiComponentDescriptor::new(id, display_name, category, role))
}

fn composite(
    id: &str,
    display_name: &str,
    category: UiComponentCategory,
    role: &str,
) -> UiComponentDescriptor {
    with_material_defaults(UiComponentDescriptor::new(id, display_name, category, role))
        .descriptor_kind(UiComponentDescriptorKind::Composite)
}

fn layout(
    id: &str,
    display_name: &str,
    layout_role: UiComponentLayoutRole,
    role: &str,
) -> UiComponentDescriptor {
    with_material_defaults(UiComponentDescriptor::new(
        id,
        display_name,
        UiComponentCategory::Container,
        role,
    ))
    .descriptor_kind(UiComponentDescriptorKind::Layout)
    .layout_role(layout_role)
    .slot(UiSlotSchema::new("content").multiple(true))
}

fn data_view(id: &str, display_name: &str, role: &str) -> UiComponentDescriptor {
    with_material_defaults(UiComponentDescriptor::new(
        id,
        display_name,
        UiComponentCategory::Collection,
        role,
    ))
    .descriptor_kind(UiComponentDescriptorKind::Composite)
    .requires_render_capability(UiRenderCapability::Scroll)
}

fn editor_panel_component(
    id: &str,
    display_name: &str,
    category: UiComponentCategory,
    role: &str,
) -> UiComponentDescriptor {
    with_material_defaults(UiComponentDescriptor::new(id, display_name, category, role))
        .descriptor_kind(UiComponentDescriptorKind::Composite)
        .requires_host_capability(UiHostCapability::Editor)
}

fn editor_panel_layout(
    id: &str,
    display_name: &str,
    layout_role: UiComponentLayoutRole,
    role: &str,
) -> UiComponentDescriptor {
    editor_panel_component(id, display_name, UiComponentCategory::Container, role)
        .descriptor_kind(UiComponentDescriptorKind::Layout)
        .layout_role(layout_role)
}

fn shell(id: &str, display_name: &str, role: &str) -> UiComponentDescriptor {
    with_material_defaults(UiComponentDescriptor::new(
        id,
        display_name,
        UiComponentCategory::Container,
        role,
    ))
    .descriptor_kind(UiComponentDescriptorKind::EditorOnly)
    .requires_host_capability(UiHostCapability::Editor)
}

fn with_material_defaults(descriptor: UiComponentDescriptor) -> UiComponentDescriptor {
    descriptor
        .default_class(MATERIAL_DARK_SKIN_ID)
        .default_class("material")
        .default_class("material-dark")
        .state(bool_prop("hovered", false))
        .state(bool_prop("pressed", false))
        .state(bool_prop("selected", false))
        .state(bool_prop("disabled", false))
        .state(bool_prop("focused", false))
        .state(bool_prop("error", false))
        .state(bool_prop("warning", false))
        .with_prop(density_prop())
}

fn density_prop() -> UiPropSchema {
    UiPropSchema::new("density", UiValueKind::Enum)
        .default_value(UiValue::Enum("compact".to_string()))
}

fn text_prop() -> UiPropSchema {
    string_prop("text")
}

fn icon_prop() -> UiPropSchema {
    string_prop("icon")
}

fn checked_prop() -> UiPropSchema {
    bool_prop("checked", false)
}

fn expanded_prop() -> UiPropSchema {
    bool_prop("expanded", true)
}

fn bool_prop(name: &str, default: bool) -> UiPropSchema {
    UiPropSchema::new(name, UiValueKind::Bool).default_value(UiValue::Bool(default))
}

fn string_prop(name: &str) -> UiPropSchema {
    default_string_prop(name, "")
}

fn required_string_prop(name: &str) -> UiPropSchema {
    string_prop(name).required(true)
}

fn default_string_prop(name: &str, default: &str) -> UiPropSchema {
    UiPropSchema::new(name, UiValueKind::String).default_value(UiValue::String(default.to_string()))
}

fn enum_prop(name: &str, default: &str) -> UiPropSchema {
    UiPropSchema::new(name, UiValueKind::Enum).default_value(UiValue::Enum(default.to_string()))
}

fn int_prop(name: &str, default: i64) -> UiPropSchema {
    UiPropSchema::new(name, UiValueKind::Int).default_value(UiValue::Int(default))
}

fn float_prop(name: &str, default: f64) -> UiPropSchema {
    UiPropSchema::new(name, UiValueKind::Float).default_value(UiValue::Float(default))
}

fn value_text_prop() -> UiPropSchema {
    string_prop("value_text")
}

fn options_prop() -> UiPropSchema {
    UiPropSchema::new("options", UiValueKind::Array).default_value(UiValue::Array(Vec::new()))
}

fn number_value_prop() -> UiPropSchema {
    UiPropSchema::new("value", UiValueKind::Float)
        .default_value(UiValue::Float(0.0))
        .range(0.0, 1.0)
        .step(0.01)
}
