use std::collections::BTreeMap;

use toml::Value;

use crate::ui::component::UiComponentDescriptorRegistry;
use zircon_runtime_interface::ui::component::{
    UiComponentCategory, UiComponentEventKind, UiDragPayloadKind, UiDropPolicy, UiHostCapability,
    UiValue, UiValueKind,
};

use zircon_runtime_interface::ui::component::{
    UiComponentDescriptor, UiDefaultNodeTemplate, UiOptionDescriptor, UiPaletteMetadata,
    UiPropSchema, UiRenderCapability, UiSlotSchema, UiWidgetEditorFallback, UiWidgetFallbackPolicy,
    UiWidgetRuntimeFallback,
};

impl UiComponentDescriptorRegistry {
    /// Builds the Runtime UI component catalog used by the editor showcase.
    pub fn editor_showcase() -> Self {
        let mut registry = Self::new();
        for descriptor in editor_showcase_descriptors() {
            registry
                .register(descriptor)
                .expect("built-in UI component descriptors must validate");
        }
        registry
    }
}

fn editor_showcase_descriptors() -> Vec<UiComponentDescriptor> {
    vec![
        layout_primitive("Container", "Container", "container"),
        layout_primitive("Overlay", "Overlay", "overlay"),
        layout_primitive("HorizontalBox", "Horizontal Box", "horizontal-box"),
        layout_primitive("VerticalBox", "Vertical Box", "vertical-box"),
        layout_primitive("FlowBox", "Flow Box", "flow-box"),
        layout_primitive("GridBox", "Grid Box", "grid-box"),
        layout_primitive("ScrollableBox", "Scrollable Box", "scrollable-box"),
        layout_primitive("Space", "Space", "space"),
        visual("Label", "Label", "label")
            .with_prop(text_prop())
            .default_prop("text", UiValue::String("Label".to_string()))
            .state(state_text_prop())
            .event(UiComponentEventKind::ValueChanged)
            .requires_render_capability(UiRenderCapability::Text),
        visual("RichLabel", "Rich Label", "rich-label")
            .with_prop(text_prop())
            .state(state_text_prop())
            .requires_render_capability(UiRenderCapability::Text),
        visual("Image", "Image", "image")
            .with_prop(UiPropSchema::new("value", UiValueKind::AssetRef))
            .with_prop(UiPropSchema::new("image", UiValueKind::AssetRef))
            .state(UiPropSchema::new("image", UiValueKind::AssetRef))
            .requires_host_capability(UiHostCapability::ImageRender)
            .requires_render_capability(UiRenderCapability::Image),
        visual("Icon", "Icon", "icon")
            .with_prop(UiPropSchema::new("value", UiValueKind::String))
            .with_prop(UiPropSchema::new("icon", UiValueKind::String))
            .state(UiPropSchema::new("icon", UiValueKind::String))
            .requires_render_capability(UiRenderCapability::Vector),
        visual("SvgIcon", "SVG Icon", "svg-icon")
            .with_prop(UiPropSchema::new("source", UiValueKind::String))
            .state(UiPropSchema::new("source", UiValueKind::String))
            .requires_render_capability(UiRenderCapability::Vector),
        visual("Separator", "Separator", "separator")
            .with_prop(text_prop())
            .requires_render_capability(UiRenderCapability::Primitive),
        feedback("ProgressBar", "Progress Bar", "progress-bar")
            .with_prop(
                UiPropSchema::new("value", UiValueKind::Float)
                    .default_value(UiValue::Float(0.5))
                    .range(0.0, 1.0),
            )
            .with_prop(validation_level_prop())
            .state(state_float_prop("value", 0.5))
            .state(state_bool_prop("focused", false)),
        feedback("Spinner", "Spinner", "spinner")
            .with_prop(text_prop())
            .state(state_bool_prop("focused", false)),
        feedback("Badge", "Badge", "badge")
            .with_prop(text_prop())
            .state(state_text_prop()),
        feedback("HelpRow", "Help Row", "help-row")
            .with_prop(text_prop())
            .with_prop(validation_level_prop())
            .with_prop(validation_message_prop())
            .state(state_text_prop()),
        input("Button", "Button", "button")
            .with_prop(text_prop())
            .default_prop("text", UiValue::String("Button".to_string()))
            .with_prop(validation_level_prop())
            .state(state_bool_prop("focused", false))
            .state(state_bool_prop("disabled", false))
            .event(UiComponentEventKind::Focus)
            .event(UiComponentEventKind::Commit),
        input("IconButton", "Icon Button", "icon-button")
            .with_prop(UiPropSchema::new("icon", UiValueKind::String))
            .with_prop(text_prop())
            .state(state_bool_prop("focused", false))
            .state(state_bool_prop("disabled", false))
            .event(UiComponentEventKind::Focus)
            .event(UiComponentEventKind::Commit)
            .requires_render_capability(UiRenderCapability::Vector),
        input("ToggleButton", "Toggle Button", "toggle-button")
            .with_prop(bool_value_prop(false))
            .with_prop(bool_prop("checked", false))
            .with_prop(text_prop())
            .state(state_bool_prop("value", false))
            .state(state_bool_prop("checked", false))
            .state(state_bool_prop("focused", false))
            .state(state_bool_prop("disabled", false))
            .event(UiComponentEventKind::Focus)
            .event(UiComponentEventKind::ValueChanged),
        input("Checkbox", "Checkbox", "checkbox")
            .with_prop(bool_value_prop(false))
            .with_prop(bool_prop("checked", false))
            .with_prop(text_prop())
            .state(state_bool_prop("value", false))
            .state(state_bool_prop("checked", false))
            .state(state_bool_prop("focused", false))
            .state(state_bool_prop("disabled", false))
            .event(UiComponentEventKind::Focus)
            .event(UiComponentEventKind::ValueChanged),
        input("Radio", "Radio", "radio")
            .with_prop(bool_value_prop(false))
            .with_prop(bool_prop("checked", false))
            .with_prop(text_prop())
            .state(state_bool_prop("value", false))
            .state(state_bool_prop("checked", false))
            .state(state_bool_prop("focused", false))
            .state(state_bool_prop("disabled", false))
            .event(UiComponentEventKind::Focus)
            .event(UiComponentEventKind::ValueChanged),
        input("SegmentedControl", "Segmented Control", "segmented-control")
            .with_prop(options_prop())
            .with_prop(UiPropSchema::new("value", UiValueKind::Enum))
            .with_prop(selection_state_prop())
            .event(UiComponentEventKind::Focus)
            .state(state_bool_prop("focused", false))
            .state(state_bool_prop("selected", false))
            .state(state_bool_prop("disabled", false))
            .event(UiComponentEventKind::SelectOption),
        input_field("InputField", "Input Field"),
        input_field("TextField", "Text Field"),
        numeric("NumberField", "Number Field", "number-field")
            .with_prop(number_value_prop())
            .with_prop(validation_level_prop())
            .event(UiComponentEventKind::Focus)
            .with_prop(
                UiPropSchema::new("min", UiValueKind::Float).default_value(UiValue::Float(0.0)),
            )
            .with_prop(
                UiPropSchema::new("max", UiValueKind::Float).default_value(UiValue::Float(100.0)),
            )
            .with_prop(
                UiPropSchema::new("step", UiValueKind::Float).default_value(UiValue::Float(1.0)),
            )
            .with_prop(
                UiPropSchema::new("large_step", UiValueKind::Float)
                    .default_value(UiValue::Float(10.0)),
            )
            .state(state_bool_prop("focused", false))
            .state(state_bool_prop("dragging", false))
            .state(state_bool_prop("disabled", false))
            .events([
                UiComponentEventKind::BeginDrag,
                UiComponentEventKind::DragDelta,
                UiComponentEventKind::LargeDragDelta,
                UiComponentEventKind::EndDrag,
                UiComponentEventKind::Commit,
                UiComponentEventKind::ValueChanged,
            ]),
        numeric("RangeField", "Range Field", "range-field")
            .with_prop(number_value_prop())
            .with_prop(validation_level_prop())
            .event(UiComponentEventKind::Focus)
            .with_prop(
                UiPropSchema::new("min", UiValueKind::Float).default_value(UiValue::Float(0.0)),
            )
            .with_prop(
                UiPropSchema::new("max", UiValueKind::Float).default_value(UiValue::Float(100.0)),
            )
            .with_prop(
                UiPropSchema::new("step", UiValueKind::Float).default_value(UiValue::Float(1.0)),
            )
            .state(state_bool_prop("focused", false))
            .state(state_bool_prop("dragging", false))
            .state(state_bool_prop("disabled", false))
            .events([
                UiComponentEventKind::DragDelta,
                UiComponentEventKind::Commit,
                UiComponentEventKind::ValueChanged,
            ]),
        numeric("ColorField", "Color Field", "color-field")
            .with_prop(UiPropSchema::new("value", UiValueKind::Color))
            .with_prop(validation_level_prop())
            .state(state_bool_prop("focused", false))
            .state(state_bool_prop("disabled", false))
            .events([
                UiComponentEventKind::Focus,
                UiComponentEventKind::ValueChanged,
                UiComponentEventKind::Commit,
            ]),
        numeric("Vector2Field", "Vector2 Field", "vector2-field")
            .with_prop(UiPropSchema::new("value", UiValueKind::Vec2))
            .with_prop(validation_level_prop())
            .with_prop(value_text_prop())
            .state(state_bool_prop("focused", false))
            .state(state_bool_prop("disabled", false))
            .events([
                UiComponentEventKind::Focus,
                UiComponentEventKind::ValueChanged,
                UiComponentEventKind::Commit,
            ]),
        numeric("Vector3Field", "Vector3 Field", "vector3-field")
            .with_prop(UiPropSchema::new("value", UiValueKind::Vec3))
            .with_prop(validation_level_prop())
            .with_prop(value_text_prop())
            .state(state_bool_prop("focused", false))
            .state(state_bool_prop("disabled", false))
            .events([
                UiComponentEventKind::Focus,
                UiComponentEventKind::ValueChanged,
                UiComponentEventKind::Commit,
            ]),
        numeric("Vector4Field", "Vector4 Field", "vector4-field")
            .with_prop(UiPropSchema::new("value", UiValueKind::Vec4))
            .with_prop(validation_level_prop())
            .with_prop(value_text_prop())
            .state(state_bool_prop("focused", false))
            .state(state_bool_prop("disabled", false))
            .events([
                UiComponentEventKind::Focus,
                UiComponentEventKind::ValueChanged,
                UiComponentEventKind::Commit,
            ]),
        selection("Dropdown", "Dropdown", "dropdown", UiValueKind::Enum),
        selection("ComboBox", "Combo Box", "combo-box", UiValueKind::Enum),
        selection("EnumField", "Enum Field", "enum-field", UiValueKind::Enum),
        selection(
            "FlagsField",
            "Flags Field",
            "flags-field",
            UiValueKind::Flags,
        )
        .with_prop(UiPropSchema::new("query", UiValueKind::String)),
        selection(
            "SearchSelect",
            "Search Select",
            "search-select",
            UiValueKind::Enum,
        )
        .with_prop(UiPropSchema::new("query", UiValueKind::String))
        .state(state_string_prop("query")),
        reference(
            "AssetField",
            "Asset Field",
            "asset-field",
            [UiDragPayloadKind::Asset],
        )
        .with_prop(UiPropSchema::new("value", UiValueKind::AssetRef))
        .with_prop(validation_level_prop())
        .event(UiComponentEventKind::Focus)
        .state(state_bool_prop("focused", false))
        .state(state_bool_prop("dragging", false))
        .state(state_bool_prop("drop_hovered", false))
        .state(state_bool_prop("active_drag_target", false))
        .state(state_bool_prop("disabled", false)),
        reference(
            "InstanceField",
            "Instance Field",
            "instance-field",
            [UiDragPayloadKind::SceneInstance],
        )
        .with_prop(UiPropSchema::new("value", UiValueKind::InstanceRef))
        .with_prop(validation_level_prop())
        .event(UiComponentEventKind::Focus)
        .state(state_bool_prop("focused", false))
        .state(state_bool_prop("dragging", false))
        .state(state_bool_prop("drop_hovered", false))
        .state(state_bool_prop("active_drag_target", false))
        .state(state_bool_prop("disabled", false)),
        reference(
            "ObjectField",
            "Object Field",
            "object-field",
            [
                UiDragPayloadKind::Asset,
                UiDragPayloadKind::SceneInstance,
                UiDragPayloadKind::Object,
            ],
        )
        .with_prop(UiPropSchema::new("value", UiValueKind::InstanceRef))
        .with_prop(validation_level_prop())
        .event(UiComponentEventKind::Focus)
        .state(state_bool_prop("focused", false))
        .state(state_bool_prop("dragging", false))
        .state(state_bool_prop("drop_hovered", false))
        .state(state_bool_prop("active_drag_target", false))
        .state(state_bool_prop("disabled", false)),
        container_descriptor("Group", "Group", "group")
            .with_prop(expanded_prop())
            .with_prop(validation_level_prop())
            .with_prop(text_prop())
            .event(UiComponentEventKind::Focus)
            .state(expanded_prop())
            .state(state_bool_prop("focused", false))
            .state(state_bool_prop("disabled", false))
            .slot(UiSlotSchema::new("content").multiple(true))
            .event(UiComponentEventKind::ToggleExpanded),
        container_descriptor("Foldout", "Foldout", "foldout")
            .with_prop(expanded_prop())
            .with_prop(validation_level_prop())
            .with_prop(text_prop())
            .event(UiComponentEventKind::Focus)
            .state(state_bool_prop("expanded", false))
            .state(state_bool_prop("focused", false))
            .state(state_bool_prop("disabled", false))
            .slot(UiSlotSchema::new("content").multiple(true))
            .event(UiComponentEventKind::ToggleExpanded),
        container_descriptor("PropertyRow", "Property Row", "property-row")
            .with_prop(text_prop())
            .with_prop(UiPropSchema::new("value", UiValueKind::String))
            .slot(UiSlotSchema::new("label"))
            .slot(UiSlotSchema::new("field")),
        container_descriptor("InspectorSection", "Inspector Section", "inspector-section")
            .with_prop(text_prop())
            .with_prop(expanded_prop())
            .slot(UiSlotSchema::new("content").multiple(true))
            .state(state_bool_prop("expanded", true))
            .event(UiComponentEventKind::ToggleExpanded),
        container_descriptor(
            "WorldSpaceSurface",
            "World Space Surface",
            "world-space-surface",
        )
        .with_prop(
            UiPropSchema::new("world_position", UiValueKind::Vec3)
                .default_value(UiValue::Vec3([0.0, 0.0, 0.0])),
        )
        .with_prop(
            UiPropSchema::new("world_rotation", UiValueKind::Vec3)
                .default_value(UiValue::Vec3([0.0, 0.0, 0.0])),
        )
        .with_prop(
            UiPropSchema::new("world_scale", UiValueKind::Vec3)
                .default_value(UiValue::Vec3([1.0, 1.0, 1.0])),
        )
        .with_prop(
            UiPropSchema::new("world_size", UiValueKind::Vec2)
                .default_value(UiValue::Vec2([1.0, 1.0])),
        )
        .with_prop(
            UiPropSchema::new("pixels_per_meter", UiValueKind::Float)
                .default_value(UiValue::Float(100.0))
                .range(1.0, 8192.0),
        )
        .with_prop(bool_prop("billboard", false))
        .with_prop(bool_prop("depth_test", true))
        .with_prop(int_prop("render_order", 0))
        .with_prop(UiPropSchema::new("camera_target", UiValueKind::String))
        .slot(UiSlotSchema::new("content").multiple(true))
        .state(UiPropSchema::new("world_position", UiValueKind::Vec3))
        .state(UiPropSchema::new("world_rotation", UiValueKind::Vec3))
        .state(UiPropSchema::new("world_scale", UiValueKind::Vec3))
        .state(UiPropSchema::new("world_size", UiValueKind::Vec2))
        .state(state_float_prop("pixels_per_meter", 100.0))
        .state(state_bool_prop("billboard", false))
        .state(state_bool_prop("depth_test", true))
        .state(state_int_prop("render_order", 0))
        .state(state_string_prop("camera_target"))
        .event(UiComponentEventKind::SetWorldTransform)
        .event(UiComponentEventKind::SetWorldSurface),
        collection("ArrayField", "Array Field", "array-field")
            .with_prop(UiPropSchema::new("items", UiValueKind::Array))
            .with_prop(UiPropSchema::new("element_type", UiValueKind::String))
            .with_prop(validation_level_prop())
            .with_prop(value_text_prop())
            .event(UiComponentEventKind::Focus)
            .state(state_array_prop("items"))
            .state(state_bool_prop("focused", false))
            .events([
                UiComponentEventKind::AddElement,
                UiComponentEventKind::SetElement,
                UiComponentEventKind::RemoveElement,
                UiComponentEventKind::MoveElement,
                UiComponentEventKind::ValueChanged,
            ]),
        collection("MapField", "Map Field", "map-field")
            .with_prop(UiPropSchema::new("entries", UiValueKind::Map))
            .with_prop(UiPropSchema::new("key_type", UiValueKind::String))
            .with_prop(UiPropSchema::new("value_type", UiValueKind::String))
            .with_prop(validation_level_prop())
            .with_prop(value_text_prop())
            .event(UiComponentEventKind::Focus)
            .state(state_map_prop("entries"))
            .state(state_bool_prop("focused", false))
            .events([
                UiComponentEventKind::AddMapEntry,
                UiComponentEventKind::SetMapEntry,
                UiComponentEventKind::RenameMapKey,
                UiComponentEventKind::RemoveMapEntry,
                UiComponentEventKind::ValueChanged,
            ]),
        collection("ListRow", "List Row", "list-row")
            .with_prop(text_prop())
            .with_prop(UiPropSchema::new("value", UiValueKind::String))
            .with_prop(bool_prop("selected", false))
            .with_prop(bool_prop("focused", false))
            .with_prop(bool_prop("hovered", false))
            .with_prop(bool_prop("pressed", false))
            .event(UiComponentEventKind::Focus)
            .event(UiComponentEventKind::Hover)
            .event(UiComponentEventKind::Press)
            .state(state_text_prop())
            .state(state_bool_prop("selected", false))
            .state(state_bool_prop("focused", false))
            .state(state_bool_prop("hovered", false))
            .state(state_bool_prop("pressed", false)),
        collection("VirtualList", "Virtual List", "virtual-list")
            .with_prop(UiPropSchema::new("items", UiValueKind::Array))
            .with_prop(UiPropSchema::new("data_source", UiValueKind::String))
            .with_prop(int_prop("total_count", 0))
            .with_prop(int_prop("viewport_start", 0))
            .with_prop(int_prop("viewport_count", 0))
            .with_prop(int_prop("visible_end", 0))
            .with_prop(int_prop("requested_start", 0))
            .with_prop(int_prop("requested_count", 0))
            .with_prop(int_prop("selected_index", -1))
            .with_prop(UiPropSchema::new("scroll_offset", UiValueKind::Float))
            .with_prop(bool_prop("loading", false))
            .with_prop(
                UiPropSchema::new("item_extent", UiValueKind::Float)
                    .default_value(UiValue::Float(24.0))
                    .range(1.0, 4096.0),
            )
            .with_prop(int_prop("overscan", 2))
            .with_prop(validation_level_prop())
            .slot(UiSlotSchema::new("row").multiple(true))
            .state(state_array_prop("items"))
            .state(state_string_prop("data_source"))
            .state(state_int_prop("total_count", 0))
            .state(state_int_prop("viewport_start", 0))
            .state(state_int_prop("viewport_count", 0))
            .state(state_int_prop("visible_end", 0))
            .state(state_int_prop("requested_start", 0))
            .state(state_int_prop("requested_count", 0))
            .state(state_int_prop("selected_index", -1))
            .state(state_float_prop("scroll_offset", 0.0))
            .state(state_bool_prop("loading", false))
            .state(state_float_prop("item_extent", 24.0))
            .state(state_int_prop("overscan", 2))
            .event(UiComponentEventKind::ValueChanged)
            .event(UiComponentEventKind::SetVisibleRange)
            .requires_host_capability(UiHostCapability::VirtualizedLayout)
            .requires_render_capability(UiRenderCapability::VirtualizedLayout),
        collection("PagedList", "Paged List", "paged-list")
            .with_prop(UiPropSchema::new("items", UiValueKind::Array))
            .with_prop(UiPropSchema::new("data_source", UiValueKind::String))
            .with_prop(int_prop("total_count", 0))
            .with_prop(int_prop("page_index", 0))
            .with_prop(int_prop("page_size", 50))
            .with_prop(int_prop("page_count", 0))
            .with_prop(int_prop("page_start", 0))
            .with_prop(int_prop("page_end", 0))
            .with_prop(bool_prop("loading", false))
            .with_prop(bool_prop("empty", false))
            .with_prop(validation_level_prop())
            .slot(UiSlotSchema::new("page").multiple(true))
            .state(state_array_prop("items"))
            .state(state_string_prop("data_source"))
            .state(state_int_prop("total_count", 0))
            .state(state_int_prop("page_index", 0))
            .state(state_int_prop("page_size", 50))
            .state(state_int_prop("page_count", 0))
            .state(state_int_prop("page_start", 0))
            .state(state_int_prop("page_end", 0))
            .state(state_bool_prop("loading", false))
            .state(state_bool_prop("empty", false))
            .event(UiComponentEventKind::ValueChanged)
            .event(UiComponentEventKind::SetPage),
        collection("TreeRow", "Tree Row", "tree-row")
            .with_prop(text_prop())
            .with_prop(expanded_prop())
            .with_prop(UiPropSchema::new("tree_depth", UiValueKind::Int))
            .with_prop(UiPropSchema::new("tree_indent_px", UiValueKind::Float))
            .event(UiComponentEventKind::Focus)
            .event(UiComponentEventKind::Hover)
            .event(UiComponentEventKind::Press)
            .state(state_text_prop())
            .state(state_bool_prop("expanded", false))
            .state(state_bool_prop("selected", false))
            .state(state_bool_prop("focused", false))
            .state(state_bool_prop("hovered", false))
            .state(state_bool_prop("pressed", false))
            .event(UiComponentEventKind::ToggleExpanded),
        input(
            "ContextActionMenu",
            "Context Action Menu",
            "context-action-menu",
        )
        .with_prop(options_prop())
        .with_prop(UiPropSchema::new("value", UiValueKind::String))
        .with_prop(bool_prop("popup_open", false))
        .with_prop(UiPropSchema::new("popup_anchor_x", UiValueKind::Float))
        .with_prop(UiPropSchema::new("popup_anchor_y", UiValueKind::Float))
        .with_prop(UiPropSchema::new("menu_items", UiValueKind::Array))
        .event(UiComponentEventKind::Focus)
        .state(state_bool_prop("focused", false))
        .state(state_bool_prop("selected", false))
        .state(state_bool_prop("popup_open", false))
        .state(UiPropSchema::new("popup_anchor_x", UiValueKind::Float))
        .state(UiPropSchema::new("popup_anchor_y", UiValueKind::Float))
        .events([
            UiComponentEventKind::OpenPopup,
            UiComponentEventKind::OpenPopupAt,
            UiComponentEventKind::ClosePopup,
            UiComponentEventKind::SelectOption,
        ]),
    ]
    .into_iter()
    .map(with_palette_metadata)
    .collect()
}

