use super::*;

#[test]
fn runtime_component_projection_applies_mui_overlay_surface_defaults() {
    let dialog = host_template_node(projected_node(
        "Dialog",
        [
            ("open", Value::Boolean(true)),
            ("text", Value::String("Confirm".into())),
        ],
    ))
    .expect("MUI Dialog should project into the host contract");

    assert_eq!(dialog.component_role.as_str(), "dialog");
    assert_eq!(dialog.text.as_str(), "Confirm");
    assert_eq!(dialog.surface_variant.as_str(), "popup");
    assert_eq!(dialog.corner_radius, 4.0);
    assert_eq!(dialog.border_width, 1.0);
    assert_eq!(dialog.elevation, 24.0);
    assert_eq!(dialog.z_index, 1300);
    assert!(dialog.popup_open);

    let confirm_dialog = host_template_node(projected_node(
        "ConfirmDialog",
        [
            ("open", Value::Boolean(true)),
            ("title", Value::String("Delete selected prefab?".into())),
            (
                "message",
                Value::String("This removes the prefab reference from the scene.".into()),
            ),
            ("confirm_text", Value::String("Delete".into())),
            ("cancel_text", Value::String("Cancel".into())),
            ("severity", Value::String("error".into())),
            ("destructive", Value::Boolean(true)),
            ("confirm_enabled", Value::Boolean(false)),
        ],
    ))
    .expect("MUI ConfirmDialog should project into the host contract");

    assert_eq!(confirm_dialog.component_role.as_str(), "confirm-dialog");
    assert_eq!(confirm_dialog.text.as_str(), "Delete selected prefab?");
    assert_eq!(
        confirm_dialog.value_text.as_str(),
        "This removes the prefab reference from the scene."
    );
    assert_variant_token(confirm_dialog.component_variant.as_str(), "error");
    assert_variant_token(confirm_dialog.component_variant.as_str(), "colorError");
    assert_variant_token(confirm_dialog.component_variant.as_str(), "destructive");
    assert_variant_token(confirm_dialog.component_variant.as_str(), "confirmDisabled");
    assert_eq!(confirm_dialog.surface_variant.as_str(), "popup");
    assert_eq!(confirm_dialog.validation_level.as_str(), "error");
    assert_eq!(confirm_dialog.border_width, 1.0);
    assert_eq!(confirm_dialog.elevation, 24.0);
    assert_eq!(confirm_dialog.z_index, 1300);
    assert!(confirm_dialog.popup_open);
    assert_eq!(confirm_dialog.actions.row_count(), 2);
    assert_eq!(
        confirm_dialog.actions.row_data(0).unwrap().label.as_str(),
        "Cancel"
    );
    assert_eq!(
        confirm_dialog.actions.row_data(1).unwrap().label.as_str(),
        "Delete"
    );

    let tooltip = host_template_node(projected_node(
        "Tooltip",
        [
            ("open", Value::Boolean(true)),
            ("text", Value::String("Hint".into())),
        ],
    ))
    .expect("MUI Tooltip should project into the host contract");

    assert_eq!(tooltip.component_role.as_str(), "tooltip");
    assert_eq!(tooltip.surface_variant.as_str(), "tooltip");
    assert_eq!(tooltip.text_tone.as_str(), "inverse");
    assert_eq!(tooltip.corner_radius, 4.0);
    assert_eq!(tooltip.elevation, 0.0);
    assert_eq!(tooltip.z_index, 1500);
    assert!(tooltip.popup_open);

    let outlined_paper = host_template_node(projected_node(
        "Paper",
        [("variant", Value::String("outlined".into()))],
    ))
    .expect("MUI Paper should project into the host contract");

    assert_eq!(outlined_paper.component_role.as_str(), "paper");
    assert_eq!(outlined_paper.component_variant.as_str(), "outlined");
    assert_eq!(outlined_paper.surface_variant.as_str(), "paper-outlined");
    assert_eq!(outlined_paper.border_width, 1.0);
    assert_eq!(outlined_paper.elevation, 0.0);
    assert_eq!(outlined_paper.z_index, 0);
}

