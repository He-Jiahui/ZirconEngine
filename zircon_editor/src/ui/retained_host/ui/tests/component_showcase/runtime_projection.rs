use super::*;

#[test]
fn component_showcase_pane_projects_runtime_component_nodes_for_template_pane() {
    let _guard = crate::tests::support::env_lock()
        .lock()
        .unwrap_or_else(|poison| poison.into_inner());
    let (_fixture, chrome, _model, _ui_asset_panes, _animation_panes) = root_shell_fixture();
    let body_spec = PaneBodySpec::new(
        "res://ui/editor/component_showcase.zui",
        PanePayloadKind::UiComponentShowcaseV1,
        PaneRouteNamespace::UiComponentShowcase,
        PaneInteractionMode::TemplateOnly,
    );
    let body = host_window::build_pane_body_presentation(
        &body_spec,
        &host_window::PanePayloadBuildContext::new(&chrome),
    );
    let mut pane = host_pane("component-showcase", "UI Component Showcase");
    pane.kind = "UiComponentShowcase".into();
    pane.pane_presentation = Some(host_window::PanePresentation::new(
        host_window::PaneShellPresentation::new(
            "UI Component Showcase",
            "ui-components",
            "Runtime components",
            "",
            None,
            false,
            blank_viewport_chrome(),
        ),
        body,
    ));

    let host_contract_pane =
        super::pane_data_conversion::to_host_contract_component_showcase_pane_from_host_pane(
            &pane,
            host_window::PaneContentSize::new(1080.0, 720.0),
        );

    let nodes = (0..host_contract_pane.nodes.row_count())
        .filter_map(|row| host_contract_pane.nodes.row_data(row))
        .collect::<Vec<_>>();
    let number = nodes
        .iter()
        .find(|node| node.control_id.as_str() == "NumberFieldDemo")
        .expect("component showcase pane should expose NumberFieldDemo");
    assert_eq!(number.component_role.as_str(), "number-field");
    assert_eq!(number.value_text.as_str(), "42");
    assert_eq!(number.value_number, 42.0);
    assert_eq!(number.value_percent, 0.42);
    assert_eq!(number.dispatch_kind.as_str(), "showcase");
    assert_eq!(
        number.action_id.as_str(),
        "ui_component_showcase.number_field_drag_update"
    );
    assert_eq!(
        number.drag_action_id.as_str(),
        "ui_component_showcase.number_field_drag_update"
    );
    assert_eq!(
        number.begin_drag_action_id.as_str(),
        "ui_component_showcase.number_field_drag_begin"
    );
    assert_eq!(
        number.end_drag_action_id.as_str(),
        "ui_component_showcase.number_field_drag_end"
    );
    assert_eq!(
        number.edit_action_id.as_str(),
        "ui_component_showcase.number_field_changed"
    );
    assert_eq!(
        number.commit_action_id.as_str(),
        "ui_component_showcase.number_field_committed"
    );

    let input = nodes
        .iter()
        .find(|node| node.control_id.as_str() == "InputFieldDemo")
        .expect("component showcase pane should expose InputFieldDemo");
    assert_eq!(
        input.edit_action_id.as_str(),
        "ui_component_showcase.input_field_changed"
    );
    assert_eq!(
        input.commit_action_id.as_str(),
        "ui_component_showcase.input_field_committed"
    );

    let text = nodes
        .iter()
        .find(|node| node.control_id.as_str() == "TextFieldDemo")
        .expect("component showcase pane should expose TextFieldDemo");
    assert_eq!(
        text.commit_action_id.as_str(),
        "ui_component_showcase.text_field_committed"
    );

    let range = nodes
        .iter()
        .find(|node| node.control_id.as_str() == "RangeFieldDemo")
        .expect("component showcase pane should expose RangeFieldDemo");
    assert_eq!(range.value_number, 68.0);
    assert_eq!(range.value_percent, 0.68);
    assert_eq!(
        range.drag_action_id.as_str(),
        "ui_component_showcase.range_field_drag_update"
    );
    assert_eq!(
        range.edit_action_id.as_str(),
        "ui_component_showcase.range_field_changed"
    );
    assert_eq!(
        range.commit_action_id.as_str(),
        "ui_component_showcase.range_field_committed"
    );

    let slider = template_node(&nodes, "SliderDemo");
    assert_eq!(slider.component_role.as_str(), "slider");
    assert_eq!(slider.component_category.as_str(), "numeric");
    assert_eq!(slider.value_text.as_str(), "42");
    assert_eq!(slider.value_number, 42.0);
    assert_eq!(slider.value_percent, 0.42);
    assert!(slider.hovered);
    assert!(slider.focused);
    assert_eq!(
        slider.drag_action_id.as_str(),
        "ui_component_showcase.slider_drag_update"
    );
    assert_eq!(
        slider.edit_action_id.as_str(),
        "ui_component_showcase.slider_changed"
    );

    let range_slider = template_node(&nodes, "RangeSliderDemo");
    assert_eq!(range_slider.component_role.as_str(), "range-slider");
    assert_eq!(range_slider.component_category.as_str(), "numeric");
    assert_eq!(range_slider.value_text.as_str(), "72");
    assert_eq!(range_slider.value_number, 72.0);
    assert_eq!(range_slider.value_percent, 0.72);
    assert_eq!(range_slider.layout_second_cell_offset_x, 28.0);
    assert!(range_slider.pressed);
    assert!(range_slider.focused);
    assert_eq!(
        range_slider.drag_action_id.as_str(),
        "ui_component_showcase.range_slider_drag_update"
    );
    assert_eq!(
        range_slider.edit_action_id.as_str(),
        "ui_component_showcase.range_slider_changed"
    );

    let tab = template_node(&nodes, "TabDemo");
    assert_eq!(tab.component_role.as_str(), "tab");
    assert_eq!(tab.component_category.as_str(), "input");
    assert_eq!(tab.value_text.as_str(), "scene");
    assert!(tab.selected);
    assert!(tab.focused);
    assert_eq!(
        tab.edit_action_id.as_str(),
        "ui_component_showcase.tab_changed"
    );

    let tab_strip = template_node(&nodes, "TabStripDemo");
    assert_eq!(tab_strip.component_role.as_str(), "tabs");
    assert_eq!(tab_strip.component_category.as_str(), "container");
    assert_eq!(tab_strip.component_variant.as_str(), "standard");
    assert_eq!(tab_strip.value_text.as_str(), "scene");
    assert_eq!(tab_strip.options_text.as_str(), "scene, assets, console");
    assert_eq!(tab_strip.options.row_count(), 3);
    assert_eq!(tab_strip.structured_options.row_count(), 3);
    let selected_tab = tab_strip
        .structured_options
        .row_data(0)
        .expect("TabStripDemo should expose the selected tab row");
    assert_eq!(selected_tab.id.as_str(), "scene");
    assert!(selected_tab.selected);
    let focused_tab = tab_strip
        .structured_options
        .row_data(1)
        .expect("TabStripDemo should expose focused and hovered tab row");
    assert_eq!(focused_tab.id.as_str(), "assets");
    assert!(focused_tab.focused);
    assert!(focused_tab.hovered);
    let disabled_tab = tab_strip
        .structured_options
        .row_data(2)
        .expect("TabStripDemo should expose disabled tab row");
    assert_eq!(disabled_tab.id.as_str(), "console");
    assert!(disabled_tab.disabled);
    assert_eq!(
        tab_strip.edit_action_id.as_str(),
        "ui_component_showcase.tab_strip_changed"
    );

    let dropdown = nodes
        .iter()
        .find(|node| node.control_id.as_str() == "DropdownDemo")
        .expect("component showcase pane should expose DropdownDemo");
    assert!(dropdown.popup_open);
    assert_eq!(dropdown.selection_state.as_str(), "multi");
    assert_eq!(dropdown.options_text.as_str(), "runtime, editor, debug");
    assert_eq!(dropdown.options.row_count(), 3);
    assert_eq!(dropdown.options.row_data(0).as_deref(), Some("runtime"));
    assert_eq!(dropdown.options.row_data(1).as_deref(), Some("editor"));
    assert_eq!(dropdown.options.row_data(2).as_deref(), Some("debug"));
    assert_eq!(dropdown.structured_options.row_count(), 3);
    let selected_option = dropdown
        .structured_options
        .row_data(0)
        .expect("DropdownDemo should project a selected structured option row");
    assert_eq!(selected_option.id.as_str(), "runtime");
    assert_eq!(selected_option.label.as_str(), "runtime");
    assert!(selected_option.selected);
    assert!(selected_option.special);
    assert!(selected_option.pressed);
    assert!(!selected_option.disabled);
    assert!(!selected_option.focused);
    assert!(!selected_option.hovered);
    let focused_option = dropdown
        .structured_options
        .row_data(1)
        .expect("DropdownDemo should project focused and hovered candidate metadata");
    assert_eq!(focused_option.id.as_str(), "editor");
    assert!(focused_option.focused);
    assert!(focused_option.hovered);
    assert!(!focused_option.pressed);
    let disabled_option = dropdown
        .structured_options
        .row_data(2)
        .expect("DropdownDemo should project disabled candidate metadata");
    assert_eq!(disabled_option.id.as_str(), "debug");
    assert!(disabled_option.disabled);

    let combo_box = nodes
        .iter()
        .find(|node| node.control_id.as_str() == "ComboBoxDemo")
        .expect("component showcase pane should expose ComboBoxDemo");
    assert!(!combo_box.popup_open);
    assert_eq!(
        combo_box.action_id.as_str(),
        "ui_component_showcase.combo_box_open_popup"
    );
    assert_eq!(combo_box.options_text.as_str(), "material, fluent, native");
    assert_eq!(combo_box.options.row_count(), 3);

    let search_select = nodes
        .iter()
        .find(|node| node.control_id.as_str() == "SearchSelectDemo")
        .expect("component showcase pane should expose SearchSelectDemo");
    assert_eq!(search_select.component_role.as_str(), "search-select");
    assert_eq!(search_select.search_query.as_str(), "number");
    assert_eq!(
        search_select.edit_action_id.as_str(),
        "ui_component_showcase.search_select_query_changed"
    );
    assert_eq!(search_select.options.row_count(), 3);
    assert_eq!(search_select.structured_options.row_count(), 3);
    let search_selected = search_select
        .structured_options
        .row_data(0)
        .expect("SearchSelectDemo should mark the current result");
    assert_eq!(search_selected.id.as_str(), "runtime.ui.NumberField");
    assert!(search_selected.selected);
    assert!(search_selected.matched);
    let search_unmatched = search_select
        .structured_options
        .row_data(1)
        .expect("SearchSelectDemo should project unmatched result state");
    assert_eq!(search_unmatched.id.as_str(), "runtime.ui.RangeField");
    assert!(!search_unmatched.matched);

    let context_menu = nodes
        .iter()
        .find(|node| node.control_id.as_str() == "ContextMenuDemo")
        .expect("component showcase pane should expose ContextMenuDemo");
    assert!(context_menu.popup_open);
    assert!(context_menu.has_popup_anchor);
    assert_eq!(context_menu.popup_anchor_x, 320.0);
    assert_eq!(context_menu.popup_anchor_y, 28.0);
    assert_eq!(context_menu.menu_items.row_count(), 4);
    assert_eq!(context_menu.structured_menu_items.row_count(), 4);
    let context_checked = context_menu
        .structured_menu_items
        .row_data(0)
        .expect("ContextMenuDemo should expose checked menu row");
    assert_eq!(context_checked.action_id.as_str(), "menu.item.inspect");
    assert_eq!(context_checked.label.as_str(), "Inspect");
    assert_eq!(context_checked.shortcut.as_str(), "Ctrl+I");
    assert!(context_checked.checked);
    assert!(context_checked.focused);
    let context_separator = context_menu
        .structured_menu_items
        .row_data(1)
        .expect("ContextMenuDemo should expose separator row");
    assert!(context_separator.separator);
    let context_pressed = context_menu
        .structured_menu_items
        .row_data(2)
        .expect("ContextMenuDemo should expose pressed menu row");
    assert_eq!(context_pressed.action_id.as_str(), "menu.item.duplicate");
    assert!(context_pressed.hovered);
    assert!(context_pressed.pressed);
    let context_loading = context_menu
        .structured_menu_items
        .row_data(3)
        .expect("ContextMenuDemo should expose loading disabled row");
    assert_eq!(context_loading.action_id.as_str(), "menu.item.archive");
    assert!(context_loading.loading);
    assert!(context_loading.disabled);

    let dropdown_popup = nodes
        .iter()
        .find(|node| node.control_id.as_str() == "DropdownPopupDemo")
        .expect("component showcase pane should expose DropdownPopupDemo");
    assert!(dropdown_popup.popup_open);
    assert_eq!(dropdown_popup.structured_options.row_count(), 4);
    let popup_selected = dropdown_popup
        .structured_options
        .row_data(0)
        .expect("DropdownPopupDemo should expose selected option row");
    assert_eq!(popup_selected.id.as_str(), "scene");
    assert_eq!(popup_selected.label.as_str(), "Scene");
    assert!(popup_selected.selected);
    let popup_focused = dropdown_popup
        .structured_options
        .row_data(1)
        .expect("DropdownPopupDemo should expose focused hovered option row");
    assert_eq!(popup_focused.id.as_str(), "assets");
    assert!(popup_focused.focused);
    assert!(popup_focused.hovered);
    let popup_disabled = dropdown_popup
        .structured_options
        .row_data(2)
        .expect("DropdownPopupDemo should expose disabled option row");
    assert_eq!(popup_disabled.id.as_str(), "console");
    assert!(popup_disabled.disabled);
    let popup_loading = dropdown_popup
        .structured_options
        .row_data(3)
        .expect("DropdownPopupDemo should expose loading option row");
    assert_eq!(popup_loading.id.as_str(), "render");
    assert!(popup_loading.loading);

    let flags = nodes
        .iter()
        .find(|node| node.control_id.as_str() == "FlagsFieldDemo")
        .expect("component showcase pane should expose FlagsFieldDemo");
    assert_eq!(flags.selection_state.as_str(), "flags");
    assert_eq!(flags.structured_options.row_count(), 3);
    assert!(
        flags
            .structured_options
            .row_data(0)
            .expect("FlagsField should mark Selectable")
            .selected
    );
    assert!(
        flags
            .structured_options
            .row_data(1)
            .expect("FlagsField should mark Draggable")
            .selected
    );
    assert!(
        !flags
            .structured_options
            .row_data(2)
            .expect("FlagsField should leave Disabled unselected")
            .selected
    );

    let progress = nodes
        .iter()
        .find(|node| node.control_id.as_str() == "ProgressBarDemo")
        .expect("component showcase pane should expose ProgressBarDemo");
    assert_eq!(progress.value_number, 0.62);
    assert_eq!(progress.value_percent, 0.62);

    let skeleton = template_node(&nodes, "SkeletonDemo");
    assert_eq!(skeleton.component_role.as_str(), "skeleton");
    assert_eq!(skeleton.component_category.as_str(), "feedback");
    assert!(
        skeleton
            .component_variant
            .split_whitespace()
            .any(|token| token == "rounded")
    );
    assert!(
        skeleton
            .component_variant
            .split_whitespace()
            .any(|token| token == "wave")
    );
    assert_eq!(skeleton.text.as_str(), "Skeleton");

    let dialog = nodes
        .iter()
        .find(|node| node.control_id.as_str() == "DialogDemo")
        .expect("component showcase pane should expose DialogDemo");
    assert_eq!(dialog.component_role.as_str(), "dialog");
    assert_eq!(dialog.text.as_str(), "Scene Settings");
    assert_eq!(
        dialog.value_text.as_str(),
        "Review scene-level settings before applying them."
    );
    assert_eq!(dialog.actions.row_count(), 1);
    assert_eq!(dialog.actions.row_data(0).unwrap().label.as_str(), "Apply");
    assert!(dialog.popup_open);

    let confirm_dialog = nodes
        .iter()
        .find(|node| node.control_id.as_str() == "ConfirmDialogDemo")
        .expect("component showcase pane should expose ConfirmDialogDemo");
    assert_eq!(confirm_dialog.component_role.as_str(), "confirm-dialog");
    assert_eq!(confirm_dialog.text.as_str(), "Delete selected prefab?");
    assert_eq!(
        confirm_dialog.value_text.as_str(),
        "This removes the prefab reference from the scene."
    );
    assert_eq!(confirm_dialog.validation_level.as_str(), "error");
    assert!(
        confirm_dialog
            .component_variant
            .split_whitespace()
            .any(|token| token == "confirmDisabled")
    );
    assert_eq!(confirm_dialog.actions.row_count(), 2);
    assert_eq!(
        confirm_dialog.actions.row_data(0).unwrap().label.as_str(),
        "Cancel"
    );
    assert_eq!(
        confirm_dialog.actions.row_data(1).unwrap().label.as_str(),
        "Delete"
    );
    assert!(confirm_dialog.popup_open);

    let command_palette = nodes
        .iter()
        .find(|node| node.control_id.as_str() == "CommandPaletteDemo")
        .expect("component showcase pane should expose CommandPaletteDemo");
    assert_eq!(command_palette.component_role.as_str(), "command-palette");
    assert_eq!(command_palette.component_category.as_str(), "input");
    assert_eq!(command_palette.component_layout_role.as_str(), "popup");
    assert!(command_palette.popup_open);
    assert_eq!(command_palette.search_query.as_str(), "build");
    assert_eq!(
        command_palette.options_text.as_str(),
        "Build Project, Build Assets"
    );
    assert_eq!(command_palette.options.row_count(), 2);
    assert_eq!(
        command_palette.options.row_data(0).as_deref(),
        Some("Build Project")
    );
    assert_eq!(command_palette.structured_options.row_count(), 2);
    let build_project = command_palette
        .structured_options
        .row_data(0)
        .expect("CommandPaletteDemo should mark the selected focused command");
    assert_eq!(build_project.id.as_str(), "build_project");
    assert_eq!(build_project.label.as_str(), "Build Project");
    assert!(build_project.selected);
    assert!(build_project.focused);
    assert!(build_project.matched);
    assert!(!build_project.disabled);
    let build_assets = command_palette
        .structured_options
        .row_data(1)
        .expect("CommandPaletteDemo should mark disabled filtered command rows");
    assert_eq!(build_assets.id.as_str(), "build_assets");
    assert_eq!(build_assets.label.as_str(), "Build Assets");
    assert!(build_assets.disabled);
    assert!(build_assets.matched);

    let notification_center = nodes
        .iter()
        .find(|node| node.control_id.as_str() == "NotificationCenterDemo")
        .expect("component showcase pane should expose NotificationCenterDemo");
    assert_eq!(
        notification_center.component_role.as_str(),
        "notification-center"
    );
    assert_eq!(notification_center.text.as_str(), "Notifications");
    assert_eq!(notification_center.value_text.as_str(), "No notifications");
    assert!(notification_center.popup_open);
    assert_eq!(
        notification_center.options_text.as_str(),
        "Build failed, Asset import complete"
    );
    assert_eq!(notification_center.structured_options.row_count(), 2);
    let build_failed = notification_center
        .structured_options
        .row_data(0)
        .expect("NotificationCenterDemo should expose selected unread notification");
    assert_eq!(build_failed.id.as_str(), "build");
    assert_eq!(build_failed.label.as_str(), "Build failed");
    assert_eq!(build_failed.description.as_str(), "Shader compile error");
    assert_eq!(build_failed.tone.as_str(), "error");
    assert!(build_failed.selected);
    assert!(build_failed.unread);
    let asset_import = notification_center
        .structured_options
        .row_data(1)
        .expect("NotificationCenterDemo should expose focused notification row");
    assert_eq!(asset_import.id.as_str(), "asset");
    assert!(asset_import.focused);
    assert!(asset_import.unread);

    let color = nodes
        .iter()
        .find(|node| node.control_id.as_str() == "ColorFieldDemo")
        .expect("component showcase pane should expose ColorFieldDemo");
    assert_eq!(
        color.value_color,
        crate::ui::retained_host::primitives::Color::from_rgb_u8(77, 137, 255)
    );
    assert_eq!(
        color.action_id.as_str(),
        "ui_component_showcase.color_field_changed"
    );

    let image = nodes
        .iter()
        .find(|node| node.control_id.as_str() == "ImageDemo")
        .expect("component showcase pane should expose ImageDemo");
    assert_eq!(
        image.media_source.as_str(),
        "ui/editor/showcase_checker.svg"
    );
    assert!(image.has_preview_image);

    let icon = nodes
        .iter()
        .find(|node| node.control_id.as_str() == "IconDemo")
        .expect("component showcase pane should expose IconDemo");
    assert_eq!(icon.icon_name.as_str(), "options-outline");
    assert!(icon.has_preview_image);
    assert!(
        icon.preview_image.size().width > 0 && icon.preview_image.size().height > 0,
        "IconDemo should resolve icon_name into a loaded Retained image"
    );

    let svg_icon = nodes
        .iter()
        .find(|node| node.control_id.as_str() == "SvgIconDemo")
        .expect("component showcase pane should expose SvgIconDemo");
    assert_eq!(
        svg_icon.media_source.as_str(),
        "ionicons/options-outline.svg"
    );
    assert!(svg_icon.has_preview_image);
    assert!(
        svg_icon.preview_image.size().width > 0 && svg_icon.preview_image.size().height > 0,
        "SvgIconDemo should resolve source into a loaded Retained SVG image"
    );

    let vector2 = nodes
        .iter()
        .find(|node| node.control_id.as_str() == "Vector2FieldDemo")
        .expect("component showcase pane should expose Vector2Demo");
    assert_eq!(vector2.vector_components.row_count(), 2);
    assert_eq!(vector2.vector_components.row_data(0), Some(12.0));
    assert_eq!(vector2.vector_components.row_data(1), Some(24.0));
    assert_eq!(
        vector2.action_id.as_str(),
        "ui_component_showcase.vector2_field_changed"
    );

    let vector3 = nodes
        .iter()
        .find(|node| node.control_id.as_str() == "Vector3FieldDemo")
        .expect("component showcase pane should expose Vector3Demo");
    assert_eq!(vector3.vector_components.row_count(), 3);
    assert_eq!(vector3.vector_components.row_data(0), Some(0.0));
    assert_eq!(vector3.vector_components.row_data(1), Some(1.0));
    assert_eq!(vector3.vector_components.row_data(2), Some(0.0));
    assert_eq!(
        vector3.action_id.as_str(),
        "ui_component_showcase.vector3_field_changed"
    );

    let vector4 = nodes
        .iter()
        .find(|node| node.control_id.as_str() == "Vector4FieldDemo")
        .expect("component showcase pane should expose Vector4Demo");
    assert_eq!(vector4.vector_components.row_count(), 4);
    assert_eq!(vector4.vector_components.row_data(3), Some(1.0));
    assert_eq!(
        vector4.action_id.as_str(),
        "ui_component_showcase.vector4_field_changed"
    );

    let inspector_section = nodes
        .iter()
        .find(|node| node.control_id.as_str() == "InspectorSectionDemo")
        .expect("component showcase pane should expose InspectorSectionDemo");
    assert_eq!(
        inspector_section.component_role.as_str(),
        "inspector-section"
    );
    assert!(inspector_section.expanded);
    assert_eq!(
        inspector_section.action_id.as_str(),
        "ui_component_showcase.inspector_section_toggled"
    );

    let asset = nodes
        .iter()
        .find(|node| node.control_id.as_str() == "AssetFieldDemo")
        .expect("component showcase pane should expose AssetFieldDemo");
    assert!(asset.drop_hovered);
    assert!(asset.active_drag_target);
    assert_eq!(asset.actions.row_count(), 3);
    assert_eq!(
        asset.actions.row_data(0).map(|action| action.label),
        Some("Find".into())
    );
    assert_eq!(
        asset.actions.row_data(1).map(|action| action.label),
        Some("Open".into())
    );
    assert_eq!(
        asset.actions.row_data(2).map(|action| action.label),
        Some("Clear".into())
    );
    for (control_id, suffix) in [
        ("InstanceFieldDemo", "InstanceField"),
        ("ObjectFieldDemo", "ObjectField"),
    ] {
        let reference = nodes
            .iter()
            .find(|node| node.control_id.as_str() == control_id)
            .unwrap_or_else(|| panic!("component showcase pane should expose {control_id}"));
        assert_eq!(reference.actions.row_count(), 3);
        for (index, (label, action_suffix)) in
            [("Find", "Locate"), ("Open", "Open"), ("Clear", "Clear")]
                .into_iter()
                .enumerate()
        {
            let action = reference
                .actions
                .row_data(index)
                .unwrap_or_else(|| panic!("{control_id} should expose action {label}"));
            let expected_action_id = expected_showcase_action_id(suffix, action_suffix);
            assert_eq!(action.label.as_str(), label);
            assert_eq!(action.action_id.as_str(), expected_action_id.as_str());
        }
    }

    let array = nodes
        .iter()
        .find(|node| node.control_id.as_str() == "ArrayFieldDemo")
        .expect("component showcase pane should expose ArrayFieldDemo");
    assert_eq!(array.actions.row_count(), 4);
    assert_eq!(array.collection_fields.row_count(), 3);
    let array_item = array
        .collection_fields
        .row_data(0)
        .expect("ArrayField should expose first child edit row");
    assert_eq!(array_item.index_text.as_str(), "#0");
    assert_eq!(array_item.key_text.as_str(), "");
    assert_eq!(array_item.value_type.as_str(), "UiComponentRef");
    assert_eq!(array_item.value_component_role.as_str(), "reference-field");
    assert_eq!(array_item.value_text.as_str(), "Label");
    assert_eq!(array_item.validation_level.as_str(), "normal");
    assert_eq!(array_item.validation_message.as_str(), "");
    assert_eq!(
        array_item.edit_action_id.as_str(),
        "ui_component_showcase.array_field_set_element"
    );
    assert_eq!(
        array_item.remove_action_id.as_str(),
        "ui_component_showcase.array_field_remove_element"
    );
    assert_eq!(array_item.move_up_action_id.as_str(), "");
    assert_eq!(
        array_item.move_down_action_id.as_str(),
        "ui_component_showcase.array_field_move_element"
    );
    assert_eq!(array_item.move_down_payload.as_str(), "array-0=1");
    assert!(!array_item.empty);

    let map = nodes
        .iter()
        .find(|node| node.control_id.as_str() == "MapFieldDemo")
        .expect("component showcase pane should expose MapFieldDemo");
    assert_eq!(map.actions.row_count(), 3);
    assert_eq!(map.collection_fields.row_count(), 2);
    let map_item = map
        .collection_fields
        .row_data(0)
        .expect("MapField should expose first key/value edit row");
    assert_eq!(map_item.key_type.as_str(), "String");
    assert_eq!(map_item.key_component_role.as_str(), "text-field");
    assert_eq!(map_item.key_text.as_str(), "speed");
    assert_eq!(map_item.value_type.as_str(), "UiValue");
    assert_eq!(map_item.value_component_role.as_str(), "number-field");
    assert_eq!(map_item.value_text.as_str(), "1");
    assert_eq!(map_item.validation_level.as_str(), "normal");
    assert_eq!(map_item.validation_message.as_str(), "");
    assert_eq!(
        map_item.edit_action_id.as_str(),
        "ui_component_showcase.map_field_set_entry"
    );
    assert_eq!(
        map_item.key_edit_action_id.as_str(),
        "ui_component_showcase.map_field_set_entry"
    );
    assert_eq!(
        map_item.remove_action_id.as_str(),
        "ui_component_showcase.map_field_remove_entry"
    );
    assert_eq!(map_item.move_up_action_id.as_str(), "");
    assert_eq!(map_item.move_down_action_id.as_str(), "");
    assert!(!map_item.empty);
    let bool_map_item = map
        .collection_fields
        .row_data(1)
        .expect("MapField should expose second key/value edit row");
    assert_eq!(bool_map_item.key_text.as_str(), "visible");
    assert_eq!(bool_map_item.value_component_role.as_str(), "checkbox");
    assert_eq!(bool_map_item.value_text.as_str(), "true");
    assert!(bool_map_item.value_checked);

    let list_row = nodes
        .iter()
        .find(|node| node.control_id.as_str() == "ListRowDemo")
        .expect("component showcase pane should expose ListRowDemo");
    assert!(list_row.selected);
    assert!(list_row.focused);
    assert!(list_row.hovered);

    let tree_row = nodes
        .iter()
        .find(|node| node.control_id.as_str() == "TreeRowDemo")
        .expect("component showcase pane should expose TreeRowDemo");
    assert!(tree_row.expanded);
    assert_eq!(tree_row.tree_depth, 2);
    assert_eq!(tree_row.tree_indent_px, 24.0);

    let context_menu = nodes
        .iter()
        .find(|node| node.control_id.as_str() == "ContextActionMenuDemo")
        .expect("component showcase pane should expose ContextActionMenuDemo");
    assert!(context_menu.has_popup_anchor);
    assert_eq!(context_menu.popup_anchor_x, 156.0);
    assert_eq!(context_menu.popup_anchor_y, 24.0);
    assert_eq!(context_menu.structured_menu_items.row_count(), 4);
    let checked_item = context_menu
        .structured_menu_items
        .row_data(0)
        .expect("ContextActionMenu should expose checked menu row");
    assert_eq!(checked_item.action_id.as_str(), "menu.item.inspect");
    assert_eq!(checked_item.label.as_str(), "Inspect");
    assert_eq!(checked_item.shortcut.as_str(), "Ctrl+I");
    assert!(checked_item.checked);
    assert!(checked_item.focused);
    assert!(!checked_item.hovered);
    assert!(!checked_item.pressed);
    assert!(!checked_item.disabled);
    assert!(!checked_item.separator);
    let separator = context_menu
        .structured_menu_items
        .row_data(1)
        .expect("ContextActionMenu should expose separator row");
    assert!(separator.separator);
    let pressed_item = context_menu
        .structured_menu_items
        .row_data(2)
        .expect("ContextActionMenu should expose pressed menu row");
    assert_eq!(pressed_item.action_id.as_str(), "menu.item.duplicate");
    assert_eq!(pressed_item.label.as_str(), "Duplicate");
    assert!(pressed_item.hovered);
    assert!(pressed_item.pressed);
    assert!(!pressed_item.focused);
    let disabled_item = context_menu
        .structured_menu_items
        .row_data(3)
        .expect("ContextActionMenu should expose disabled menu row");
    assert_eq!(disabled_item.action_id.as_str(), "menu.item.delete");
    assert_eq!(disabled_item.label.as_str(), "Delete");
    assert_eq!(disabled_item.shortcut.as_str(), "Del");
    assert!(disabled_item.disabled);

    let event_log = nodes
        .iter()
        .find(|node| node.control_id.as_str() == "ComponentShowcaseEventLog")
        .expect("component showcase pane should expose event log node");
    assert!(event_log.text.contains("Registered events"));
}
