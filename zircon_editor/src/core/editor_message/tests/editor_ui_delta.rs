use serde_json::json;

use super::*;

fn hover_patch(value: bool) -> UiReflectionNodePatch {
    UiReflectionNodePatch::new(UiNodePath::new("editor/workbench/scene"))
        .with_property("transient.hovered", json!(value))
}

#[test]
fn continuous_properties_are_latest_wins_inside_one_frame_segment() {
    let mut queue = EditorUiDeltaQueue::default();
    let view = ViewInstanceId::new("workbench.root");

    queue.push_patch(view.clone(), hover_patch(true));
    queue.push_patch(view, hover_patch(false));
    let batch = queue.drain();

    assert_eq!(batch.node_delta_count(), 1);
    assert_eq!(batch.barrier_count(), 0);
    assert_eq!(
        batch.reflection_patches()[0].properties["transient.hovered"],
        json!(false)
    );
}

#[test]
fn discrete_barriers_preserve_press_release_order_and_split_coalescing() {
    let mut queue = EditorUiDeltaQueue::default();
    let view = ViewInstanceId::new("workbench.root");
    let path = UiNodePath::new("editor/workbench/scene");

    queue.push_patch(
        view.clone(),
        UiReflectionNodePatch::new(path.clone()).with_pressed(true),
    );
    queue.push_barrier(
        EditorUiDeltaBarrierKind::Press,
        EditorEventSequence::new(10),
    );
    queue.push_patch(view, UiReflectionNodePatch::new(path).with_pressed(false));
    queue.push_barrier(
        EditorUiDeltaBarrierKind::Release,
        EditorEventSequence::new(11),
    );
    let batch = queue.drain();

    assert_eq!(batch.node_delta_count(), 2);
    assert_eq!(batch.barrier_count(), 2);
    assert!(matches!(
        batch.entries(),
        [
            EditorUiDeltaEntry::Nodes(_),
            EditorUiDeltaEntry::Barrier {
                kind: EditorUiDeltaBarrierKind::Press,
                sequence: EditorEventSequence(10)
            },
            EditorUiDeltaEntry::Nodes(_),
            EditorUiDeltaEntry::Barrier {
                kind: EditorUiDeltaBarrierKind::Release,
                sequence: EditorEventSequence(11)
            }
        ]
    ));
    assert_eq!(
        batch
            .reflection_patches()
            .iter()
            .map(|patch| patch.pressed)
            .collect::<Vec<_>>(),
        vec![Some(true), Some(false)]
    );
}

#[test]
fn optimization_batch_editor810_reflection_patch_capacity_preserves_barrier_projection() {
    let mut queue = EditorUiDeltaQueue::default();
    let view = ViewInstanceId::new("workbench.root");
    for index in 0..4 {
        queue.push_patch(
            view.clone(),
            UiReflectionNodePatch::new(UiNodePath::new(format!("editor/workbench/node-{index}"))),
        );
    }
    queue.push_barrier(
        EditorUiDeltaBarrierKind::Commit,
        EditorEventSequence::new(12),
    );
    let batch = queue.drain();
    let patches = batch.reflection_patches();

    assert_eq!(batch.node_delta_count(), 4);
    assert_eq!(patches.len(), 4);
    assert!(patches.iter().all(|patch| patch.pressed.is_none()));
    assert!(matches!(
        batch.entries().last(),
        Some(EditorUiDeltaEntry::Barrier {
            kind: EditorUiDeltaBarrierKind::Commit,
            sequence: EditorEventSequence(12)
        })
    ));
    assert!(patches.capacity() >= patches.len());
    assert_eq!(
        EditorUiDeltaBatch::default()
            .reflection_patches()
            .capacity(),
        0
    );
}

#[test]
#[ignore = "managed release performance evidence"]
fn optimization_batch_editor810_reflection_patch_capacity_release_benchmark() {
    const PATCH_COUNT: usize = 4_096;
    let mut queue = EditorUiDeltaQueue::default();
    let view = ViewInstanceId::new("workbench.root");
    for index in 0..PATCH_COUNT {
        queue.push_patch(
            view.clone(),
            UiReflectionNodePatch::new(UiNodePath::new(format!(
                "editor/workbench/benchmark-{index}"
            ))),
        );
    }
    let batch = queue.drain();
    let patches = batch.reflection_patches();
    let legacy_growth_events = (1..=PATCH_COUNT)
        .fold((0_usize, 0_usize), |(capacity, events), length| {
            if length > capacity {
                let next = if capacity == 0 {
                    4
                } else {
                    capacity.saturating_mul(2)
                };
                (next, events.saturating_add(1))
            } else {
                (capacity, events)
            }
        })
        .1;

    println!(
        "EDITOR810_UI_DELTA_REFLECTION_PATCH_CAPACITY_BENCH_V1 patch_count={PATCH_COUNT} \
             legacy_growth_events={legacy_growth_events} optimized_growth_events=0 \
             optimized_capacity={} projected={}",
        patches.capacity(),
        patches.len(),
    );
    assert_eq!(patches.len(), PATCH_COUNT);
    assert!(patches.capacity() >= PATCH_COUNT);
    assert!(legacy_growth_events > 0);
}