#[test]
fn runtime_component_projection_applies_mui_surface_card_defaults() {
    let app_bar = host_template_node(projected_node("AppBar", []))
        .expect("MUI AppBar should project local MUI surface defaults");

    assert_eq!(app_bar.component_role.as_str(), "app-bar");
    assert_eq!(app_bar.surface_variant.as_str(), "primary");
    assert_eq!(app_bar.text_tone.as_str(), "inverse");
    assert_eq!(app_bar.corner_radius, 0.0);
    assert_eq!(app_bar.elevation, 4.0);
    assert_eq!(app_bar.z_index, 1100);

    let transparent_app_bar = host_template_node(projected_node(
        "AppBar",
        [("color", Value::String("transparent".into()))],
    ))
    .expect("MUI transparent AppBar should preserve transparent surface semantics");

    assert_eq!(transparent_app_bar.surface_variant.as_str(), "transparent");
    assert_eq!(transparent_app_bar.text_tone.as_str(), "primary");

    let card = host_template_node(projected_node("Card", []))
        .expect("MUI Card should project Paper-backed surface defaults");

    assert_eq!(card.component_role.as_str(), "card");
    assert_eq!(card.surface_variant.as_str(), "paper");
    assert_eq!(card.corner_radius, 4.0);
    assert_eq!(card.border_width, 0.0);
    assert_eq!(card.elevation, 1.0);

    let raised_card =
        host_template_node(projected_node("Card", [("raised", Value::Boolean(true))]))
            .expect("MUI raised Card should project elevation 8");

    assert_eq!(raised_card.elevation, 8.0);

    let outlined_card = host_template_node(projected_node(
        "Card",
        [("variant", Value::String("outlined".into()))],
    ))
    .expect("MUI outlined Card should project outlined Paper defaults");

    assert_eq!(outlined_card.component_variant.as_str(), "outlined");
    assert_eq!(outlined_card.surface_variant.as_str(), "paper-outlined");
    assert_eq!(outlined_card.border_width, 1.0);
    assert_eq!(outlined_card.elevation, 0.0);

    let square_paper =
        host_template_node(projected_node("Paper", [("square", Value::Boolean(true))]))
            .expect("MUI square Paper should disable retained corner radius");

    assert_eq!(square_paper.component_role.as_str(), "paper");
    assert_eq!(square_paper.corner_radius, 0.0);

    let card_header = host_template_node(projected_node(
        "CardHeader",
        [("title", Value::String("Scene".into()))],
    ))
    .expect("MUI CardHeader should project title text fallback");

    assert_eq!(card_header.component_role.as_str(), "card-header");
    assert_eq!(card_header.text.as_str(), "Scene");
}

#[test]
fn runtime_component_projection_applies_mui_feedback_visual_defaults() {
    let outlined_alert = host_template_node(projected_node(
        "Alert",
        [
            ("severity", Value::String("warning".into())),
            ("variant", Value::String("outlined".into())),
            ("text", Value::String("Careful".into())),
        ],
    ))
    .expect("MUI Alert should project visual defaults into the host contract");

    assert_eq!(outlined_alert.component_role.as_str(), "alert");
    assert_eq!(outlined_alert.text.as_str(), "Careful");
    assert_variant_token(outlined_alert.component_variant.as_str(), "outlined");
    assert_variant_token(outlined_alert.component_variant.as_str(), "warning");
    assert_variant_token(outlined_alert.component_variant.as_str(), "colorWarning");
    assert_variant_token(outlined_alert.component_variant.as_str(), "hasIcon");
    assert_eq!(outlined_alert.surface_variant.as_str(), "alert");
    assert_eq!(outlined_alert.validation_level.as_str(), "warning");
    assert_eq!(outlined_alert.text_tone.as_str(), "warning");
    assert_eq!(outlined_alert.corner_radius, 4.0);
    assert_eq!(outlined_alert.border_width, 1.0);
    assert_eq!(outlined_alert.elevation, 0.0);

    let filled_alert = host_template_node(projected_node(
        "Alert",
        [
            ("severity", Value::String("error".into())),
            ("variant", Value::String("filled".into())),
        ],
    ))
    .expect("MUI filled Alert should project contrast text defaults");

    assert_variant_token(filled_alert.component_variant.as_str(), "filled");
    assert_variant_token(filled_alert.component_variant.as_str(), "error");
    assert_variant_token(filled_alert.component_variant.as_str(), "colorError");
    assert_variant_token(filled_alert.component_variant.as_str(), "hasIcon");
    assert_eq!(filled_alert.validation_level.as_str(), "error");
    assert_eq!(filled_alert.text_tone.as_str(), "inverse");
    assert_eq!(filled_alert.border_width, 0.0);

    let closable_alert = host_template_node(projected_node(
        "Alert",
        [
            ("icon", Value::Boolean(false)),
            ("onClose", Value::String("close-alert".into())),
        ],
    ))
    .expect("MUI closable Alert should project close action metadata");

    assert_variant_token(closable_alert.component_variant.as_str(), "hasAction");
    assert_variant_token(closable_alert.component_variant.as_str(), "hasCloseAction");
    assert!(
        !closable_alert
            .component_variant
            .split_whitespace()
            .any(|part| part == "hasIcon"),
        "icon=false should suppress retained Alert icon metadata"
    );

    let snackbar = host_template_node(projected_node(
        "Snackbar",
        [
            ("open", Value::Boolean(true)),
            ("message", Value::String("Saved".into())),
        ],
    ))
    .expect("MUI Snackbar should project message and overlay visual defaults");

    assert_eq!(snackbar.component_role.as_str(), "snackbar");
    assert_eq!(snackbar.text.as_str(), "Saved");
    assert_eq!(snackbar.surface_variant.as_str(), "snackbar");
    assert_eq!(snackbar.text_tone.as_str(), "inverse");
    assert_eq!(snackbar.corner_radius, 4.0);
    assert_eq!(snackbar.elevation, 6.0);
    assert_eq!(snackbar.z_index, 1400);
    assert!(snackbar.popup_open);

    let snackbar_content = host_template_node(projected_node(
        "SnackbarContent",
        [("message", Value::String("Content".into()))],
    ))
    .expect("MUI SnackbarContent should project content visual defaults");

    assert_eq!(snackbar_content.component_role.as_str(), "snackbar-content");
    assert_eq!(snackbar_content.text.as_str(), "Content");
    assert_eq!(snackbar_content.surface_variant.as_str(), "snackbar");
    assert_eq!(snackbar_content.text_tone.as_str(), "inverse");
    assert_eq!(snackbar_content.corner_radius, 4.0);
    assert_eq!(snackbar_content.elevation, 6.0);
    assert_eq!(snackbar_content.z_index, 1400);
}

