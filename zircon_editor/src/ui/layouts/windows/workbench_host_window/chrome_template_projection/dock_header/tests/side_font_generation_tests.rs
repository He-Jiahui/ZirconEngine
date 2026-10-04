use super::*;

fn physical_metrics(scale: f32) -> HostControlMetrics {
    let _scope = crate::ui::retained_host::enter_host_paint_theme_scope(
        crate::ui::retained_host::host_paint_theme_snapshot_at_scale_for_test(scale),
    );
    current_host_metrics()
}

fn sidebar_tabs() -> ModelRc<TabData> {
    model_rc(
        [("Outliner", true), ("Assets", false), ("Plugins", false)]
            .into_iter()
            .enumerate()
            .map(|(row, (title, active))| TabData {
                id: format!("tool-{row}").into(),
                title: title.into(),
                active,
                closeable: true,
                ..Default::default()
            })
            .collect(),
    )
}

#[test]
fn side_header_active_titles_fit_the_physical_button_budget_at_both_scales() {
    for scale in [1.0, 1.5] {
        let metrics = physical_metrics(scale);
        for tabs in [
            sidebar_tabs(),
            model_rc(vec![TabData {
                title: "Details".into(),
                active: true,
                closeable: true,
                ..Default::default()
            }]),
        ] {
            let nodes = side_dock_header_nodes_with_metrics(
                &tabs,
                320.0 * scale,
                30.0 * scale,
                [7; 3],
                scale,
                metrics,
            );
            let active = nodes
                .iter()
                .find(|node| node.control_id.as_str() == "DockTab0")
                .unwrap();
            assert_eq!(active.text, tabs.get(0).unwrap().title);
            assert_eq!(active.font_size, DOCUMENT_TAB_TITLE_FONT_SIZE * scale);
            let title = measure_runtime_text_width_with_style(
                active.text.as_str(),
                active.font_size,
                UiTextRunPaintStyle {
                    strong: true,
                    ..Default::default()
                },
            );
            let glyph = (metrics.row_height - metrics.gap_l).max(1.0) + metrics.button_icon_gap;
            let reserve = (crate::ui::workbench::document_tabs::DOCUMENT_TAB_CLOSE_EXTENT
                + crate::ui::workbench::document_tabs::DOCUMENT_TAB_CLOSE_RIGHT_INSET)
                * scale;
            assert!(
                active.frame.width + 0.01
                    >= title
                        + glyph
                        + metrics.button_pad_x * 2.0
                        + metrics.text_clip_guard
                        + reserve
            );
            let frames = dock_tab_frames(&nodes, &tabs);
            assert_eq!(frames.row_count(), tabs.row_count());
            for frame in frames.iter() {
                assert!(
                    frame.frame.x >= 0.0
                        && frame.frame.x + frame.frame.width <= 320.0 * scale + 0.01
                );
            }
        }
    }
}

#[test]
fn positive_physical_header_height_contains_tabs_close_controls_and_text_at_high_scale() {
    let scale = 1.5;
    let metrics = physical_metrics(scale);
    let tabs = sidebar_tabs();
    let nodes = side_dock_header_nodes_with_metrics(&tabs, 480.0, 31.0, [11; 3], scale, metrics);
    let header = nodes
        .iter()
        .find(|node| node.control_id.as_str() == DOCK_HEADER_BAR_CONTROL_ID)
        .unwrap();
    assert_eq!(header.frame.height, 31.0);
    for node in nodes.iter() {
        assert!(node.frame.y >= 0.0 && node.frame.y + node.frame.height <= 31.0 + 0.01);
    }
    let active = nodes
        .iter()
        .find(|node| node.control_id.as_str() == "DockTab0")
        .unwrap();
    assert_eq!(active.font_size, DOCUMENT_TAB_TITLE_FONT_SIZE * scale);
    let line_height = (active.font_size * metrics.line_height_ratio)
        .round()
        .max(active.font_size.ceil());
    assert!(line_height <= active.frame.height);
    assert!((metrics.row_height - metrics.gap_l).max(1.0) <= active.frame.height);
    let close = nodes
        .iter()
        .find(|node| node.control_id.as_str() == "DockTabClose0")
        .unwrap();
    assert!((metrics.row_height - metrics.gap_l).max(1.0) <= close.frame.height);
    // The existing native body starts after this published band; no control
    // may expand into the Search row at physical y31.
    assert_eq!(header.frame.y + header.frame.height, 31.0);
}

