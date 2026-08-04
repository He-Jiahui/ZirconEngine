use super::super::super::style_selector::{
    WORKBENCH_TEXT_FIELD_BORDER as FIELD_BORDER,
    WORKBENCH_TEXT_FIELD_DISABLED_BORDER as FIELD_DISABLED_BORDER,
    WORKBENCH_TEXT_FIELD_DISABLED_SURFACE as FIELD_DISABLED_SURFACE,
    WORKBENCH_TEXT_FIELD_DISABLED_TEXT as FIELD_DISABLED_TEXT,
    WORKBENCH_TEXT_FIELD_FOCUSED_BORDER as FIELD_FOCUSED_BORDER,
    WORKBENCH_TEXT_FIELD_FOCUSED_SURFACE as FIELD_FOCUSED_SURFACE,
    WORKBENCH_TEXT_FIELD_STEPPER_DIVIDER as FIELD_STEPPER_DIVIDER,
    WORKBENCH_TEXT_FIELD_SURFACE as FIELD_SURFACE,
};
use super::super::super::template_nodes::paint_template_nodes_for_test;
use super::super::metrics::workbench_field_metrics;
use super::super::push_field_commands;
use super::super::style::field_style;
use super::support::{changed_pixel_count, pixel_at, positioned_field_node};
use crate::ui::layouts::common::model_rc;
use crate::ui::retained_host::host_contract::data::FrameRect;
use crate::ui::retained_host::host_contract::paint_theme::PALETTE;

#[test]
fn workbench_field_paints_surface_border_and_text() {
    let bytes = paint_template_nodes_for_test(
        200,
        48,
        model_rc(vec![positioned_field_node(
            "WorkbenchInputText",
            "Text field",
            12.0,
            8.0,
            170.0,
            32.0,
        )]),
    );

    assert_eq!(pixel_at(&bytes, 200, 170, 24), FIELD_SURFACE);
    assert_eq!(pixel_at(&bytes, 200, 80, 8), FIELD_BORDER);
    assert!(changed_pixel_count(&bytes, 200, 22, 16, 64, 18) > 0);
}

#[test]
fn focused_workbench_field_uses_primary_focused_border() {
    let mut node = positioned_field_node(
        "WorkbenchInputFocused",
        "Focused input",
        12.0,
        8.0,
        170.0,
        32.0,
    );
    node.focused = true;
    let bytes = paint_template_nodes_for_test(200, 48, model_rc(vec![node]));

    assert_eq!(pixel_at(&bytes, 200, 170, 24), FIELD_FOCUSED_SURFACE);
    assert_eq!(pixel_at(&bytes, 200, 80, 8), FIELD_FOCUSED_BORDER);
    assert_eq!(FIELD_FOCUSED_BORDER, PALETTE.focus_ring);
    assert_ne!(FIELD_FOCUSED_BORDER, PALETTE.accent);
}

#[test]
fn disabled_workbench_field_paints_placeholder_tone() {
    let mut node = positioned_field_node("WorkbenchInputDisabled", "", 12.0, 8.0, 170.0, 32.0);
    node.disabled = true;
    let text_color = field_style(&node).text;
    let bytes = paint_template_nodes_for_test(200, 48, model_rc(vec![node]));

    assert_eq!(pixel_at(&bytes, 200, 170, 24), FIELD_DISABLED_SURFACE);
    assert_eq!(pixel_at(&bytes, 200, 80, 8), FIELD_DISABLED_BORDER);
    assert_eq!(text_color, FIELD_DISABLED_TEXT);
    assert!(changed_pixel_count(&bytes, 200, 22, 16, 90, 18) > 0);
}

#[test]
fn stepper_workbench_field_paints_right_arrows() {
    let node = positioned_field_node("WorkbenchInputStepper", "42", 12.0, 8.0, 67.0, 32.0);
    let stepper_divider = field_style(&node).stepper_divider;
    let bytes = paint_template_nodes_for_test(112, 48, model_rc(vec![node]));

    assert_eq!(stepper_divider, FIELD_STEPPER_DIVIDER);
    assert_eq!(pixel_at(&bytes, 112, 61, 16), stepper_divider);
    assert!(changed_pixel_count(&bytes, 112, 64, 15, 12, 20) > 0);
}

