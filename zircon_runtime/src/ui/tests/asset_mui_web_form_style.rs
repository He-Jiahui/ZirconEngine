use crate::ui::template::{UiAssetLoader, UiDocumentCompiler};
use toml::Value;
use zircon_runtime_interface::ui::template::UiTemplateNode;

const FORM_STYLE_TOML: &str = r##"
[asset]
kind = "style"
id = "ui.tests.mui_web_form_style"
version = 1
display_name = "MUI Web Form Style"

[[stylesheets]]
id = "mui_web_form"

[[stylesheets.rules]]
selector = ".MuiButtonBase-root.MuiButtonBase-focusVisible"
set = { self = { state_layer_enabled = true } }

[[stylesheets.rules]]
selector = ".MuiInputBase-root.MuiInputBase-formControl.MuiInputBase-sizeSmall.MuiInputBase-adornedStart.MuiInputBase-fullWidth.MuiInputBase-readOnly.Mui-readOnly"
set = { self = { validation_level = "inputbase" } }

[[stylesheets.rules]]
selector = ".MuiInputBase-input.MuiInputBase-inputTypeSearch.input-extra"
set = { self = { text_tone = "info" } }

[[stylesheets.rules]]
selector = ".MuiFilledInput-root.MuiInputBase-root.MuiFilledInput-underline.MuiFilledInput-adornedEnd.MuiInputBase-adornedEnd"
set = { self = { surface_variant = "filled-input" } }

[[stylesheets.rules]]
selector = ".MuiOutlinedInput-notchedOutline.notched-extra"
set = { self = { border_width = 2.0 } }

[[stylesheets.rules]]
selector = ".MuiFormControl-root.MuiFormControl-marginDense.MuiFormControl-fullWidth"
set = { self = { surface_variant = "form-control" } }

[[stylesheets.rules]]
selector = ".MuiFormControlLabel-root.MuiFormControlLabel-labelPlacementStart.MuiFormControlLabel-required.Mui-error"
set = { self = { validation_level = "label-error" } }

[[stylesheets.rules]]
selector = ".MuiFormControlLabel-label.label-extra"
set = { self = { text_tone = "warning" } }

[[stylesheets.rules]]
selector = ".MuiFormGroup-root.MuiFormGroup-row"
set = { self = { text_align = "center" } }

[[stylesheets.rules]]
selector = ".MuiRadioGroup-root.MuiRadioGroup-row"
set = { self = { role = "radiogroup-row" } }

[[stylesheets.rules]]
selector = ".MuiFormHelperText-root.MuiFormHelperText-sizeSmall.MuiFormHelperText-contained.MuiFormHelperText-filled.MuiFormHelperText-required"
set = { self = { text_tone = "muted" } }

[[stylesheets.rules]]
selector = ".MuiFormLabel-root.MuiFormLabel-colorSecondary.MuiFormLabel-filled.MuiFormLabel-required"
set = { self = { text_tone = "secondary" } }

[[stylesheets.rules]]
selector = ".MuiInputAdornment-root.MuiInputAdornment-positionStart.MuiInputAdornment-filled.MuiInputAdornment-disablePointerEvents.MuiInputAdornment-hiddenLabel.MuiInputAdornment-sizeSmall"
set = { self = { surface_variant = "adornment" } }

[[stylesheets.rules]]
selector = ".MuiInputLabel-root.MuiInputLabel-formControl.MuiInputLabel-sizeSmall.MuiInputLabel-shrink.MuiInputLabel-animated.MuiInputLabel-outlined.MuiInputLabel-required"
set = { self = { text_tone = "label" } }

[[stylesheets.rules]]
selector = ".MuiNativeSelect-select.MuiNativeSelect-outlined.MuiNativeSelect-multiple"
set = { self = { surface_variant = "native-select" } }

[[stylesheets.rules]]
selector = ".MuiNativeSelect-icon.MuiNativeSelect-iconOutlined.MuiNativeSelect-iconOpen"
set = { self = { text_tone = "select-icon" } }

