use super::shared::*;
use zircon_runtime_interface::ui::component::UiPropSchema;

const FIELD_VARIANTS: [&str; 3] = ["standard", "filled", "outlined"];
const FIELD_SIZES: [&str; 2] = ["small", "medium"];
const MARGINS: [&str; 3] = ["dense", "none", "normal"];
const PALETTE_COLORS: [&str; 7] = [
    "primary",
    "secondary",
    "error",
    "info",
    "success",
    "warning",
    "default",
];

pub(super) fn descriptors() -> Vec<UiComponentDescriptor> {
    vec![
        button_base(),
        input_base("InputBase", "Input Base", "input-base"),
        input_base("FilledInput", "Filled Input", "filled-input")
            .with_prop(bool_prop("disableUnderline", false))
            .with_prop(default_string_prop("variant", "filled")),
        input_base("OutlinedInput", "Outlined Input", "outlined-input")
            .with_prop(default_string_prop("label", ""))
            .with_prop(bool_prop("notched", false))
            .with_prop(default_string_prop("variant", "outlined"))
            .slot(UiSlotSchema::new("notchedOutline")),
        form_control(),
        form_control_label(),
        form_group(),
        form_helper_text(),
        form_label(),
        input_adornment(),
        input_label(),
        native_select(),
        radio_group(),
    ]
}

fn button_base() -> UiComponentDescriptor {
    primitive(
        "ButtonBase",
        "Button Base",
        UiComponentCategory::Input,
        "button-base",
    )
    .with_prop(default_string_prop("component", "button"))
    .with_prop(bool_prop("centerRipple", false))
    .with_prop(bool_prop("disableRipple", false))
    .with_prop(bool_prop("disableTouchRipple", false))
    .with_prop(bool_prop("focusRipple", false))
    .with_prop(default_string_prop("focusVisibleClassName", ""))
    .with_prop(any_prop("TouchRippleProps"))
    .slot(UiSlotSchema::new("touchRipple"))
    .events([
        UiComponentEventKind::Focus,
        UiComponentEventKind::Press,
        UiComponentEventKind::Commit,
    ])
}

fn input_base(id: &str, display_name: &str, role: &str) -> UiComponentDescriptor {
    with_text_input_events(
        primitive(id, display_name, UiComponentCategory::Input, role)
            .with_prop(default_string_prop("value", ""))
            .with_prop(value_text_prop())
            .with_prop(default_string_prop("defaultValue", ""))
            .with_prop(default_string_prop("placeholder", ""))
            .with_prop(default_string_prop("type", "text"))
            .with_prop(default_string_prop("name", ""))
            .with_prop(default_string_prop("component", "div"))
            .with_prop(default_string_prop("inputComponent", "input"))
            .with_prop(mui_enum_prop("color", "primary", PALETTE_COLORS))
            .with_prop(mui_enum_prop("size", "medium", FIELD_SIZES))
            .with_prop(bool_prop("autoFocus", false))
            .with_prop(bool_prop("readOnly", false))
            .with_prop(bool_prop("inputReadOnly", false))
            .with_prop(bool_prop("fullWidth", false))
            .with_prop(bool_prop("hiddenLabel", false))
            .with_prop(bool_prop("formControl", false))
            .with_prop(bool_prop("multiline", false))
            .with_prop(default_string_prop("startAdornment", ""))
            .with_prop(default_string_prop("endAdornment", ""))
            .with_prop(int_prop("minRows", 0))
            .with_prop(int_prop("maxRows", 0))
            .with_prop(map_prop("inputProps"))
            .slot(UiSlotSchema::new("root"))
            .slot(UiSlotSchema::new("input"))
            .slot(UiSlotSchema::new("startAdornment"))
            .slot(UiSlotSchema::new("endAdornment")),
    )
}

fn form_control() -> UiComponentDescriptor {
    composite(
        "FormControl",
        "Form Control",
        UiComponentCategory::Container,
        "form-control",
    )
    .with_prop(default_string_prop("component", "div"))
    .with_prop(mui_enum_prop("color", "primary", PALETTE_COLORS))
    .with_prop(mui_enum_prop("margin", "none", MARGINS))
    .with_prop(mui_enum_prop("size", "medium", FIELD_SIZES))
    .with_prop(mui_enum_prop("variant", "outlined", FIELD_VARIANTS))
    .with_prop(bool_prop("fullWidth", false))
    .with_prop(bool_prop("hiddenLabel", false))
    .with_prop(bool_prop("required", false))
    .slot(UiSlotSchema::new("label"))
    .slot(UiSlotSchema::new("input"))
    .slot(UiSlotSchema::new("helperText"))
}

