use std::{hint::black_box, time::Instant};

use super::*;
use crate::ui::surface::{input::prepare_editable_text_properties_with_edit, UiSurface};
use zircon_runtime_interface::ui::{
    event_ui::{UiNodePath, UiTreeId},
    layout::UiFrame,
    surface::{UiEditableTextState, UiTextCaret, UiTextCaretAffinity},
    tree::{UiTemplateNodeMetadata, UiTreeNode},
    widget::{UiWidgetBehavior, UiWidgetContract},
};

const OWNER: UiNodeId = UiNodeId::new(1);
const MARKER: &str = "RUNTIME82_METADATA_BATCH_UNCHANGED_STRING_V1";

fn tree_with_value(value: toml::Value) -> UiTree {
    let mut tree = UiTree::new(UiTreeId::new("runtime82.metadata.batch"));
    tree.insert_root(
        UiTreeNode::new(OWNER, UiNodePath::new("root/input"))
            .with_frame(UiFrame::new(0.0, 0.0, 320.0, 30.0))
            .with_template_metadata(UiTemplateNodeMetadata {
                component: "InputField".to_owned(),
                attributes: [
                    ("content".to_owned(), value),
                    ("caret_offset".to_owned(), toml::Value::Integer(0)),
                    (
                        "caret_affinity".to_owned(),
                        toml::Value::String("downstream".to_owned()),
                    ),
                    ("selection_anchor".to_owned(), toml::Value::Integer(0)),
                    ("selection_focus".to_owned(), toml::Value::Integer(0)),
                    ("composition_start".to_owned(), toml::Value::Integer(0)),
                    ("composition_end".to_owned(), toml::Value::Integer(0)),
                    (
                        "composition_text".to_owned(),
                        toml::Value::String(String::new()),
                    ),
                    (
                        "composition_restore_text".to_owned(),
                        toml::Value::String(String::new()),
                    ),
                    (
                        "composition_clauses".to_owned(),
                        toml::Value::Array(Vec::new()),
                    ),
                ]
                .into_iter()
                .collect(),
                widget: UiWidgetContract {
                    behavior: UiWidgetBehavior::TextInput,
                    value_property: Some("content".to_owned()),
                    ..Default::default()
                },
                ..Default::default()
            }),
    );
    tree
}

fn attributes(tree: &UiTree) -> &BTreeMap<String, toml::Value> {
    &tree
        .node(OWNER)
        .unwrap()
        .template_metadata
        .as_ref()
        .unwrap()
        .attributes
}

