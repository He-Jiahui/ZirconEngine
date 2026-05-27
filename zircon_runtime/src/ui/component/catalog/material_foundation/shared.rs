use std::collections::BTreeMap;

pub(super) use zircon_runtime_interface::ui::component::{
    UiComponentCategory, UiComponentDescriptor, UiComponentDescriptorKind, UiComponentEventKind,
    UiComponentLayoutRole, UiHostCapability, UiOptionDescriptor, UiPropSchema, UiRenderCapability,
    UiSlotSchema, UiValue, UiValueKind,
};
use zircon_runtime_interface::ui::skin::{
    FYROX_PANEL_PRESET_ID, JETBRAINS_SHELL_PRESET_ID, MATERIAL_DARK_SKIN_ID,
    UNREAL_WINDOW_MODEL_PRESET_ID,
};
use zircon_runtime_interface::ui::style::{
    ButtonColor, ButtonIconPlacement, ButtonSize, ButtonVariant,
};

pub(super) fn primitive(
    id: &str,
    display_name: &str,
    category: UiComponentCategory,
    role: &str,
) -> UiComponentDescriptor {
    with_material_defaults(UiComponentDescriptor::new(id, display_name, category, role))
}

pub(super) fn composite(
    id: &str,
    display_name: &str,
    category: UiComponentCategory,
    role: &str,
) -> UiComponentDescriptor {
    with_material_defaults(UiComponentDescriptor::new(id, display_name, category, role))
        .descriptor_kind(UiComponentDescriptorKind::Composite)
}