#[test]
fn narrow_side_header_keeps_hidden_tools_in_the_existing_overflow_model() {
    let tabs = sidebar_tabs();
    let nodes =
        side_dock_header_nodes_with_metrics(&tabs, 180.0, 45.0, [8; 3], 1.5, physical_metrics(1.5));
    let overflow = nodes
        .iter()
        .find(|node| node.control_id.as_str() == DOCK_TAB_OVERFLOW_CONTROL_ID)
        .expect("narrow real tab model uses overflow");
    assert!(overflow.frame.width > 0.0 && overflow.frame.x + overflow.frame.width <= 180.0 + 0.01);
    assert_eq!(tabs.row_count(), 3);
    assert_eq!(tabs.get(1).unwrap().title.as_str(), "Assets");
    assert_eq!(tabs.get(2).unwrap().title.as_str(), "Plugins");
}

#[test]
fn side_header_cache_tracks_scale_and_paint_metrics_without_mutating_published_frames() {
    clear_side_dock_header_projection_cache_for_tests();
    let tabs = sidebar_tabs();
    let metrics = physical_metrics(1.0);
    let first = side_dock_header_nodes_with_metrics(&tabs, 480.0, 45.0, [9; 3], 1.0, metrics);
    let old_frame = first.get(1).unwrap().frame.clone();
    let unchanged = side_dock_header_nodes_with_metrics(&tabs, 480.0, 45.0, [9; 3], 1.0, metrics);
    assert!(first.shares_values_with(&unchanged));
    let scaled =
        side_dock_header_nodes_with_metrics(&tabs, 480.0, 45.0, [9; 3], 1.5, physical_metrics(1.5));
    assert!(!first.shares_values_with(&scaled));
    let mut changed_metrics = physical_metrics(1.5);
    changed_metrics.button_pad_x += 3.0;
    let changed =
        side_dock_header_nodes_with_metrics(&tabs, 480.0, 45.0, [9; 3], 1.5, changed_metrics);
    assert!(!scaled.shares_values_with(&changed));
    assert_eq!(first.get(1).unwrap().frame, old_frame);
    let font_changed =
        side_dock_header_nodes_with_metrics(&tabs, 480.0, 45.0, [10; 3], 1.5, changed_metrics);
    assert!(!changed.shares_values_with(&font_changed));
}

#[test]
fn side_dock_reprojects_tabs_when_resolved_font_generation_changes() {
    clear_side_dock_header_projection_cache_for_tests();
    let tabs = model_rc(vec![TabData {
        id: "editor.hierarchy".into(),
        slot: "left".into(),
        title: "Hierarchy".into(),
        icon_key: "layers".into(),
        active: true,
        closeable: false,
    }]);

    let first = side_dock_header_nodes_with_text_generation(&tabs, 320.0, 31.0, [1; 3]);
    let unchanged = side_dock_header_nodes_with_text_generation(&tabs, 320.0, 31.0, [1; 3]);
    let resolved = side_dock_header_nodes_with_text_generation(&tabs, 320.0, 31.0, [2; 3]);

    assert!(first.shares_values_with(&unchanged));
    assert!(!first.shares_values_with(&resolved));
    assert_eq!(side_dock_header_projection_builds_for_tests(), 2);
}

#[test]
fn unresolved_font_measure_preserves_first_frame_side_tab_width() {
    let controls = EditorControlTokens::workbench_dense();
    let density = EditorDensityTokens::workbench_dense();
    let unresolved =
        side_dock_tab_preferred_width_from_measured("Hierarchy", 0.0, controls, density);
    let resolved =
        side_dock_tab_preferred_width_from_measured("Hierarchy", 88.0, controls, density);

    assert!(unresolved > controls.default_height * 3.0 + density.gap_medium * 2.0);
    assert_eq!(
        resolved,
        88.0 + controls.default_height * 0.5 + density.gap_small + density.gap_medium * 2.0
    );
}