fn base_descriptor(
    id: &str,
    display_name: &str,
    category: UiComponentCategory,
    role: &str,
) -> UiComponentDescriptor {
    UiComponentDescriptor::new(id, display_name, category, role)
        .requires_host_capability(UiHostCapability::Runtime)
        .requires_render_capability(UiRenderCapability::Primitive)
        .fallback_policy(UiWidgetFallbackPolicy::new(
            UiWidgetEditorFallback::Placeholder,
            UiWidgetRuntimeFallback::RejectNode,
        ))
}

fn visual(id: &str, display_name: &str, role: &str) -> UiComponentDescriptor {
    base_descriptor(id, display_name, UiComponentCategory::Visual, role)
}

fn input(id: &str, display_name: &str, role: &str) -> UiComponentDescriptor {
    base_descriptor(id, display_name, UiComponentCategory::Input, role)
        .requires_host_capability(UiHostCapability::PointerInput)
        .requires_host_capability(UiHostCapability::KeyboardNavigation)
}

fn numeric(id: &str, display_name: &str, role: &str) -> UiComponentDescriptor {
    base_descriptor(id, display_name, UiComponentCategory::Numeric, role)
        .requires_host_capability(UiHostCapability::PointerInput)
        .requires_host_capability(UiHostCapability::KeyboardNavigation)
        .requires_host_capability(UiHostCapability::TextInput)
        .requires_render_capability(UiRenderCapability::Text)
}

