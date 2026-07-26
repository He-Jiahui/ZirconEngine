use crate::ui::{
    event_ui::UiNodeId,
    layout::UiFrame,
    surface::{
        UiClipMode, UiClipState, UiDrawEffect, UiPaintElement, UiRenderCommand,
        UiRenderCommandKind, UiResolvedStyle,
    },
};

use super::{UiBatchPlan, UiBatchSplitReason, UiClipStack};

fn solid_element(
    node_id: u64,
    z_index: i32,
    paint_order: u64,
    clip: Option<UiClipState>,
) -> UiPaintElement {
    let mut element = UiRenderCommand {
        node_id: UiNodeId::new(node_id),
        kind: UiRenderCommandKind::Quad,
        frame: UiFrame::new(node_id as f32 * 8.0, 0.0, 8.0, 8.0),
        clip_frame: None,
        z_index,
        style: UiResolvedStyle::default(),
        text_layout: None,
        text: None,
        image: None,
        opacity: 1.0,
    }
    .to_paint_element(paint_order);
    element.clip = clip;
    element
}

#[test]
fn batch_plan_sorts_layers_without_encoding_layer_in_the_batch_key() {
    let elements = vec![
        solid_element(1, 20, 0, None),
        solid_element(2, 0, 10, None),
        solid_element(3, 0, 20, None),
    ];

    let plan = UiBatchPlan::from_paint_elements(&elements);

    assert_eq!(plan.ordered_element_indices, vec![1, 2, 0]);
    assert_eq!(plan.batches.len(), 2);
    assert_eq!(plan.batches[0].layer, 0);
    assert_eq!(plan.batches[0].source_indices, vec![1, 2]);
    assert_eq!(plan.batches[1].layer, 20);
    assert_eq!(plan.batches[1].source_indices, vec![0]);
    assert_eq!(
        plan.batches[1].split_reason,
        UiBatchSplitReason::LayerChanged
    );
}

#[test]
fn batch_plan_reuses_equivalent_clip_handles_and_splits_different_states() {
    let shared_scissor = UiClipState {
        mode: UiClipMode::Scissor,
        frame: UiFrame::new(0.0, 0.0, 80.0, 40.0),
    };
    let stencil = UiClipState {
        mode: UiClipMode::Stencil,
        frame: UiFrame::new(0.0, 0.0, 80.0, 40.0),
    };
    let elements = vec![
        solid_element(1, 0, 0, Some(shared_scissor.clone())),
        solid_element(2, 0, 1, Some(shared_scissor)),
        solid_element(3, 0, 2, Some(stencil)),
    ];

    let plan = UiBatchPlan::from_paint_elements(&elements);

    assert_eq!(plan.clip_states().len(), 2);
    assert_eq!(plan.batches.len(), 2);
    assert_eq!(plan.batches[0].source_indices, vec![0, 1]);
    assert_eq!(plan.batches[1].source_indices, vec![2]);
    assert_ne!(plan.batches[0].key.clip, plan.batches[1].key.clip);
    assert_eq!(
        plan.batches[1].split_reason,
        UiBatchSplitReason::ClipChanged
    );
}

#[test]
fn batch_plan_round_trips_without_serializing_backend_clip_indices() {
    let plan = UiBatchPlan::from_paint_elements(&[solid_element(
        1,
        0,
        0,
        Some(UiClipState {
            mode: UiClipMode::Scissor,
            frame: UiFrame::new(0.0, 0.0, 80.0, 40.0),
        }),
    )]);

    let json = serde_json::to_string(&plan).expect("batch plan serializes");
    let restored: UiBatchPlan = serde_json::from_str(&json).expect("batch plan deserializes");

    assert_eq!(restored, plan);
}

#[test]
fn clip_stack_intersects_nested_axis_aligned_scissors() {
    let mut clips = UiClipStack::default();
    clips.push(UiClipState {
        mode: UiClipMode::Scissor,
        frame: UiFrame::new(0.0, 0.0, 100.0, 80.0),
    });
    let nested = clips.push(UiClipState {
        mode: UiClipMode::Scissor,
        frame: UiFrame::new(60.0, 40.0, 100.0, 80.0),
    });

    assert_eq!(
        clips
            .resolve(&nested)
            .expect("nested clip is retained")
            .frame,
        UiFrame::new(60.0, 40.0, 40.0, 40.0)
    );
    assert_eq!(clips.pop(), Some(nested));
}

#[test]
fn clip_stack_keeps_disjoint_scissors_empty() {
    let mut clips = UiClipStack::default();
    clips.push(UiClipState {
        mode: UiClipMode::Scissor,
        frame: UiFrame::new(0.0, 0.0, 20.0, 20.0),
    });
    let disjoint = clips.push(UiClipState {
        mode: UiClipMode::Scissor,
        frame: UiFrame::new(40.0, 40.0, 20.0, 20.0),
    });

    assert_eq!(
        clips
            .resolve(&disjoint)
            .expect("disjoint clip is retained as an empty scissor")
            .frame,
        UiFrame::new(40.0, 40.0, 0.0, 0.0)
    );
}

#[test]
fn batch_plan_splits_when_draw_effect_flags_change() {
    let first = solid_element(1, 0, 0, None);
    let mut second = solid_element(2, 0, 1, None);
    second.effects.effects.push(UiDrawEffect::DisabledEffect);

    let plan = UiBatchPlan::from_paint_elements(&[first, second]);

    assert_eq!(plan.batches.len(), 2);
    assert_eq!(
        plan.batches[1].split_reason,
        UiBatchSplitReason::DrawEffectsChanged
    );
}
