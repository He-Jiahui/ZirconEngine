use super::*;

#[test]
fn inline_rename_focus_only_replaces_the_matching_row_text() {
    let focus = HostTextInputFocusData {
        control_id: HIERARCHY_INLINE_RENAME_CONTROL_ID.into(),
        dispatch_kind: "hierarchy_inline_rename:7".into(),
        value_text: "Renamed".into(),
        ..HostTextInputFocusData::default()
    };
    let selected = SceneNodeData {
        id: "7".into(),
        selected: true,
        ..SceneNodeData::default()
    };
    let other_selected = SceneNodeData {
        id: "8".into(),
        selected: true,
        ..SceneNodeData::default()
    };

    assert_eq!(
        inline_hierarchy_rename_value(&selected, Some(&focus)),
        Some("Renamed")
    );
    assert_eq!(
        inline_hierarchy_rename_value(&other_selected, Some(&focus)),
        None
    );
}

#[test]
fn visible_row_range_is_bounded_by_the_clipped_viewport() {
    let viewport = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 240.0,
        height: 100.0,
    };
    let metrics = hierarchy_row_metrics_from_host_metrics(
        crate::ui::retained_host::host_contract::paint_theme::METRICS,
    );
    let range = visible_hierarchy_row_range(&viewport, &viewport, 560.0, 10_000, metrics);
    let maximum_rows =
        (viewport.height / (metrics.row_height + metrics.row_gap)).ceil() as usize + 2;

    assert!(range.start > 0);
    assert!(range.end < 10_000);
    assert!(range.len() <= maximum_rows);
}

#[test]
fn visible_row_range_is_empty_for_an_empty_clip() {
    let viewport = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 240.0,
        height: 100.0,
    };
    let empty_clip = FrameRect {
        height: 0.0,
        ..viewport.clone()
    };

    assert!(visible_hierarchy_row_range(
        &viewport,
        &empty_clip,
        0.0,
        10_000,
        hierarchy_row_metrics_from_host_metrics(
            crate::ui::retained_host::host_contract::paint_theme::METRICS,
        ),
    )
    .is_empty());
}