fn feedback(id: &str, display_name: &str, role: &str) -> UiComponentDescriptor {
    base_descriptor(id, display_name, UiComponentCategory::Feedback, role)
}

fn collection(id: &str, display_name: &str, role: &str) -> UiComponentDescriptor {
    base_descriptor(id, display_name, UiComponentCategory::Collection, role)
        .requires_render_capability(UiRenderCapability::Scroll)
}

fn container_descriptor(id: &str, display_name: &str, role: &str) -> UiComponentDescriptor {
    base_descriptor(id, display_name, UiComponentCategory::Container, role)
}

fn layout_primitive(id: &str, display_name: &str, role: &str) -> UiComponentDescriptor {
    let descriptor =
        container_descriptor(id, display_name, role).default_node_template(layout_template(id));
    if id == "Space" {
        descriptor
    } else {
        descriptor.slot(UiSlotSchema::new("content").multiple(true))
    }
}

fn layout_template(widget_type: &str) -> UiDefaultNodeTemplate {
    let template = UiDefaultNodeTemplate::native(widget_type);
    match widget_type {
        "Container" | "Overlay" | "FlowBox" | "GridBox" => {
            template.with_layout(container_layout(widget_type))
        }
        "HorizontalBox" | "VerticalBox" => template.with_layout(box_layout(widget_type)),
        "ScrollableBox" => template.with_layout(scrollable_layout()),
        _ => template,
    }
}

