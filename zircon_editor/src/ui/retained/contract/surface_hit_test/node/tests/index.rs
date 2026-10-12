use crate::ui::retained_host::host_contract::data::{
    FrameRect, HostWindowPresentationData, PaneData, TemplateNodeFrameData, TemplatePaneNodeData,
};
use zircon_runtime_interface::ui::layout::UiSize;

use super::super::surface_frame_builder::{
    reset_template_surface_frame_build_count, template_surface_frame_build_count,
};
use super::super::{
    hit_test_pane_template_node, hit_test_workbench_window_template_node_with_index,
    rebuild_pane_template_hit_artifacts, HostWorkbenchHitIndex,
};
use super::support::{hit_test_workbench_window_template_node, model};

#[test]
fn pane_hit_test_skips_the_full_popup_scan_when_no_popup_is_open() {
    let nodes = (0..10_000)
        .map(|row| TemplatePaneNodeData {
            node_id: format!("node-{row}").into(),
            control_id: format!("Control{row}").into(),
            action_id: format!("action.{row}").into(),
            frame: TemplateNodeFrameData {
                x: 8.0,
                y: row as f32 * 28.0,
                width: 120.0,
                height: 20.0,
            },
            ..TemplatePaneNodeData::default()
        })
        .collect();
    let mut pane = PaneData {
        id: "editor.scale#1".into(),
        kind: "TemplateV2".into(),
        template_v2: crate::ui::retained_host::host_contract::data::TemplateV2PaneData {
            nodes: model(nodes),
        },
        ..PaneData::default()
    };
    rebuild_pane_template_hit_artifacts(&mut pane, UiSize::new(160.0, 280_000.0));
    let index = pane
        .body_template_hit_index
        .as_ref()
        .expect("pane hit index should be built")
        .clone();
    let body = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 160.0,
        height: 280_000.0,
    };

    let hit = hit_test_pane_template_node(&pane, &body, 16.0, 279_980.0, 0.0)
        .expect("the final pane control should remain hit-testable");

    assert_eq!(hit.control_id.as_str(), "Control9999");
    assert_eq!(index.query_count_for_test(), 1);
    assert_eq!(index.last_popup_candidate_visit_count_for_test(), 0);
}

#[test]
fn workbench_hit_test_does_not_rebuild_a_template_surface() {
    let presentation = HostWindowPresentationData {
        workbench_window_nodes: model(vec![TemplatePaneNodeData {
            node_id: "button".into(),
            control_id: "WorkbenchButton".into(),
            role: "Button".into(),
            action_id: "workbench.button.click".into(),
            frame: TemplateNodeFrameData {
                x: 10.0,
                y: 20.0,
                width: 120.0,
                height: 32.0,
            },
            ..TemplatePaneNodeData::default()
        }]),
        ..HostWindowPresentationData::default()
    };
    reset_template_surface_frame_build_count();
    let index = HostWorkbenchHitIndex::from_presentation(&presentation);

    for _ in 0..1_000 {
        let hit =
            hit_test_workbench_window_template_node_with_index(&presentation, &index, 24.0, 30.0)
                .expect("workbench button should remain hit-testable");
        assert_eq!(hit.control_id.as_str(), "WorkbenchButton");
    }

    assert_eq!(
        template_surface_frame_build_count(),
        0,
        "pointer routing must consume committed node geometry without rebuilding a UiSurface"
    );
    assert_eq!(index.last_candidate_visit_count_for_test(), 1);
}

#[test]
fn workbench_paint_index_limits_single_region_damage_to_nearby_rows() {
    let nodes = (0..100)
        .map(|row| TemplatePaneNodeData {
            node_id: format!("row-{row}"),
            frame: TemplateNodeFrameData {
                x: 0.0,
                y: row as f32 * 100.0,
                width: 24.0,
                height: 20.0,
            },
            ..TemplatePaneNodeData::default()
        })
        .collect();
    let presentation = HostWindowPresentationData {
        workbench_window_nodes: model(nodes),
        ..HostWindowPresentationData::default()
    };
    let index = HostWorkbenchHitIndex::from_presentation(&presentation);

    let rows = index.paint_rows_for_clip(&FrameRect {
        x: 0.0,
        y: 5_000.0,
        width: 24.0,
        height: 20.0,
    });

    assert!(rows.contains(&50));
    assert!(rows.len() <= 2, "single-cell damage visited {rows:?}");
}

#[test]
fn workbench_paint_index_returns_candidates_in_stable_z_order() {
    let presentation = HostWindowPresentationData {
        workbench_window_nodes: model(vec![
            TemplatePaneNodeData {
                node_id: "front".into(),
                z_index: 2,
                frame: TemplateNodeFrameData {
                    x: 0.0,
                    y: 0.0,
                    width: 24.0,
                    height: 20.0,
                },
                ..TemplatePaneNodeData::default()
            },
            TemplatePaneNodeData {
                node_id: "back".into(),
                z_index: 0,
                frame: TemplateNodeFrameData {
                    x: 0.0,
                    y: 0.0,
                    width: 24.0,
                    height: 20.0,
                },
                ..TemplatePaneNodeData::default()
            },
            TemplatePaneNodeData {
                node_id: "middle".into(),
                z_index: 1,
                frame: TemplateNodeFrameData {
                    x: 0.0,
                    y: 0.0,
                    width: 24.0,
                    height: 20.0,
                },
                ..TemplatePaneNodeData::default()
            },
        ]),
        ..HostWindowPresentationData::default()
    };
    let index = HostWorkbenchHitIndex::from_presentation(&presentation);

    assert_eq!(
        index.paint_rows_for_clip(&FrameRect {
            x: 0.0,
            y: 0.0,
            width: 24.0,
            height: 20.0,
        }),
        vec![1, 2, 0]
    );
}

