use std::sync::OnceLock;

use crate::ui::component::UiComponentDescriptorRegistry;
use zircon_runtime_interface::ui::component::{
    UiComponentDescriptorKind, UiComponentEventKind, UiComponentLayoutRole, UiDragPayloadKind,
    UiHostCapability, UiValue, UiValueKind,
};

use zircon_runtime_interface::ui::component::{
    UiComponentDescriptor, UiPropSchema, UiRenderCapability, UiSlotSchema,
};

use descriptor_builders::{
    bool_prop, bool_value_prop, collection, container_descriptor, editor_collection,
    editor_feedback, expanded_prop, feedback, input, input_field, int_prop, layout_primitive,
    number_value_prop, numeric, options_prop, popup_descriptor, reference, selection,
    selection_state_prop, state_array_prop, state_bool_prop, state_float_prop, state_int_prop,
    state_map_prop, state_string_prop, state_text_prop, text_prop, validation_level_prop,
    validation_message_prop, value_text_prop, visual, with_palette_metadata,
};

mod descriptor_builders;

static EDITOR_SHOWCASE_REGISTRY: OnceLock<UiComponentDescriptorRegistry> = OnceLock::new();

impl UiComponentDescriptorRegistry {
    /// Builds the Runtime UI component catalog used by the editor showcase.
    pub fn editor_showcase() -> Self {
        EDITOR_SHOWCASE_REGISTRY
            .get_or_init(build_editor_showcase_registry)
            .clone()
    }
}

fn build_editor_showcase_registry() -> UiComponentDescriptorRegistry {
    let mut registry = UiComponentDescriptorRegistry::new();
    for descriptor in editor_showcase_descriptors() {
        registry
            .register(descriptor)
            .expect("built-in UI component descriptors must validate");
    }
    registry
}

#[inline(never)]
fn build_editor_showcase_descriptor(
    factory: impl FnOnce() -> UiComponentDescriptor,
) -> UiComponentDescriptor {
    with_palette_metadata(factory())
}

