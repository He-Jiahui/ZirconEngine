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

    let popup = host_template_node(projected_node(
        "DropdownPopup",
        [
            ("layout_padding_left", Value::Float(8.0)),
            ("layout_padding_right", Value::Float(8.0)),
            ("layout_padding_top", Value::Float(4.0)),
            ("layout_padding_bottom", Value::Float(4.0)),
            ("layout_spacing", Value::Float(4.0)),
        ],
    ))
    .expect("popup content metrics should project into the host contract");

    assert_eq!(popup.layout_padding_left, 8.0);
    assert_eq!(popup.layout_padding_right, 8.0);
    assert_eq!(popup.layout_padding_top, 4.0);
    assert_eq!(popup.layout_padding_bottom, 4.0);
    assert_eq!(popup.layout_spacing, 4.0);

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

#[test]
fn live_workbench_tree_owner_focus_reaches_source_commands_and_native_pixels() {
    use crate::ui::layouts::common::model_rc;
    use crate::ui::retained_host::paint_template_nodes_for_test;
    use crate::ui::template_runtime::EditorUiHostRuntime;
    use zircon_runtime_interface::ui::{layout::UiSize, surface::UiRenderCommandKind};
    let mut runtime = EditorUiHostRuntime::default();
    let document_id = crate::ui::template_runtime::builtin::UNREAL_EDITOR_WINDOW_DOCUMENT_ID;
    runtime
        .load_builtin_host_templates_for_document_ids(&[document_id])
        .unwrap();
    let projection = runtime.project_document(document_id).unwrap();
    let runtime = std::sync::Arc::new(runtime);
    let mut bridge = crate::ui::retained_host::callback_dispatch::BuiltinWorkbenchWindowTemplateSurfaceBridge::new_with_runtime(
        std::sync::Arc::clone(&runtime), UiSize::new(1280.0, 800.0),
    ).unwrap();
    let rows = [(1, "Camera"), (2, "Sun")].into_iter().map(|(entity, name)| {
        zircon_runtime::scene::WorldInspectionHierarchyRow {
            entity, parent: None, depth: 0, display_name: name.into(),
            kind: "Entity".into(), subtree_hash: 0, active_in_hierarchy: true,
            has_children: false,
        }
    }).collect::<Vec<_>>();
    let entries = crate::ui::workbench::snapshot::SceneEntries::from_hierarchy_rows_at_generation(rows, [], 1);
    bridge.resync_scene_hierarchy(&entries).unwrap();
    assert_eq!(bridge.scene_entity_for_control("WorkbenchSceneRootItem"), Some(1));
    assert_eq!(bridge.scene_entity_for_control("WorkbenchSceneEnvironmentItem"), Some(2));
    let mut surface = bridge.surface().clone();
    let find = |surface: &zircon_runtime::ui::surface::UiSurface, control: &str| {
        surface
            .tree
            .nodes
            .values()
            .find(|node| {
                node.template_metadata
                    .as_ref()
                    .and_then(|metadata| metadata.control_id.as_deref())
                    == Some(control)
            })
            .expect("actual owning workbench control")
            .node_id
    };
    let owner = find(&surface, "WorkbenchSceneTree");
    let row = find(&surface, "WorkbenchSceneRootItem");
    let sibling = find(&surface, "WorkbenchSceneEnvironmentItem");
    let outside = surface
        .tree
        .nodes
        .values()
        .find(|node| {
            node.is_focus_candidate() && surface.tree_owner_focused(node.node_id).is_none()
        })
        .unwrap()
        .node_id;
    // Change selection through the existing runtime metadata/transaction route.
    for id in [row, sibling] {
        surface
            .mutate_property(zircon_runtime::ui::surface::UiPropertyMutationRequest::new(
                id,
                "selected",
                zircon_runtime_interface::ui::component::UiValue::Bool(true),
            ))
            .unwrap();
    }
    let size = UiSize::new(1280.0, 800.0);
    surface.compute_layout(size).unwrap();
    let row_fill = |surface: &zircon_runtime::ui::surface::UiSurface, id| {
        surface
            .render_extract
            .list
            .commands
            .iter()
            .find(|command| command.node_id == id && command.kind == UiRenderCommandKind::Quad)
            .and_then(|command| command.style.background_color.clone())
    };
    let inactive = row_fill(&surface, row).expect("selected row inactive surface");
    let inactive_sibling = row_fill(&surface, sibling).unwrap();
    for target in [owner, row, sibling] {
        surface.focus_node(target).unwrap();
        surface.rebuild_dirty(size).unwrap();
        assert_eq!(surface.tree_owner_focused(row), Some(true));
        assert_ne!(row_fill(&surface, row), Some(inactive.clone()));
        assert_ne!(
            row_fill(&surface, sibling),
            Some(inactive_sibling.clone()),
            "owner siblings must invalidate"
        );
        let model = runtime
            .build_host_model_with_surface(&projection, &surface)
            .unwrap();
        let retained = crate::ui::template_runtime::RetainedUiHostAdapter::build_projection(&model);
        assert_eq!(
            retained
                .node_by_control_id("WorkbenchSceneRootItem")
                .unwrap()
                .tree_owner_focused,
            Some(true)
        );
        let node = host_template_node(
            model
                .node_by_control_id("WorkbenchSceneRootItem")
                .unwrap()
                .clone(),
        )
        .unwrap();
        assert_eq!(node.tree_owner_focused, Some(true));
        let frame = surface.arranged_node(row).unwrap().frame;
        let pixels = paint_template_nodes_for_test(1280, 800, model_rc(vec![node]));
        let x = (frame.x + frame.width * 0.65) as usize;
        let y = (frame.y + frame.height * 0.5) as usize;
        let index = (y * 1280 + x) * 4;
        let active =
            zircon_runtime_interface::ui::design_tokens::EditorDesignTokens::workbench_dark()
                .palette
                .surface_selected
                .to_u8();
        assert_eq!(
            &pixels[index..index + 4],
            &active,
            "actual source projection must paint active selection"
        );
    }
    surface.focus_node(outside).unwrap();
    surface.rebuild_dirty(size).unwrap();
    assert_eq!(row_fill(&surface, row), Some(inactive.clone()));
    assert_eq!(row_fill(&surface, sibling), Some(inactive_sibling));
    let model = runtime
        .build_host_model_with_surface(&projection, &surface)
        .unwrap();
    let node = host_template_node(
        model
            .node_by_control_id("WorkbenchSceneRootItem")
            .unwrap()
            .clone(),
    )
    .unwrap();
    assert_eq!(node.tree_owner_focused, Some(false));
    let frame = surface.arranged_node(row).unwrap().frame;
    let pixels = paint_template_nodes_for_test(1280, 800, model_rc(vec![node]));
    let index = (((frame.y + frame.height * 0.5) as usize) * 1280
        + (frame.x + frame.width * 0.65) as usize)
        * 4;
    let expected =
        zircon_runtime_interface::ui::design_tokens::EditorDesignTokens::workbench_dark()
            .palette
            .surface_selected_inactive
            .to_u8();
    assert_eq!(
        &pixels[index..index + 4],
        &expected,
        "known lost owner focus must reach native raster"
    );
    surface.focus_node(owner).unwrap();
    surface.clear_focus();
    surface.rebuild_dirty(size).unwrap();
    assert_eq!(surface.focus.focused, None);
    assert_eq!(row_fill(&surface, row), Some(inactive));
    for flag in ["disabled", "loading"] {
        surface
            .mutate_property(zircon_runtime::ui::surface::UiPropertyMutationRequest::new(
                row,
                flag,
                zircon_runtime_interface::ui::component::UiValue::Bool(true),
            ))
            .unwrap();
        surface.rebuild_dirty(size).unwrap();
        assert_eq!(
            row_fill(&surface, row),
            None,
            "unavailable row must suppress selection fill"
        );
        let model = runtime
            .build_host_model_with_surface(&projection, &surface)
            .unwrap();
        let node = host_template_node(
            model
                .node_by_control_id("WorkbenchSceneRootItem")
                .unwrap()
                .clone(),
        )
        .unwrap();
        let frame = surface.arranged_node(row).unwrap().frame;
        let pixels = paint_template_nodes_for_test(1280, 800, model_rc(vec![node]));
        let index = (((frame.y + frame.height * 0.5) as usize) * 1280
            + (frame.x + frame.width * 0.65) as usize)
            * 4;
        assert_eq!(
            &pixels[index..index + 4],
            &[0, 0, 0, 255],
            "unavailable row must remain clear in native raster"
        );
        surface
            .mutate_property(zircon_runtime::ui::surface::UiPropertyMutationRequest::new(
                row,
                flag,
                zircon_runtime_interface::ui::component::UiValue::Bool(false),
            ))
            .unwrap();
    }
    surface.focus_node(owner).unwrap();
    surface
        .mutate_property(zircon_runtime::ui::surface::UiPropertyMutationRequest::new(
            owner,
            "disabled",
            zircon_runtime_interface::ui::component::UiValue::Bool(true),
        ))
        .unwrap();
    surface.rebuild_dirty(size).unwrap();
    assert_eq!(
        surface.focus.focused, None,
        "disabled owner must lose real focus through existing admission"
    );
    assert_eq!(surface.tree_owner_focused(row), Some(false));
}

