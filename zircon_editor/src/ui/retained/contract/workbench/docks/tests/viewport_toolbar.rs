use super::*;
use crate::ui::retained_host::host_contract::paint_frame::HostRecordedPaintKind;
use crate::ui::retained_host::host_contract::paint_theme::{METRICS, PALETTE};

#[test]
fn projected_toolbar_paints_all_16_source_icons_at_100_and_150_percent() {
    let _guard = crate::tests::support::env_lock().lock().unwrap();
    let mut bridge =
        crate::ui::retained_host::callback_dispatch::BuiltinViewportToolbarTemplateBridge::new()
            .unwrap();
    let repo = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap();
    for scale in [1.0, 1.5] {
        bridge.admit_layout_context(scale, 1);
        let mut pane = PaneData {
            kind: "Scene".into(),
            show_toolbar: true,
            ..Default::default()
        };
        pane.viewport.toolbar_surface_key = "document:icons".into();
        pane.viewport.toolbar_template_nodes = bridge
            .paint_nodes_for_size(zircon_runtime_interface::ui::layout::UiSize::new(
                640.0 * scale,
                28.0 * scale,
            ))
            .unwrap();
        let toolbar = FrameRect {
            x: 120.0,
            y: 80.0,
            width: 640.0 * scale,
            height: 28.0 * scale,
        };
        let evidence = crate::ui::retained_host::host_contract::paint_template_nodes::PaintEvidenceScope::begin(repo, 96.0 * scale).unwrap();
        let mut frame = HostRgbaFrame::recording_only(1200, 200);
        draw_viewport_toolbar(&mut frame, &pane, &toolbar, &toolbar);
        let commands = frame.into_recorded_commands();
        let audit = evidence.finish().unwrap();
        assert_eq!(
            audit["assetAudit"]["complete"].as_bool(),
            Some(true),
            "{audit}"
        );
        let resources = audit["assetAudit"]["resources"].as_array().unwrap();
        let controls: Vec<_> = pane
            .viewport
            .toolbar_template_nodes
            .iter()
            .filter(|node| !node.icon_name.is_empty())
            .collect();
        assert_eq!(controls.len(), 16);
        for node in controls {
            assert!(
                resources
                    .iter()
                    .any(|resource| resource["sourceNodeId"].as_str()
                        == Some(node.source_node_id.as_str())
                        && resource["loaded"].as_bool() == Some(true)),
                "icon {} must be loaded with its source owner",
                node.control_id
            );
            assert!(
                commands.iter().any(|command| matches!(
                    &command.kind,
                    HostRecordedPaintKind::Image { .. }
                ) && command.frame.x >= toolbar.x + node.frame.x
                    && command.frame.y >= toolbar.y + node.frame.y
                    && command.frame.x + command.frame.width
                        <= toolbar.x + node.frame.x + node.frame.width + 0.01
                    && command.frame.y + command.frame.height
                        <= toolbar.y + node.frame.y + node.frame.height + 0.01),
                "icon {} must contribute real ink inside its measured control",
                node.control_id
            );
        }
    }
}

#[test]
fn source_toolbar_hover_focus_and_mode_change_share_live_painter_state() {
    let _guard = crate::tests::support::env_lock().lock().unwrap();
    let mut bridge =
        crate::ui::retained_host::callback_dispatch::BuiltinViewportToolbarTemplateBridge::new()
            .unwrap();
    let mut pane = PaneData {
        kind: "Scene".into(),
        show_toolbar: true,
        ..Default::default()
    };
    pane.viewport.toolbar_surface_key = "document:state".into();
    pane.viewport.mode = "Select".into();
    pane.viewport.toolbar_template_nodes = bridge
        .paint_nodes_for_size(zircon_runtime_interface::ui::layout::UiSize::new(
            640.0, 28.0,
        ))
        .unwrap();
    let toolbar = FrameRect {
        x: 40.0,
        y: 70.0,
        width: 640.0,
        height: 28.0,
    };
    let paint = |pane: &PaneData, interaction| {
        let mut frame = HostRgbaFrame::recording_only(800, 200);
        frame.set_pane_interaction_state(std::sync::Arc::new(interaction));
        draw_viewport_toolbar(&mut frame, pane, &toolbar, &toolbar);
        frame.into_recorded_commands()
    };
    let normal = paint(&pane, Default::default());
    let interaction = crate::ui::retained_host::host_contract::data::HostPaneInteractionStateData {
        hovered_template_control_id: "document:state::ActivateSceneMode".into(),
        focused_template_control_id: "document:state::ActivateSceneMode".into(),
        template_focus_visible: true,
        ..Default::default()
    };
    let hovered = paint(&pane, interaction.clone());
    assert_ne!(
        normal, hovered,
        "source-owned icon must react to shared hover/focus state"
    );
    pane.viewport.mode = "Transform.Move".into();
    let changed = paint(&pane, Default::default());
    assert_ne!(
        normal, changed,
        "live selected mode must alter shared source-owned paint"
    );
    let nodes = crate::ui::retained_host::host_contract::viewport_chrome_state::live_toolbar_nodes(
        &pane.viewport,
        Some(&interaction),
    );
    let mode = nodes
        .iter()
        .find(|node| node.control_id.as_str() == "document:state::ActivateSceneMode")
        .unwrap();
    assert!(mode.selected && mode.focused && mode.focus_visible);
    let other_leaf = crate::ui::retained_host::host_contract::data::SceneViewportChromeData {
        toolbar_surface_key: "document:other".into(),
        ..pane.viewport.clone()
    };
    let nodes = crate::ui::retained_host::host_contract::viewport_chrome_state::live_toolbar_nodes(
        &other_leaf,
        Some(&interaction),
    );
    assert!(
        !nodes
            .iter()
            .find(|node| node.control_id.as_str() == "document:other::ActivateSceneMode")
            .unwrap()
            .focused
    );
}