[[stylesheets.rules]]
selector = ".MuiScopedCssBaseline-root"
set = { self = { color_scheme = "scoped" } }

[[stylesheets.rules]]
selector = ".MuiTextField-root.MuiFormControl-root.MuiTextField-outlined.MuiTextField-sizeSmall.MuiTextField-colorSecondary.MuiTextField-fullWidth.MuiTextField-required.MuiTextField-focused.MuiTextField-error"
set = { self = { surface_variant = "textfield-root", component_variant = "outlined" } }

[[stylesheets.rules]]
selector = ".MuiTextField-input.MuiOutlinedInput-root.MuiInputBase-root.MuiOutlinedInput-focused.MuiOutlinedInput-error.MuiOutlinedInput-sizeSmall.MuiOutlinedInput-colorSecondary.MuiOutlinedInput-adornedStart.text-field-input-extra"
set = { self = { surface_variant = "textfield-input", component_variant = "outlined focused error" } }

[[stylesheets.rules]]
selector = ".MuiTextField-htmlInput.MuiOutlinedInput-input.MuiInputBase-input.MuiInputBase-inputTypeSearch.Mui-readOnly.text-field-html-input-extra"
set = { self = { text_tone = "textfield-html-input" } }

[[stylesheets.rules]]
selector = ".MuiTextField-inputLabel.MuiInputLabel-root.MuiInputLabel-outlined.MuiInputLabel-sizeSmall.MuiInputLabel-shrink.MuiInputLabel-focused.MuiInputLabel-error.MuiInputLabel-required"
set = { self = { text_tone = "textfield-label" } }

[[stylesheets.rules]]
selector = ".MuiTextField-formHelperText.MuiFormHelperText-root.MuiFormHelperText-contained.MuiFormHelperText-sizeSmall.MuiFormHelperText-focused.MuiFormHelperText-error.MuiFormHelperText-required"
set = { self = { text_tone = "textfield-helper" } }

[[stylesheets.rules]]
selector = ".MuiAutocomplete-root.MuiAutocomplete-expanded.MuiAutocomplete-focused.MuiAutocomplete-fullWidth.MuiAutocomplete-hasClearIcon.MuiAutocomplete-hasPopupIcon"
set = { self = { validation_level = "autocomplete-root" } }

[[stylesheets.rules]]
selector = ".MuiAutocomplete-inputRoot.MuiAutocomplete-hasClearIcon.MuiAutocomplete-hasPopupIcon.autocomplete-input-root-extra"
set = { self = { surface_variant = "autocomplete-input-root" } }

[[stylesheets.rules]]
selector = ".MuiAutocomplete-input.MuiAutocomplete-inputFocused.autocomplete-input-extra"
set = { self = { text_tone = "autocomplete-input" } }

[[stylesheets.rules]]
selector = ".MuiAutocomplete-tag.MuiAutocomplete-tagSizeSmall.autocomplete-tag-extra"
set = { self = { surface_variant = "autocomplete-tag" } }

[[stylesheets.rules]]
selector = ".MuiAutocomplete-endAdornment"
set = { self = { surface_variant = "autocomplete-end-adornment" } }

[[stylesheets.rules]]
selector = ".MuiAutocomplete-clearIndicator"
set = { self = { surface_variant = "autocomplete-clear-indicator" } }

[[stylesheets.rules]]
selector = ".MuiAutocomplete-popupIndicator.MuiAutocomplete-popupIndicatorOpen.autocomplete-popup-extra"
set = { self = { text_tone = "autocomplete-popup" } }

[[stylesheets.rules]]
selector = ".MuiAutocomplete-popper.MuiAutocomplete-popperDisablePortal.autocomplete-popper-extra"
set = { self = { role = "autocomplete-popper" } }

