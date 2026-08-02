use super::*;

#[test]
fn componentized_workbench_window_template_bridge_projects_input_selection_and_feedback_controls() {
    let _guard = env_lock().lock().unwrap();
    let (bridge, nodes) = componentized_workbench_projection_fixture();
    let component_inputs = template_contract_node(&nodes, "WorkbenchComponentInputs");
    let component_selection = template_contract_node(&nodes, "WorkbenchComponentSelection");
    let component_sliders = template_contract_node(&nodes, "WorkbenchComponentSliders");
    let component_labs = template_contract_node(&nodes, "WorkbenchComponentLabs");
    let component_list = template_contract_node(&nodes, "WorkbenchComponentList");
    let component_lower_row = template_contract_node(&nodes, "WorkbenchComponentLowerRow");

    let component_inputs = template_contract_node(&nodes, "WorkbenchComponentInputs");
    let component_selection = template_contract_node(&nodes, "WorkbenchComponentSelection");
    let component_sliders = template_contract_node(&nodes, "WorkbenchComponentSliders");
    let component_labs = template_contract_node(&nodes, "WorkbenchComponentLabs");
    let labs_tabs = template_contract_node(&nodes, "WorkbenchLabsTabs");
    let component_list = template_contract_node(&nodes, "WorkbenchComponentList");
    assert_eq!(component_inputs.frame.width, 214.0);
    assert_eq!(component_selection.frame.width, 168.0);
    assert_eq!(component_sliders.frame.width, 260.0);
    assert_eq!(component_labs.frame.width, 236.0);
    assert_eq!(labs_tabs.frame.width, 216.0);
    assert_eq!(
        style_color_u8(labs_tabs.button_style.element.background_color.as_ref()),
        Some([20, 25, 29, 255])
    );
    assert!(component_selection.frame.x > component_inputs.frame.x + component_inputs.frame.width);
    assert!(
        component_sliders.frame.x > component_selection.frame.x + component_selection.frame.width
    );
    assert!(component_labs.frame.x > component_sliders.frame.x + component_sliders.frame.width);
    assert!(component_list.frame.x > component_labs.frame.x + component_labs.frame.width);
    assert_eq!(component_selection.layout_content_offset_x, 9.0);
    let checkbox_on = template_contract_node(&nodes, "WorkbenchCheckboxOn");
    assert_eq!(checkbox_on.layout_icon_size, 16.0);
    assert_eq!(checkbox_on.layout_content_offset_x, 9.0);
    assert_eq!(
        checkbox_on.label_color,
        crate::ui::retained_host::primitives::Color::from_rgb_u8(130, 140, 147)
    );
    assert_eq!(
        style_color_u8(checkbox_on.button_style.element.background_color.as_ref()),
        Some([23, 57, 66, 255])
    );
    assert_eq!(
        style_color_u8(checkbox_on.button_style.element.border_color.as_ref()),
        Some([42, 166, 184, 255])
    );
    let checkbox_off = template_contract_node(&nodes, "WorkbenchCheckboxOff");
    assert_eq!(checkbox_off.layout_icon_size, 16.0);
    assert_eq!(checkbox_off.layout_content_offset_x, 9.0);
    assert_eq!(
        checkbox_off.label_color,
        crate::ui::retained_host::primitives::Color::from_rgb_u8(130, 140, 147)
    );
    assert_eq!(
        style_color_u8(checkbox_off.button_style.element.background_color.as_ref()),
        Some([20, 26, 30, 255])
    );
    assert_eq!(
        style_color_u8(checkbox_off.button_style.element.border_color.as_ref()),
        Some([66, 78, 86, 255])
    );
    let radio_on = template_contract_node(&nodes, "WorkbenchRadioOn");
    assert_eq!(radio_on.layout_icon_size, 16.0);
    assert_eq!(radio_on.layout_content_offset_x, 9.0);
    assert_eq!(radio_on.value_number, 7.0);
    assert_eq!(
        radio_on.label_color,
        crate::ui::retained_host::primitives::Color::from_rgb_u8(130, 140, 147)
    );
    assert_eq!(
        radio_on.value_color,
        crate::ui::retained_host::primitives::Color::from_rgb_u8(42, 166, 184)
    );
    assert_eq!(
        style_color_u8(radio_on.button_style.element.background_color.as_ref()),
        Some([27, 39, 45, 255])
    );
    assert_eq!(
        style_color_u8(radio_on.button_style.element.border_color.as_ref()),
        Some([76, 91, 99, 255])
    );
    let radio_off = template_contract_node(&nodes, "WorkbenchRadioOff");
    assert_eq!(radio_off.layout_icon_size, 16.0);
    assert_eq!(radio_off.layout_content_offset_x, 9.0);
    assert_eq!(radio_off.value_number, 7.0);
    assert_eq!(
        radio_off.label_color,
        crate::ui::retained_host::primitives::Color::from_rgb_u8(130, 140, 147)
    );
    assert_eq!(
        radio_off.value_color,
        crate::ui::retained_host::primitives::Color::from_rgb_u8(42, 166, 184)
    );
    assert_eq!(
        style_color_u8(radio_off.button_style.element.background_color.as_ref()),
        Some([20, 26, 30, 255])
    );
    assert_eq!(
        style_color_u8(radio_off.button_style.element.border_color.as_ref()),
        Some([66, 78, 86, 255])
    );
    let toggle = template_contract_node(&nodes, "WorkbenchToggleOn");
    assert_eq!(toggle.value_number, 34.0);
    assert_eq!(toggle.layout_icon_size, 12.0);
    assert_eq!(toggle.layout_content_offset_x, 10.0);
    assert_eq!(toggle.layout_content_offset_y, 18.0);
    assert_eq!(
        style_color_u8(toggle.button_style.element.background_color.as_ref()),
        Some([23, 57, 66, 255])
    );
    assert_eq!(
        style_color_u8(toggle.button_style.element.foreground_color.as_ref()),
        Some([164, 174, 180, 255])
    );
    assert_eq!(
        style_color_u8(toggle.button_style.element.border_color.as_ref()),
        Some([65, 75, 84, 255])
    );
    assert!(toggle.frame.x >= component_labs.frame.x);
    assert!(
        toggle.frame.x + toggle.frame.width
            <= component_labs.frame.x + component_labs.frame.width + 0.001
    );
    let component_table = template_contract_node(&nodes, "WorkbenchComponentTable");
    let component_feedback = template_contract_node(&nodes, "WorkbenchComponentFeedback");
    assert_eq!(component_table.frame.width, 590.0);
    assert!(component_table.frame.y >= component_lower_row.frame.y);
    assert!(component_feedback.frame.y >= component_lower_row.frame.y);
    let feedback_alerts = template_contract_node(&nodes, "WorkbenchFeedbackAlerts");
    let feedback_toast_column = template_contract_node(&nodes, "WorkbenchFeedbackToastColumn");
    assert_eq!(feedback_alerts.frame.width, 390.0);
    assert_eq!(feedback_toast_column.frame.width, 390.0);
    assert!(feedback_alerts.frame.x >= component_feedback.frame.x);
    assert!(
        feedback_alerts.frame.x + feedback_alerts.frame.width
            <= component_feedback.frame.x + component_feedback.frame.width + 0.001
    );
    let info_alert = template_contract_node(&nodes, "WorkbenchInfoAlert");
    let success_alert = template_contract_node(&nodes, "WorkbenchSuccessAlert");
    let warning_alert = template_contract_node(&nodes, "WorkbenchWarningAlert");
    let error_alert = template_contract_node(&nodes, "WorkbenchErrorAlert");
    assert!(info_alert.frame.x >= feedback_alerts.frame.x);
    assert!(
        info_alert.frame.x + info_alert.frame.width
            <= feedback_alerts.frame.x + feedback_alerts.frame.width + 0.001
    );
    for alert in [&info_alert, &success_alert, &warning_alert, &error_alert] {
        assert_eq!(alert.frame.height, 30.0);
    }
    assert!(
        (success_alert.frame.y - (info_alert.frame.y + info_alert.frame.height + 4.0)).abs()
            < 0.001
    );
    assert!(
        (warning_alert.frame.y - (success_alert.frame.y + success_alert.frame.height + 4.0)).abs()
            < 0.001
    );
    assert!(
        (error_alert.frame.y - (warning_alert.frame.y + warning_alert.frame.height + 4.0)).abs()
            < 0.001
    );
    assert!(
        error_alert.frame.y + error_alert.frame.height
            <= feedback_alerts.frame.y + feedback_alerts.frame.height + 0.001
    );
    let feedback_tooltip = template_contract_node(&nodes, "WorkbenchTooltipRoot");
    let standalone_toast = template_contract_node(&nodes, "WorkbenchToastRoot");
    assert!(feedback_tooltip.frame.x > feedback_alerts.frame.x + feedback_alerts.frame.width);
    assert!(
        feedback_toast_column.frame.x > feedback_tooltip.frame.x + feedback_tooltip.frame.width
    );
    assert!(standalone_toast.frame.x >= feedback_toast_column.frame.x);
    assert!(
        standalone_toast.frame.x + standalone_toast.frame.width
            <= feedback_toast_column.frame.x + feedback_toast_column.frame.width + 0.001
    );
    assert_eq!(standalone_toast.frame.height, 30.0);
    assert!(standalone_toast.frame.y > feedback_tooltip.frame.y);
    assert!(
        standalone_toast.frame.y + standalone_toast.frame.height
            <= component_feedback.frame.y + component_feedback.frame.height + 0.001
    );
    assert!(bridge
        .host_projection()
        .node_by_control_id("WorkbenchFeedbackToastOffset")
        .is_none());
    let table_item = template_contract_node(&nodes, "WorkbenchTableItem");
    assert_eq!(table_item.role.as_str(), "Table");
    assert_eq!(table_item.component_role.as_str(), "table");
    assert_eq!(table_item.options.row_count(), 4);
    assert_eq!(table_item.options.row_data(0).as_deref(), Some("Item_01"));
    assert_eq!(table_item.options.row_data(1).as_deref(), Some("Mesh"));
    assert_eq!(table_item.options.row_data(2).as_deref(), Some("2.4 MB"));
    assert_eq!(table_item.options.row_data(3).as_deref(), Some("2m ago"));
    assert_eq!(table_item.layout_first_cell_offset_x, 4.0);
    assert!(!table_item.selected);
    let table = template_contract_node(&nodes, "WorkbenchTableSelected");
    assert_eq!(table.role.as_str(), "Table");
    assert_eq!(table.component_role.as_str(), "table");
    assert_eq!(table.options.row_count(), 4);
    assert_eq!(table.layout_offset_x, -1.0);
    assert_eq!(table.layout_offset_y, -1.5);
    assert_eq!(table.options.row_data(0).as_deref(), Some("Item_02"));
    assert_eq!(table.options.row_data(1).as_deref(), Some("Material"));
    assert_eq!(table.options.row_data(2).as_deref(), Some("512 KB"));
    assert_eq!(table.options.row_data(3).as_deref(), Some("10m ago"));
    assert_eq!(
        render_background_for_control(&bridge, "WorkbenchTableSelected").as_deref(),
        Some("#12383d")
    );
    let table_header = template_contract_node(&nodes, "WorkbenchTableHeader");
    assert!(table_header.frame.x >= component_table.frame.x);
    assert!(
        table_header.frame.x + table_header.frame.width
            <= component_table.frame.x + component_table.frame.width + 0.001
    );
    assert_eq!(table_header.layout_content_offset_x, -1.0);
    assert_eq!(table_header.layout_content_offset_y, 3.0);
    assert_eq!(table_header.layout_first_cell_offset_x, 0.0);
    assert!(
        (table_item.frame.y - (table_header.frame.y + table_header.frame.height)).abs() < 0.001
    );
    assert!((table.frame.y - (table_item.frame.y + table_item.frame.height)).abs() < 0.001);
    assert_eq!(table.layout_first_cell_offset_x, 0.0);
    let table_tail = template_contract_node(&nodes, "WorkbenchTableTail");
    assert!((table_tail.frame.y - (table.frame.y + table.frame.height)).abs() < 0.001);
    assert_eq!(table_tail.layout_content_offset_y, -0.5);
    assert_eq!(table_tail.layout_first_cell_offset_x, 6.0);
    assert_eq!(table_tail.layout_second_cell_offset_x, 2.0);
    assert_eq!(table_tail.layout_fourth_cell_offset_x, -2.0);
    assert_eq!(
        table_tail.value_color,
        crate::ui::retained_host::primitives::Color::from_rgb_u8(170, 181, 186)
    );
    let segmented = template_contract_node(&nodes, "WorkbenchInputSegmented");
    assert_eq!(segmented.label_text.as_str(), "Segmented Control");
    assert_eq!(
        segmented.label_color,
        crate::ui::retained_host::primitives::Color::from_rgb_u8(161, 172, 178)
    );
    assert!((segmented.label_brightness - 0.94).abs() < 0.001);
    assert_eq!(segmented.layout_offset_x, -0.5);
    assert_eq!(segmented.frame.height, 48.0);
    let labs_tabs = template_contract_node(&nodes, "WorkbenchLabsTabs");
    assert!(segmented.frame.x >= component_labs.frame.x);
    assert!(
        segmented.frame.x + segmented.frame.width
            <= component_labs.frame.x + component_labs.frame.width + 0.001
    );
    assert!(segmented.frame.y > labs_tabs.frame.y + labs_tabs.frame.height);
    assert!(toggle.frame.y > segmented.frame.y + segmented.frame.height);
    let input_slider = template_contract_node(&nodes, "WorkbenchInputSlider");
    assert_eq!(input_slider.label_text.as_str(), "Value");
    assert_eq!(input_slider.value_text.as_str(), "0.75");
    assert_eq!(input_slider.layout_offset_x, -18.0);
    assert_eq!(input_slider.layout_offset_y, 1.0);
    assert_eq!(input_slider.layout_icon_size, 0.0);
    assert_eq!(input_slider.layout_content_offset_x, -10.0);
    assert_eq!(input_slider.layout_first_cell_offset_x, 18.0);
    assert_eq!(
        input_slider.icon_color,
        crate::ui::retained_host::primitives::Color::from_argb_u8(0, 0, 0, 0)
    );
    assert_eq!(
        style_color_u8(input_slider.button_style.element.border_color.as_ref()),
        Some([42, 50, 56, 255])
    );
    assert_eq!(
        input_slider.state_layer_color,
        crate::ui::retained_host::primitives::Color::from_argb_u8(0, 0, 0, 0)
    );
    assert!(input_slider.frame.x >= component_sliders.frame.x);
    assert!(
        input_slider.frame.x + input_slider.frame.width
            <= component_sliders.frame.x + component_sliders.frame.width + 0.001
    );
    assert_eq!(
        input_slider.label_color,
        crate::ui::retained_host::primitives::Color::from_argb_u8(0, 0, 0, 0)
    );
    assert_eq!(
        style_color_u8(input_slider.button_style.element.background_color.as_ref()),
        Some([17, 22, 26, 255])
    );
    assert_eq!(
        input_slider.value_color,
        crate::ui::retained_host::primitives::Color::from_argb_u8(255, 216, 227, 231)
    );
    let range_slider = template_contract_node(&nodes, "WorkbenchInputRangeSlider");
    assert_eq!(range_slider.label_text.as_str(), "Range");
    assert_eq!(range_slider.value_text.as_str(), "0.80");
    assert_eq!(range_slider.layout_offset_x, -18.0);
    assert_eq!(range_slider.layout_icon_size, 11.0);
    assert_eq!(range_slider.layout_content_offset_x, -10.0);
    assert_eq!(range_slider.layout_first_cell_offset_x, 18.0);
    assert_eq!(range_slider.layout_second_cell_offset_x, 20.0);
    assert_eq!(
        range_slider.icon_color,
        crate::ui::retained_host::primitives::Color::from_argb_u8(0, 0, 0, 0)
    );
    assert_eq!(range_slider.frame.height, 46.0);
    assert_eq!(range_slider.value_color, input_slider.value_color);
    assert!(range_slider.frame.x >= component_sliders.frame.x);
    assert!(
        range_slider.frame.x + range_slider.frame.width
            <= component_sliders.frame.x + component_sliders.frame.width + 0.001
    );
    let steps_slider = template_contract_node(&nodes, "WorkbenchInputStepsSlider");
    assert_eq!(steps_slider.label_text.as_str(), "Steps");
    assert_eq!(steps_slider.value_text.as_str(), "3");
    assert_eq!(steps_slider.layout_offset_x, -18.0);
    assert_eq!(steps_slider.layout_icon_size, 0.0);
    assert_eq!(steps_slider.layout_content_offset_x, -10.0);
    assert_eq!(steps_slider.layout_first_cell_offset_x, 18.0);
    assert_eq!(steps_slider.layout_third_cell_offset_x, 5.0);
    assert_eq!(
        steps_slider.icon_color,
        crate::ui::retained_host::primitives::Color::from_argb_u8(0, 0, 0, 0)
    );
    assert!(steps_slider.frame.x >= component_sliders.frame.x);
    assert!(
        steps_slider.frame.x + steps_slider.frame.width
            <= component_sliders.frame.x + component_sliders.frame.width + 0.001
    );
    let list_group = template_contract_node(&nodes, "WorkbenchListGroup");
    let menu_title = template_contract_node(&nodes, "WorkbenchMenuTitle");
    let popup_menu = template_contract_node(&nodes, "WorkbenchPopupMenu");
    assert!(list_group.frame.x >= component_list.frame.x);
    assert!(menu_title.frame.y > list_group.frame.y + list_group.frame.height);
    assert!(popup_menu.frame.y > menu_title.frame.y + menu_title.frame.height);
    assert_eq!(steps_slider.value_color, input_slider.value_color);
    let input_focused = template_contract_node(&nodes, "WorkbenchInputFocused");
    assert!(input_focused.focused);
    assert_eq!(
        render_border_for_control(&bridge, "WorkbenchInputFocused").as_deref(),
        Some("#2a3238")
    );
    let input_disabled = template_contract_node(&nodes, "WorkbenchInputDisabled");
    assert!(input_disabled.disabled);
    assert!((input_disabled.button_style.element.opacity - 0.94).abs() < 0.001);
    let selection_title = template_contract_node(&nodes, "WorkbenchSelectionTitle");
    assert_eq!(
        selection_title.label_color,
        crate::ui::retained_host::primitives::Color::from_rgb_u8(131, 141, 148)
    );
    let labs_tab_one = template_contract_node(&nodes, "WorkbenchLabsTabOne");
    let labs_tab_two = template_contract_node(&nodes, "WorkbenchLabsTabTwo");
    let labs_tab_three = template_contract_node(&nodes, "WorkbenchLabsTabThree");
    assert_eq!(labs_tab_one.text.as_str(), "Tab 1");
    assert_eq!(labs_tab_two.text.as_str(), "Tab 2");
    assert_eq!(labs_tab_three.text.as_str(), "Tab 3");
    assert_eq!(labs_tab_one.layout_offset_x, 3.0);
    assert_eq!(labs_tab_one.layout_offset_y, 2.0);
    assert!(labs_tab_one.selected);
    assert!(!labs_tab_two.selected);
    assert!(!labs_tab_three.selected);
    let list_item = template_contract_node(&nodes, "WorkbenchListItem");
    assert_eq!(
        list_item.value_color,
        crate::ui::retained_host::primitives::Color::from_rgb_u8(197, 208, 213)
    );
    assert_eq!(
        list_item.icon_color,
        crate::ui::retained_host::primitives::Color::from_rgb_u8(141, 156, 164)
    );
    let list_selected = template_contract_node(&nodes, "WorkbenchListSelected");
    assert_eq!(
        list_selected.value_color,
        crate::ui::retained_host::primitives::Color::from_rgb_u8(53, 199, 208)
    );
    assert_eq!(
        list_selected.icon_color,
        crate::ui::retained_host::primitives::Color::from_rgb_u8(122, 230, 240)
    );
    assert_eq!(
        render_background_for_control(&bridge, "WorkbenchListSelected").as_deref(),
        Some("#12383d")
    );
    let list_disabled = template_contract_node(&nodes, "WorkbenchListDisabled");
    assert_eq!(
        list_disabled.value_color,
        crate::ui::retained_host::primitives::Color::from_rgb_u8(88, 101, 108)
    );
    assert_eq!(
        list_disabled.icon_color,
        crate::ui::retained_host::primitives::Color::from_rgb_u8(88, 101, 108)
    );
    assert!((list_selected.frame.y - (list_item.frame.y + list_item.frame.height)).abs() < 0.001);
    assert!(
        (list_disabled.frame.y - (list_selected.frame.y + list_selected.frame.height)).abs()
            < 0.001
    );
    let position_axis_x = template_contract_node(&nodes, "WorkbenchTransformPositionAxisX");
    let position_value_x = template_contract_node(&nodes, "WorkbenchTransformPositionX");
    assert_eq!(
        position_axis_x.label_color,
        crate::ui::retained_host::primitives::Color::from_rgb_u8(86, 104, 113)
    );
    assert_eq!(
        position_value_x.value_color,
        crate::ui::retained_host::primitives::Color::from_rgb_u8(146, 158, 164)
    );
    let scale_axis_x = template_contract_node(&nodes, "WorkbenchTransformScaleAxisX");
    let scale_value_x = template_contract_node(&nodes, "WorkbenchTransformScaleX");
    assert_eq!(
        scale_axis_x.label_color,
        crate::ui::retained_host::primitives::Color::from_rgb_u8(104, 118, 126)
    );
    assert_eq!(
        scale_value_x.value_color,
        crate::ui::retained_host::primitives::Color::from_rgb_u8(146, 158, 164)
    );
}
