use std::collections::BTreeMap;

use super::*;
use crate::ui::template_runtime::RetainedUiHostBindingProjection;
use toml::Value;
use zircon_runtime_interface::ui::layout::UiFrame;

#[test]
fn runtime_component_projection_preserves_virtualization_and_pagination_metadata() {
    let virtual_list = host_template_node(projected_node(
        "VirtualList",
        [
            ("item_extent", Value::Float(32.0)),
            ("overscan", Value::Integer(4)),
            ("total_count", Value::Integer(1000)),
            ("viewport_start", Value::Integer(120)),
            ("viewport_count", Value::Integer(40)),
        ],
    ))
    .expect("VirtualList should project into the host contract");

    assert!(virtual_list.virtualization_enabled);
    assert_eq!(virtual_list.virtualization_item_extent, 32.0);
    assert_eq!(virtual_list.virtualization_overscan, 4);
    assert_eq!(virtual_list.virtualization_total_count, 1000);
    assert_eq!(virtual_list.virtualization_visible_start, 120);
    assert_eq!(virtual_list.virtualization_visible_count, 40);

    let paged_list = host_template_node(projected_node(
        "PagedList",
        [
            ("total_count", Value::Integer(2500)),
            ("page_index", Value::Integer(3)),
            ("page_size", Value::Integer(100)),
            ("page_count", Value::Integer(25)),
        ],
    ))
    .expect("PagedList should project into the host contract");

    assert!(!paged_list.virtualization_enabled);
    assert_eq!(paged_list.pagination_total_count, 2500);
    assert_eq!(paged_list.pagination_page_index, 3);
    assert_eq!(paged_list.pagination_page_size, 100);
    assert_eq!(paged_list.pagination_page_count, 25);
}

#[test]
fn runtime_component_projection_slices_virtualized_visible_collection_items() {
    let virtual_list = host_template_node(projected_node(
        "VirtualList",
        [
            (
                "collection_items",
                string_array((0..20).map(|index| format!("Item {index}"))),
            ),
            ("viewport_start", Value::Integer(10)),
            ("viewport_count", Value::Integer(5)),
            ("overscan", Value::Integer(2)),
        ],
    ))
    .expect("VirtualList should project a visible collection window");

    assert_eq!(virtual_list.collection_items.row_count(), 9);
    assert_eq!(
        virtual_list.collection_items.row_data(0).as_deref(),
        Some("Item 8")
    );
    assert_eq!(
        virtual_list.collection_items.row_data(8).as_deref(),
        Some("Item 16")
    );
}

#[test]
fn runtime_component_projection_clamps_virtualized_collection_window_edges() {
    let negative_start = host_template_node(projected_node(
        "VirtualList",
        [
            (
                "collection_items",
                string_array((0..5).map(|index| format!("Item {index}"))),
            ),
            ("viewport_start", Value::Integer(-10)),
            ("viewport_count", Value::Integer(2)),
            ("overscan", Value::Integer(1)),
        ],
    ))
    .expect("VirtualList should project a negative start deterministically");

    assert_eq!(negative_start.collection_items.row_count(), 3);
    assert_eq!(
        negative_start.collection_items.row_data(0).as_deref(),
        Some("Item 0")
    );
    assert_eq!(
        negative_start.collection_items.row_data(2).as_deref(),
        Some("Item 2")
    );

    let zero_count = host_template_node(projected_node(
        "VirtualList",
        [
            (
                "collection_items",
                string_array((0..5).map(|index| format!("Item {index}"))),
            ),
            ("viewport_start", Value::Integer(1)),
            ("viewport_count", Value::Integer(0)),
            ("overscan", Value::Integer(10)),
        ],
    ))
    .expect("VirtualList should project a zero visible count deterministically");

    assert_eq!(zero_count.collection_items.row_count(), 0);

    let oversized_overscan = host_template_node(projected_node(
        "VirtualList",
        [
            (
                "collection_items",
                string_array((0..4).map(|index| format!("Item {index}"))),
            ),
            ("viewport_start", Value::Integer(2)),
            ("viewport_count", Value::Integer(1)),
            ("overscan", Value::Integer(50)),
        ],
    ))
    .expect("VirtualList should project oversized overscan deterministically");

    assert_eq!(oversized_overscan.collection_items.row_count(), 4);
}