[[stylesheets.rules]]
selector = ".MuiAutocomplete-paper"
set = { self = { surface_variant = "autocomplete-paper" } }

[[stylesheets.rules]]
selector = ".MuiAutocomplete-listbox"
set = { self = { surface_variant = "autocomplete-listbox" } }

[[stylesheets.rules]]
selector = ".MuiAutocomplete-loading"
set = { self = { surface_variant = "autocomplete-loading" } }

[[stylesheets.rules]]
selector = ".MuiAutocomplete-noOptions"
set = { self = { surface_variant = "autocomplete-no-options" } }

[[stylesheets.rules]]
selector = ".MuiAutocomplete-option.MuiAutocomplete-focused.MuiAutocomplete-focusVisible"
set = { self = { surface_variant = "autocomplete-option" } }

[[stylesheets.rules]]
selector = ".MuiAutocomplete-groupLabel"
set = { self = { surface_variant = "autocomplete-group-label" } }

[[stylesheets.rules]]
selector = ".MuiAutocomplete-groupUl"
set = { self = { surface_variant = "autocomplete-group-ul" } }
"##;

const FORM_LAYOUT_TOML: &str = r##"
[asset]
kind = "layout"
id = "ui.tests.mui_web_form_style_layout"
version = 1
display_name = "MUI Web Form Style Layout"

[imports]
styles = ["asset://ui/tests/mui_web_form_style.ui"]

[root]
node_id = "form_root"
kind = "native"
type = "VerticalBox"
control_id = "FormRoot"

[[root.children]]
[root.children.node]
node_id = "button_base"
kind = "native"
type = "ButtonBase"
control_id = "ButtonBaseRoot"
props = { focusVisible = true }

[[root.children]]
[root.children.node]
node_id = "input_base"
kind = "native"
type = "InputBase"
control_id = "InputBaseRoot"
props = { formControl = true, size = "small", startAdornment = "$", fullWidth = true, readOnly = true, type = "search", slotProps = { input = { className = "input-extra" } } }

[[root.children.node.children]]
mount = "input"
[root.children.node.children.node]
node_id = "input_base_input"
kind = "native"
type = "Label"
control_id = "InputBaseInput"
props = { text = "Input" }

[[root.children]]
[root.children.node]
node_id = "filled_input"
kind = "native"
type = "FilledInput"
control_id = "FilledInputRoot"
props = { endAdornment = "kg", size = "small" }

[[root.children]]
[root.children.node]
node_id = "outlined_input"
kind = "native"
type = "OutlinedInput"
control_id = "OutlinedInputRoot"
props = { notched = true, slotProps = { notchedOutline = { className = "notched-extra" } } }

[[root.children.node.children]]
mount = "notchedOutline"
[root.children.node.children.node]
node_id = "notched_outline"
kind = "native"
type = "Label"
control_id = "NotchedOutline"
props = { text = "Outline" }

[[root.children]]
[root.children.node]
node_id = "form_control"
kind = "native"
type = "FormControl"
control_id = "FormControlRoot"
props = { margin = "dense", fullWidth = true }

[[root.children]]
[root.children.node]
node_id = "form_control_label"
kind = "native"
type = "FormControlLabel"
control_id = "FormControlLabelRoot"
props = { labelPlacement = "start", required = true, error = true, slotProps = { label = { className = "label-extra" } } }

[[root.children.node.children]]
mount = "label"
[root.children.node.children.node]
node_id = "form_control_label_text"
kind = "native"
type = "Label"
control_id = "FormControlLabelText"
props = { text = "Label" }

[[root.children]]
[root.children.node]
node_id = "form_group"
kind = "native"
type = "FormGroup"
control_id = "FormGroupRoot"
props = { row = true }

[[root.children]]
[root.children.node]
node_id = "radio_group"
kind = "native"
type = "RadioGroup"
control_id = "RadioGroupRoot"
props = { row = true }

