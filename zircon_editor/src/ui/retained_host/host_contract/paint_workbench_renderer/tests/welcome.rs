use super::*;

#[test]
fn authoritative_layout_keeps_a_collapsed_recent_panel_absent() {
    let body = FrameRect {
        x: 10.0,
        y: 20.0,
        width: 640.0,
        height: 360.0,
    };
    let layout = WelcomePaneLayoutData {
        has_nodes: true,
        outer_panel: Some(FrameRect {
            x: 16.0,
            y: 12.0,
            width: 608.0,
            height: 336.0,
        }),
        main_panel: Some(FrameRect {
            x: 16.0,
            y: 12.0,
            width: 608.0,
            height: 336.0,
        }),
        ..WelcomePaneLayoutData::default()
    };

    let (recent, main) = resolve_welcome_panel_frames(&layout, &body);

    assert_eq!(recent, None);
    assert_eq!(main.expect("project task panel").width, 608.0);
}

#[test]
fn legacy_layout_without_projected_nodes_retains_panel_fallbacks() {
    let body = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 900.0,
        height: 620.0,
    };

    let (recent, main) = resolve_welcome_panel_frames(&WelcomePaneLayoutData::default(), &body);

    let recent = recent.expect("legacy recent panel");
    let main = main.expect("legacy main panel");
    assert!(recent.width >= 220.0);
    assert_eq!(main.x, recent.x + recent.width);
    assert_eq!(main.x + main.width, body.width - WELCOME_COLUMN_INSET);
}