#[test]
fn runtime_component_projection_preserves_primary_click_binding_id() {
    let mut button = projected_node("Button", [("text", Value::String("Apply".to_owned()))]);
    button.bindings.push(RetainedUiHostBindingProjection {
        binding_id: "InspectorPaneBody/ApplyDraft".to_owned(),
        event_kind: UiEventKind::Click,
        route_id: None,
    });

    let projected = host_template_node(button)
        .expect("button with a primary click binding should project into the host contract");

    assert_eq!(
        projected.binding_id.as_str(),
        "InspectorPaneBody/ApplyDraft"
    );
    assert_eq!(projected.action_id.as_str(), "");
}

#[test]
fn runtime_component_projection_derives_text_edit_targets_from_change_and_submit_bindings() {
    let mut input = projected_node(
        "InputField",
        [("value_text", Value::String("Draft".into()))],
    );
    input.control_id = Some("NameField".to_owned());
    input.bindings.push(RetainedUiHostBindingProjection {
        binding_id: "InspectorView/NameField".to_owned(),
        event_kind: UiEventKind::Change,
        route_id: None,
    });
    input.bindings.push(RetainedUiHostBindingProjection {
        binding_id: "InspectorView/ApplyBatchButton".to_owned(),
        event_kind: UiEventKind::Submit,
        route_id: None,
    });

    let projected = host_template_node(input)
        .expect("input with change and commit bindings should project edit targets");

    assert_eq!(projected.component_role.as_str(), "input-field");
    assert_eq!(projected.edit_action_id.as_str(), "InspectorView/NameField");
    assert_eq!(
        projected.commit_action_id.as_str(),
        "InspectorView/ApplyBatchButton"
    );
}

#[test]
fn runtime_component_projection_preserves_material_visual_metadata() {
    let button = host_template_node(projected_node(
        "Button",
        [
            ("surface_variant", Value::String("accent".to_owned())),
            ("text_tone", Value::String("muted".to_owned())),
            ("button_variant", Value::String("primary".to_owned())),
            ("font_size", Value::Float(13.0)),
            ("font_weight", Value::Integer(600)),
            ("text_align", Value::String("center".to_owned())),
            ("overflow", Value::String("clip".to_owned())),
            ("corner_radius", Value::Float(5.0)),
            ("border_width", Value::Float(1.0)),
            ("elevation", Value::Float(3.0)),
            ("z_index", Value::Integer(17)),
            ("state_layer_enabled", Value::Boolean(true)),
            ("state_layer_color", Value::String("#80eaff".to_owned())),
            ("ripple_enabled", Value::Boolean(true)),
            ("ripple_pressed_x", Value::Float(24.0)),
            ("ripple_pressed_y", Value::Float(12.0)),
            ("clip_ripple", Value::Boolean(false)),
            ("validation_level", Value::String("error".to_owned())),
            ("selected", Value::Boolean(true)),
            ("hovered", Value::Boolean(true)),
            ("pressed", Value::Boolean(true)),
            ("focused", Value::Boolean(true)),
            ("disabled", Value::Boolean(true)),
        ],
    ))
    .expect("material button metadata should project into the host contract");

    assert_eq!(button.surface_variant.as_str(), "accent");
    assert_eq!(button.text_tone.as_str(), "muted");
    assert_eq!(button.button_variant.as_str(), "primary");
    assert_eq!(button.validation_level.as_str(), "error");
    assert!(button.selected);
    assert!(button.hovered);
    assert!(button.pressed);
    assert!(button.focused);
    assert!(button.disabled);
    assert_eq!(button.font_size, 13.0);
    assert_eq!(button.font_weight, 600);
    assert_eq!(button.text_align.as_str(), "center");
    assert_eq!(button.overflow.as_str(), "clip");
    assert_eq!(button.corner_radius, 5.0);
    assert_eq!(button.border_width, 1.0);
    assert_eq!(button.elevation, 3.0);
    assert_eq!(button.z_index, 17);
    assert!(button.state_layer_enabled);
    assert_eq!(button.state_layer_color, Color::from_rgb_u8(128, 234, 255));
    assert!(button.ripple_enabled);
    assert_eq!(button.ripple_pressed_x, 24.0);
    assert_eq!(button.ripple_pressed_y, 12.0);
    assert!(button.ripple_unclipped);
}

