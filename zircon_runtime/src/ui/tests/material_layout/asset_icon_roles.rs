use super::*;

#[test]
fn asset_value_nodes_render_as_image_or_icon_not_text() {
    let image_command = render_material_leaf_command(
        "Image",
        r#"
value = "ui/editor/showcase_checker.svg"
image = "ui/editor/showcase_checker.svg"
"#,
    );
    let icon_command = render_material_leaf_command(
        "Icon",
        r#"
value = "ionicons/options-outline.svg"
icon = "options-outline"
"#,
    );
    let svg_icon_command = render_material_leaf_command(
        "SvgIcon",
        r#"
value = "ionicons/options-outline.svg"
source = "ionicons/options-outline.svg"
"#,
    );

    assert_eq!(image_command.kind, UiRenderCommandKind::Image);
    assert_eq!(image_command.text.as_deref(), None);
    assert_eq!(
        image_command.image,
        Some(UiVisualAssetRef::Image(
            "ui/editor/showcase_checker.svg".to_string()
        ))
    );
    assert_eq!(icon_command.kind, UiRenderCommandKind::Image);
    assert_eq!(icon_command.text.as_deref(), None);
    assert_eq!(
        icon_command.image,
        Some(UiVisualAssetRef::Icon("options-outline".to_string()))
    );
    assert_eq!(svg_icon_command.kind, UiRenderCommandKind::Image);
    assert_eq!(svg_icon_command.text.as_deref(), None);
    assert_eq!(
        svg_icon_command.image,
        Some(UiVisualAssetRef::Image(
            "ionicons/options-outline.svg".to_string()
        ))
    );
}

#[test]
fn icon_button_label_is_accessibility_text_not_rendered_text() {
    let icon_button_command = render_material_leaf_command(
        "IconButton",
        r#"
label = "Focus Console"
icon = "search-outline"
layout_min_width = 40
layout_min_height = 40
layout_icon_size = 18
layout_padding_left = 0
layout_padding_right = 0
layout_padding_top = 0
layout_padding_bottom = 0
"#,
    );

    assert_eq!(icon_button_command.kind, UiRenderCommandKind::Image);
    assert_eq!(icon_button_command.text.as_deref(), None);
}

#[test]
fn common_native_material_roles_use_authored_layout_metrics() {
    let progress = measure_material_leaf(
        "ProgressBar",
        r#"
layout_min_height = 8
"#,
        intrinsic_constraints(),
    );
    let spinner = measure_material_leaf(
        "Spinner",
        r#"
text = "Loading"
font_size = 12
line_height = 14
layout_padding_left = 16
layout_padding_right = 16
layout_spacing = 8
layout_min_height = 40
layout_icon_size = 18
"#,
        intrinsic_constraints(),
    );
    let menu = measure_material_leaf(
        "ContextActionMenu",
        r#"
text = "Inspect"
font_size = 12
line_height = 14
layout_padding_left = 16
layout_padding_right = 16
layout_spacing = 8
layout_min_height = 40
"#,
        intrinsic_constraints(),
    );

    assert_eq!(progress, DesiredSize::new(0.0, 8.0));
    assert_eq!(spinner, DesiredSize::new(74.0, 40.0));
    assert_eq!(menu, DesiredSize::new(74.0, 40.0));
}