#[test]
fn toolbar_palette_uses_runtime_surface_border_and_text_roles() {
    let mut palette = PALETTE;
    palette.surface = [1, 2, 3, 255];
    palette.border = [4, 5, 6, 255];
    palette.text_muted = [7, 8, 9, 255];

    assert_eq!(
        viewport_toolbar_palette(palette),
        ViewportToolbarPalette {
            surface: [1, 2, 3, 255],
            border: [4, 5, 6, 255],
            text: [7, 8, 9, 255],
        }
    );
}

#[test]
fn scene_mode_protocol_symbols_project_to_user_facing_labels() {
    assert_eq!(scene_mode_label("Select"), "Select");
    assert_eq!(scene_mode_label("Transform.Rotate"), "Rotate");
    assert_eq!(scene_mode_label("Custom:terrain.paint"), "terrain.paint");
}

#[test]
fn toolbar_slots_follow_natural_text_widths_when_space_allows() {
    let toolbar = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 640.0,
        height: 30.0,
    };
    let slots = viewport_toolbar_label_slots(
        &toolbar,
        ["Move", "World", "Center", "Lit", "Grid"],
        METRICS,
    );

    assert!(slots[0].width > 0.0);
    assert!(slots[0].width < slots[1].width);
    assert!(slots
        .windows(2)
        .all(|pair| pair[0].x + pair[0].width <= pair[1].x));
    assert!(slots[4].x + slots[4].width <= toolbar.x + toolbar.width);
}

#[test]
fn toolbar_slots_compact_evenly_inside_a_narrow_toolbar() {
    let toolbar = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 120.0,
        height: 30.0,
    };
    let slots = viewport_toolbar_label_slots(
        &toolbar,
        [
            "Translate Long Tool Name",
            "World Coordinates",
            "Selection Center",
            "Lit With Shadows",
            "Visible And Snap",
        ],
        METRICS,
    );

    assert!(slots
        .windows(2)
        .all(|pair| pair[0].x + pair[0].width <= pair[1].x));
    assert!((slots[0].width - slots[1].width).abs() < f32::EPSILON);
    assert!(slots[4].x + slots[4].width <= toolbar.x + toolbar.width);
}

#[test]
fn toolbar_labels_use_finite_runtime_text_slots_with_ellipsis() {
    let toolbar = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 120.0,
        height: 30.0,
    };
    let mut frame = HostRgbaFrame::recording_only(128, 40);

    draw_viewport_toolbar_labels(
        &mut frame,
        [
            "Translate Long Tool Name",
            "World Coordinates",
            "Selection Center",
            "Lit With Shadows",
            "Visible And Snap",
        ],
        &toolbar,
        &toolbar,
        viewport_toolbar_palette(PALETTE),
        METRICS,
    );

    let texts = frame
        .into_recorded_commands()
        .into_iter()
        .filter_map(|command| match command.kind {
            HostRecordedPaintKind::Text { text, .. } => Some((text, command.frame)),
            _ => None,
        })
        .collect::<Vec<_>>();

    assert_eq!(texts.len(), 5);
    assert!(texts.iter().all(|(text, _)| text.ends_with('\u{2026}')));
    assert!(texts.iter().all(|(_, frame)| {
        frame.x >= toolbar.x && frame.x + frame.width <= toolbar.x + toolbar.width
    }));
}
