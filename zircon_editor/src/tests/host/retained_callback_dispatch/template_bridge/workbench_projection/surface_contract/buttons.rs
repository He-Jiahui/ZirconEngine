use super::*;

#[test]
fn componentized_workbench_window_template_bridge_projects_buttons_and_icon_controls() {
    let _guard = env_lock().lock().unwrap();
    let (bridge, nodes) = componentized_workbench_projection_fixture();
    let component_buttons = template_contract_node(&nodes, "WorkbenchComponentButtons");

    let primary_button = template_contract_node(&nodes, "WorkbenchPrimaryButton");
    let secondary_button = template_contract_node(&nodes, "WorkbenchSecondaryButton");
    let tertiary_button = template_contract_node(&nodes, "WorkbenchTertiaryButton");
    let outline_button = template_contract_node(&nodes, "WorkbenchOutlineButton");
    let button_icon = template_contract_node(&nodes, "WorkbenchButtonIcon");
    let button_delete = template_contract_node(&nodes, "WorkbenchButtonDelete");
    let disabled_button = template_contract_node(&nodes, "WorkbenchDisabledButton");
    let buttons_title = template_contract_node(&nodes, "WorkbenchButtonsTitle");
    assert!((buttons_title.frame.x - (component_buttons.frame.x + 8.0)).abs() < 0.001);
    assert!((buttons_title.frame.y - (component_buttons.frame.y + 4.0)).abs() < 0.001);
    assert!(buttons_title.frame.width <= component_buttons.frame.width - 16.0 + 0.001);
    assert!(primary_button.frame.x >= component_buttons.frame.x + 8.0 - 0.001);
    assert!((primary_button.label_brightness - 1.0).abs() < 0.001);
    assert!((secondary_button.label_brightness - 1.01).abs() < 0.001);
    assert_eq!(primary_button.layout_offset_x, 3.0);
    assert_eq!(primary_button.layout_offset_y, -1.0);
    assert_eq!(primary_button.font_size, 12.22);
    assert_eq!(
        style_color_u8(
            primary_button
                .button_style
                .element
                .background_color
                .as_ref()
        ),
        Some([18, 57, 65, 255])
    );
    assert_eq!(
        style_color_u8(primary_button.button_style.element.border_color.as_ref()),
        Some([53, 199, 208, 255])
    );
    assert_eq!(secondary_button.layout_offset_x, 1.0);
    assert_eq!(secondary_button.layout_offset_y, -1.0);
    assert_eq!(secondary_button.font_size, 12.22);
    assert_eq!(
        style_color_u8(
            secondary_button
                .button_style
                .element
                .background_color
                .as_ref()
        ),
        Some([29, 35, 40, 255])
    );
    assert_eq!(tertiary_button.role.as_str(), "Button");
    assert_eq!(tertiary_button.text.as_str(), "Tertiary");
    assert_eq!(tertiary_button.button_variant.as_str(), "text");
    assert_eq!(tertiary_button.layout_offset_x, 1.0);
    assert_eq!(tertiary_button.corner_radius, 5.0);
    assert_eq!(outline_button.text.as_str(), "Outline");
    assert_eq!(outline_button.layout_offset_x, 1.0);
    assert_eq!(outline_button.corner_radius, 5.0);
    assert_eq!(
        style_color_u8(
            tertiary_button
                .button_style
                .element
                .background_color
                .as_ref()
        ),
        Some([29, 35, 40, 255])
    );
    assert_eq!(
        style_color_u8(tertiary_button.button_style.element.border_color.as_ref()),
        Some([42, 50, 56, 255])
    );
    assert_eq!(
        style_color_u8(
            tertiary_button
                .button_style
                .element
                .foreground_color
                .as_ref()
        ),
        Some([216, 227, 231, 255])
    );
    assert_eq!(
        style_color_u8(outline_button.button_style.element.border_color.as_ref()),
        Some([42, 50, 56, 255])
    );
    assert_eq!(
        style_color_u8(
            outline_button
                .button_style
                .element
                .foreground_color
                .as_ref()
        ),
        Some([216, 227, 231, 255])
    );
    assert_eq!(button_icon.text.as_str(), "Icon");
    assert_eq!(
        button_icon.icon_name.as_str(),
        "zircon_editor_shell/controls/add.svg"
    );
    assert_eq!(button_icon.layout_offset_x, 3.0);
    assert_eq!(button_icon.layout_offset_y, 1.0);
    assert!((button_icon.label_brightness - 1.02).abs() < 0.001);
    assert_eq!(button_icon.corner_radius, 5.0);
    assert_eq!(
        style_color_u8(button_icon.button_style.element.background_color.as_ref()),
        Some([29, 35, 40, 255])
    );
    assert_eq!(
        style_color_u8(button_icon.button_style.element.border_color.as_ref()),
        Some([42, 50, 56, 255])
    );
    assert_eq!(
        style_color_u8(button_icon.button_style.element.foreground_color.as_ref()),
        Some([216, 227, 231, 255])
    );
    assert_eq!(
        button_delete.icon_name.as_str(),
        "zircon_editor_shell/controls/delete.svg"
    );
    assert_eq!(button_delete.validation_level.as_str(), "danger");
    assert_eq!(button_delete.corner_radius, 5.0);
    assert!((button_delete.label_brightness - 1.02).abs() < 0.001);
    assert_eq!(
        style_color_u8(button_delete.button_style.element.foreground_color.as_ref()),
        Some([216, 227, 231, 255])
    );
    assert!(disabled_button.disabled);
    assert_eq!(disabled_button.layout_offset_x, -1.0);
    assert_eq!(disabled_button.layout_offset_y, 3.5);
    assert_eq!(
        style_color_u8(
            disabled_button
                .button_style
                .element
                .background_color
                .as_ref()
        ),
        Some([39, 44, 48, 255])
    );
    assert_eq!(
        style_color_u8(disabled_button.button_style.element.border_color.as_ref()),
        Some([52, 61, 68, 255])
    );
    assert_eq!(
        style_color_u8(
            disabled_button
                .button_style
                .element
                .foreground_color
                .as_ref()
        ),
        Some([216, 227, 231, 255])
    );
    assert!((disabled_button.button_style.element.opacity - 0.72).abs() < 0.001);
    let dropdown_button = template_contract_node(&nodes, "WorkbenchDropdownButton");
    assert_eq!(dropdown_button.role.as_str(), "Dropdown");
    assert_eq!(dropdown_button.value_text.as_str(), "Dropdown");
    assert_eq!(dropdown_button.options.row_count(), 3);
    assert_eq!(dropdown_button.layout_offset_x, -1.0);
    assert!((dropdown_button.label_brightness - 1.005).abs() < 0.001);
    assert_eq!(dropdown_button.layout_offset_y, 3.5);
    assert_eq!(
        dropdown_button.value_color,
        crate::ui::retained_host::primitives::Color::from_rgb_u8(127, 138, 145)
    );
    assert_eq!(
        dropdown_button.icon_color,
        crate::ui::retained_host::primitives::Color::from_rgb_u8(103, 115, 122)
    );
    assert_eq!(
        style_color_u8(dropdown_button.button_style.element.border_color.as_ref()),
        Some([42, 50, 56, 255])
    );
    let mini_add = template_contract_node(&nodes, "WorkbenchMiniAdd");
    assert_eq!(
        style_color_u8(mini_add.button_style.element.background_color.as_ref()),
        Some([23, 28, 32, 255])
    );
    assert_eq!(
        style_color_u8(mini_add.button_style.element.border_color.as_ref()),
        Some([42, 50, 56, 255])
    );
    assert_eq!(mini_add.corner_radius, 5.0);
    let mini_eye = template_contract_node(&nodes, "WorkbenchMiniEye");
    let mini_eye_off = template_contract_node(&nodes, "WorkbenchMiniEyeOff");
    let mini_lock = template_contract_node(&nodes, "WorkbenchMiniLock");
    let mini_more = template_contract_node(&nodes, "WorkbenchMiniMore");
    let mini_delete = template_contract_node(&nodes, "WorkbenchMiniDelete");
    assert_eq!(mini_eye.role.as_str(), "IconButton");
    assert_eq!(
        mini_eye.icon_name.as_str(),
        "zircon_editor_shell/scene/eye.svg"
    );
    assert_eq!(mini_eye.value_number, 18.0);
    assert!((mini_eye.layout_offset_y - 1.35).abs() < 0.001);
    assert_eq!(mini_eye.frame.width, 38.0);
    assert_eq!(
        mini_eye.icon_color,
        crate::ui::retained_host::primitives::Color::from_rgb_u8(152, 163, 168)
    );
    assert_eq!(
        style_color_u8(mini_eye.button_style.element.background_color.as_ref()),
        Some([23, 28, 32, 255])
    );
    assert_eq!(
        style_color_u8(mini_eye.button_style.element.border_color.as_ref()),
        Some([42, 50, 56, 255])
    );
    assert_eq!(mini_eye.corner_radius, 5.0);
    assert_eq!(
        mini_eye_off.icon_name.as_str(),
        "zircon_editor_shell/scene/eye-off.svg"
    );
    assert_eq!(
        mini_lock.icon_name.as_str(),
        "zircon_editor_shell/scene/lock.svg"
    );
    assert_eq!(
        mini_more.icon_name.as_str(),
        "zircon_editor_shell/toolbar/more-vertical.svg"
    );
    assert_eq!(mini_delete.corner_radius, 5.0);
    assert_eq!(
        style_color_u8(mini_delete.button_style.element.border_color.as_ref()),
        Some([236, 111, 98, 255])
    );
    let icon_toggle = template_contract_node(&nodes, "WorkbenchIconToggleSegmented");
    assert_eq!(icon_toggle.value_text.as_str(), "grid");
    assert_eq!(icon_toggle.options.row_count(), 3);
    assert_eq!(icon_toggle.options.row_data(0).as_deref(), Some("grid"));
    assert_eq!(icon_toggle.options.row_data(1).as_deref(), Some("list"));
    assert_eq!(icon_toggle.options.row_data(2).as_deref(), Some("columns"));
    assert_eq!(icon_toggle.layout_offset_y, 1.0);
    assert!(icon_toggle.has_selected_segment_border_width);
    assert_eq!(icon_toggle.selected_segment_border_width, 0.0);
    assert_eq!(icon_toggle.selected_segment_underline_height, 2.0);
    assert_eq!(
        icon_toggle.selected_segment_underline_color,
        crate::ui::retained_host::primitives::Color::from_argb_u8(255, 42, 166, 184)
    );
}