fn container_layout(kind: &str) -> BTreeMap<String, Value> {
    BTreeMap::from([(
        "container".to_string(),
        table_value(&[("kind", Value::String(kind.to_string()))]),
    )])
}

fn box_layout(kind: &str) -> BTreeMap<String, Value> {
    BTreeMap::from([(
        "container".to_string(),
        table_value(&[
            ("kind", Value::String(kind.to_string())),
            ("gap", Value::Integer(0)),
        ]),
    )])
}

fn scrollable_layout() -> BTreeMap<String, Value> {
    BTreeMap::from([(
        "container".to_string(),
        table_value(&[
            ("kind", Value::String("ScrollableBox".to_string())),
            ("axis", Value::String("Vertical".to_string())),
            ("gap", Value::Integer(0)),
            ("scrollbar_visibility", Value::String("Auto".to_string())),
        ]),
    )])
}

fn table_value(entries: &[(&str, Value)]) -> Value {
    Value::Table(
        entries
            .iter()
            .map(|(key, value)| ((*key).to_string(), value.clone()))
            .collect(),
    )
}

fn reference(
    id: &str,
    display_name: &str,
    role: &str,
    accepts: impl IntoIterator<Item = UiDragPayloadKind>,
) -> UiComponentDescriptor {
    base_descriptor(id, display_name, UiComponentCategory::Reference, role)
        .requires_host_capability(UiHostCapability::PointerInput)
        .with_prop(bool_prop("drop_hovered", false))
        .with_prop(bool_prop("active_drag_target", false))
        .drop_policy(UiDropPolicy::new(accepts))
        .event(UiComponentEventKind::DropHover)
        .event(UiComponentEventKind::ActiveDragTarget)
        .event(UiComponentEventKind::DropReference)
        .event(UiComponentEventKind::ClearReference)
        .event(UiComponentEventKind::LocateReference)
        .event(UiComponentEventKind::OpenReference)
}

