use super::*;

#[test]
fn mui_x_date_time_picker_utility_classes_match_retained_targets() {
    let style = UiAssetLoader::load_toml_str(MUI_X_STYLE_TOML).unwrap();
    let layout = UiAssetLoader::load_toml_str(MUI_X_LAYOUT_TOML).unwrap();
    let mut compiler = UiDocumentCompiler::default();
    compiler
        .register_style_import("asset://ui/tests/mui_web_mui_x_style.ui", style)
        .unwrap();
    let compiled = compiler.compile(&layout).unwrap();
    let root = &compiled.template_instance().root;

    let pickers = find_node(root, "DateTimePickersRoot");
    assert_eq!(
        str_attr(pickers, "surface_variant"),
        Some("date-time-picker")
    );
    assert_classes(
        pickers,
        &[
            "MuiDateTimePickers-root",
            "MuiDateTimePicker-root",
            "MuiPickersLayout-Desktop",
        ],
    );

    let mobile_date_picker = find_node(root, "DatePickerMobileRoot");
    assert_eq!(
        str_attr(mobile_date_picker, "surface_variant"),
        Some("date-picker-mobile")
    );
    assert_classes(
        mobile_date_picker,
        &["MuiDatePicker-root", "MuiPickersLayout-Mobile"],
    );

    let static_time_picker = find_node(root, "TimePickerStaticRoot");
    assert_eq!(
        str_attr(static_time_picker, "surface_variant"),
        Some("time-picker-static")
    );
    assert_classes(
        static_time_picker,
        &["MuiTimePicker-root", "MuiPickersLayout-Static"],
    );

    let picker_state_flags = find_node(root, "DateTimePickerStateFlagsRoot");
    assert_eq!(
        str_attr(picker_state_flags, "validation_level"),
        Some("picker-state-flags")
    );
    assert_classes(
        picker_state_flags,
        &[
            "MuiDateTimePicker-root",
            "MuiPickersLayout-Desktop",
            "MuiPickers-readOnly",
            "MuiPickers-ampm",
            "MuiPickers-hasDateBounds",
            "MuiPickers-hasViews",
        ],
    );

    let picker_value_state = find_node(root, "DateTimePickerValueStateRoot");
    assert_eq!(
        str_attr(picker_value_state, "text_tone"),
        Some("picker-value-bound")
    );
    assert_classes(
        picker_value_state,
        &[
            "MuiDateTimePicker-root",
            "MuiPickers-hasValue",
            "MuiPickers-hasView",
            "MuiPickers-hasFormat",
        ],
    );

    let picker_field = find_node(root, "DateTimePickerField");
    assert_eq!(
        str_attr(picker_field, "text_tone"),
        Some("picker-field-state")
    );
    assert_classes(
        picker_field,
        &[
            "MuiPickersField",
            "MuiPickersField-readOnly",
            "MuiPickersField-hasValue",
            "MuiPickersField-hasFormat",
        ],
    );

    let picker_layout = find_node(root, "DateTimePickerLayout");
    assert_eq!(
        str_attr(picker_layout, "surface_variant"),
        Some("picker-layout-views")
    );
    assert_classes(
        picker_layout,
        &[
            "MuiPickersLayout",
            "MuiPickersLayout-Desktop",
            "MuiPickersLayout-hasViews",
        ],
    );

    let picker_toolbar = find_node(root, "DateTimePickerToolbar");
    assert_eq!(
        str_attr(picker_toolbar, "text_tone"),
        Some("picker-toolbar-state")
    );
    assert_classes(
        picker_toolbar,
        &[
            "MuiPickersToolbar",
            "MuiPickersToolbar-ampm",
            "MuiPickersToolbar-hasViews",
        ],
    );

    let picker_popper = find_node(root, "DateTimePickerPopper");
    assert_eq!(
        str_attr(picker_popper, "surface_variant"),
        Some("picker-popper-open-bounds")
    );
    assert_classes(
        picker_popper,
        &[
            "MuiPickersPopper",
            "MuiPickersPopper-open",
            "MuiPickersPopper-hasDateBounds",
        ],
    );
}
