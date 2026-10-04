use super::*;

fn rect() -> FrameRect {
    FrameRect {
        x: 0.0,
        y: 0.0,
        width: 260.0,
        height: 48.0,
    }
}

fn node(font_size: f32) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        font_size,
        ..TemplatePaneNodeData::default()
    }
}

#[test]
fn alert_message_frame_uses_the_available_message_band() {
    let node = node(13.0);
    let frame = alert_message_frame(&node, &rect(), 0.0, 260.0)
        .expect("alert message has enough horizontal space")
        .0;

    assert!((frame.width - 260.0).abs() <= 0.01);
}

#[test]
fn alert_message_frame_clamps_to_available_message_space() {
    let node = node(13.0);
    let frame = alert_message_frame(&node, &rect(), 0.0, 44.0)
        .expect("alert message has enough horizontal space")
        .0;

    assert!((frame.width - 44.0).abs() <= 0.01);
}

#[test]
fn tall_alert_message_frame_reserves_a_multiline_content_band() {
    let node = node(13.0);
    let alert = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 260.0,
        height: 88.0,
    };
    let frame = alert_message_frame(&node, &alert, 16.0, 244.0)
        .expect("tall alert has message space")
        .0;
    let line_height = alert_message_line_height(alert_font_size(&node));

    assert!(frame.height >= line_height * 2.0);
    assert!(frame.y >= alert.y);
    assert!(frame.bottom() <= alert.bottom());
}