[[root.children]]
[root.children.node]
node_id = "helper_text"
kind = "native"
type = "FormHelperText"
control_id = "HelperTextRoot"
props = { text = "Hint", size = "small", variant = "outlined", filled = true, required = true }

[[root.children]]
[root.children.node]
node_id = "form_label"
kind = "native"
type = "FormLabel"
control_id = "FormLabelRoot"
props = { text = "Name", color = "secondary", filled = true, required = true }

[[root.children]]
[root.children.node]
node_id = "input_adornment"
kind = "native"
type = "InputAdornment"
control_id = "AdornmentRoot"
props = { text = "$", position = "start", variant = "filled", disablePointerEvents = true, hiddenLabel = true, size = "small" }

[[root.children]]
[root.children.node]
node_id = "input_label"
kind = "native"
type = "InputLabel"
control_id = "InputLabelRoot"
props = { text = "Label", formControl = true, size = "small", shrink = true, variant = "outlined", required = true }

[[root.children]]
[root.children.node]
node_id = "native_select"
kind = "native"
type = "NativeSelect"
control_id = "NativeSelectRoot"
props = { variant = "outlined", multiple = true, open = true }

[[root.children.node.children]]
mount = "select"
[root.children.node.children.node]
node_id = "native_select_select"
kind = "native"
type = "Label"
control_id = "NativeSelectSelect"
props = { text = "Select" }

[[root.children.node.children]]
mount = "icon"
[root.children.node.children.node]
node_id = "native_select_icon"
kind = "native"
type = "Label"
control_id = "NativeSelectIcon"
props = { text = "Icon" }

[[root.children]]
[root.children.node]
node_id = "scoped_baseline"
kind = "native"
type = "ScopedCssBaseline"
control_id = "ScopedBaselineRoot"
props = { enableColorScheme = true }

[[root.children]]
[root.children.node]
node_id = "text_field"
kind = "native"
type = "TextField"
control_id = "TextFieldRoot"
props = { label = "Search", value = "atlas", variant = "outlined", size = "small", color = "secondary", fullWidth = true, required = true, focused = true, error = true, readOnly = true, type = "search", startAdornment = "?", slotProps = { input = { className = "text-field-input-extra" }, htmlInput = { className = "text-field-html-input-extra" } } }

[[root.children.node.children]]
mount = "input"
[root.children.node.children.node]
node_id = "text_field_input"
kind = "native"
type = "InputField"
control_id = "TextFieldInput"
props = { text = "Atlas" }

[[root.children.node.children]]
mount = "htmlInput"
[root.children.node.children.node]
node_id = "text_field_html_input"
kind = "native"
type = "Label"
control_id = "TextFieldHtmlInput"
props = { text = "Atlas" }

[[root.children.node.children]]
mount = "inputLabel"
[root.children.node.children.node]
node_id = "text_field_input_label"
kind = "native"
type = "Label"
control_id = "TextFieldInputLabel"
props = { text = "Search" }

[[root.children.node.children]]
mount = "formHelperText"
[root.children.node.children.node]
node_id = "text_field_helper"
kind = "native"
type = "Label"
control_id = "TextFieldHelper"
props = { text = "Required" }

[[root.children]]
[root.children.node]
node_id = "autocomplete"
kind = "native"
type = "Autocomplete"
control_id = "AutocompleteRoot"
props = { query = "at", value = "atlas", value_text = "Atlas", selected_options = ["atlas"], focused_options = ["atlas"], options = ["atlas", "asset"], size = "small", fullWidth = true, popup_open = true, focused = true, inputFocused = true, disablePortal = true, forcePopupIcon = "auto", freeSolo = false, slotProps = { inputRoot = { className = "autocomplete-input-root-extra" }, input = { className = "autocomplete-input-extra" }, tag = { className = "autocomplete-tag-extra" }, popupIndicator = { className = "autocomplete-popup-extra" }, popper = { className = "autocomplete-popper-extra" } } }