#[test]
fn runtime_component_projection_preserves_mui_divider_visual_metadata() {
    let divider = host_template_node(projected_node(
        "Divider",
        [
            ("variant", Value::String("middle".to_owned())),
            ("orientation", Value::String("vertical".to_owned())),
            ("flexItem", Value::Boolean(true)),
            ("textAlign", Value::String("right".to_owned())),
            ("text", Value::String("Section".to_owned())),
        ],
    ))
    .expect("MUI Divider should project visual metadata into the host contract");

    assert_eq!(divider.component_role.as_str(), "divider");
    assert_variant_token(&divider.component_variant, "middle");
    assert_variant_token(&divider.component_variant, "vertical");
    assert_variant_token(&divider.component_variant, "flexItem");
    assert_variant_token(&divider.component_variant, "withChildren");
    assert_variant_token(&divider.component_variant, "textAlignRight");
    assert_eq!(divider.text_align.as_str(), "right");
    assert_eq!(divider.text.as_str(), "Section");
}

#[test]
fn runtime_component_projection_preserves_mui_timeline_dot_color_metadata() {
    let dot = host_template_node(projected_node(
        "TimelineDot",
        [
            ("variant", Value::String("outlined".to_owned())),
            ("color", Value::String("secondary".to_owned())),
        ],
    ))
    .expect("MUI TimelineDot should project color metadata into the host contract");

    assert_eq!(dot.component_role.as_str(), "timeline-dot");
    assert_variant_token(&dot.component_variant, "outlined");
    assert_variant_token(&dot.component_variant, "secondary");
}

#[test]
fn runtime_component_projection_preserves_mui_badge_overlay_metadata_and_display_value() {
    let badge = host_template_node(projected_node(
        "Badge",
        [
            ("badgeContent", Value::Integer(120)),
            ("max", Value::Integer(99)),
            ("variant", Value::String("standard".to_owned())),
            ("color", Value::String("error".to_owned())),
            ("overlap", Value::String("circular".to_owned())),
            (
                "anchorOrigin",
                toml_table([
                    ("vertical", Value::String("bottom".to_owned())),
                    ("horizontal", Value::String("left".to_owned())),
                ]),
            ),
        ],
    ))
    .expect("MUI Badge should project badgeContent and overlay metadata into the host contract");

    assert_eq!(badge.component_role.as_str(), "badge");
    assert_eq!(badge.value_text.as_str(), "99+");
    assert_variant_token(&badge.component_variant, "standard");
    assert_variant_token(&badge.component_variant, "error");
    assert_variant_token(&badge.component_variant, "circular");
    assert_variant_token(&badge.component_variant, "bottom");
    assert_variant_token(&badge.component_variant, "left");
    assert_variant_token(&badge.component_variant, "overlapCircular");
    assert_variant_token(&badge.component_variant, "anchorOriginBottomLeftCircular");
}

#[test]
fn runtime_component_projection_marks_mui_badge_zero_content_invisible_unless_show_zero() {
    let hidden = host_template_node(projected_node(
        "Badge",
        [
            ("badgeContent", Value::Integer(0)),
            ("showZero", Value::Boolean(false)),
        ],
    ))
    .expect("MUI Badge zero content should project into the host contract");

    assert_variant_token(&hidden.component_variant, "standard");
    assert_variant_token(&hidden.component_variant, "invisible");

    let visible = host_template_node(projected_node(
        "Badge",
        [
            ("badgeContent", Value::Integer(0)),
            ("showZero", Value::Boolean(true)),
        ],
    ))
    .expect("MUI Badge showZero content should project into the host contract");

    assert_eq!(visible.value_text.as_str(), "0");
    assert!(
        !visible
            .component_variant
            .split_whitespace()
            .any(|part| part == "invisible"),
        "showZero badge should not be marked invisible"
    );
}

#[test]
fn runtime_component_projection_marks_mui_badge_empty_content_invisible() {
    let empty_content = host_template_node(projected_node(
        "Badge",
        [("badgeContent", Value::String(String::new()))],
    ))
    .expect("MUI Badge empty content should project into the host contract");

    assert_variant_token(&empty_content.component_variant, "standard");
    assert_variant_token(&empty_content.component_variant, "invisible");
}

#[test]
fn runtime_component_projection_keeps_mui_badge_string_zero_visible() {
    let string_zero = host_template_node(projected_node(
        "Badge",
        [("badgeContent", Value::String("0".to_owned()))],
    ))
    .expect("MUI Badge string zero content should project into the host contract");

    assert_eq!(string_zero.value_text.as_str(), "0");
    assert!(
        !string_zero
            .component_variant
            .split_whitespace()
            .any(|part| part == "invisible"),
        "string zero badge content should stay visible like local MUI"
    );
}