fn projected_properties(text: String, caret: i64) -> [(&'static str, UiValue); 10] {
    [
        ("content", UiValue::String(text)),
        ("caret_offset", UiValue::Int(caret)),
        ("caret_affinity", UiValue::String("downstream".to_owned())),
        ("selection_anchor", UiValue::Int(0)),
        ("selection_focus", UiValue::Int(0)),
        ("composition_start", UiValue::Int(0)),
        ("composition_end", UiValue::Int(0)),
        ("composition_text", UiValue::String(String::new())),
        ("composition_restore_text", UiValue::String(String::new())),
        ("composition_clauses", UiValue::Array(Vec::new())),
    ]
}

#[test]
fn unchanged_text_with_caret_mutation_reports_only_the_caret() {
    let text = "e\u{301} body";
    let mut tree = tree_with_value(toml::Value::String(text.to_owned()));
    let before_dirty = tree.node(OWNER).unwrap().dirty;
    let result = mutate_tree_metadata_properties(
        &mut tree,
        OWNER,
        [
            ("content", UiValue::String(text.to_owned())),
            ("caret_offset", UiValue::Int(3)),
        ],
        UiBindingSourceKind::WidgetBehavior,
    )
    .unwrap();
    assert_eq!(attributes(&tree)["content"].as_str(), Some(text));
    assert_eq!(attributes(&tree)["caret_offset"].as_integer(), Some(3));
    assert_eq!(result.changes.len(), 1);
    assert_eq!(result.changes[0].property, "caret_offset");
    assert_eq!(result.reflected_updates.len(), 1);
    assert_eq!(result.reflected_updates[0].previous, Some(UiValue::Int(0)));
    assert_eq!(result.reflected_updates[0].value, UiValue::Int(3));
    assert!(result.dirty.render && !result.dirty.text && !result.dirty.layout);
    assert_eq!(
        tree.node(OWNER).unwrap().dirty,
        before_dirty,
        "caller owns dirty publication"
    );
}

#[test]
fn changed_string_keeps_previous_value_and_reflected_receipt() {
    let mut tree = tree_with_value(toml::Value::String("before".to_owned()));
    let result = mutate_tree_metadata_properties(
        &mut tree,
        OWNER,
        [("content", UiValue::String("after".to_owned()))],
        UiBindingSourceKind::WidgetBehavior,
    )
    .unwrap();
    assert_eq!(attributes(&tree)["content"].as_str(), Some("after"));
    assert_eq!(result.changes.len(), 1);
    assert_eq!(result.changes[0].value, UiValue::String("after".to_owned()));
    assert_eq!(
        result.reflected_updates[0].previous,
        Some(UiValue::String("before".to_owned()))
    );
    assert_eq!(
        result.reflected_updates[0].value,
        UiValue::String("after".to_owned())
    );
    assert_eq!(
        result.reflected_updates[0].status,
        UiBindingUpdateStatus::Applied
    );
}

#[test]
fn missing_and_repeated_text_properties_keep_sequential_batch_semantics() {
    let mut old_tree = tree_with_value(toml::Value::String("initial".to_owned()));
    old_tree
        .node_mut(OWNER)
        .unwrap()
        .template_metadata
        .as_mut()
        .unwrap()
        .attributes
        .remove("content");
    let mut new_tree = old_tree.clone();
    let properties = || {
        [
            ("content", UiValue::String("first".to_owned())),
            ("content", UiValue::String("first".to_owned())),
            ("content", UiValue::String("last".to_owned())),
        ]
    };
    let old = legacy_batch(
        &mut old_tree,
        OWNER,
        properties(),
        UiBindingSourceKind::WidgetBehavior,
    )
    .unwrap();
    let new = mutate_tree_metadata_properties(
        &mut new_tree,
        OWNER,
        properties(),
        UiBindingSourceKind::WidgetBehavior,
    )
    .unwrap();
    assert_eq!(new, old);
    assert_eq!(new.changes.len(), 2);
    assert_eq!(new.reflected_updates[0].previous, None);
    assert_eq!(
        new.reflected_updates[1].previous,
        Some(UiValue::String("first".to_owned()))
    );
    assert_eq!(attributes(&new_tree)["content"].as_str(), Some("last"));
    assert_eq!(new_tree, old_tree);
}

#[test]
fn datetime_numeric_and_stringlike_variants_preserve_old_batch_semantics() {
    let date: toml::value::Datetime = "2026-09-27".parse().unwrap();
    for (previous, proposed) in [
        (
            toml::Value::Datetime(date),
            UiValue::String("2026-09-27".to_owned()),
        ),
        (toml::Value::Integer(42), UiValue::String("42".to_owned())),
        (
            toml::Value::String("same".to_owned()),
            UiValue::Color("same".to_owned()),
        ),
        (
            toml::Value::String("same".to_owned()),
            UiValue::Enum("same".to_owned()),
        ),
        (toml::Value::String("".to_owned()), UiValue::Null),
    ] {
        let mut old_tree = tree_with_value(previous);
        let mut new_tree = old_tree.clone();
        let old = legacy_batch(
            &mut old_tree,
            OWNER,
            [("content", proposed.clone())],
            UiBindingSourceKind::WidgetBehavior,
        )
        .unwrap();
        let new = mutate_tree_metadata_properties(
            &mut new_tree,
            OWNER,
            [("content", proposed)],
            UiBindingSourceKind::WidgetBehavior,
        )
        .unwrap();
        assert_eq!(new, old);
        assert_eq!(new_tree, old_tree);
    }
}

#[test]
fn caret_only_surface_commit_preserves_document_epoch_and_text_layout_revision() {
    let text = "e\u{301} body";
    let mut surface = UiSurface::new(UiTreeId::new("runtime82.metadata.batch"));
    surface.tree = tree_with_value(toml::Value::String(text.to_owned()));
    let mut state = UiEditableTextState {
        text: text.to_owned(),
        caret: UiTextCaret {
            offset: text.len(),
            affinity: UiTextCaretAffinity::Downstream,
        },
        ..Default::default()
    };
    prepare_editable_text_properties_with_edit(
        &mut surface,
        OWNER,
        "content",
        &state,
        UiBindingSourceKind::WidgetBehavior,
        None,
    )
    .unwrap()
    .commit()
    .unwrap();
    surface.rebuild();
    let epoch = surface.input.text_document_epoch(OWNER);
    let revision = surface
        .tree
        .node(OWNER)
        .unwrap()
        .layout_cache
        .retained_text_layout_revision()
        .expect("reusable text layout revision");
    state.caret.offset = 3;
    let receipt = prepare_editable_text_properties_with_edit(
        &mut surface,
        OWNER,
        "content",
        &state,
        UiBindingSourceKind::WidgetBehavior,
        None,
    )
    .unwrap()
    .commit()
    .unwrap();
    assert!(!receipt.value_changed && !receipt.text_changed);
    assert!(receipt.dirty.render && !receipt.dirty.text && !receipt.dirty.layout);
    assert_eq!(surface.input.text_document_epoch(OWNER), epoch);
    assert_eq!(
        surface
            .tree
            .node(OWNER)
            .unwrap()
            .layout_cache
            .retained_text_layout_revision(),
        Some(revision)
    );
    assert_eq!(attributes(&surface.tree)["content"].as_str(), Some(text));
    assert_eq!(
        attributes(&surface.tree)["caret_offset"].as_integer(),
        Some(3)
    );
    assert!(receipt
        .binding_report
        .unwrap()
        .updates
        .iter()
        .all(|update| update.target.property.as_deref() != Some("content")));
}

#[test]
fn changed_surface_text_advances_epoch_and_emits_body_binding() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime82.metadata.batch"));
    surface.tree = tree_with_value(toml::Value::String("old".to_owned()));
    let state = UiEditableTextState {
        text: "new".to_owned(),
        caret: UiTextCaret {
            offset: 3,
            affinity: UiTextCaretAffinity::Downstream,
        },
        ..Default::default()
    };
    let receipt = prepare_editable_text_properties_with_edit(
        &mut surface,
        OWNER,
        "content",
        &state,
        UiBindingSourceKind::WidgetBehavior,
        None,
    )
    .unwrap()
    .commit()
    .unwrap();
    assert!(receipt.value_changed && receipt.text_changed);
    assert!(receipt.dirty.layout && receipt.dirty.text && receipt.dirty.render);
    assert_eq!(surface.input.text_document_epoch(OWNER), Some(1));
    assert_eq!(attributes(&surface.tree)["content"].as_str(), Some("new"));
    let binding = receipt.binding_report.expect("body binding update");
    assert!(binding.updates.iter().any(|update| {
        update.target.property.as_deref() == Some("content")
            && update.previous == Some(UiValue::String("old".to_owned()))
    }));
}

