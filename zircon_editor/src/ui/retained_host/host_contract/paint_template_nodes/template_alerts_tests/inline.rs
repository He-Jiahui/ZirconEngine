use super::super::super::super::paint_text::HostTextLayoutPolicy;
use super::super::super::super::paint_theme::PALETTE;
use super::super::super::style_selector::{
    WORKBENCH_ALERT_INFO_SURFACE, WORKBENCH_ALERT_WARNING_SURFACE,
};
use super::super::super::template_nodes::paint_template_nodes_for_test;
use super::support::{changed_pixel_count, pixel_at, positioned_alert_node};
use crate::ui::layouts::common::model_rc;

#[test]
fn workbench_info_alert_paints_tinted_surface_icon_and_label() {
    let bytes = paint_template_nodes_for_test(
        192,
        48,
        model_rc(vec![positioned_alert_node(
            "WorkbenchInfoAlert",
            "Info Alert",
            "info",
            8.0,
            8.0,
            160.0,
            32.0,
        )]),
    );

    assert_eq!(pixel_at(&bytes, 192, 150, 24), WORKBENCH_ALERT_INFO_SURFACE);
    assert_eq!(pixel_at(&bytes, 192, 25, 24), PALETTE.info);
    assert!(changed_pixel_count(&bytes, 192, 38, 16, 62, 18) > 0);
    assert_eq!(pixel_at(&bytes, 192, 176, 24), [0, 0, 0, 255]);
}

#[test]
fn workbench_warning_alert_uses_warning_tone() {
    let bytes = paint_template_nodes_for_test(
        192,
        48,
        model_rc(vec![positioned_alert_node(
            "WorkbenchWarningAlert",
            "Warning Alert",
            "warning",
            8.0,
            8.0,
            160.0,
            32.0,
        )]),
    );

    assert_eq!(
        pixel_at(&bytes, 192, 150, 24),
        WORKBENCH_ALERT_WARNING_SURFACE
    );
    assert_eq!(pixel_at(&bytes, 192, 27, 18), PALETTE.warning);
    assert!(changed_pixel_count(&bytes, 192, 38, 16, 84, 18) > 0);
}

#[test]
fn tall_inline_alert_requests_runtime_word_wrap_for_its_content_band() {
    let height = 76.0;
    let node = positioned_alert_node(
        "WorkbenchInfoAlert",
        "Imported assets require validation before opening this project.",
        "info",
        8.0,
        8.0,
        220.0,
        height,
    );
    let rect = crate::ui::retained_host::host_contract::data::FrameRect {
        x: 8.0,
        y: 8.0,
        width: 220.0,
        height,
    };
    let mut commands = Vec::new();

    assert!(super::super::push_alert_commands(
        &mut commands,
        &node,
        &rect,
        &rect,
        1,
        1.0,
    ));

    let text = commands
        .iter()
        .find(|command| command.text.is_some())
        .expect("inline alert content command");
    assert_eq!(text.text_layout_policy, HostTextLayoutPolicy::WordWrap);
    let metrics = super::super::template_alerts::layout::alert_metrics();
    assert!(text.frame.height >= metrics.line_height * 2.0);
}
