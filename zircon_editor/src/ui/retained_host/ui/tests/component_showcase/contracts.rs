use super::*;

#[test]
fn component_showcase_template_metadata_is_owned_by_rust_contracts() {
    let template_nodes = source("src/ui/retained_host/host_contract/data/template_nodes.rs");
    let showcase_asset = component_showcase_contract_source();

    for required in [
        "pub component_category: SharedString",
        "pub component_layout_role: SharedString",
        "pub component_variant: SharedString",
        "pub label_text: SharedString",
        "pub label_color: Color",
        "pub label_brightness: f32",
        "pub layout_offset_x: f32",
        "pub layout_offset_y: f32",
        "pub layout_icon_size: f32",
        "pub layout_content_offset_x: f32",
        "pub layout_content_offset_y: f32",
        "pub layout_first_cell_offset_x: f32",
        "pub layout_second_cell_offset_x: f32",
        "pub layout_third_cell_offset_x: f32",
        "pub layout_fourth_cell_offset_x: f32",
        "pub value_number: f32",
        "pub value_percent: f32",
        "pub value_color: Color",
        "pub icon_color: Color",
        "pub icon_stroke_width: f32",
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
        "pub has_clip_frame: bool",
        "pub clip_frame: TemplateNodeFrameData",
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
        "SliderDemo",
        "RangeSliderDemo",
        "TabDemo",
        "TabStripDemo",
        "ColorFieldDemo",
        "SkeletonDemo",
        "Vector3FieldDemo",
        "ContextMenuDemo",
        "DropdownPopupDemo",
        "DialogDemo",
        "ConfirmDialogDemo",
        "CommandPaletteDemo",
        "NotificationCenterDemo",
        "ui_component_showcase.number_field_drag_update",
        "ui_component_showcase.slider_drag_update",
        "ui_component_showcase.slider_changed",
        "ui_component_showcase.range_slider_drag_update",
        "ui_component_showcase.range_slider_changed",
        "ui_component_showcase.tab_changed",
        "ui_component_showcase.tab_strip_changed",
        "ui_component_showcase.input_field_changed",
    ] {
        assert!(
            showcase_asset.contains(required),
            "component showcase TOML missing `{required}`"
        );
    }
}

#[test]
fn component_showcase_option_and_action_callbacks_are_rust_wired() {
    let template_nodes = source("src/ui/retained_host/host_contract/data/template_nodes.rs");
    let callbacks = source("src/ui/retained_host/host_contract/globals.rs");
    let wiring = source("src/ui/retained_host/app/callback_wiring.rs");
    let pane_actions = source("src/ui/retained_host/app/pane_surface_actions.rs");
    let showcase_event_inputs = source("src/ui/retained_host/app/showcase_event_inputs.rs");

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
    for required in [
        "AssetFieldClear",
        "AssetFieldLocate",
        "AssetFieldOpen",
        "asset_field_drop_hovered",
        "asset_field_active_drag_target",
        "list_row_hovered",
        "list_row_pressed",
        "array_field_changed",
        "map_field_changed",
        "range_field_drag_update",
    ] {
        assert!(
            showcase_event_inputs.contains(required),
            "showcase action input missing `{required}`"
        );
    }
}