[[root.children.node.children]]
mount = "inputRoot"
[root.children.node.children.node]
node_id = "autocomplete_input_root"
kind = "native"
type = "Label"
control_id = "AutocompleteInputRoot"
props = { text = "Input Root" }

[[root.children.node.children]]
mount = "input"
[root.children.node.children.node]
node_id = "autocomplete_input"
kind = "native"
type = "Label"
control_id = "AutocompleteInput"
props = { text = "Input" }

[[root.children.node.children]]
mount = "tag"
[root.children.node.children.node]
node_id = "autocomplete_tag"
kind = "native"
type = "Label"
control_id = "AutocompleteTag"
props = { text = "Atlas" }

[[root.children.node.children]]
mount = "endAdornment"
[root.children.node.children.node]
node_id = "autocomplete_end_adornment"
kind = "native"
type = "Label"
control_id = "AutocompleteEndAdornment"
props = { text = "End" }

[[root.children.node.children]]
mount = "clearIndicator"
[root.children.node.children.node]
node_id = "autocomplete_clear_indicator"
kind = "native"
type = "Label"
control_id = "AutocompleteClearIndicator"
props = { text = "Clear" }

[[root.children.node.children]]
mount = "popupIndicator"
[root.children.node.children.node]
node_id = "autocomplete_popup_indicator"
kind = "native"
type = "Label"
control_id = "AutocompletePopupIndicator"
props = { text = "Open" }

[[root.children.node.children]]
mount = "popper"
[root.children.node.children.node]
node_id = "autocomplete_popper"
kind = "native"
type = "Label"
control_id = "AutocompletePopper"
props = { text = "Popper" }

[[root.children.node.children]]
mount = "paper"
[root.children.node.children.node]
node_id = "autocomplete_paper"
kind = "native"
type = "Label"
control_id = "AutocompletePaper"
props = { text = "Paper" }

[[root.children.node.children]]
mount = "listbox"
[root.children.node.children.node]
node_id = "autocomplete_listbox"
kind = "native"
type = "Label"
control_id = "AutocompleteListbox"
props = { text = "Listbox" }

[[root.children.node.children]]
mount = "loading"
[root.children.node.children.node]
node_id = "autocomplete_loading"
kind = "native"
type = "Label"
control_id = "AutocompleteLoading"
props = { text = "Loading" }

[[root.children.node.children]]
mount = "noOptions"
[root.children.node.children.node]
node_id = "autocomplete_no_options"
kind = "native"
type = "Label"
control_id = "AutocompleteNoOptions"
props = { text = "No Options" }

[[root.children.node.children]]
mount = "option"
[root.children.node.children.node]
node_id = "autocomplete_option"
kind = "native"
type = "Label"
control_id = "AutocompleteOption"
props = { text = "Atlas", focused = true, focusVisible = true }

[[root.children.node.children]]
mount = "groupLabel"
[root.children.node.children.node]
node_id = "autocomplete_group_label"
kind = "native"
type = "Label"
control_id = "AutocompleteGroupLabel"
props = { text = "Group" }

[[root.children.node.children]]
mount = "groupUl"
[root.children.node.children.node]
node_id = "autocomplete_group_ul"
kind = "native"
type = "Label"
control_id = "AutocompleteGroupUl"
props = { text = "Group Ul" }
"##;

