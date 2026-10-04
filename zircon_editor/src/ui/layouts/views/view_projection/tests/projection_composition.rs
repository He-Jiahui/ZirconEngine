use std::cell::Cell;

use super::*;

#[test]
fn same_generation_text_delta_patches_only_the_composed_target_row() {
    clear_for_tests();
    let compose_calls = Cell::new(0_u64);
    let stable = Rc::new(ViewTemplateNodeData {
        node_id: "stable".into(),
        control_id: "Stable".into(),
        text: "stable".into(),
        ..ViewTemplateNodeData::default()
    });
    let changing = Rc::new(ViewTemplateNodeData {
        node_id: "changing".into(),
        control_id: "Changing".into(),
        text: "before".into(),
        ..ViewTemplateNodeData::default()
    });
    let first_rows = Rc::new(vec![Rc::clone(&stable), Rc::clone(&changing)]);
    let first = compose_model(
        "composition.incremental_text",
        ViewTemplateNodeProjection {
            base_rows: Rc::clone(&first_rows),
            row_patches: Rc::new(BTreeMap::new()),
            source_frame: None,
        },
        &7_u64,
        |nodes| {
            compose_calls.set(compose_calls.get() + 1);
            nodes[1].surface_variant = "composed".into();
        },
    );

    let changed = Rc::new(ViewTemplateNodeData {
        text: "after".into(),
        ..changing.as_ref().clone()
    });
    let next = compose_model(
        "composition.incremental_text",
        ViewTemplateNodeProjection {
            base_rows: Rc::clone(&first_rows),
            row_patches: Rc::new(BTreeMap::from([(1, changed)])),
            source_frame: None,
        },
        &7_u64,
        |_| panic!("same-generation text patch must not rerun full composition"),
    );

    assert_eq!(compose_calls.get(), 1);
    assert_eq!(composition_counts_for_tests(), (1, 2, 1));
    assert!(first.shares_row_with(&next, 0));
    assert!(!first.shares_row_with(&next, 1));
    assert_eq!(next.get(1).map(|node| node.text.as_str()), Some("after"));
    assert_eq!(
        next.get(1).map(|node| node.surface_variant.as_str()),
        Some("composed")
    );
}

#[test]
fn topology_change_reruns_full_composition() {
    clear_for_tests();
    let compose_calls = Cell::new(0_u64);
    let stable = Rc::new(ViewTemplateNodeData {
        node_id: "stable".into(),
        control_id: "Stable".into(),
        ..ViewTemplateNodeData::default()
    });
    let _ = compose_model(
        "composition.topology",
        ViewTemplateNodeProjection {
            base_rows: Rc::new(vec![Rc::clone(&stable)]),
            row_patches: Rc::new(BTreeMap::new()),
            source_frame: None,
        },
        &11_u64,
        |_| compose_calls.set(compose_calls.get() + 1),
    );

    let appended = Rc::new(ViewTemplateNodeData {
        node_id: "appended".into(),
        control_id: "Appended".into(),
        ..ViewTemplateNodeData::default()
    });
    let next = compose_model(
        "composition.topology",
        ViewTemplateNodeProjection {
            base_rows: Rc::new(vec![stable, appended]),
            row_patches: Rc::new(BTreeMap::new()),
            source_frame: None,
        },
        &11_u64,
        |_| compose_calls.set(compose_calls.get() + 1),
    );

    assert_eq!(compose_calls.get(), 2);
    assert_eq!(next.row_count(), 2);
    assert_eq!(composition_counts_for_tests(), (2, 3, 0));
}
