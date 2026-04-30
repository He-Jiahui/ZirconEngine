use super::*;

fn source(relative: &str) -> String {
    std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(relative))
        .unwrap_or_else(|error| panic!("read `{relative}`: {error}"))
}

#[test]
fn component_showcase_template_metadata_is_owned_by_rust_contracts() {
    let template_nodes = source("src/ui/slint_host/host_contract/data/template_nodes.rs");
    let showcase_asset = source("assets/ui/editor/component_showcase.ui.toml");

    for required in [
        "pub value_number: f32",
        "pub value_percent: f32",
        "pub value_color: Color",
        "pub media_source: SharedString",
        "pub icon_name: SharedString",
        "pub has_preview_image: bool",
        "pub preview_image: Image",
        "pub vector_components: ModelRc<f32>",
        "pub dispatch_kind: SharedString",
        "pub begin_drag_action_id: SharedString",
        "pub drag_action_id: SharedString",
        "pub commit_action_id: SharedString",
        "pub edit_action_id: SharedString",
    ] {
        assert!(
            template_nodes.contains(required),
            "template node DTO missing `{required}`"
        );
    }

    for required in [
        "NumberFieldDemo",
        "InputFieldDemo",
        "RangeFieldDemo",
        "ColorFieldDemo",
        "Vector3FieldDemo",
        "UiComponentShowcase/NumberFieldDragUpdate",
        "UiComponentShowcase/InputFieldChanged",
    ] {
        assert!(
            showcase_asset.contains(required),
            "component showcase TOML missing `{required}`"
        );
    }
}

#[test]
fn component_showcase_option_and_action_callbacks_are_rust_wired() {
    let template_nodes = source("src/ui/slint_host/host_contract/data/template_nodes.rs");
    let callbacks = source("src/ui/slint_host/host_contract/globals.rs");
    let wiring = source("src/ui/slint_host/app/callback_wiring.rs");
    let pane_actions = source("src/ui/slint_host/app/pane_surface_actions.rs");
    let showcase_event_inputs = source("src/ui/slint_host/app/showcase_event_inputs.rs");

    for required in [
        "pub(crate) struct TemplatePaneActionData",
        "pub(crate) struct TemplatePaneOptionData",
        "pub structured_options: ModelRc<TemplatePaneOptionData>",
        "pub actions: ModelRc<TemplatePaneActionData>",
    ] {
        assert!(
            template_nodes.contains(required),
            "template node DTO missing `{required}`"
        );
    }
    assert!(callbacks.contains("on_component_showcase_option_selected"));
    assert!(wiring.contains("pane_surface_host.on_component_showcase_option_selected("));
    assert!(pane_actions.contains("dispatch_component_showcase_option_selected"));
    for required in ["AssetFieldClear", "AssetFieldLocate", "AssetFieldOpen"] {
        assert!(
            showcase_event_inputs.contains(required),
            "showcase action input missing `{required}`"
        );
    }
}