#[test]
fn mui_web_form_utility_classes_match_local_material_contracts() {
    let style = UiAssetLoader::load_toml_str(FORM_STYLE_TOML).unwrap();
    let layout = UiAssetLoader::load_toml_str(FORM_LAYOUT_TOML).unwrap();
    let mut compiler = UiDocumentCompiler::default();
    compiler
        .register_style_import("asset://ui/tests/mui_web_form_style.ui", style)
        .unwrap();

    let compiled = compiler.compile(&layout).unwrap();
    let root = &compiled.template_instance().root;

    assert_eq!(
        bool_attr(find_node(root, "ButtonBaseRoot"), "state_layer_enabled"),
        Some(true)
    );
    let input_base = find_node(root, "InputBaseRoot");
    assert_eq!(str_attr(input_base, "validation_level"), Some("inputbase"));
    assert_not_classes(
        input_base,
        &["MuiInputBase-colorPrimary", "MuiInputBase-sizeMedium"],
    );
    assert_eq!(
        str_attr(find_node(root, "InputBaseInput"), "text_tone"),
        Some("info")
    );

    let filled = find_node(root, "FilledInputRoot");
    assert_eq!(str_attr(filled, "surface_variant"), Some("filled-input"));
    assert_classes(
        filled,
        &[
            "MuiFilledInput-root",
            "MuiInputBase-root",
            "MuiFilledInput-underline",
            "MuiFilledInput-adornedEnd",
            "MuiInputBase-adornedEnd",
        ],
    );
    assert_eq!(
        float_attr(find_node(root, "NotchedOutline"), "border_width"),
        Some(2.0)
    );
    assert_eq!(
        str_attr(find_node(root, "FormControlRoot"), "surface_variant"),
        Some("form-control")
    );
    assert_eq!(
        str_attr(find_node(root, "FormControlLabelRoot"), "validation_level"),
        Some("label-error")
    );
    assert_eq!(
        str_attr(find_node(root, "FormControlLabelText"), "text_tone"),
        Some("warning")
    );
    assert_eq!(
        str_attr(find_node(root, "FormGroupRoot"), "text_align"),
        Some("center")
    );
    assert_eq!(
        str_attr(find_node(root, "RadioGroupRoot"), "role"),
        Some("radiogroup-row")
    );
    assert_eq!(
        str_attr(find_node(root, "HelperTextRoot"), "text_tone"),
        Some("muted")
    );
    assert_eq!(
        str_attr(find_node(root, "FormLabelRoot"), "text_tone"),
        Some("secondary")
    );
    assert_eq!(
        str_attr(find_node(root, "AdornmentRoot"), "surface_variant"),
        Some("adornment")
    );
    assert_eq!(
        str_attr(find_node(root, "InputLabelRoot"), "text_tone"),
        Some("label")
    );
    assert_eq!(
        str_attr(find_node(root, "NativeSelectSelect"), "surface_variant"),
        Some("native-select")
    );
    assert_eq!(
        str_attr(find_node(root, "NativeSelectIcon"), "text_tone"),
        Some("select-icon")
    );
    assert_eq!(
        str_attr(find_node(root, "ScopedBaselineRoot"), "color_scheme"),
        Some("scoped")
    );

    let text_field = find_node(root, "TextFieldRoot");
    assert_eq!(
        str_attr(text_field, "surface_variant"),
        Some("textfield-root")
    );
    assert_eq!(
        str_attr(text_field, "component_variant"),
        Some("outlined")
    );
    assert_classes(
        text_field,
        &[
            "MuiTextField-root",
            "MuiFormControl-root",
            "MuiTextField-outlined",
            "MuiTextField-sizeSmall",
            "MuiTextField-colorSecondary",
            "MuiTextField-fullWidth",
            "MuiTextField-required",
            "MuiTextField-focused",
            "MuiTextField-error",
        ],
    );
    let text_field_input = find_node(root, "TextFieldInput");
    assert_eq!(
        str_attr(text_field_input, "surface_variant"),
        Some("textfield-input")
    );
    assert_classes(
        text_field_input,
        &[
            "MuiTextField-input",
            "MuiOutlinedInput-root",
            "MuiInputBase-root",
            "MuiOutlinedInput-focused",
            "MuiOutlinedInput-error",
            "MuiOutlinedInput-sizeSmall",
            "MuiOutlinedInput-colorSecondary",
            "MuiOutlinedInput-adornedStart",
        ],
    );
    assert_eq!(
        str_attr(find_node(root, "TextFieldHtmlInput"), "text_tone"),
        Some("textfield-html-input")
    );
    assert_eq!(
        str_attr(find_node(root, "TextFieldInputLabel"), "text_tone"),
        Some("textfield-label")
    );
    assert_eq!(
        str_attr(find_node(root, "TextFieldHelper"), "text_tone"),
        Some("textfield-helper")
    );

    let autocomplete = find_node(root, "AutocompleteRoot");
    assert_eq!(
        str_attr(autocomplete, "validation_level"),
        Some("autocomplete-root")
    );
    assert_not_classes(
        autocomplete,
        &["MuiAutocomplete-colorPrimary", "MuiAutocomplete-sizeMedium"],
    );
    assert_eq!(
        str_attr(find_node(root, "AutocompleteInputRoot"), "surface_variant"),
        Some("autocomplete-input-root")
    );
    assert_eq!(
        str_attr(find_node(root, "AutocompleteInput"), "text_tone"),
        Some("autocomplete-input")
    );
    assert_eq!(
        str_attr(find_node(root, "AutocompleteTag"), "surface_variant"),
        Some("autocomplete-tag")
    );
    assert_eq!(
        str_attr(find_node(root, "AutocompletePopupIndicator"), "text_tone"),
        Some("autocomplete-popup")
    );
    assert_eq!(
        str_attr(find_node(root, "AutocompletePopper"), "role"),
        Some("autocomplete-popper")
    );
    for (control_id, expected_variant) in [
        ("AutocompleteEndAdornment", "autocomplete-end-adornment"),
        ("AutocompleteClearIndicator", "autocomplete-clear-indicator"),
        ("AutocompletePaper", "autocomplete-paper"),
        ("AutocompleteListbox", "autocomplete-listbox"),
        ("AutocompleteLoading", "autocomplete-loading"),
        ("AutocompleteNoOptions", "autocomplete-no-options"),
        ("AutocompleteOption", "autocomplete-option"),
        ("AutocompleteGroupLabel", "autocomplete-group-label"),
        ("AutocompleteGroupUl", "autocomplete-group-ul"),
    ] {
        assert_eq!(
            str_attr(find_node(root, control_id), "surface_variant"),
            Some(expected_variant),
            "{control_id} should receive the local MUI Autocomplete slot utility class"
        );
    }
}