pub(super) fn layout(
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

pub(super) fn data_view(id: &str, display_name: &str, role: &str) -> UiComponentDescriptor {
    with_material_defaults(UiComponentDescriptor::new(
        id,
        display_name,
        UiComponentCategory::Collection,
        role,
    ))
    .descriptor_kind(UiComponentDescriptorKind::Composite)
    .requires_render_capability(UiRenderCapability::Scroll)
}

pub(super) fn editor_panel_component(
    id: &str,
    display_name: &str,
    category: UiComponentCategory,
    role: &str,
) -> UiComponentDescriptor {
    with_material_defaults(UiComponentDescriptor::new(id, display_name, category, role))
        .descriptor_kind(UiComponentDescriptorKind::Composite)
        .requires_host_capability(UiHostCapability::Editor)
}

pub(super) fn editor_panel_layout(
    id: &str,
    display_name: &str,
    layout_role: UiComponentLayoutRole,
    role: &str,
) -> UiComponentDescriptor {
    editor_panel_component(id, display_name, UiComponentCategory::Container, role)
        .descriptor_kind(UiComponentDescriptorKind::Layout)
        .layout_role(layout_role)
}

pub(super) fn shell(id: &str, display_name: &str, role: &str) -> UiComponentDescriptor {
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
        .state(bool_prop("checked", false))
        .state(bool_prop("open", false))
        .state(bool_prop("popup_open", false))
        .with_prop(density_prop())
        .with_prop(enum_prop("surface_variant", "default"))
        .with_prop(button_variant_prop())
        .with_prop(enum_prop("text_tone", "primary"))
        .with_prop(enum_prop("validation_level", "normal"))
        .with_prop(float_prop("corner_radius", 10.0))
        .with_prop(float_prop("border_width", 1.0))
        .with_prop(float_prop("elevation", 0.0))
        .with_prop(default_string_prop("mui_variant", ""))
        .with_prop(default_string_prop("mui_color", "primary"))
        .with_prop(default_string_prop("mui_size", "medium"))
        .with_prop(map_prop("mui_slots"))
        .with_prop(map_prop("mui_slot_props"))
        .with_prop(map_prop("mui_sx"))
        .with_prop(array_prop("mui_classes"))
        .with_prop(map_prop("slots"))
        .with_prop(map_prop("slotProps"))
        .with_prop(map_prop("sx"))
        .with_prop(map_prop("classes"))
        .with_prop(default_string_prop("className", ""))
}

pub(super) fn popup_position_props(
    descriptor: UiComponentDescriptor,
    default_placement: &str,
) -> UiComponentDescriptor {
    descriptor
        .with_prop(enum_prop("placement", default_placement))
        .with_prop(optional_float_prop("popup_anchor_x"))
        .with_prop(optional_float_prop("popup_anchor_y"))
        .with_prop(float_prop("popup_anchor_width", 0.0))
        .with_prop(float_prop("popup_anchor_height", 0.0))
        .with_prop(default_string_prop("anchor_origin_vertical", ""))
        .with_prop(default_string_prop("anchor_origin_horizontal", ""))
        .with_prop(default_string_prop("transform_origin_vertical", ""))
        .with_prop(default_string_prop("transform_origin_horizontal", ""))
        .with_prop(float_prop("popup_offset_x", 0.0))
        .with_prop(float_prop("popup_offset_y", 0.0))
}

pub(super) fn modal_interaction_props(descriptor: UiComponentDescriptor) -> UiComponentDescriptor {
    descriptor
        .with_prop(bool_prop("disable_auto_focus", false))
        .with_prop(bool_prop("disable_enforce_focus", false))
        .with_prop(bool_prop("disable_restore_focus", false))
        .with_prop(bool_prop("disable_escape_key_down", false))
        .with_prop(bool_prop("close_on_backdrop_click", true))
        .with_prop(bool_prop("keep_mounted", false))
        .with_prop(bool_prop("aria_modal", true))
        .with_prop(default_string_prop("aria_labelledby", ""))
        .with_prop(default_string_prop("aria_describedby", ""))
}

pub(super) fn overlay_layer_props(descriptor: UiComponentDescriptor) -> UiComponentDescriptor {
    descriptor
        .with_prop(optional_int_prop("z_index"))
        .with_prop(bool_prop("disable_portal", false))
        .with_prop(default_string_prop("portal_layer", "overlay"))
}

pub(super) fn transition_props(
    descriptor: UiComponentDescriptor,
    transition_kind: &str,
    timeout_ms: i64,
    easing: &str,
) -> UiComponentDescriptor {
    descriptor
        .with_prop(default_string_prop("transition_kind", transition_kind))
        .with_prop(bool_prop("in", true))
        .with_prop(default_string_prop("transition_status", "entered"))
        .with_prop(float_prop("transition_progress", 1.0))
        .with_prop(int_prop("timeout_ms", timeout_ms))
        .with_prop(int_prop("transition_duration_ms", timeout_ms))
        .with_prop(default_string_prop("easing", easing))
        .with_prop(default_string_prop("transition_easing", easing))
        .with_prop(bool_prop("mount_on_enter", false))
        .with_prop(bool_prop("mountOnEnter", false))
        .with_prop(bool_prop("unmount_on_exit", false))
        .with_prop(bool_prop("unmountOnExit", false))
}

pub(super) fn virtualized_range_props(descriptor: UiComponentDescriptor) -> UiComponentDescriptor {
    descriptor
        .with_prop(int_prop("total_count", 0))
        .with_prop(int_prop("item_count", 0))
        .with_prop(int_prop("itemCount", 0))
        .with_prop(int_prop("row_count", 0))
        .with_prop(int_prop("rowCount", 0))
        .with_prop(int_prop("viewport_start", 0))
        .with_prop(int_prop("viewport_count", 20))
        .with_prop(int_prop("visible_end", 0))
        .with_prop(int_prop("requested_start", 0))
        .with_prop(int_prop("requested_count", 0))
        .with_prop(float_prop("item_extent", 24.0))
        .with_prop(float_prop("itemSize", 24.0))
        .with_prop(float_prop("row_height", 24.0))
        .with_prop(float_prop("rowHeight", 24.0))
        .with_prop(int_prop("overscan", 2))
        .with_prop(int_prop("overscan_count", 2))
        .with_prop(int_prop("overscanCount", 2))
        .with_prop(float_prop("scroll_offset", 0.0))
        .with_prop(float_prop("scrollTop", 0.0))
        .with_prop(bool_prop("disable_virtualization", false))
        .with_prop(bool_prop("disableVirtualization", false))
}

pub(super) fn density_prop() -> UiPropSchema {
    UiPropSchema::new("density", UiValueKind::Enum)
        .default_value(UiValue::Enum("compact".to_string()))
}

pub(super) fn text_prop() -> UiPropSchema {
    string_prop("text")
}

pub(super) fn icon_prop() -> UiPropSchema {
    string_prop("icon")
}

pub(super) fn checked_prop() -> UiPropSchema {
    bool_prop("checked", false)
}

pub(super) fn expanded_prop() -> UiPropSchema {
    bool_prop("expanded", true)
}

pub(super) fn bool_prop(name: &str, default: bool) -> UiPropSchema {
    UiPropSchema::new(name, UiValueKind::Bool).default_value(UiValue::Bool(default))
}

pub(super) fn string_prop(name: &str) -> UiPropSchema {
    default_string_prop(name, "")
}

pub(super) fn required_string_prop(name: &str) -> UiPropSchema {
    string_prop(name).required(true)
}

pub(super) fn default_string_prop(name: &str, default: &str) -> UiPropSchema {
    UiPropSchema::new(name, UiValueKind::String).default_value(UiValue::String(default.to_string()))
}

pub(super) fn enum_prop(name: &str, default: &str) -> UiPropSchema {
    UiPropSchema::new(name, UiValueKind::Enum).default_value(UiValue::Enum(default.to_string()))
}

pub(super) fn enum_prop_with_options(
    name: &str,
    default: &str,
    options: impl IntoIterator<Item = UiOptionDescriptor>,
) -> UiPropSchema {
    enum_prop(name, default).with_options(options)
}

pub(super) fn button_variant_prop() -> UiPropSchema {
    enum_prop_with_options(
        "button_variant",
        "default",
        enum_options(ButtonVariant::OPTIONS),
    )
}

pub(super) fn button_color_prop() -> UiPropSchema {
    enum_prop_with_options(
        "button_color",
        "primary",
        enum_options(ButtonColor::OPTIONS),
    )
}

pub(super) fn button_size_prop() -> UiPropSchema {
    enum_prop_with_options("button_size", "medium", enum_options(ButtonSize::OPTIONS))
}

pub(super) fn button_icon_placement_prop(default: &str) -> UiPropSchema {
    enum_prop_with_options(
        "icon_placement",
        default,
        enum_options(ButtonIconPlacement::OPTIONS),
    )
}

pub(super) fn with_text_input_events(descriptor: UiComponentDescriptor) -> UiComponentDescriptor {
    descriptor
        .events([
            UiComponentEventKind::Focus,
            UiComponentEventKind::ValueChanged,
            UiComponentEventKind::Commit,
        ])
        .requires_host_capability(UiHostCapability::TextInput)
}

fn enum_options<const N: usize>(options: [&'static str; N]) -> Vec<UiOptionDescriptor> {
    options
        .into_iter()
        .map(|option| {
            UiOptionDescriptor::new(
                option,
                enum_label(option),
                UiValue::Enum(option.to_string()),
            )
        })
        .collect()
}

pub(super) fn enum_option_descriptor(option: &str) -> UiOptionDescriptor {
    UiOptionDescriptor::new(
        option,
        enum_label(option),
        UiValue::Enum(option.to_string()),
    )
}

fn enum_label(option: &str) -> String {
    option
        .split('_')
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => format!("{}{}", first.to_ascii_uppercase(), chars.as_str()),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

pub(super) fn int_prop(name: &str, default: i64) -> UiPropSchema {
    UiPropSchema::new(name, UiValueKind::Int).default_value(UiValue::Int(default))
}

fn optional_int_prop(name: &str) -> UiPropSchema {
    UiPropSchema::new(name, UiValueKind::Int)
}

pub(super) fn float_prop(name: &str, default: f64) -> UiPropSchema {
    UiPropSchema::new(name, UiValueKind::Float).default_value(UiValue::Float(default))
}

pub(super) fn optional_float_prop(name: &str) -> UiPropSchema {
    UiPropSchema::new(name, UiValueKind::Float)
}

pub(super) fn value_text_prop() -> UiPropSchema {
    string_prop("value_text")
}

pub(super) fn options_prop() -> UiPropSchema {
    UiPropSchema::new("options", UiValueKind::Array).default_value(UiValue::Array(Vec::new()))
}

pub(super) fn array_prop(name: &str) -> UiPropSchema {
    UiPropSchema::new(name, UiValueKind::Array).default_value(UiValue::Array(Vec::new()))
}

pub(super) fn map_prop(name: &str) -> UiPropSchema {
    UiPropSchema::new(name, UiValueKind::Map).default_value(UiValue::Map(BTreeMap::new()))
}

pub(super) fn any_prop(name: &str) -> UiPropSchema {
    UiPropSchema::new(name, UiValueKind::Any).default_value(UiValue::Null)
}

pub(super) fn number_value_prop() -> UiPropSchema {
    UiPropSchema::new("value", UiValueKind::Float)
        .default_value(UiValue::Float(0.0))
        .range(0.0, 1.0)
        .step(0.01)
}

pub(super) fn field_number_value_prop() -> UiPropSchema {
    UiPropSchema::new("value", UiValueKind::Float)
        .default_value(UiValue::Float(0.0))
        .range(0.0, 100.0)
        .step(1.0)
}

pub(super) fn workbench_skin_prop() -> UiPropSchema {
    default_string_prop("skin_id", MATERIAL_DARK_SKIN_ID)
}

pub(super) fn fyrox_panel_preset_prop() -> UiPropSchema {
    default_string_prop("panel_preset_id", FYROX_PANEL_PRESET_ID)
}

pub(super) fn jetbrains_shell_preset_prop() -> UiPropSchema {
    default_string_prop("shell_preset_id", JETBRAINS_SHELL_PRESET_ID)
}

pub(super) fn unreal_window_model_preset_prop() -> UiPropSchema {
    default_string_prop("window_model_preset_id", UNREAL_WINDOW_MODEL_PRESET_ID)
}
