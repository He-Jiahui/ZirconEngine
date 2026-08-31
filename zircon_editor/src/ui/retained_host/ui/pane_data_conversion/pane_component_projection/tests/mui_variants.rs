use super::*;

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
fn runtime_component_projection_ignores_transition_state_without_a_transition_kind() {
    let paper = host_template_node(projected_node(
        "Paper",
        [
            ("transition_in", Value::Boolean(false)),
            ("transition_progress", Value::Float(0.25)),
            ("transition_duration_ms", Value::Integer(999)),
            ("transition_easing", Value::String("linear".to_owned())),
            ("transition_direction", Value::String("left".to_owned())),
        ],
    ))
    .expect("a non-transition component should still project into the host contract");

    assert!(paper.transition_kind.is_empty());
    assert!(paper.transition_in);
    assert!(paper.transition_entered);
    assert_eq!(paper.transition_progress, 1.0);
    assert_eq!(paper.transition_duration_ms, 0);
    assert!(paper.transition_easing.is_empty());
    assert!(paper.transition_direction.is_empty());
}