#[test]
fn runtime_component_projection_preserves_mui_chip_visual_metadata() {
    let chip = host_template_node(projected_node(
        "Chip",
        [
            ("variant", Value::String("outlined".to_owned())),
            ("size", Value::String("small".to_owned())),
            ("color", Value::String("warning".to_owned())),
            ("clickable", Value::Boolean(true)),
            (
                "onDelete",
                Value::String("MaterialLab.Chip.Delete".to_owned()),
            ),
            ("deleteIcon", Value::String("cancel".to_owned())),
            ("focusVisible", Value::Boolean(true)),
        ],
    ))
    .expect("MUI Chip should project visual metadata into the host contract");

    assert_eq!(chip.component_role.as_str(), "chip");
    assert_variant_token(&chip.component_variant, "outlined");
    assert_variant_token(&chip.component_variant, "small");
    assert_variant_token(&chip.component_variant, "sizeSmall");
    assert_variant_token(&chip.component_variant, "warning");
    assert_variant_token(&chip.component_variant, "colorWarning");
    assert_variant_token(&chip.component_variant, "clickable");
    assert_variant_token(&chip.component_variant, "deletable");
    assert_variant_token(&chip.component_variant, "hasDeleteIcon");
    assert_variant_token(&chip.component_variant, "focusVisible");
}

#[test]
fn runtime_component_projection_preserves_mui_skeleton_shape_animation_and_child_tokens() {
    let skeleton = host_template_node(projected_node(
        "Skeleton",
        [
            ("variant", Value::String("text".to_owned())),
            ("animation", Value::String("wave".to_owned())),
            ("hasChildren", Value::Boolean(true)),
        ],
    ))
    .expect("MUI Skeleton should project visual metadata into the host contract");

    assert_eq!(skeleton.component_role.as_str(), "skeleton");
    assert_variant_token(&skeleton.component_variant, "text");
    assert_variant_token(&skeleton.component_variant, "wave");
    assert_variant_token(&skeleton.component_variant, "withChildren");
    assert_variant_token(&skeleton.component_variant, "fitContent");
    assert_variant_token(&skeleton.component_variant, "heightAuto");
}

#[test]
fn runtime_component_projection_preserves_mui_feedback_variant_open_and_progress_state() {
    let progress = host_template_node(projected_node(
        "Progress",
        [
            ("variant", Value::String("circular".to_owned())),
            ("value", Value::Float(68.0)),
        ],
    ))
    .expect("MUI Progress should project into the host contract");

    assert_eq!(progress.component_role.as_str(), "progress");
    assert_eq!(progress.component_variant.as_str(), "circular");
    assert_eq!(progress.value_number, 68.0);
    assert_eq!(progress.value_percent, 0.68);

    let backdrop = host_template_node(projected_node(
        "Backdrop",
        [
            ("open", Value::Boolean(true)),
            ("invisible", Value::Boolean(true)),
        ],
    ))
    .expect("MUI Backdrop should project into the host contract");

    assert_eq!(backdrop.component_role.as_str(), "backdrop");
    assert_eq!(backdrop.component_variant.as_str(), "invisible");
    assert_eq!(backdrop.z_index, 1299);
    assert!(backdrop.popup_open);

    let fade = host_template_node(projected_node(
        "Fade",
        [
            ("in", Value::Boolean(true)),
            ("transition_progress", Value::Float(0.5)),
            ("timeout_ms", Value::Integer(225)),
            (
                "easing",
                Value::String("cubic-bezier(0.4, 0, 0.2, 1)".to_owned()),
            ),
        ],
    ))
    .expect("MUI Fade should project transition metadata into the host contract");

    assert_eq!(fade.component_role.as_str(), "fade");
    assert_eq!(fade.transition_kind.as_str(), "fade");
    assert!(fade.transition_in);
    assert!(!fade.transition_entered);
    assert_eq!(fade.transition_progress, 0.5);
    assert_eq!(fade.transition_duration_ms, 225);
    assert_eq!(
        fade.transition_easing.as_str(),
        "cubic-bezier(0.4, 0, 0.2, 1)"
    );

    let slide = host_template_node(projected_node("Slide", []))
        .expect("MUI Slide should project transition defaults into the host contract");

    assert_eq!(slide.transition_kind.as_str(), "slide");
    assert_eq!(slide.transition_direction.as_str(), "down");
    assert_eq!(slide.transition_duration_ms, 225);
    assert_eq!(
        slide.transition_easing.as_str(),
        "cubic-bezier(0.0, 0, 0.2, 1)"
    );

    let collapse = host_template_node(projected_node("Collapse", [("in", Value::Boolean(false))]))
        .expect("MUI Collapse should project exit transition defaults into the host contract");

    assert_eq!(collapse.transition_kind.as_str(), "collapse");
    assert!(!collapse.transition_in);
    assert_eq!(collapse.transition_progress, 0.0);
    assert_eq!(collapse.transition_duration_ms, 300);
}

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
    assert_eq!(dialog.surface_variant.as_str(), "popup");
    assert_eq!(dialog.corner_radius, 4.0);
    assert_eq!(dialog.elevation, 24.0);
    assert_eq!(dialog.z_index, 1300);
    assert!(dialog.popup_open);

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
}

