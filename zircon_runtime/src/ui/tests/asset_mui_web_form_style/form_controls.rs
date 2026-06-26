use super::*;

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
    assert_eq!(str_attr(text_field, "component_variant"), Some("outlined"));
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