#[test]
fn live_workbench_tab_keyboard_and_pointer_focus_project_native_ring_modality() {
    use crate::ui::layouts::common::model_rc;
    use crate::ui::retained_host::paint_template_nodes_for_test;
    use crate::ui::template_runtime::EditorUiHostRuntime;
    use zircon_runtime::ui::dispatch::{UiNavigationDispatcher, UiPointerDispatcher};
    use zircon_runtime_interface::ui::{
        dispatch::UiPointerEvent,
        layout::{UiPoint, UiSize},
        surface::{
            UiNavigationEventKind, UiPointerButton, UiPointerEventKind, UiRenderCommandKind,
        },
    };
    let document_id = crate::ui::template_runtime::builtin::UNREAL_EDITOR_WINDOW_DOCUMENT_ID;
    let mut runtime = EditorUiHostRuntime::default();
    runtime
        .load_builtin_host_templates_for_document_ids(&[document_id])
        .unwrap();
    let projection = runtime.project_document(document_id).unwrap();
    let mut surface = runtime.build_shared_surface(document_id).unwrap();
    let size = UiSize::new(1280.0, 800.0);
    surface.compute_layout(size).unwrap();
    let control = "WorkbenchDrawerTabComponents";
    let target = surface
        .tree
        .nodes
        .values()
        .find(|node| {
            node.template_metadata
                .as_ref()
                .and_then(|m| m.control_id.as_deref())
                == Some(control)
                && node.is_focus_candidate()
        })
        .expect("actual authored Tab is a keyboard focus candidate")
        .node_id;
    surface
        .mutate_property(zircon_runtime::ui::surface::UiPropertyMutationRequest::new(
            target,
            "selected",
            zircon_runtime_interface::ui::component::UiValue::Bool(true),
        ))
        .unwrap();
    for (property, value) in [
        (
            "focus_border_color",
            zircon_runtime_interface::ui::component::UiValue::String("#b14ed0".into()),
        ),
        (
            "focus_border_width",
            zircon_runtime_interface::ui::component::UiValue::Float(2.0),
        ),
    ] {
        surface
            .mutate_property(zircon_runtime::ui::surface::UiPropertyMutationRequest::new(
                target, property, value,
            ))
            .unwrap();
    }
    let dispatcher = UiNavigationDispatcher::default();
    for _ in 0..=surface.tree.nodes.len() {
        surface
            .dispatch_navigation_event(&dispatcher, UiNavigationEventKind::Next)
            .unwrap();
        if surface.focus.focused == Some(target) {
            break;
        }
    }
    assert_eq!(
        surface.focus.focused,
        Some(target),
        "real navigation must reach the authored Tab"
    );
    assert!(surface.focus.focus_visible.visible);
    surface.rebuild_dirty(size).unwrap();
    let actual_frame = surface.arranged_node(target).unwrap().frame;
    let ring = surface
        .render_extract
        .list
        .commands
        .iter()
        .find(|c| {
            c.node_id == target
                && c.kind == UiRenderCommandKind::Quad
                && c.style.border_width == 2.0
        })
        .expect("real navigation emits source focus outline");
    assert_eq!(ring.frame, actual_frame);
    assert_eq!(ring.style.background_color.as_deref(), Some("#00000000"));
    let source_focus = [177, 78, 208, 255];
    let expected_focus = format!(
        "#{:02x}{:02x}{:02x}",
        source_focus[0], source_focus[1], source_focus[2]
    );
    assert_eq!(
        ring.style.border_color.as_deref(),
        Some(expected_focus.as_str())
    );
    let ring_z = ring.z_index;
    let underline = surface
        .render_extract
        .list
        .commands
        .iter()
        .find(|c| {
            c.node_id == target
                && c.kind == UiRenderCommandKind::Quad
                && c.frame.height < actual_frame.height
        })
        .expect("selected source Tab still emits its underline");
    assert!(
        ring_z < underline.z_index,
        "selected underline remains above source focus border"
    );
    let project = |surface: &zircon_runtime::ui::surface::UiSurface| {
        let model = runtime
            .build_host_model_with_surface(&projection, surface)
            .unwrap();
        host_template_node(model.node_by_control_id(control).unwrap().clone()).unwrap()
    };
    let keyboard = project(&surface);
    assert!(keyboard.focused && keyboard.focus_visible_known && keyboard.focus_visible);
    let x =
        (keyboard.frame.x + keyboard.layout_offset_x + keyboard.frame.width * 0.5).floor() as usize;
    let y = (keyboard.frame.y + keyboard.layout_offset_y).floor() as usize;
    let pixels = paint_template_nodes_for_test(1280, 800, model_rc(vec![keyboard]));
    let offset = (y * 1280 + x) * 4;
    let focus = source_focus;
    assert_eq!(&pixels[offset..offset + 4], &focus);
    assert_eq!(
        &pixels[offset + 1280 * 4..offset + 1280 * 4 + 4],
        &focus,
        "authored two pixel outline reaches native second row"
    );
    assert_ne!(
        &pixels[offset + 1280 * 8..offset + 1280 * 8 + 4],
        &focus,
        "authored outline stays within its two pixel depth"
    );
    let frame = surface.arranged_node(target).unwrap().frame;
    let event = UiPointerEvent::new(
        UiPointerEventKind::Down,
        UiPoint::new(frame.x + frame.width * 0.5, frame.y + frame.height * 0.5),
    )
    .with_button(UiPointerButton::Primary);
    surface
        .dispatch_pointer_event(&UiPointerDispatcher::default(), event)
        .unwrap();
    assert_eq!(surface.focus.focused, Some(target));
    assert!(
        !surface.focus.focus_visible.visible,
        "actual pointer focus hides keyboard outline"
    );
    surface.rebuild_dirty(size).unwrap();
    let pointer = project(&surface);
    assert!(pointer.focused && pointer.focus_visible_known && !pointer.focus_visible);
    let pixels = paint_template_nodes_for_test(1280, 800, model_rc(vec![pointer]));
    assert_ne!(&pixels[offset..offset + 4], &focus);
    assert!(
        !surface
            .render_extract
            .list
            .commands
            .iter()
            .any(|c| c.node_id == target
                && c.kind == UiRenderCommandKind::Quad
                && c.style.border_width > 0.0),
        "pointer modality emits no source focus border"
    );
    for _ in 0..=surface.tree.nodes.len() {
        surface
            .dispatch_navigation_event(&dispatcher, UiNavigationEventKind::Next)
            .unwrap();
        if surface.focus.focused == Some(target) {
            break;
        }
    }
    assert_eq!(surface.focus.focused, Some(target));
    assert!(surface.focus.focus_visible.visible);
    surface
        .mutate_property(zircon_runtime::ui::surface::UiPropertyMutationRequest::new(
            target,
            "focus_border_width",
            zircon_runtime_interface::ui::component::UiValue::Float(0.0),
        ))
        .unwrap();
    surface.rebuild_dirty(size).unwrap();
    assert!(
        !surface
            .render_extract
            .list
            .commands
            .iter()
            .any(|c| c.node_id == target && c.style.border_width > 0.0),
        "explicit authored zero width suppresses source outline despite visible keyboard focus"
    );
    let zero_width = project(&surface);
    assert_eq!(zero_width.authored_focus_border_width, Some(0.0));
    let pixels = paint_template_nodes_for_test(1280, 800, model_rc(vec![zero_width]));
    assert_ne!(&pixels[offset..offset + 4], &focus);
    surface
        .mutate_property(zircon_runtime::ui::surface::UiPropertyMutationRequest::new(
            target,
            "focus_border_width",
            zircon_runtime_interface::ui::component::UiValue::Float(2.0),
        ))
        .unwrap();
    surface
        .mutate_property(zircon_runtime::ui::surface::UiPropertyMutationRequest::new(
            target,
            "disabled",
            zircon_runtime_interface::ui::component::UiValue::Bool(true),
        ))
        .unwrap();
    surface.rebuild_dirty(size).unwrap();
    assert!(
        !surface
            .render_extract
            .list
            .commands
            .iter()
            .any(|c| c.node_id == target
                && c.kind == UiRenderCommandKind::Quad
                && c.style.border_width > 0.0),
        "disabled source suppresses an existing keyboard outline"
    );
    let disabled = project(&surface);
    assert!(disabled.disabled || disabled.button_style.disabled);
    let pixels = paint_template_nodes_for_test(1280, 800, model_rc(vec![disabled]));
    assert_ne!(&pixels[offset..offset + 4], &focus);
}