#[test]
fn workbench_hit_test_preserves_reverse_paint_order_and_clip_frames() {
    let presentation = HostWindowPresentationData {
        workbench_window_nodes: model(vec![
            TemplatePaneNodeData {
                node_id: "back".into(),
                control_id: "BackButton".into(),
                action_id: "back.click".into(),
                frame: TemplateNodeFrameData {
                    x: 10.0,
                    y: 10.0,
                    width: 100.0,
                    height: 80.0,
                },
                ..TemplatePaneNodeData::default()
            },
            TemplatePaneNodeData {
                node_id: "front".into(),
                control_id: "FrontButton".into(),
                action_id: "front.click".into(),
                frame: TemplateNodeFrameData {
                    x: 10.0,
                    y: 10.0,
                    width: 100.0,
                    height: 80.0,
                },
                has_clip_frame: true,
                clip_frame: TemplateNodeFrameData {
                    x: 50.0,
                    y: 10.0,
                    width: 60.0,
                    height: 80.0,
                },
                ..TemplatePaneNodeData::default()
            },
        ]),
        ..HostWindowPresentationData::default()
    };

    let clipped_front = hit_test_workbench_window_template_node(&presentation, 20.0, 20.0)
        .expect("the back node should remain hittable outside the front clip");
    let visible_front = hit_test_workbench_window_template_node(&presentation, 60.0, 20.0)
        .expect("the front node should be hittable inside its clip");

    assert_eq!(clipped_front.control_id.as_str(), "BackButton");
    assert_eq!(visible_front.control_id.as_str(), "FrontButton");
}

#[test]
fn indexed_workbench_hit_test_preserves_reverse_paint_order_and_clip_frames() {
    let presentation = HostWindowPresentationData {
        workbench_window_nodes: model(vec![
            TemplatePaneNodeData {
                node_id: "back".into(),
                control_id: "BackButton".into(),
                action_id: "back.click".into(),
                frame: TemplateNodeFrameData {
                    x: 10.0,
                    y: 10.0,
                    width: 100.0,
                    height: 80.0,
                },
                ..TemplatePaneNodeData::default()
            },
            TemplatePaneNodeData {
                node_id: "front".into(),
                control_id: "FrontButton".into(),
                action_id: "front.click".into(),
                frame: TemplateNodeFrameData {
                    x: 10.0,
                    y: 10.0,
                    width: 100.0,
                    height: 80.0,
                },
                has_clip_frame: true,
                clip_frame: TemplateNodeFrameData {
                    x: 50.0,
                    y: 10.0,
                    width: 60.0,
                    height: 80.0,
                },
                ..TemplatePaneNodeData::default()
            },
        ]),
        ..HostWindowPresentationData::default()
    };
    let index = HostWorkbenchHitIndex::from_presentation(&presentation);

    let clipped_front =
        hit_test_workbench_window_template_node_with_index(&presentation, &index, 20.0, 20.0)
            .expect("the indexed back node should remain hittable outside the front clip");
    let visible_front =
        hit_test_workbench_window_template_node_with_index(&presentation, &index, 60.0, 20.0)
            .expect("the indexed front node should be hittable inside its clip");

    assert_eq!(clipped_front.control_id.as_str(), "BackButton");
    assert_eq!(visible_front.control_id.as_str(), "FrontButton");
}

#[test]
fn indexed_workbench_hit_test_visits_only_the_point_bucket() {
    let nodes = (0..10_000)
        .map(|row| TemplatePaneNodeData {
            node_id: format!("node-{row}").into(),
            control_id: format!("Control{row}").into(),
            action_id: format!("action.{row}").into(),
            frame: TemplateNodeFrameData {
                x: 10.0,
                y: row as f32 * 32.0,
                width: 120.0,
                height: 24.0,
            },
            ..TemplatePaneNodeData::default()
        })
        .collect();
    let presentation = HostWindowPresentationData {
        workbench_window_nodes: model(nodes),
        ..HostWindowPresentationData::default()
    };
    let index = HostWorkbenchHitIndex::from_presentation(&presentation);

    let hit = hit_test_workbench_window_template_node_with_index(
        &presentation,
        &index,
        20.0,
        9_999.0 * 32.0 + 12.0,
    )
    .expect("the final indexed node should remain hittable");

    assert_eq!(hit.control_id.as_str(), "Control9999");
    assert!(
        index.last_candidate_visit_count_for_test() <= 2,
        "a stable hit should not scan the 10k-node model"
    );
}
