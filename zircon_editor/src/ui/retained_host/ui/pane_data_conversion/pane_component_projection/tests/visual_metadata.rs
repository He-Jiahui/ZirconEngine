use super::*;

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
fn runtime_component_projection_projects_popup_option_state_metadata_for_native_painter() {
    let popup = host_template_node(projected_node(
        "DropdownPopup",
        [
            ("open", Value::Boolean(true)),
            (
                "options",
                string_array(
                    [
                        "scene|label=Scene",
                        "assets|label=Assets,checked",
                        "console|label=Console",
                        "render|label=Render",
                    ]
                    .into_iter()
                    .map(str::to_owned),
                ),
            ),
            (
                "selected_options",
                string_array(["assets"].into_iter().map(str::to_owned)),
            ),
            (
                "disabled_options",
                string_array(["render"].into_iter().map(str::to_owned)),
            ),
            (
                "pressed_options",
                string_array(["assets"].into_iter().map(str::to_owned)),
            ),
            (
                "loading_options",
                string_array(["scene"].into_iter().map(str::to_owned)),
            ),
            ("focused_index", Value::Integer(2)),
            ("hovered_option_id", Value::String("console".to_owned())),
        ],
    ))
    .expect("DropdownPopup should project option state metadata into host rows");

    assert!(popup.popup_open);
    assert_eq!(popup.structured_options.row_count(), 4);

    let scene = popup.structured_options.row_data(0).unwrap();
    assert_eq!(scene.id.as_str(), "scene");
    assert_eq!(scene.label.as_str(), "Scene");
    assert!(scene.loading);

    let assets = popup.structured_options.row_data(1).unwrap();
    assert_eq!(assets.id.as_str(), "assets");
    assert_eq!(assets.label.as_str(), "Assets");
    assert!(assets.selected);
    assert!(assets.pressed);

    let console = popup.structured_options.row_data(2).unwrap();
    assert_eq!(console.id.as_str(), "console");
    assert_eq!(console.label.as_str(), "Console");
    assert!(console.focused);
    assert!(console.hovered);

    let render = popup.structured_options.row_data(3).unwrap();
    assert_eq!(render.id.as_str(), "render");
    assert_eq!(render.label.as_str(), "Render");
    assert!(render.disabled);
}

#[test]
fn runtime_component_projection_projects_command_palette_commands_for_native_painter() {
    let palette = host_template_node(projected_node(
        "CommandPalette",
        [
            ("open", Value::Boolean(true)),
            ("query", Value::String("build".to_owned())),
            (
                "commands",
                Value::Array(vec![
                    toml_table([
                        ("id", Value::String("open_scene".to_owned())),
                        ("label", Value::String("Open Scene".to_owned())),
                        ("source", Value::String("workbench".to_owned())),
                        ("shortcut", Value::String("Ctrl+O".to_owned())),
                    ]),
                    toml_table([
                        ("id", Value::String("build_project".to_owned())),
                        ("label", Value::String("Build Project".to_owned())),
                        ("source", Value::String("workbench".to_owned())),
                    ]),
                    Value::String(
                        "reload_runtime|label=Reload Runtime|source=runtime|shortcut=Ctrl+R"
                            .to_owned(),
                    ),
                    Value::String(
                        "build_assets|label=Build Assets|source=workbench|disabled=true".to_owned(),
                    ),
                ]),
            ),
            (
                "filtered_commands",
                string_array(
                    ["build_project", "build_assets"]
                        .into_iter()
                        .map(str::to_owned),
                ),
            ),
            (
                "selected_command_id",
                Value::String("build_project".to_owned()),
            ),
            ("focused_index", Value::Integer(0)),
            (
                "recent_commands",
                string_array(["open_scene"].into_iter().map(str::to_owned)),
            ),
        ],
    ))
    .expect("CommandPalette should project command rows into the host contract");

    assert_eq!(palette.component_role.as_str(), "command-palette");
    assert_eq!(palette.component_category.as_str(), "input");
    assert_eq!(palette.component_layout_role.as_str(), "popup");
    assert!(palette.popup_open);
    assert_eq!(palette.search_query.as_str(), "build");
    assert_eq!(palette.options_text.as_str(), "Build Project, Build Assets");
    assert_eq!(palette.options.row_count(), 2);
    assert_eq!(
        palette.options.row_data(0).as_deref(),
        Some("Build Project")
    );
    assert_eq!(palette.structured_options.row_count(), 2);

    let build_project = palette.structured_options.row_data(0).unwrap();
    assert_eq!(build_project.id.as_str(), "build_project");
    assert_eq!(build_project.label.as_str(), "Build Project");
    assert!(build_project.selected);
    assert!(build_project.focused);
    assert!(build_project.matched);
    assert!(!build_project.disabled);

    let build_assets = palette.structured_options.row_data(1).unwrap();
    assert_eq!(build_assets.id.as_str(), "build_assets");
    assert_eq!(build_assets.label.as_str(), "Build Assets");
    assert!(build_assets.disabled);
    assert!(build_assets.matched);
}