#[test]
fn authored_field_asset_colors_reach_native_quads_and_text_for_available_states() {
    use crate::ui::layouts::common::model_rc;
    use crate::ui::retained_host::paint_template_nodes_for_test;
    let source = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/assets/ui/editor/components/workbench/primitives/inputs/field.zui"
    ));
    let asset: Value = source.parse().expect("parse owning WorkbenchField asset");
    let props = asset["nodes"]["root"]["props"]
        .as_table()
        .expect("owning props");
    for control in ["WorkbenchFieldRoot", "SearchEdited"] {
        for state in [
            "normal", "hovered", "focused", "pressed", "selected", "checked",
        ] {
            let mut attributes = props.clone();
            // Deliberately distinguish authored state values from base: a blind base override fails.
            for (key, color) in [
                ("background_color", "#5d6165"),
                ("border_color", "#6a6f74"),
                ("foreground_color", "#e4c46a"),
                ("hover_background_color", "#556677"),
                ("pressed_background_color", "#445566"),
                ("focused_background_color", "#778899"),
                ("hover_border_color", "#8090a0"),
                ("focus_border_color", "#a0b0c0"),
            ] {
                attributes.insert(key.into(), Value::String(color.into()));
            }
            attributes.insert("value".into(), Value::String("Authored".into()));
            for flag in [
                "hovered", "focused", "pressed", "selected", "checked", "disabled",
            ] {
                attributes.insert(flag.into(), Value::Boolean(flag == state));
            }
            let mut projection = projected_node("InputField", []);
            projection.attributes = attributes.into_iter().collect();
            projection.control_id = Some(control.into());
            projection.frame = UiFrame::new(12.0, 8.0, 170.0, 32.0);
            let node = host_template_node(projection).expect("project real asset fields");
            let pixels = paint_template_nodes_for_test(200, 48, model_rc(vec![node]));
            let pixel = |x: usize, y: usize| {
                let i = (y * 200 + x) * 4;
                [pixels[i], pixels[i + 1], pixels[i + 2], pixels[i + 3]]
            };
            let surface = match state {
                "hovered" => [85, 102, 119, 255],
                "focused" => [119, 136, 153, 255],
                "pressed" => [68, 85, 102, 255],
                _ => [93, 97, 101, 255],
            };
            let border = match state {
                "hovered" => [128, 144, 160, 255],
                "focused" | "pressed" => [160, 176, 192, 255],
                _ => [106, 111, 116, 255],
            };
            assert_eq!(pixel(170, 24), surface, "{control}/{state} surface");
            assert_eq!(pixel(80, 8), border, "{control}/{state} border");
            assert!(
                (16..36).any(|y| (38..130).any(|x| {
                    let [r, g, b, _] = pixel(x, y);
                    r > 130 && g > 115 && r > g && g > b
                })),
                "{control}/{state} authored text must reach pixels"
            );
        }
    }
}

