use zircon_runtime_interface::ui::{
    ecs::{UiEcsDirtyDomains, UiEcsInteractionState, UiEcsNodeProjection, UiEcsProjectionSnapshot},
    event_ui::{UiNodeId, UiNodePath, UiTreeId},
    layout::UiFrame,
    pipeline::{
        UiPipelineDirtyReason, UiPipelineFrameReport, UiPipelineStage, UiPipelineStageCounters,
        UiPipelineStageReport,
    },
    surface::UiSurfaceDebugSnapshot,
};

use super::model::EditorUiDebugReflectorModel;

#[test]
fn ui_debug_reflector_schedule_sections_project_pipeline_and_ecs_projection() {
    let snapshot = UiSurfaceDebugSnapshot {
        tree_id: UiTreeId::new("editor.schedule.reflector.test"),
        pipeline_report: UiPipelineFrameReport::from_stage_reports(
            12,
            vec![
                UiPipelineStageReport::new(
                    UiPipelineStage::InputCollect,
                    3,
                    vec![UiPipelineDirtyReason::Input],
                    UiPipelineStageCounters {
                        input_event_count: 2,
                        pointer_move_count: 1,
                        ..UiPipelineStageCounters::default()
                    },
                ),
                UiPipelineStageReport::new(
                    UiPipelineStage::RenderExtract,
                    7,
                    vec![UiPipelineDirtyReason::Render],
                    UiPipelineStageCounters {
                        render_extract_command_count: 4,
                        batch_count: 2,
                        ..UiPipelineStageCounters::default()
                    },
                ),
            ],
        ),
        ecs_projection: UiEcsProjectionSnapshot::from_nodes(
            UiTreeId::new("editor.schedule.reflector.test"),
            vec![UiNodeId::new(1)],
            vec![UiEcsNodeProjection {
                node_id: UiNodeId::new(1),
                node_path: UiNodePath::new("root/input"),
                component: "TextInput".to_string(),
                frame: UiFrame::new(0.0, 0.0, 80.0, 24.0),
                dirty: UiEcsDirtyDomains {
                    text: true,
                    render: true,
                    ..UiEcsDirtyDomains::default()
                },
                interaction: UiEcsInteractionState {
                    visible: true,
                    enabled: true,
                    focused: true,
                    focusable: true,
                    ..UiEcsInteractionState::default()
                },
                render_command_count: 3,
                hit_entry_count: 1,
                ..UiEcsNodeProjection::default()
            }],
        ),
        ..UiSurfaceDebugSnapshot::default()
    };

    let model =
        EditorUiDebugReflectorModel::from_snapshot(&snapshot).with_schedule_sections(&snapshot);

    assert!(model.sections.iter().any(|section| {
        section.title == "Pipeline"
            && section.lines.iter().any(|line| line == "frame: 12")
            && section
                .lines
                .iter()
                .any(|line| line.contains("stages: completed=2 total=2 ordered=false"))
            && section
                .lines
                .iter()
                .any(|line| line == "totals: input=2,pointer_move=1,render=4,batch=2")
            && section.lines.iter().any(|line| {
                line.contains("stage=input_collect")
                    && line.contains("dirty=Input")
                    && line.contains("counters=input=2,pointer_move=1")
            })
    }));
    assert!(model.sections.iter().any(|section| {
        section.title == "ECS Projection"
            && section
                .lines
                .iter()
                .any(|line| line == "nodes: total=1 dirty=1 roots=1")
            && section.lines.iter().any(|line| {
                line == "dirty domains: layout=0 text=1 input=0 picking=0 a11y=0 render=1"
            })
            && section
                .lines
                .iter()
                .any(|line| line == "interaction: focused=1 hovered=0 pressed=0 disabled=0")
            && section.lines.iter().any(|line| {
                line.contains("schedule mask:")
                    && line.contains("text_measure")
                    && line.contains("render_extract")
            })
            && section.lines.iter().any(|line| {
                line.contains("schedule impacts:")
                    && line.contains("text_measure=1 nodes")
                    && line.contains("render_extract=1 nodes")
            })
            && section
                .lines
                .iter()
                .any(|line| line.contains("dirty-domain impacts:") && line.contains("Text=1"))
    }));
}