#[test]
fn runtime_component_projection_projects_popup_menu_loading_flags_for_native_painter() {
    let menu = host_template_node(projected_node(
        "ContextActionMenu",
        [
            ("popup_open", Value::Boolean(true)),
            (
                "menu_items",
                string_array(
                    ["Archive|loading", "Delete|danger,disabled"]
                        .into_iter()
                        .map(str::to_owned),
                ),
            ),
        ],
    ))
    .expect("ContextActionMenu should project menu-item state flags into host rows");

    let archive = menu.structured_menu_items.row_data(0).unwrap();
    assert_eq!(archive.label.as_str(), "Archive");
    assert!(archive.loading);

    let delete = menu.structured_menu_items.row_data(1).unwrap();
    assert_eq!(delete.label.as_str(), "Delete");
    assert!(delete.disabled);
    assert!(!delete.loading);
}

#[test]
fn runtime_component_projection_preserves_segmented_selected_style_metadata() {
    let segmented = host_template_node(projected_node(
        "SegmentedControl",
        [
            ("selected_segment_border_width", Value::Float(0.0)),
            ("selected_segment_underline_height", Value::Float(1.0)),
            (
                "selected_segment_underline_color",
                Value::String("#32d3de7a".to_owned()),
            ),
        ],
    ))
    .expect("segmented selected style metadata should project into the host contract");

    assert!(segmented.has_selected_segment_border_width);
    assert_eq!(segmented.selected_segment_border_width, 0.0);
    assert_eq!(segmented.selected_segment_underline_height, 1.0);
    assert_eq!(
        segmented.selected_segment_underline_color,
        Color::from_argb_u8(122, 50, 211, 222)
    );
}

#[test]
fn runtime_component_projection_prioritizes_color_field_value_over_visual_aliases() {
    let color_field = host_template_node(projected_node(
        "ColorField",
        [
            ("value", Value::String("#4d89ff".to_owned())),
            ("color", Value::String("#e6f1f4".to_owned())),
            ("foreground_color", Value::String("#e6f1f4".to_owned())),
        ],
    ))
    .expect("ColorField should project into the host contract");

    assert_eq!(color_field.component_role.as_str(), "color-field");
    assert_eq!(color_field.value_color, Color::from_rgb_u8(77, 137, 255));
}