#[test]
fn authored_transparent_field_asset_colors_do_not_fall_back_to_host_palette() {
    use crate::ui::layouts::common::model_rc;
    use crate::ui::retained_host::paint_template_nodes_for_test;
    let source = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/assets/ui/editor/components/workbench/primitives/inputs/field.zui"
    ));
    let asset: Value = source.parse().expect("parse owning WorkbenchField asset");
    let props = asset["nodes"]["root"]["props"]
        .as_table()
        .expect("owning props");
    for authored_color in ["transparent", "#00000000"] {
        for control in ["WorkbenchFieldRoot", "SearchEdited"] {
            for state in [
                "normal", "hovered", "focused", "pressed", "selected", "checked",
            ] {
                let mut attributes = props.clone();
                for key in [
                    "background_color",
                    "hover_background_color",
                    "pressed_background_color",
                    "focused_background_color",
                    "border_color",
                    "hover_border_color",
                    "focus_border_color",
                    "foreground_color",
                    "placeholder_color",
                ] {
                    attributes.insert(key.into(), Value::String(authored_color.into()));
                }
                attributes.insert("value".into(), Value::String("Authored".into()));
                for flag in [
                    "hovered", "focused", "pressed", "selected", "checked", "disabled",
                ] {
                    attributes.insert(flag.into(), Value::Boolean(flag == state));
                }
                let mut projection = projected_node("InputField", []);
                projection.attributes = attributes.into_iter().collect();
                projection.control_id = Some(control.into());
                projection.frame = UiFrame::new(12.0, 8.0, 170.0, 32.0);
                let node =
                    host_template_node(projection).expect("project transparent source fields");
                assert!(
                    node.button_style.element.background_color.is_some(),
                    "explicit transparency must remain present in the source projection"
                );
                let pixels = paint_template_nodes_for_test(200, 48, model_rc(vec![node]));
                // Quad fill, border and text must all preserve the underlying opaque black
                // canvas, including toolbar and transient states. Palette fallback changes it.
                for (x, y) in [(170, 24), (80, 8)] {
                    let index = (y * 200 + x) * 4;
                    assert_eq!(
                        &pixels[index..index + 4],
                        &[0, 0, 0, 255],
                        "{authored_color}/{control}/{state} surface/border fell back"
                    );
                }
                // Search has a separately owned SVG prefix. Inspect the text slot after it.
                for y in 16..36 {
                    for x in 48..130 {
                        let index = (y * 200 + x) * 4;
                        assert_eq!(
                            &pixels[index..index + 4],
                            &[0, 0, 0, 255],
                            "{authored_color}/{control}/{state} unexpectedly painted ({x},{y})"
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn authored_field_state_cascade_matches_runtime_commands_and_native_raster() {
    use crate::ui::layouts::common::model_rc;
    use crate::ui::retained_host::paint_template_nodes_for_test;
    use zircon_runtime::ui::surface::UiSurface;
    use zircon_runtime_interface::ui::{
        event_ui::{UiNodeId, UiNodePath, UiTreeId},
        layout::UiSize,
        surface::UiRenderCommandKind,
        tree::{UiTemplateNodeMetadata, UiTreeNode},
    };
    let source = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/assets/ui/editor/components/workbench/primitives/inputs/field.zui"
    ));
    let asset: Value = source.parse().unwrap();
    let props = asset["nodes"]["root"]["props"].as_table().unwrap();
    let as_rgba = |color: &str| {
        let values = BTreeMap::from([(
            "foreground_color".to_owned(),
            Value::String(color.to_owned()),
        )]);
        style_color_u8(
            zircon_runtime::ui::style::resolve_button_style_from_values(&values)
                .element
                .foreground_color
                .as_ref(),
        )
        .expect("real runtime command color")
    };
    for scenario in [
        "pressed",
        "disabled",
        "loading",
        "error",
        "error-default",
        "transparent-disabled",
        "transparent-disabled-keyword",
    ] {
        let mut attrs = props.clone();
        for (key, color) in [
            ("background_color", "#2d3135"),
            ("border_color", "#6a6f74"),
            ("foreground_color", "#e4c46a"),
            ("hover_background_color", "#556677"),
            ("pressed_background_color", "#445566"),
            ("focus_border_color", "#a0b0c0"),
            ("hover_border_color", "#8090a0"),
            ("disabled_background_color", "#414243"),
            ("disabled_border_color", "#717273"),
            ("disabled_foreground_color", "#f4d47a"),
            ("error_border_color", "#f56a96"),
        ] {
            attrs.insert(key.into(), Value::String(color.into()));
        }
        attrs.insert("value".into(), Value::String("Authored".into()));
        for flag in ["hovered", "focused", "pressed", "disabled", "loading"] {
            attrs.insert(
                flag.into(),
                Value::Boolean(match flag {
                    "hovered" | "focused" | "pressed" => true,
                    "disabled" => {
                        scenario == "disabled" || scenario.starts_with("transparent-disabled")
                    }
                    "loading" => scenario == "loading",
                    _ => false,
                }),
            );
        }
        let invalid = matches!(scenario, "error" | "error-default" | "disabled" | "loading");
        attrs.insert(
            "validation_level".into(),
            Value::String(if invalid { "error" } else { "" }.into()),
        );
        if scenario == "error-default" {
            attrs.remove("error_border_color");
        }
        if scenario.starts_with("transparent-disabled") {
            for key in [
                "disabled_background_color",
                "disabled_border_color",
                "disabled_foreground_color",
            ] {
                attrs.insert(
                    key.into(),
                    Value::String(
                        if scenario.ends_with("keyword") {
                            "transparent"
                        } else {
                            "#00000000"
                        }
                        .into(),
                    ),
                );
            }
        }
        let mut projection = projected_node("InputField", []);
        projection.attributes = attrs.clone().into_iter().collect();
        projection.control_id = Some("SearchEdited".into());
        projection.frame = UiFrame::new(12.0, 8.0, 170.0, 32.0);
        let native = host_template_node(projection).unwrap();
        let pixels = paint_template_nodes_for_test(200, 48, model_rc(vec![native]));
        let id = UiNodeId::new(1);
        let mut surface = UiSurface::new(UiTreeId::new("authored-field-parity"));
        let mut source_node = UiTreeNode::new(id, UiNodePath::new("root")).with_template_metadata(
            UiTemplateNodeMetadata {
                component: "InputField".into(),
                attributes: attrs.into_iter().collect(),
                ..Default::default()
            },
        );
        source_node.layout_stretch_width = true;
        source_node.layout_stretch_height = true;
        surface.tree.insert_root(source_node);
        surface.compute_layout(UiSize::new(200.0, 48.0)).unwrap();
        let quad = surface
            .render_extract
            .list
            .commands
            .iter()
            .find(|command| command.node_id == id && command.kind == UiRenderCommandKind::Quad)
            .unwrap();
        let source_surface = as_rgba(quad.style.background_color.as_deref().unwrap());
        let source_border = as_rgba(quad.style.border_color.as_deref().unwrap());
        let pixel = |x: usize, y: usize| {
            let i = (y * 200 + x) * 4;
            &pixels[i..i + 4]
        };
        if scenario.starts_with("transparent-disabled") {
            assert_eq!(source_surface[3], 0);
            assert_eq!(source_border[3], 0);
            assert_eq!(pixel(170, 24), &[0, 0, 0, 255]);
            assert_eq!(pixel(80, 8), &[0, 0, 0, 255]);
        } else {
            assert_eq!(
                pixel(170, 24),
                &source_surface,
                "{scenario}: source/native fill"
            );
            assert_eq!(
                pixel(80, 8),
                &source_border,
                "{scenario}: source/native border"
            );
            if matches!(scenario, "disabled" | "loading") {
                assert_eq!(source_surface, [65, 66, 67, 255]);
                assert_eq!(source_border, [113, 114, 115, 255]);
                let text = surface
                    .render_extract
                    .list
                    .commands
                    .iter()
                    .find(|command| {
                        command.node_id == id && command.kind == UiRenderCommandKind::Text
                    })
                    .unwrap();
                assert_eq!(
                    as_rgba(text.style.foreground_color.as_deref().unwrap()),
                    [244, 212, 122, 255]
                );
                assert!(
                    (16..36).any(|y| (48..130).any(|x| {
                        let p = pixel(x, y);
                        p[0] > 130 && p[1] > 115 && p[0] > p[1] && p[1] > p[2]
                    })),
                    "disabled authored foreground must reach real raster"
                );
            }
            if scenario == "error" {
                assert_eq!(source_border, [245, 106, 150, 255]);
            }
            if scenario == "error-default" {
                assert_eq!(source_border, zircon_runtime_interface::ui::design_tokens::EditorDesignTokens::workbench_dark().palette.error.to_u8());
            }
        }
    }
}