fn find_node<'a>(node: &'a UiTemplateNode, control_id: &str) -> &'a UiTemplateNode {
    if node.control_id.as_deref() == Some(control_id) {
        return node;
    }
    for child in &node.children {
        if let Some(found) = find_node_opt(child, control_id) {
            return found;
        }
    }
    panic!("missing node `{control_id}`");
}

fn find_node_opt<'a>(node: &'a UiTemplateNode, control_id: &str) -> Option<&'a UiTemplateNode> {
    if node.control_id.as_deref() == Some(control_id) {
        return Some(node);
    }
    node.children
        .iter()
        .find_map(|child| find_node_opt(child, control_id))
}

fn str_attr<'a>(node: &'a UiTemplateNode, name: &str) -> Option<&'a str> {
    node.attributes.get(name).and_then(Value::as_str)
}

fn float_attr(node: &UiTemplateNode, name: &str) -> Option<f64> {
    node.attributes.get(name).and_then(Value::as_float)
}

fn bool_attr(node: &UiTemplateNode, name: &str) -> Option<bool> {
    node.attributes.get(name).and_then(Value::as_bool)
}

fn assert_classes(node: &UiTemplateNode, expected: &[&str]) {
    for class_name in expected {
        assert!(
            node.classes.iter().any(|value| value == class_name),
            "missing {class_name} in {:?}",
            node.classes
        );
    }
}

fn assert_not_classes(node: &UiTemplateNode, unexpected: &[&str]) {
    for class_name in unexpected {
        assert!(
            !node.classes.iter().any(|value| value == class_name),
            "unexpected {class_name} in {:?}",
            node.classes
        );
    }
}