#[test]
fn component_showcase_pane_projects_runtime_component_nodes_for_template_pane() {
    let _guard = crate::tests::support::env_lock()
        .lock()
        .unwrap_or_else(|poison| poison.into_inner());
    let (_fixture, chrome, _model, _ui_asset_panes, _animation_panes) = root_shell_fixture();
    let body_spec = PaneBodySpec::new(
        "editor.window.ui_component_showcase",
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
        "UiComponentShowcase/NumberFieldDragUpdate"
    );
    assert_eq!(
        number.drag_action_id.as_str(),
        "UiComponentShowcase/NumberFieldDragUpdate"
    );
    assert_eq!(
        number.begin_drag_action_id.as_str(),
        "UiComponentShowcase/NumberFieldDragBegin"
    );
    assert_eq!(
        number.end_drag_action_id.as_str(),
        "UiComponentShowcase/NumberFieldDragEnd"
    );
    assert_eq!(
        number.edit_action_id.as_str(),
        "UiComponentShowcase/NumberFieldChanged"
    );
    assert_eq!(
        number.commit_action_id.as_str(),
        "UiComponentShowcase/NumberFieldCommitted"
    );

    let input = nodes
        .iter()
        .find(|node| node.control_id.as_str() == "InputFieldDemo")
        .expect("component showcase pane should expose InputFieldDemo");
    assert_eq!(
        input.edit_action_id.as_str(),
        "UiComponentShowcase/InputFieldChanged"
    );
    assert_eq!(
        input.commit_action_id.as_str(),
        "UiComponentShowcase/InputFieldCommitted"
    );

    let text = nodes
        .iter()
        .find(|node| node.control_id.as_str() == "TextFieldDemo")
        .expect("component showcase pane should expose TextFieldDemo");
    assert_eq!(
        text.commit_action_id.as_str(),
        "UiComponentShowcase/TextFieldCommitted"
    );

    let range = nodes
        .iter()
        .find(|node| node.control_id.as_str() == "RangeFieldDemo")
        .expect("component showcase pane should expose RangeFieldDemo");
    assert_eq!(range.value_number, 68.0);
    assert_eq!(range.value_percent, 0.68);
    assert_eq!(
        range.drag_action_id.as_str(),
        "UiComponentShowcase/RangeFieldDragUpdate"
    );
    assert_eq!(
        range.edit_action_id.as_str(),
        "UiComponentShowcase/RangeFieldChanged"
    );
    assert_eq!(
        range.commit_action_id.as_str(),
        "UiComponentShowcase/RangeFieldCommitted"
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
        "UiComponentShowcase/ComboBoxOpenPopup"
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
        "UiComponentShowcase/SearchSelectQueryChanged"
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

    let color = nodes
        .iter()
        .find(|node| node.control_id.as_str() == "ColorFieldDemo")
        .expect("component showcase pane should expose ColorFieldDemo");
    assert_eq!(color.value_color, slint::Color::from_rgb_u8(77, 137, 255));
    assert_eq!(
        color.action_id.as_str(),
        "UiComponentShowcase/ColorFieldChanged"
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
        "IconDemo should resolve icon_name into a loaded Slint image"
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
        "SvgIconDemo should resolve source into a loaded Slint SVG image"
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
        "UiComponentShowcase/Vector2FieldChanged"
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
        "UiComponentShowcase/Vector3FieldChanged"
    );

    let vector4 = nodes
        .iter()
        .find(|node| node.control_id.as_str() == "Vector4FieldDemo")
        .expect("component showcase pane should expose Vector4Demo");
    assert_eq!(vector4.vector_components.row_count(), 4);
    assert_eq!(vector4.vector_components.row_data(3), Some(1.0));
    assert_eq!(
        vector4.action_id.as_str(),
        "UiComponentShowcase/Vector4FieldChanged"
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
        "UiComponentShowcase/InspectorSectionToggled"
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
            let expected_action_id = format!("UiComponentShowcase/{suffix}{action_suffix}");
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
        "UiComponentShowcase/ArrayFieldSetElement"
    );
    assert_eq!(
        array_item.remove_action_id.as_str(),
        "UiComponentShowcase/ArrayFieldRemoveElement"
    );
    assert_eq!(array_item.move_up_action_id.as_str(), "");
    assert_eq!(
        array_item.move_down_action_id.as_str(),
        "UiComponentShowcase/ArrayFieldMoveElement"
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
        "UiComponentShowcase/MapFieldSetEntry"
    );
    assert_eq!(
        map_item.key_edit_action_id.as_str(),
        "UiComponentShowcase/MapFieldSetEntry"
    );
    assert_eq!(
        map_item.remove_action_id.as_str(),
        "UiComponentShowcase/MapFieldRemoveEntry"
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
    assert_eq!(checked_item.action_id.as_str(), "Inspect");
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
    assert_eq!(pressed_item.action_id.as_str(), "Duplicate");
    assert_eq!(pressed_item.label.as_str(), "Duplicate");
    assert!(pressed_item.hovered);
    assert!(pressed_item.pressed);
    assert!(!pressed_item.focused);
    let disabled_item = context_menu
        .structured_menu_items
        .row_data(3)
        .expect("ContextActionMenu should expose disabled menu row");
    assert_eq!(disabled_item.action_id.as_str(), "Delete");
    assert_eq!(disabled_item.label.as_str(), "Delete");
    assert_eq!(disabled_item.shortcut.as_str(), "Del");
    assert!(disabled_item.disabled);

    let event_log = nodes
        .iter()
        .find(|node| node.control_id.as_str() == "ComponentShowcaseEventLog")
        .expect("component showcase pane should expose event log node");
    assert!(event_log.text.contains("Registered events"));
}

#[test]
fn component_showcase_pane_uses_supplied_runtime_demo_state() {
    let _guard = crate::tests::support::env_lock()
        .lock()
        .unwrap_or_else(|poison| poison.into_inner());
    let (_fixture, chrome, _model, _ui_asset_panes, _animation_panes) = root_shell_fixture();
    let body_spec = PaneBodySpec::new(
        "editor.window.ui_component_showcase",
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

    let mut runtime = EditorUiHostRuntime::default();
    runtime
        .load_builtin_host_templates()
        .expect("built-in host templates should load");
    let binding = runtime
        .project_document("editor.window.ui_component_showcase")
        .expect("showcase projection should compile")
        .bindings
        .into_iter()
        .find(|binding| binding.binding_id == "UiComponentShowcase/NumberFieldDragUpdate")
        .expect("showcase should expose NumberField drag binding")
        .binding;
    runtime
        .apply_showcase_demo_binding(&binding, UiComponentShowcaseDemoEventInput::DragDelta(5.0))
        .expect("showcase runtime should accept NumberField drag input");
    let combo_open_binding = runtime
        .project_document("editor.window.ui_component_showcase")
        .expect("showcase projection should compile")
        .bindings
        .into_iter()
        .find(|binding| binding.binding_id == "UiComponentShowcase/ComboBoxOpenPopup")
        .expect("showcase should expose ComboBox open binding")
        .binding;
    runtime
        .apply_showcase_demo_binding(&combo_open_binding, UiComponentShowcaseDemoEventInput::None)
        .expect("showcase runtime should accept ComboBox popup input");
    let asset_drop_binding = runtime
        .project_document("editor.window.ui_component_showcase")
        .expect("showcase projection should compile")
        .bindings
        .into_iter()
        .find(|binding| binding.binding_id == "UiComponentShowcase/AssetFieldDropped")
        .expect("showcase should expose AssetField drop binding")
        .binding;
    runtime
        .apply_showcase_demo_binding(
            &asset_drop_binding,
            UiComponentShowcaseDemoEventInput::DropReference {
                payload: UiDragPayload::new(
                    UiDragPayloadKind::Asset,
                    "res://textures/grid.albedo.png",
                )
                .with_source(UiDragSourceMetadata::asset(
                    "browser",
                    "AssetBrowserContentPanel",
                    "asset-uuid-1",
                    "res://textures/grid.albedo.png",
                    "Grid Albedo",
                    "Texture",
                    "png",
                )),
            },
        )
        .expect("showcase runtime should accept AssetField drop input");

    let host_contract_pane =
        super::pane_data_conversion::to_host_contract_component_showcase_pane_from_host_pane_with_runtime(
            &pane,
            host_window::PaneContentSize::new(1080.0, 720.0),
            &runtime,
        );

    let nodes = (0..host_contract_pane.nodes.row_count())
        .filter_map(|row| host_contract_pane.nodes.row_data(row))
        .collect::<Vec<_>>();
    let number = nodes
        .iter()
        .find(|node| node.control_id.as_str() == "NumberFieldDemo")
        .expect("component showcase pane should expose NumberFieldDemo");
    assert_eq!(number.value_text.as_str(), "47");

    let combo_box = nodes
        .iter()
        .find(|node| node.control_id.as_str() == "ComboBoxDemo")
        .expect("component showcase pane should expose ComboBoxDemo");
    assert!(combo_box.popup_open);
    assert_eq!(
        combo_box.action_id.as_str(),
        "UiComponentShowcase/ComboBoxChanged"
    );

    let asset = nodes
        .iter()
        .find(|node| node.control_id.as_str() == "AssetFieldDemo")
        .expect("component showcase pane should expose AssetFieldDemo");
    assert_eq!(asset.value_text.as_str(), "res://textures/grid.albedo.png");
    assert_eq!(asset.drop_source_summary.as_str(), "Texture: Grid Albedo");

    let event_log = nodes
        .iter()
        .find(|node| node.control_id.as_str() == "ComponentShowcaseEventLog")
        .expect("component showcase pane should expose event log node");
    assert!(
        event_log
            .text
            .contains("NumberFieldDemo -> DragDelta.NumberField = 47"),
        "event log should reflect the supplied runtime state: {}",
        event_log.text
    );
}