#[test]
fn missing_node_and_missing_metadata_keep_the_existing_contract() {
    let mut missing = UiTree::new(UiTreeId::new("missing"));
    assert!(matches!(
        mutate_tree_metadata_properties(&mut missing, OWNER, [("content", UiValue::String("x".to_owned()))], UiBindingSourceKind::WidgetBehavior),
        Err(UiTreeError::MissingNode(id)) if id == OWNER
    ));
    missing.insert_root(UiTreeNode::new(OWNER, UiNodePath::new("root")));
    let before = missing.clone();
    let result = mutate_tree_metadata_properties(
        &mut missing,
        OWNER,
        [("content", UiValue::String("x".to_owned()))],
        UiBindingSourceKind::WidgetBehavior,
    )
    .unwrap();
    assert_eq!(result, UiMetadataPropertyBatchMutation::default());
    assert_eq!(missing, before);
}

// Frozen HEAD metadata_batch.rs production function; shared dirty/report helpers
// are unchanged. Do not replace the allocation-producing to_toml comparison.
fn legacy_batch<P>(
    tree: &mut UiTree,
    node_id: UiNodeId,
    properties: impl IntoIterator<Item = (P, UiValue)>,
    source_kind: UiBindingSourceKind,
) -> Result<UiMetadataPropertyBatchMutation, UiTreeError>
where
    P: AsRef<str> + Into<String>,
{
    let node = tree
        .node_mut(node_id)
        .ok_or(UiTreeError::MissingNode(node_id))?;
    let Some(metadata) = node.template_metadata.as_mut() else {
        return Ok(UiMetadataPropertyBatchMutation::default());
    };

    let mut batch = UiMetadataPropertyBatchMutation::default();
    for (property, value) in properties {
        let property_name = property.as_ref();
        let next = value.to_toml();
        if metadata.attributes.get(property_name) == Some(&next) {
            continue;
        }

        let previous = metadata
            .attributes
            .get(property_name)
            .map(UiValue::from_toml);
        set_metadata_value(&mut metadata.attributes, property_name, next);
        let dirty =
            metadata_attribute_dirty(metadata.component.as_str(), property_name, value.kind());
        merge_dirty_flags(&mut batch.dirty, dirty);
        batch
            .reflected_updates
            .push(reflected_property_update_with_source_kind(
                node_id,
                property_name,
                source_kind,
                previous,
                value.clone(),
                dirty,
                UiBindingUpdateStatus::Applied,
                None,
            ));
        let property = property.into();
        batch.changes.push(UiMetadataPropertyChange {
            property,
            value,
            dirty,
        });
    }

    Ok(batch)
}