fn selection(
    id: &str,
    display_name: &str,
    role: &str,
    value_kind: UiValueKind,
) -> UiComponentDescriptor {
    base_descriptor(id, display_name, UiComponentCategory::Selection, role)
        .requires_host_capability(UiHostCapability::PointerInput)
        .requires_host_capability(UiHostCapability::KeyboardNavigation)
        .with_prop(UiPropSchema::new("value", value_kind))
        .with_prop(options_prop())
        .with_prop(value_text_prop())
        .with_prop(
            UiPropSchema::new("multiple", UiValueKind::Bool).default_value(UiValue::Bool(false)),
        )
        .with_prop(selection_state_prop())
        .with_prop(validation_level_prop())
        .with_prop(bool_prop("popup_open", false))
        .with_prop(option_ids_prop("disabled_options"))
        .with_prop(option_ids_prop("special_options"))
        .with_prop(option_ids_prop("focused_options"))
        .with_prop(option_ids_prop("hovered_options"))
        .with_prop(option_ids_prop("pressed_options"))
        .event(UiComponentEventKind::Focus)
        .state(state_bool_prop("focused", false))
        .state(state_bool_prop("popup_open", false))
        .state(state_bool_prop("selected", false))
        .events([
            UiComponentEventKind::OpenPopup,
            UiComponentEventKind::ClosePopup,
            UiComponentEventKind::SelectOption,
            UiComponentEventKind::ValueChanged,
        ])
}