fn form_control_label() -> UiComponentDescriptor {
    composite(
        "FormControlLabel",
        "Form Control Label",
        UiComponentCategory::Input,
        "form-control-label",
    )
    .with_prop(default_string_prop("label", ""))
    .with_prop(value_text_prop())
    .with_prop(default_string_prop("component", "label"))
    .with_prop(mui_enum_prop(
        "labelPlacement",
        "end",
        ["bottom", "end", "start", "top"],
    ))
    .with_prop(bool_prop("checked", false))
    .with_prop(bool_prop("disableTypography", false))
    .with_prop(bool_prop("required", false))
    .slot(UiSlotSchema::new("control").required(true))
    .slot(UiSlotSchema::new("label"))
    .slot(UiSlotSchema::new("asterisk"))
    .events([
        UiComponentEventKind::Focus,
        UiComponentEventKind::ValueChanged,
    ])
}

fn form_group() -> UiComponentDescriptor {
    composite(
        "FormGroup",
        "Form Group",
        UiComponentCategory::Container,
        "form-group",
    )
    .with_prop(bool_prop("row", false))
    .slot(UiSlotSchema::new("content").multiple(true))
}

fn form_helper_text() -> UiComponentDescriptor {
    primitive(
        "FormHelperText",
        "Form Helper Text",
        UiComponentCategory::Visual,
        "form-helper-text",
    )
    .with_prop(text_prop())
    .with_prop(default_string_prop("component", "p"))
    .with_prop(mui_enum_prop("size", "medium", FIELD_SIZES))
    .with_prop(mui_enum_prop("variant", "outlined", FIELD_VARIANTS))
    .with_prop(bool_prop("filled", false))
    .with_prop(bool_prop("required", false))
}

fn form_label() -> UiComponentDescriptor {
    primitive(
        "FormLabel",
        "Form Label",
        UiComponentCategory::Visual,
        "form-label",
    )
    .with_prop(text_prop())
    .with_prop(default_string_prop("component", "label"))
    .with_prop(mui_enum_prop("color", "primary", PALETTE_COLORS))
    .with_prop(bool_prop("filled", false))
    .with_prop(bool_prop("required", false))
    .slot(UiSlotSchema::new("asterisk"))
}

fn input_adornment() -> UiComponentDescriptor {
    composite(
        "InputAdornment",
        "Input Adornment",
        UiComponentCategory::Visual,
        "input-adornment",
    )
    .with_prop(text_prop())
    .with_prop(default_string_prop("component", "div"))
    .with_prop(mui_enum_prop("position", "end", ["end", "start"]))
    .with_prop(mui_enum_prop("size", "medium", FIELD_SIZES))
    .with_prop(mui_enum_prop("variant", "standard", FIELD_VARIANTS))
    .with_prop(bool_prop("disablePointerEvents", false))
    .with_prop(bool_prop("disableTypography", false))
    .with_prop(bool_prop("hiddenLabel", false))
    .slot(UiSlotSchema::new("content").multiple(true))
}

fn input_label() -> UiComponentDescriptor {
    primitive(
        "InputLabel",
        "Input Label",
        UiComponentCategory::Visual,
        "input-label",
    )
    .with_prop(text_prop())
    .with_prop(default_string_prop("component", "label"))
    .with_prop(mui_enum_prop("margin", "none", ["dense", "none"]))
    .with_prop(mui_enum_prop("size", "medium", FIELD_SIZES))
    .with_prop(mui_enum_prop("variant", "outlined", FIELD_VARIANTS))
    .with_prop(bool_prop("disableAnimation", false))
    .with_prop(bool_prop("formControl", false))
    .with_prop(bool_prop("required", false))
    .with_prop(bool_prop("shrink", false))
    .slot(UiSlotSchema::new("asterisk"))
}

fn native_select() -> UiComponentDescriptor {
    primitive(
        "NativeSelect",
        "Native Select",
        UiComponentCategory::Selection,
        "native-select",
    )
    .with_prop(options_prop())
    .with_prop(default_string_prop("value", ""))
    .with_prop(default_string_prop("defaultValue", ""))
    .with_prop(mui_enum_prop("variant", "standard", FIELD_VARIANTS))
    .with_prop(bool_prop("multiple", false))
    .with_prop(bool_prop("open", false))
    .with_prop(map_prop("inputProps"))
    .with_prop(default_string_prop("IconComponent", "ArrowDropDown"))
    .slot(UiSlotSchema::new("select"))
    .slot(UiSlotSchema::new("icon"))
    .slot(UiSlotSchema::new("nativeInput"))
    .event(UiComponentEventKind::ValueChanged)
}

fn radio_group() -> UiComponentDescriptor {
    composite(
        "RadioGroup",
        "Radio Group",
        UiComponentCategory::Selection,
        "radio-group",
    )
    .with_prop(options_prop())
    .with_prop(default_string_prop("value", ""))
    .with_prop(default_string_prop("defaultValue", ""))
    .with_prop(default_string_prop("name", ""))
    .with_prop(bool_prop("row", false))
    .slot(UiSlotSchema::new("items").multiple(true))
    .events([
        UiComponentEventKind::Focus,
        UiComponentEventKind::SelectOption,
        UiComponentEventKind::ValueChanged,
    ])
}

fn mui_enum_prop<const N: usize>(
    name: &str,
    default: &str,
    options: [&'static str; N],
) -> UiPropSchema {
    enum_prop_with_options(
        name,
        default,
        options.into_iter().map(enum_option_descriptor),
    )
}