#[test]
#[ignore = "managed Windows Release Runtime82 metadata batch comparison"]
fn runtime82_metadata_batch_borrowed_unchanged_string_release_profile() {
    assert!(!cfg!(debug_assertions), "run with --release");
    const WARMUPS: usize = 5;
    const SAMPLES: usize = 31;
    eprintln!("{MARKER} os={} arch={} crate={} processor={:?} profile=release warmups={WARMUPS} samples={SAMPLES}", std::env::consts::OS, std::env::consts::ARCH, env!("CARGO_PKG_VERSION"), std::env::var("PROCESSOR_IDENTIFIER").ok());
    for bytes in [1_000, 10_000, 1_000_000] {
        let text = "x".repeat(bytes);
        let mut old_tree = tree_with_value(toml::Value::String(text.clone()));
        let mut new_tree = old_tree.clone();
        let mut old_ns = Vec::with_capacity(SAMPLES);
        let mut new_ns = Vec::with_capacity(SAMPLES);
        for sample in 0..WARMUPS + SAMPLES {
            let caret = (sample % 2 + 1) as i64;
            // Owned proposed values are required by both signatures. Construct
            // both copies outside the measured interval, then retain tree state.
            let old_properties = projected_properties(text.clone(), caret);
            let new_properties = projected_properties(text.clone(), caret);
            let (old, old_elapsed, new, new_elapsed) = if sample % 2 == 0 {
                let started = Instant::now();
                let old = black_box(
                    legacy_batch(
                        black_box(&mut old_tree),
                        OWNER,
                        black_box(old_properties),
                        UiBindingSourceKind::WidgetBehavior,
                    )
                    .unwrap(),
                );
                let old_elapsed = started.elapsed().as_nanos();
                let started = Instant::now();
                let new = black_box(
                    mutate_tree_metadata_properties(
                        black_box(&mut new_tree),
                        OWNER,
                        black_box(new_properties),
                        UiBindingSourceKind::WidgetBehavior,
                    )
                    .unwrap(),
                );
                (old, old_elapsed, new, started.elapsed().as_nanos())
            } else {
                let started = Instant::now();
                let new = black_box(
                    mutate_tree_metadata_properties(
                        black_box(&mut new_tree),
                        OWNER,
                        black_box(new_properties),
                        UiBindingSourceKind::WidgetBehavior,
                    )
                    .unwrap(),
                );
                let new_elapsed = started.elapsed().as_nanos();
                let started = Instant::now();
                let old = black_box(
                    legacy_batch(
                        black_box(&mut old_tree),
                        OWNER,
                        black_box(old_properties),
                        UiBindingSourceKind::WidgetBehavior,
                    )
                    .unwrap(),
                );
                (old, started.elapsed().as_nanos(), new, new_elapsed)
            };
            assert_eq!(old, new);
            assert_eq!(old.changes.len(), 1, "each sample changes the caret");
            assert_eq!(old.changes[0].property, "caret_offset");
            assert_eq!(old_tree, new_tree);
            if sample >= WARMUPS {
                old_ns.push(old_elapsed);
                new_ns.push(new_elapsed);
            }
        }
        let old_summary = summary(&old_ns);
        let new_summary = summary(&new_ns);
        eprintln!("{MARKER} bytes={bytes} old_ns={old_ns:?} new_ns={new_ns:?} old_p50_p95_p99={old_summary:?} new_p50_p95_p99={new_summary:?}");
        if bytes == 1_000_000 {
            assert!(
                new_summary.1.saturating_mul(100) <= old_summary.1.saturating_mul(80),
                "1M unchanged-body batch p95 must improve by at least 20%"
            );
        }
    }
}

fn summary(samples: &[u128]) -> (u128, u128, u128) {
    let mut ordered = samples.to_vec();
    ordered.sort_unstable();
    let percentile = |p: usize| ordered[(ordered.len() * p).div_ceil(100) - 1];
    (percentile(50), percentile(95), percentile(99))
}