#[test]
fn runtime_component_projection_preserves_world_space_metadata() {
    let world_surface = host_template_node(projected_node(
        "WorldSpaceSurface",
        [
            ("world_position", float_array([1.0, 2.0, 3.0])),
            ("world_rotation", float_array([10.0, 20.0, 30.0])),
            ("world_scale", float_array([2.0, 2.5, 3.0])),
            ("world_size", float_array([4.0, 2.0, 0.0])),
            ("pixels_per_meter", Value::Float(128.0)),
            ("billboard", Value::Boolean(true)),
            ("depth_test", Value::Boolean(true)),
            ("render_order", Value::Integer(7)),
            ("camera_target", Value::String("viewport-main".to_owned())),
        ],
    ))
    .expect("WorldSpaceSurface should project into the host contract");

    assert!(world_surface.world_space_enabled);
    assert_eq!(world_surface.world_position_x, 1.0);
    assert_eq!(world_surface.world_position_y, 2.0);
    assert_eq!(world_surface.world_position_z, 3.0);
    assert_eq!(world_surface.world_rotation_x, 10.0);
    assert_eq!(world_surface.world_rotation_y, 20.0);
    assert_eq!(world_surface.world_rotation_z, 30.0);
    assert_eq!(world_surface.world_scale_x, 2.0);
    assert_eq!(world_surface.world_scale_y, 2.5);
    assert_eq!(world_surface.world_scale_z, 3.0);
    assert_eq!(world_surface.world_width, 4.0);
    assert_eq!(world_surface.world_height, 2.0);
    assert_eq!(world_surface.world_pixels_per_meter, 128.0);
    assert!(world_surface.world_billboard);
    assert!(world_surface.world_depth_test);
    assert_eq!(world_surface.world_render_order, 7);
    assert_eq!(world_surface.world_camera_target.as_str(), "viewport-main");
}

fn projected_node(
    component: &str,
    attributes: impl IntoIterator<Item = (&'static str, Value)>,
) -> RetainedUiHostNodeProjection {
    RetainedUiHostNodeProjection {
        node_id: format!("{component}Node"),
        parent_id: None,
        component: component.to_owned(),
        control_id: Some(format!("{component}Control")),
        frame: UiFrame::new(0.0, 0.0, 320.0, 240.0),
        clip_frame: None,
        z_index: 0,
        attributes: attributes
            .into_iter()
            .map(|(name, value)| (name.to_owned(), value))
            .collect::<BTreeMap<_, _>>(),
        style_tokens: BTreeMap::new(),
        bindings: Vec::new(),
    }
}

fn float_array(values: [f64; 3]) -> Value {
    Value::Array(values.into_iter().map(Value::Float).collect())
}

fn string_array(values: impl Iterator<Item = String>) -> Value {
    Value::Array(values.map(Value::String).collect())
}

fn toml_table(values: impl IntoIterator<Item = (&'static str, Value)>) -> Value {
    let mut table = toml::map::Map::new();
    for (name, value) in values {
        table.insert(name.to_owned(), value);
    }
    Value::Table(table)
}

fn assert_variant_token(component_variant: &str, expected: &str) {
    assert!(
        component_variant
            .split_whitespace()
            .any(|part| part == expected),
        "expected component_variant `{component_variant}` to contain `{expected}`"
    );
}