#[test]
fn runtime_component_projection_positions_mui_popups_from_anchor_metadata() {
    let mut popper_node = projected_node(
        "Popper",
        [
            ("open", Value::Boolean(true)),
            ("placement", Value::String("bottom-start".into())),
            ("popup_anchor_x", Value::Float(100.0)),
            ("popup_anchor_y", Value::Float(50.0)),
            ("popup_anchor_width", Value::Float(30.0)),
            ("popup_anchor_height", Value::Float(10.0)),
        ],
    );
    popper_node.frame = UiFrame::new(0.0, 0.0, 80.0, 40.0);
    let popper = host_template_node(popper_node)
        .expect("MUI Popper should project anchor metadata into the host contract");

    assert_eq!(popper.frame.x, 100.0);
    assert_eq!(popper.frame.y, 60.0);

    let mut tooltip_node = projected_node(
        "Tooltip",
        [
            ("open", Value::Boolean(true)),
            ("popup_anchor_x", Value::Float(100.0)),
            ("popup_anchor_y", Value::Float(50.0)),
            ("popup_anchor_width", Value::Float(30.0)),
            ("popup_anchor_height", Value::Float(10.0)),
        ],
    );
    tooltip_node.frame = UiFrame::new(0.0, 0.0, 80.0, 20.0);
    let tooltip = host_template_node(tooltip_node)
        .expect("MUI Tooltip should project anchor metadata into the host contract");

    assert_eq!(tooltip.frame.x, 75.0);
    assert_eq!(tooltip.frame.y, 22.0);

    let mut menu_node = projected_node(
        "Menu",
        [
            ("open", Value::Boolean(true)),
            ("popup_anchor_x", Value::Float(100.0)),
            ("popup_anchor_y", Value::Float(50.0)),
            ("popup_anchor_width", Value::Float(30.0)),
            ("popup_anchor_height", Value::Float(10.0)),
        ],
    );
    menu_node.frame = UiFrame::new(0.0, 0.0, 96.0, 48.0);
    let menu = host_template_node(menu_node)
        .expect("MUI Menu should project anchor metadata into the host contract");

    assert_eq!(menu.frame.x, 100.0);
    assert_eq!(menu.frame.y, 60.0);

    let mut context_node = projected_node(
        "ContextMenu",
        [
            ("open", Value::Boolean(true)),
            ("popup_anchor_x", Value::Float(120.0)),
            ("popup_anchor_y", Value::Float(70.0)),
            ("popup_anchor_width", Value::Float(24.0)),
            ("popup_anchor_height", Value::Float(12.0)),
        ],
    );
    context_node.frame = UiFrame::new(0.0, 0.0, 96.0, 48.0);
    let context = host_template_node(context_node)
        .expect("ContextMenu should project anchor metadata into the host contract");

    assert_eq!(context.component_role.as_str(), "context-menu");
    assert_eq!(context.frame.x, 120.0);
    assert_eq!(context.frame.y, 82.0);

    let mut dropdown_popup_node = projected_node(
        "DropdownPopup",
        [
            ("open", Value::Boolean(true)),
            ("popup_anchor_x", Value::Float(140.0)),
            ("popup_anchor_y", Value::Float(90.0)),
            ("popup_anchor_width", Value::Float(80.0)),
            ("popup_anchor_height", Value::Float(28.0)),
        ],
    );
    dropdown_popup_node.frame = UiFrame::new(0.0, 0.0, 128.0, 96.0);
    let dropdown_popup = host_template_node(dropdown_popup_node)
        .expect("DropdownPopup should project anchor metadata into the host contract");

    assert_eq!(dropdown_popup.component_role.as_str(), "dropdown-popup");
    assert_eq!(dropdown_popup.frame.x, 140.0);
    assert_eq!(dropdown_popup.frame.y, 118.0);
}