#[test]
fn runtime_component_projection_maps_workbench_metric_aliases() {
    let toggle = host_template_node(projected_node(
        "Toggle",
        [
            ("layout_spacing", Value::Float(10.0)),
            ("track_width", Value::Float(34.0)),
            ("track_height", Value::Float(18.0)),
            ("thumb_size", Value::Float(12.0)),
        ],
    ))
    .expect("toggle metric aliases should project into the host contract");

    assert_eq!(toggle.layout_content_offset_x, 10.0);
    assert_eq!(toggle.value_number, 34.0);
    assert_eq!(toggle.layout_content_offset_y, 18.0);
    assert_eq!(toggle.layout_icon_size, 12.0);

    let toast = host_template_node(projected_node(
        "Alert",
        [
            ("status_mark_size", Value::Float(18.0)),
            ("status_mark_color", Value::String("#209fa9".to_owned())),
            ("action_color", Value::String("#238f98".to_owned())),
        ],
    ))
    .expect("toast metric aliases should project into the host contract");

    assert_eq!(toast.value_number, 18.0);
    assert_eq!(toast.label_color, Color::from_rgb_u8(32, 159, 169));
    assert_eq!(toast.value_color, Color::from_rgb_u8(35, 143, 152));

    let tooltip = host_template_node(projected_node(
        "Tooltip",
        [
            ("arrow_size", Value::Float(8.0)),
            ("arrow_color", Value::String("#171c20".to_owned())),
            ("label_color", Value::String("#a8b3b8".to_owned())),
            ("icon_color", Value::String("#259ca7".to_owned())),
            ("icon_stroke_width", Value::Float(1.45)),
            ("layout_content_offset_y", Value::Float(56.0)),
        ],
    ))
    .expect("tooltip metric aliases should project into the host contract");

    assert_eq!(tooltip.component_role.as_str(), "tooltip");
    assert_eq!(tooltip.value_number, 8.0);
    assert_eq!(tooltip.value_color, Color::from_rgb_u8(23, 28, 32));
    assert_eq!(tooltip.label_color, Color::from_rgb_u8(168, 179, 184));
    assert_eq!(tooltip.icon_color, Color::from_rgb_u8(37, 156, 167));
    assert_eq!(tooltip.icon_stroke_width, 1.45);
    assert_eq!(tooltip.layout_content_offset_y, 56.0);

    let slider = host_template_node(projected_node(
        "Slider",
        [
            ("thumb_color", Value::String("#b7f1f8".to_owned())),
            ("thumb_outline_color", Value::String("#2ab1bc33".to_owned())),
            ("thumb_halo_color", Value::String("#32d3de3d".to_owned())),
        ],
    ))
    .expect("slider thumb aliases should project into the host contract");

    assert_eq!(slider.icon_color, Color::from_rgb_u8(183, 241, 248));
    assert_eq!(
        slider.state_layer_color,
        Color::from_argb_u8(61, 50, 211, 222)
    );
    assert_eq!(
        style_color_u8(slider.button_style.element.border_color.as_ref()),
        Some([42, 177, 188, 51])
    );
}

#[test]
fn runtime_component_projection_honors_optional_feedback_icons() {
    let hidden_icon = host_template_node(projected_node(
        "Tooltip",
        [
            ("icon", Value::String("info".to_owned())),
            ("show_icon", Value::Boolean(false)),
        ],
    ))
    .expect("tooltip metadata should project into the host contract");
    assert!(hidden_icon.icon_name.is_empty());

    let visible_icon = host_template_node(projected_node(
        "Tooltip",
        [
            ("icon", Value::String("info".to_owned())),
            ("show_icon", Value::Boolean(true)),
        ],
    ))
    .expect("tooltip metadata should project into the host contract");
    assert_eq!(visible_icon.icon_name.as_str(), "info");

    let hidden_alias_icon = host_template_node(projected_node(
        "Tooltip",
        [
            ("icon", Value::String("info".to_owned())),
            ("showIcon", Value::Boolean(false)),
        ],
    ))
    .expect("tooltip alias metadata should project into the host contract");
    assert!(hidden_alias_icon.icon_name.is_empty());

    let hidden_alert_icon = host_template_node(projected_node(
        "Alert",
        [
            ("icon", Value::String("warning".to_owned())),
            ("show_icon", Value::Boolean(false)),
            ("severity", Value::String("warning".to_owned())),
        ],
    ))
    .expect("alert metadata should project into the host contract");
    assert!(hidden_alert_icon.icon_name.is_empty());
}

#[test]
fn runtime_component_projection_loads_editor_svg_image_preview() {
    let image = host_template_node(projected_node(
        "Image",
        [
            (
                "image",
                Value::String("zircon_editor_shell/toolbar/select.svg".to_owned()),
            ),
            (
                "value",
                Value::String("zircon_editor_shell/toolbar/select.svg".to_owned()),
            ),
        ],
    ))
    .expect("editor svg image should project into the host contract");

    assert_eq!(image.component_role.as_str(), "image");
    assert_eq!(
        image.media_source.as_str(),
        "zircon_editor_shell/toolbar/select.svg"
    );
    assert!(image.has_preview_image);

    let preview_size = image.preview_image.size();
    assert!(preview_size.width > 0);
    assert!(preview_size.height > 0);
}