fn input_field(id: &str, display_name: &str) -> UiComponentDescriptor {
    input(id, display_name, "input-field")
        .requires_host_capability(UiHostCapability::TextInput)
        .requires_render_capability(UiRenderCapability::Text)
        .with_prop(
            UiPropSchema::new("value", UiValueKind::String)
                .default_value(UiValue::String(String::new())),
        )
        .with_prop(UiPropSchema::new("placeholder", UiValueKind::String))
        .with_prop(validation_level_prop())
        .state(state_string_prop("value"))
        .state(state_bool_prop("focused", false))
        .state(state_bool_prop("disabled", false))
        .events([
            UiComponentEventKind::ValueChanged,
            UiComponentEventKind::Commit,
            UiComponentEventKind::Focus,
        ])
}

fn with_palette_metadata(descriptor: UiComponentDescriptor) -> UiComponentDescriptor {
    let template = if descriptor.default_node_template.is_empty() {
        default_template_from_descriptor(&descriptor)
    } else {
        descriptor.default_node_template.clone()
    };
    let sort_key = format!(
        "{:02}.{}",
        category_sort_key(descriptor.category),
        descriptor.display_name
    );
    let palette = UiPaletteMetadata::new(
        descriptor.display_name.clone(),
        descriptor.category,
        sort_key,
        template.clone(),
    );
    descriptor.default_node_template(template).palette(palette)
}