fn editor_showcase_descriptors() -> Vec<UiComponentDescriptor> {
    macro_rules! push_descriptors {
        ($target:expr, $($descriptor:expr),+ $(,)?) => {
            $(
                $target.push(build_editor_showcase_descriptor(|| $descriptor));
            )+
        };
    }

    let mut descriptors = Vec::with_capacity(70);
    push_descriptors!(
        descriptors,
        layout_primitive("Container", "Container", "container"),
        layout_primitive("Overlay", "Overlay", "overlay"),
        layout_primitive("ListView", "List View", "list-view"),
        layout_primitive("FlexBox", "Flex Box", "flex-box"),
        layout_primitive("HorizontalBox", "Horizontal Box", "horizontal-box"),
        layout_primitive("HorizontalGroup", "Horizontal Group", "horizontal-group"),
        layout_primitive("VerticalBox", "Vertical Box", "vertical-box"),
        layout_primitive("VerticalGroup", "Vertical Group", "vertical-group"),
        layout_primitive("FlowBox", "Flow Box", "flow-box"),
        layout_primitive("GridBox", "Grid Box", "grid-box"),
        layout_primitive("GridGroup", "Grid Group", "grid-group"),
        layout_primitive("ScrollableBox", "Scrollable Box", "scrollable-box"),
        layout_primitive("CanvasBox", "Canvas Box", "canvas-box"),
        layout_primitive("SizeBox", "Size Box", "size-box"),
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
        visual("Text", "Text", "text")
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
        visual("Svg", "SVG", "svg")
            .with_prop(UiPropSchema::new("source", UiValueKind::String))
            .state(UiPropSchema::new("source", UiValueKind::String))
            .requires_render_capability(UiRenderCapability::Vector),
        visual("Canvas", "Canvas", "canvas")
            .with_prop(UiPropSchema::new("commands", UiValueKind::Array))
            .state(state_array_prop("commands"))
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
            .with_prop(bool_prop("indeterminate", false))
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
        input("Toggle", "Toggle", "toggle")
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
        input("RadioField", "Radio Field", "radio-field")
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
            .descriptor_kind(UiComponentDescriptorKind::EditorOnly)
            .with_prop(expanded_prop())
            .with_prop(validation_level_prop())
            .with_prop(text_prop())
            .event(UiComponentEventKind::Focus)
            .state(state_bool_prop("expanded", false))
            .state(state_bool_prop("focused", false))
            .state(state_bool_prop("disabled", false))
            .slot(UiSlotSchema::new("content").multiple(true))
            .event(UiComponentEventKind::ToggleExpanded),
        popup_descriptor(),
        container_descriptor("PropertyRow", "Property Row", "property-row")
            .descriptor_kind(UiComponentDescriptorKind::Composite)
            .with_prop(text_prop())
            .with_prop(UiPropSchema::new("value", UiValueKind::String))
            .slot(UiSlotSchema::new("label"))
            .slot(UiSlotSchema::new("field")),
        container_descriptor("InspectorSection", "Inspector Section", "inspector-section")
            .descriptor_kind(UiComponentDescriptorKind::Composite)
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
        .requires_host_capability(UiHostCapability::WorldSpaceUi)
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
            .descriptor_kind(UiComponentDescriptorKind::Layout)
            .layout_role(UiComponentLayoutRole::VirtualList)
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
        editor_collection("TreeView", "Tree View", "tree-view")
            .with_prop(UiPropSchema::new("items", UiValueKind::Array))
            .with_prop(int_prop("selected_index", -1))
            .with_prop(expanded_prop())
            .slot(UiSlotSchema::new("row").multiple(true))
            .state(state_array_prop("items"))
            .state(state_int_prop("selected_index", -1))
            .state(state_bool_prop("expanded", true))
            .event(UiComponentEventKind::ValueChanged)
            .event(UiComponentEventKind::SelectOption)
            .event(UiComponentEventKind::ToggleExpanded),
        editor_collection("EditableTable", "Editable Table", "editable-table")
            .with_prop(UiPropSchema::new("rows", UiValueKind::Array))
            .with_prop(UiPropSchema::new("columns", UiValueKind::Array))
            .with_prop(int_prop("selected_row", -1))
            .with_prop(int_prop("selected_column", -1))
            .slot(UiSlotSchema::new("cell").multiple(true))
            .state(state_array_prop("rows"))
            .state(state_array_prop("columns"))
            .state(state_int_prop("selected_row", -1))
            .state(state_int_prop("selected_column", -1))
            .events([
                UiComponentEventKind::ValueChanged,
                UiComponentEventKind::Commit,
                UiComponentEventKind::SelectOption,
                UiComponentEventKind::SetElement,
            ]),
        editor_collection("Table", "Table", "table")
            .with_prop(UiPropSchema::new("rows", UiValueKind::Array))
            .with_prop(UiPropSchema::new("columns", UiValueKind::Array))
            .slot(UiSlotSchema::new("cell").multiple(true))
            .state(state_array_prop("rows"))
            .state(state_array_prop("columns"))
            .events([
                UiComponentEventKind::ValueChanged,
                UiComponentEventKind::Commit,
                UiComponentEventKind::SelectOption,
                UiComponentEventKind::SetElement,
            ]),
        editor_feedback("MessageBox", "Message Box", "message-box")
            .with_prop(UiPropSchema::new("severity", UiValueKind::String))
            .with_prop(text_prop())
            .with_prop(UiPropSchema::new("rich_text", UiValueKind::String))
            .with_prop(bool_prop("open", false))
            .with_prop(UiPropSchema::new("actions", UiValueKind::Array))
            .state(state_string_prop("severity"))
            .state(state_text_prop())
            .state(state_string_prop("rich_text"))
            .state(state_bool_prop("open", false))
            .state(state_array_prop("actions"))
            .events([
                UiComponentEventKind::OpenPopup,
                UiComponentEventKind::ClosePopup,
                UiComponentEventKind::SelectOption,
            ]),
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
    );
    descriptors
}

#[cfg(test)]
mod tests {
    #[test]
    fn editor_showcase_catalog_builds_on_small_stack() {
        std::thread::Builder::new()
            .stack_size(256 * 1024)
            .spawn(|| {
                let registry = super::build_editor_showcase_registry();
                assert!(registry.len() >= 40);
                assert!(registry.contains("Container"));
                assert!(registry.contains("ContextActionMenu"));
            })
            .expect("spawn small-stack showcase catalog test")
            .join()
            .expect("showcase catalog should not overflow the stack");
    }
}