#[test]
fn number_field_stepper_property_paints_right_arrows_without_the_legacy_control_id() {
    let mut node = positioned_field_node("WorkbenchNumberFieldDemo", "42", 12.0, 8.0, 67.0, 32.0);
    node.component_role = "number-field".into();
    node.layout_stepper = true;
    let stepper_divider = field_style(&node).stepper_divider;
    let bytes = paint_template_nodes_for_test(112, 48, model_rc(vec![node]));

    assert_eq!(pixel_at(&bytes, 112, 61, 16), stepper_divider);
    assert!(changed_pixel_count(&bytes, 112, 64, 15, 12, 20) > 0);
}

#[test]
fn stepper_workbench_field_honors_declared_layout_offset() {
    let mut node = positioned_field_node("WorkbenchInputStepper", "42", 12.0, 8.0, 67.0, 32.0);
    node.layout_offset_x = 5.0;
    node.layout_offset_y = 6.0;
    let bytes = paint_template_nodes_for_test(128, 72, model_rc(vec![node]));

    let stepper_divider = field_style(&positioned_field_node(
        "WorkbenchInputStepper",
        "42",
        12.0,
        8.0,
        67.0,
        32.0,
    ))
    .stepper_divider;

    assert_eq!(pixel_at(&bytes, 128, 66, 20), stepper_divider);
    assert_eq!(pixel_at(&bytes, 128, 14, 24), [0, 0, 0, 255]);
}

#[test]
fn search_workbench_field_paints_left_icon_before_placeholder_text() {
    let mut search = positioned_field_node("SearchEdited", "", 12.0, 10.0, 184.0, 28.0);
    search.text = "Search".into();
    let bytes = paint_template_nodes_for_test(220, 48, model_rc(vec![search]));

    assert_ne!(pixel_at(&bytes, 220, 28, 25), [0, 0, 0, 255]);
    assert_ne!(pixel_at(&bytes, 220, 28, 25), FIELD_SURFACE);
    assert!(changed_pixel_count(&bytes, 220, 40, 17, 48, 15) > 0);
}

#[test]
fn search_workbench_field_prefers_shell_search_asset_pixels() {
    let mut search = positioned_field_node("SearchEdited", "", 12.0, 10.0, 184.0, 28.0);
    search.text = "Search".into();
    let rect = FrameRect {
        x: 12.0,
        y: 10.0,
        width: 184.0,
        height: 28.0,
    };
    let mut commands = Vec::new();

    assert!(push_field_commands(
        &mut commands,
        &search,
        &rect,
        &rect,
        0,
        1.0,
    ));

    let icon_commands = commands
        .iter()
        .filter(|command| command.image_pixels.is_some())
        .collect::<Vec<_>>();
    assert_eq!(
        icon_commands.len(),
        1,
        "search fields should render the glass glyph through the shared shell SVG asset"
    );
    let icon = icon_commands[0];
    let metrics = workbench_field_metrics();
    assert_eq!(icon.frame.width, metrics.search_icon_size);
    assert_eq!(icon.frame.height, metrics.search_icon_size);
    assert!(
        icon.image_pixels
            .as_ref()
            .map(|image| !image.resource_key.starts_with("missing-icon:"))
            .unwrap_or(false)
    );
}

#[test]
fn search_workbench_field_paints_a_clear_action_only_for_a_nonempty_query() {
    let mut search = positioned_field_node("SearchEdited", "material", 12.0, 10.0, 184.0, 28.0);
    search.component_role = "search-field".into();
    search.has_clear_action = true;
    let bytes = paint_template_nodes_for_test(220, 48, model_rc(vec![search]));

    let mut empty_search = positioned_field_node("SearchEdited", "", 12.0, 10.0, 184.0, 28.0);
    empty_search.component_role = "search-field".into();
    empty_search.has_clear_action = true;
    let empty_bytes = paint_template_nodes_for_test(220, 48, model_rc(vec![empty_search]));

    assert!(
        bytes
            .chunks_exact(4)
            .zip(empty_bytes.chunks_exact(4))
            .enumerate()
            .any(|(index, (query, empty))| {
                let x = index % 220;
                let y = index / 220;
                (166..188).contains(&x) && (14..34).contains(&y) && query != empty
            }),
        "a nonempty search should paint its trailing clear affordance"
    );
}