fn default_template_from_descriptor(descriptor: &UiComponentDescriptor) -> UiDefaultNodeTemplate {
    let mut props = descriptor
        .prop_schema
        .iter()
        .filter_map(|schema| {
            schema
                .default_value
                .as_ref()
                .map(|value| (schema.name.clone(), value.to_toml()))
        })
        .collect::<BTreeMap<_, _>>();
    for (name, value) in &descriptor.default_props {
        let _ = props.insert(name.clone(), value.to_toml());
    }
    UiDefaultNodeTemplate::native(descriptor.id.as_str())
        .with_node_id_prefix(descriptor.role.as_str())
        .with_props(props)
}

fn category_sort_key(category: UiComponentCategory) -> u8 {
    match category {
        UiComponentCategory::Container => 0,
        UiComponentCategory::Visual => 1,
        UiComponentCategory::Input => 2,
        UiComponentCategory::Numeric => 3,
        UiComponentCategory::Selection => 4,
        UiComponentCategory::Reference => 5,
        UiComponentCategory::Collection => 6,
        UiComponentCategory::Feedback => 7,
    }
}

fn text_prop() -> UiPropSchema {
    UiPropSchema::new("text", UiValueKind::String).default_value(UiValue::String(String::new()))
}

fn state_text_prop() -> UiPropSchema {
    UiPropSchema::new("text", UiValueKind::String).default_value(UiValue::String(String::new()))
}

