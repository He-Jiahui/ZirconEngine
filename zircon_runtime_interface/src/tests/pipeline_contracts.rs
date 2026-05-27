use crate::ui::pipeline::{
    UiPipelineDirtyReason, UiPipelineFrameReport, UiPipelineStage, UiPipelineStageCounters,
    UiPipelineStageReport,
};

fn round_trip<T>(value: &T) -> T
where
    T: serde::Serialize + serde::de::DeserializeOwned,
{
    serde_json::from_str(&serde_json::to_string(value).unwrap()).unwrap()
}

#[test]
fn ui_pipeline_stage_order_matches_bevy_aligned_schedule_contract() {
    assert_eq!(
        UiPipelineStage::ORDER,
        [
            UiPipelineStage::InputCollect,
            UiPipelineStage::Focus,
            UiPipelineStage::WidgetBehavior,
            UiPipelineStage::TextMeasure,
            UiPipelineStage::Layout,
            UiPipelineStage::PostLayout,
            UiPipelineStage::Picking,
            UiPipelineStage::A11yExtract,
            UiPipelineStage::RenderExtract,
            UiPipelineStage::BatchPrepare,
        ]
    );

    let names: Vec<&'static str> = UiPipelineStage::ordered()
        .iter()
        .map(|stage| stage.as_str())
        .collect();

    assert_eq!(
        names,
        vec![
            "input_collect",
            "focus",
            "widget_behavior",
            "text_measure",
            "layout",
            "post_layout",
            "picking",
            "a11y_extract",
            "render_extract",
            "batch_prepare",
        ]
    );
    assert_eq!(
        round_trip(&UiPipelineStage::RenderExtract),
        UiPipelineStage::RenderExtract
    );
    assert_eq!(
        serde_json::from_str::<UiPipelineStage>("\"focus_interaction\"").unwrap(),
        UiPipelineStage::FocusInteraction
    );
    assert_eq!(
        serde_json::from_str::<UiPipelineStage>("\"hit_grid\"").unwrap(),
        UiPipelineStage::HitGrid
    );
}

#[test]
fn ui_pipeline_frame_report_records_stage_timings_dirty_reasons_and_counters() {
    let reports = vec![
        UiPipelineStageReport::new(
            UiPipelineStage::InputCollect,
            7,
            vec![UiPipelineDirtyReason::Input],
            UiPipelineStageCounters {
                input_event_count: 3,
                pointer_move_count: 1,
                ..UiPipelineStageCounters::default()
            },
        ),
        UiPipelineStageReport::new(
            UiPipelineStage::Focus,
            11,
            vec![UiPipelineDirtyReason::Focus],
            UiPipelineStageCounters {
                focus_change_count: 1,
                ..UiPipelineStageCounters::default()
            },
        ),
        UiPipelineStageReport::new(
            UiPipelineStage::WidgetBehavior,
            13,
            vec![UiPipelineDirtyReason::WidgetBehavior],
            UiPipelineStageCounters {
                widget_behavior_count: 2,
                ..UiPipelineStageCounters::default()
            },
        ),
        UiPipelineStageReport::new(
            UiPipelineStage::TextMeasure,
            17,
            vec![UiPipelineDirtyReason::Text],
            UiPipelineStageCounters {
                text_measure_count: 4,
                ..UiPipelineStageCounters::default()
            },
        ),
        UiPipelineStageReport::new(
            UiPipelineStage::Layout,
            19,
            vec![UiPipelineDirtyReason::LayoutMetrics],
            UiPipelineStageCounters {
                layout_node_count: 8,
                incremental_layout_count: 1,
                ..UiPipelineStageCounters::default()
            },
        ),
        UiPipelineStageReport::new(
            UiPipelineStage::PostLayout,
            23,
            vec![UiPipelineDirtyReason::Layout],
            UiPipelineStageCounters {
                stack_node_count: 8,
                ..UiPipelineStageCounters::default()
            },
        ),
        UiPipelineStageReport::new(
            UiPipelineStage::Picking,
            29,
            vec![UiPipelineDirtyReason::Picking],
            UiPipelineStageCounters {
                picking_candidate_count: 7,
                hit_grid_rebuild_count: 1,
                ..UiPipelineStageCounters::default()
            },
        ),
        UiPipelineStageReport::new(
            UiPipelineStage::A11yExtract,
            31,
            vec![UiPipelineDirtyReason::A11y],
            UiPipelineStageCounters {
                accessibility_node_count: 6,
                ..UiPipelineStageCounters::default()
            },
        ),
        UiPipelineStageReport::new(
            UiPipelineStage::RenderExtract,
            37,
            vec![UiPipelineDirtyReason::Render],
            UiPipelineStageCounters {
                render_extract_command_count: 12,
                render_command_reuse_count: 9,
                render_command_rebuild_count: 3,
                ..UiPipelineStageCounters::default()
            },
        ),
        UiPipelineStageReport::new(
            UiPipelineStage::BatchPrepare,
            41,
            vec![UiPipelineDirtyReason::Render],
            UiPipelineStageCounters {
                batch_count: 5,
                ..UiPipelineStageCounters::default()
            },
        ),
    ];

    let frame = UiPipelineFrameReport::from_stage_reports(42, reports);

    assert!(frame.is_complete_ordered());
    assert!(frame.missing_required_stages().is_empty());
    assert_eq!(frame.completed_stage_count(), 10);
    assert_eq!(frame.total_elapsed_micros, 228);
    assert_eq!(frame.totals.widget_behavior_count, 2);
    assert_eq!(frame.totals.text_measure_count, 4);
    assert_eq!(frame.totals.layout_node_count, 8);
    assert_eq!(frame.totals.picking_candidate_count, 7);
    assert_eq!(frame.totals.hit_grid_rebuild_count, 1);
    assert_eq!(frame.totals.accessibility_node_count, 6);
    assert_eq!(frame.totals.render_command_reuse_count, 9);
    assert_eq!(frame.totals.batch_count, 5);
    assert_eq!(
        frame
            .stage_report(UiPipelineStage::RenderExtract)
            .unwrap()
            .dirty_reasons,
        vec![UiPipelineDirtyReason::Render]
    );
    assert_eq!(round_trip(&frame), frame);
}

#[test]
fn ui_pipeline_counters_express_repeated_pointer_move_fast_path() {
    let input = UiPipelineStageReport::new(
        UiPipelineStage::InputCollect,
        17,
        vec![UiPipelineDirtyReason::Input],
        UiPipelineStageCounters {
            input_event_count: 100,
            pointer_move_count: 100,
            ..UiPipelineStageCounters::default()
        },
    );
    let content = UiPipelineStageReport::skipped(
        UiPipelineStage::TextMeasure,
        vec![UiPipelineDirtyReason::Input],
    );
    let layout =
        UiPipelineStageReport::skipped(UiPipelineStage::Layout, vec![UiPipelineDirtyReason::Input]);

    let frame = UiPipelineFrameReport::from_stage_reports(7, vec![input, content, layout]);

    assert_eq!(frame.totals.pointer_move_count, 100);
    assert_eq!(frame.totals.template_reload_count, 0);
    assert_eq!(frame.totals.full_layout_count, 0);
    assert_eq!(frame.totals.layout_node_count, 0);
    assert!(frame.repeated_pointer_move_fast_path_holds(100));
    assert_eq!(round_trip(&frame), frame);
}
