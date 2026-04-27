use std::collections::BTreeMap;

use super::{
    UiComponentCategory, UiComponentDescriptor, UiComponentEventKind, UiDragPayloadKind,
    UiDropPolicy, UiOptionDescriptor, UiPropSchema, UiSlotSchema, UiValue, UiValueKind,
};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct UiComponentDescriptorRegistry {
    descriptors: BTreeMap<String, UiComponentDescriptor>,
}

impl UiComponentDescriptorRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn editor_showcase() -> Self {
        let mut registry = Self::new();
        for descriptor in editor_showcase_descriptors() {
            registry.register(descriptor);
        }
        registry
    }

    pub fn register(&mut self, descriptor: UiComponentDescriptor) {
        self.descriptors.insert(descriptor.id.clone(), descriptor);
    }

    pub fn descriptor(&self, component_id: &str) -> Option<&UiComponentDescriptor> {
        self.descriptors.get(component_id)
    }

    pub fn descriptors(&self) -> impl Iterator<Item = &UiComponentDescriptor> {
        self.descriptors.values()
    }
}

fn editor_showcase_descriptors() -> Vec<UiComponentDescriptor> {
    vec![
        visual("Label", "Label", "label")
            .with_prop(text_prop())
            .state(state_text_prop())
            .event(UiComponentEventKind::ValueChanged),
        visual("RichLabel", "Rich Label", "rich-label")
            .with_prop(text_prop())
            .state(state_text_prop()),
        visual("Image", "Image", "image")
            .with_prop(UiPropSchema::new("image", UiValueKind::AssetRef))
            .state(UiPropSchema::new("image", UiValueKind::AssetRef)),
        visual("Icon", "Icon", "icon")
            .with_prop(UiPropSchema::new("icon", UiValueKind::String))
            .state(UiPropSchema::new("icon", UiValueKind::String)),
        visual("SvgIcon", "SVG Icon", "svg-icon")
            .with_prop(UiPropSchema::new("source", UiValueKind::String))
            .state(UiPropSchema::new("source", UiValueKind::String)),
        visual("Separator", "Separator", "separator").with_prop(text_prop()),
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
            .event(UiComponentEventKind::Commit),
        input("ToggleButton", "Toggle Button", "toggle-button")
            .with_prop(bool_value_prop(false))
            .with_prop(text_prop())
            .state(state_bool_prop("value", false))
            .state(state_bool_prop("checked", false))
            .state(state_bool_prop("focused", false))
            .state(state_bool_prop("disabled", false))
            .event(UiComponentEventKind::Focus)
            .event(UiComponentEventKind::ValueChanged),
        input("Checkbox", "Checkbox", "checkbox")
            .with_prop(bool_value_prop(false))
            .with_prop(text_prop())
            .state(state_bool_prop("value", false))
            .state(state_bool_prop("checked", false))
            .state(state_bool_prop("focused", false))
            .state(state_bool_prop("disabled", false))
            .event(UiComponentEventKind::Focus)
            .event(UiComponentEventKind::ValueChanged),
        input("Radio", "Radio", "radio")
            .with_prop(bool_value_prop(false))
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
        .with_prop(
            UiPropSchema::new("multiple", UiValueKind::Bool).default_value(UiValue::Bool(false)),
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
        .state(state_bool_prop("disabled", false)),
        container("Group", "Group", "group")
            .with_prop(expanded_prop())
            .with_prop(validation_level_prop())
            .with_prop(text_prop())
            .event(UiComponentEventKind::Focus)
            .state(expanded_prop())
            .state(state_bool_prop("focused", false))
            .state(state_bool_prop("disabled", false))
            .slot(UiSlotSchema::new("content").multiple(true))
            .event(UiComponentEventKind::ToggleExpanded),
        container("Foldout", "Foldout", "foldout")
            .with_prop(expanded_prop())
            .with_prop(validation_level_prop())
            .with_prop(text_prop())
            .event(UiComponentEventKind::Focus)
            .state(state_bool_prop("expanded", false))
            .state(state_bool_prop("focused", false))
            .state(state_bool_prop("disabled", false))
            .slot(UiSlotSchema::new("content").multiple(true))
            .event(UiComponentEventKind::ToggleExpanded),
        container("PropertyRow", "Property Row", "property-row")
            .with_prop(text_prop())
            .with_prop(UiPropSchema::new("value", UiValueKind::String))
            .slot(UiSlotSchema::new("label"))
            .slot(UiSlotSchema::new("field")),
        container("InspectorSection", "Inspector Section", "inspector-section")
            .with_prop(text_prop())
            .slot(UiSlotSchema::new("content").multiple(true))
            .state(state_bool_prop("expanded", true))
            .event(UiComponentEventKind::ToggleExpanded),
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
            .with_prop(value_text_prop())
            .event(UiComponentEventKind::Focus)
            .state(state_map_prop("entries"))
            .state(state_bool_prop("focused", false))
            .events([
                UiComponentEventKind::AddMapEntry,
                UiComponentEventKind::SetMapEntry,
                UiComponentEventKind::RemoveMapEntry,
                UiComponentEventKind::ValueChanged,
            ]),
        collection("ListRow", "List Row", "list-row")
            .with_prop(text_prop())
            .with_prop(UiPropSchema::new("value", UiValueKind::String))
            .event(UiComponentEventKind::Focus)
            .state(state_text_prop())
            .state(state_bool_prop("selected", false))
            .state(state_bool_prop("focused", false)),
        collection("TreeRow", "Tree Row", "tree-row")
            .with_prop(text_prop())
            .with_prop(expanded_prop())
            .event(UiComponentEventKind::Focus)
            .state(state_text_prop())
            .state(state_bool_prop("expanded", false))
            .state(state_bool_prop("selected", false))
            .state(state_bool_prop("focused", false))
            .event(UiComponentEventKind::ToggleExpanded),
        input(
            "ContextActionMenu",
            "Context Action Menu",
            "context-action-menu",
        )
        .with_prop(options_prop())
        .with_prop(UiPropSchema::new("value", UiValueKind::String))
        .event(UiComponentEventKind::Focus)
        .state(state_bool_prop("focused", false))
        .state(state_bool_prop("selected", false))
        .state(state_bool_prop("popup_open", false))
        .events([
            UiComponentEventKind::OpenPopup,
            UiComponentEventKind::ClosePopup,
            UiComponentEventKind::SelectOption,
        ]),
    ]
}

fn visual(id: &str, display_name: &str, role: &str) -> UiComponentDescriptor {
    UiComponentDescriptor::new(id, display_name, UiComponentCategory::Visual, role)
}

fn input(id: &str, display_name: &str, role: &str) -> UiComponentDescriptor {
    UiComponentDescriptor::new(id, display_name, UiComponentCategory::Input, role)
}

fn numeric(id: &str, display_name: &str, role: &str) -> UiComponentDescriptor {
    UiComponentDescriptor::new(id, display_name, UiComponentCategory::Numeric, role)
}

fn feedback(id: &str, display_name: &str, role: &str) -> UiComponentDescriptor {
    UiComponentDescriptor::new(id, display_name, UiComponentCategory::Feedback, role)
}

fn collection(id: &str, display_name: &str, role: &str) -> UiComponentDescriptor {
    UiComponentDescriptor::new(id, display_name, UiComponentCategory::Collection, role)
}

fn container(id: &str, display_name: &str, role: &str) -> UiComponentDescriptor {
    UiComponentDescriptor::new(id, display_name, UiComponentCategory::Container, role)
}

fn reference(
    id: &str,
    display_name: &str,
    role: &str,
    accepts: impl IntoIterator<Item = UiDragPayloadKind>,
) -> UiComponentDescriptor {
    UiComponentDescriptor::new(id, display_name, UiComponentCategory::Reference, role)
        .drop_policy(UiDropPolicy::new(accepts))
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
    UiComponentDescriptor::new(id, display_name, UiComponentCategory::Selection, role)
        .with_prop(UiPropSchema::new("value", value_kind))
        .with_prop(options_prop())
        .with_prop(value_text_prop())
        .with_prop(
            UiPropSchema::new("multiple", UiValueKind::Bool).default_value(UiValue::Bool(false)),
        )
        .with_prop(selection_state_prop())
        .with_prop(validation_level_prop())
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

fn selection_state_prop() -> UiPropSchema {
    UiPropSchema::new("selection_state", UiValueKind::String)
}

fn state_bool_prop(name: &str, default: bool) -> UiPropSchema {
    UiPropSchema::new(name, UiValueKind::Bool).default_value(UiValue::Bool(default))
}

fn state_float_prop(name: &str, default: f64) -> UiPropSchema {
    UiPropSchema::new(name, UiValueKind::Float).default_value(UiValue::Float(default))
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