fn state_string_prop(name: &str) -> UiPropSchema {
    UiPropSchema::new(name, UiValueKind::String)
}

fn bool_value_prop(default: bool) -> UiPropSchema {
    UiPropSchema::new("value", UiValueKind::Bool).default_value(UiValue::Bool(default))
}

fn bool_prop(name: &str, default: bool) -> UiPropSchema {
    UiPropSchema::new(name, UiValueKind::Bool).default_value(UiValue::Bool(default))
}

fn int_prop(name: &str, default: i64) -> UiPropSchema {
    UiPropSchema::new(name, UiValueKind::Int).default_value(UiValue::Int(default))
}

fn number_value_prop() -> UiPropSchema {
    UiPropSchema::new("value", UiValueKind::Float)
        .default_value(UiValue::Float(0.0))
        .range(0.0, 100.0)
        .step(1.0)
}

fn value_text_prop() -> UiPropSchema {
    UiPropSchema::new("value_text", UiValueKind::String)
        .default_value(UiValue::String(String::new()))
}

fn validation_level_prop() -> UiPropSchema {
    UiPropSchema::new("validation_level", UiValueKind::String)
}

fn validation_message_prop() -> UiPropSchema {
    UiPropSchema::new("validation_message", UiValueKind::String)
}

fn options_prop() -> UiPropSchema {
    UiPropSchema::new("options", UiValueKind::Array)
        .default_value(UiValue::Array(vec![
            UiValue::Enum("primary".to_string()),
            UiValue::Enum("secondary".to_string()),
            UiValue::Enum("tertiary".to_string()),
        ]))
        .with_options([
            UiOptionDescriptor::new("primary", "Primary", UiValue::Enum("primary".to_string())),
            UiOptionDescriptor::new(
                "secondary",
                "Secondary",
                UiValue::Enum("secondary".to_string()),
            )
            .disabled(true),
            UiOptionDescriptor::new(
                "tertiary",
                "Tertiary",
                UiValue::Enum("tertiary".to_string()),
            )
            .special_condition("mixed"),
        ])
}

fn option_ids_prop(name: &str) -> UiPropSchema {
    UiPropSchema::new(name, UiValueKind::Array).default_value(UiValue::Array(Vec::new()))
}

fn selection_state_prop() -> UiPropSchema {
    UiPropSchema::new("selection_state", UiValueKind::String)
}

fn state_bool_prop(name: &str, default: bool) -> UiPropSchema {
    UiPropSchema::new(name, UiValueKind::Bool).default_value(UiValue::Bool(default))
}

fn state_float_prop(name: &str, default: f64) -> UiPropSchema {
    UiPropSchema::new(name, UiValueKind::Float).default_value(UiValue::Float(default))
}

fn state_int_prop(name: &str, default: i64) -> UiPropSchema {
    UiPropSchema::new(name, UiValueKind::Int).default_value(UiValue::Int(default))
}

fn state_array_prop(name: &str) -> UiPropSchema {
    UiPropSchema::new(name, UiValueKind::Array)
}

fn state_map_prop(name: &str) -> UiPropSchema {
    UiPropSchema::new(name, UiValueKind::Map)
}

fn expanded_prop() -> UiPropSchema {
    UiPropSchema::new("expanded", UiValueKind::Bool).default_value(UiValue::Bool(true))
}
