use std::{hint::black_box, time::Instant};

use super::*;
use zircon_runtime_interface::ui::{
    event_ui::{UiNodePath, UiTreeId},
    layout::UiFrame,
    tree::{UiTemplateNodeMetadata, UiTreeNode},
    widget::UiWidgetContract,
};

const OWNER: UiNodeId = UiNodeId::new(7);
const MARKER: &str = "RUNTIME82_TEXT_STATE_REPEATED_BOUNDARY_V1";

fn surface_with_text(text: &str, offsets: &[(&str, toml::Value)]) -> UiSurface {
    let mut surface = UiSurface::new(UiTreeId::new("runtime82.text.state.boundaries"));
    let mut attributes = std::collections::BTreeMap::from([(
        "content".to_owned(),
        toml::Value::String(text.to_owned()),
    )]);
    for (key, value) in offsets {
        attributes.insert((*key).to_owned(), value.clone());
    }
    surface.tree.insert_root(
        UiTreeNode::new(OWNER, UiNodePath::new("root/input"))
            .with_frame(UiFrame::new(0.0, 0.0, 320.0, 30.0))
            .with_template_metadata(UiTemplateNodeMetadata {
                component: "InputField".to_owned(),
                attributes,
                widget: UiWidgetContract {
                    behavior: UiWidgetBehavior::TextInput,
                    value_property: Some("content".to_owned()),
                    ..Default::default()
                },
                ..Default::default()
            }),
    );
    surface
}

fn offset(value: i64) -> toml::Value {
    toml::Value::Integer(value)
}

#[test]
fn absent_optional_offsets_keep_a_plain_editable_state() {
    let surface = surface_with_text("e\u{301} body", &[]);
    let state = editable_text_state_for_node(&surface, OWNER).unwrap();
    assert_eq!(
        state,
        legacy_editable_text_state_for_node(&surface, OWNER).unwrap()
    );
    assert_eq!(state.text, "e\u{301} body");
    assert_eq!(state.caret.offset, state.text.len());
    assert!(state.selection.is_none());
    assert!(state.composition.is_none());
}

#[test]
fn repeated_non_ascii_offsets_reuse_a_single_grapheme_result() {
    let surface = surface_with_text(
        "a\u{301}b",
        &[
            ("caret_offset", offset(1)),
            ("selection_anchor", offset(1)),
            ("selection_focus", offset(1)),
            ("composition_start", offset(1)),
            ("composition_end", offset(1)),
            ("composition_text", toml::Value::String("x".to_owned())),
            (
                "composition_restore_text",
                toml::Value::String(String::new()),
            ),
        ],
    );
    let state = editable_text_state_for_node(&surface, OWNER).unwrap();
    assert_eq!(
        state,
        legacy_editable_text_state_for_node(&surface, OWNER).unwrap()
    );
    assert_eq!(state.caret.offset, 0);
    assert_eq!(state.selection.as_ref().unwrap().anchor, 0);
    assert_eq!(state.selection.as_ref().unwrap().focus, 0);
    assert_eq!(state.composition.as_ref().unwrap().range.start, 0);
    assert_eq!(state.composition.as_ref().unwrap().range.end, 0);
}

#[test]
fn distinct_unicode_crlf_and_out_of_range_offsets_match_legacy() {
    let text = "\u{0600}a\u{301}b\r\nc";
    for offsets in [
        vec![
            ("caret_offset", offset(7)),
            ("selection_anchor", offset(1)),
            ("selection_focus", offset(3)),
            ("composition_start", offset(5)),
            ("composition_end", offset(8)),
            ("composition_text", toml::Value::String("x".to_owned())),
        ],
        vec![
            ("caret_offset", offset(999)),
            ("selection_anchor", offset(0)),
            ("selection_focus", offset(999)),
            ("composition_start", offset(3)),
            ("composition_end", offset(999)),
            ("composition_text", toml::Value::String("x".to_owned())),
        ],
    ] {
        let surface = surface_with_text(text, &offsets);
        assert_eq!(
            editable_text_state_for_node(&surface, OWNER),
            legacy_editable_text_state_for_node(&surface, OWNER)
        );
    }
}

#[test]
fn ascii_offsets_and_non_editable_nodes_keep_the_old_contract() {
    let surface = surface_with_text(
        "plain ascii",
        &[
            ("caret_offset", offset(7)),
            ("selection_anchor", offset(2)),
            ("selection_focus", offset(7)),
        ],
    );
    assert_eq!(
        editable_text_state_for_node(&surface, OWNER),
        legacy_editable_text_state_for_node(&surface, OWNER)
    );
    assert_eq!(
        editable_text_state_for_node(&surface, UiNodeId::new(999)),
        None
    );
    assert_eq!(
        legacy_editable_text_state_for_node(&surface, UiNodeId::new(999)),
        None
    );
    let mut missing_metadata = UiSurface::new(UiTreeId::new("missing_metadata"));
    missing_metadata
        .tree
        .insert_root(UiTreeNode::new(OWNER, UiNodePath::new("bare")));
    assert_eq!(editable_text_state_for_node(&missing_metadata, OWNER), None);
    let mut not_editable = surface_with_text("label", &[]);
    let metadata = not_editable
        .tree
        .node_mut(OWNER)
        .unwrap()
        .template_metadata
        .as_mut()
        .unwrap();
    metadata.component = "Label".to_owned();
    metadata.widget = UiWidgetContract::default();
    assert_eq!(editable_text_state_for_node(&not_editable, OWNER), None);
    assert_eq!(
        legacy_editable_text_state_for_node(&not_editable, OWNER),
        None
    );
}

#[test]
fn incomplete_selection_and_composition_fields_remain_absent() {
    for fields in [
        vec![("selection_anchor", offset(1))],
        vec![("selection_focus", offset(1))],
        vec![
            ("composition_start", offset(1)),
            ("composition_end", offset(1)),
        ],
        vec![
            ("composition_start", offset(1)),
            ("composition_text", toml::Value::String("x".to_owned())),
        ],
    ] {
        let surface = surface_with_text("e\u{301}b", &fields);
        let state = editable_text_state_for_node(&surface, OWNER).unwrap();
        assert_eq!(
            state,
            legacy_editable_text_state_for_node(&surface, OWNER).unwrap()
        );
        assert!(state.selection.is_none());
        assert!(state.composition.is_none());
    }
}

#[test]
fn boundary_reuse_is_scoped_to_the_current_materialization() {
    let fields = [
        ("caret_offset", offset(2)),
        ("selection_anchor", offset(2)),
        ("selection_focus", offset(2)),
        ("composition_start", offset(2)),
        ("composition_end", offset(2)),
        ("composition_text", toml::Value::String(String::new())),
    ];
    let mut surface = surface_with_text("a\r\nb", &fields);
    let crlf = editable_text_state_for_node(&surface, OWNER).unwrap();
    assert_eq!(crlf.caret.offset, 1);
    assert_eq!(crlf.selection.as_ref().unwrap().anchor, 1);
    assert_eq!(crlf.composition.as_ref().unwrap().range.end, 1);
    surface
        .tree
        .node_mut(OWNER)
        .unwrap()
        .template_metadata
        .as_mut()
        .unwrap()
        .attributes
        .insert("content".to_owned(), toml::Value::String("abcd".to_owned()));
    let ascii = editable_text_state_for_node(&surface, OWNER).unwrap();
    assert_eq!(ascii.caret.offset, 2);
    assert_eq!(ascii.selection.as_ref().unwrap().focus, 2);
    assert_eq!(ascii.composition.as_ref().unwrap().range.start, 2);
    assert_eq!(
        ascii,
        legacy_editable_text_state_for_node(&surface, OWNER).unwrap()
    );
}

#[test]
fn committed_surface_projection_materializes_the_same_complete_state() {
    let text = "ééé";
    let surface = surface_with_committed_offsets(text, 4);
    let state = editable_text_state_for_node(&surface, OWNER).unwrap();
    assert_eq!(
        state,
        legacy_editable_text_state_for_node(&surface, OWNER).unwrap()
    );
    assert_eq!(state.text, text);
    assert_eq!(state.caret.offset, 4);
    assert_eq!(state.selection.as_ref().unwrap().anchor, 4);
    assert_eq!(state.selection.as_ref().unwrap().focus, 4);
    assert!(state.composition.is_none());
}

fn surface_with_committed_offsets(text: &str, caret_offset: usize) -> UiSurface {
    let mut surface = surface_with_text(text, &[]);
    let state = UiEditableTextState {
        text: text.to_owned(),
        caret: UiTextCaret {
            offset: caret_offset,
            affinity: UiTextCaretAffinity::Downstream,
        },
        ..Default::default()
    };
    super::super::editable_text::prepare_editable_text_properties_with_edit(
        &mut surface,
        OWNER,
        "content",
        &state,
        zircon_runtime_interface::ui::binding::UiBindingSourceKind::WidgetBehavior,
        None,
    )
    .unwrap()
    .commit()
    .unwrap();
    surface
}

// Frozen HEAD materializer; private helpers remain shared and unchanged.
fn legacy_editable_text_state_for_node(
    surface: &UiSurface,
    target: UiNodeId,
) -> Option<UiEditableTextState> {
    crate::profile_scope!("runtime", "ui_text.edit", "state_materialize");
    let metadata = surface
        .tree
        .nodes
        .get(&target)?
        .template_metadata
        .as_ref()?;
    if !is_editable_text_component(metadata) {
        return None;
    }
    let property = editable_value_property(surface, target)?;
    let text = if is_number_field_metadata(metadata) {
        number_field_text(metadata, property.as_str())
    } else {
        string_attribute(metadata, property.as_str())
            .or_else(|| string_attribute(metadata, "value_text"))
            .or_else(|| string_attribute(metadata, "text"))
            .or_else(|| metadata.widget.value.as_ref().map(UiValue::display_text))
            .unwrap_or_default()
    };
    super::super::editable_text::profile::record_state_materialization(text.len());
    let caret_offset = usize_attribute(metadata, "caret_offset").unwrap_or(text.len());
    let selection = usize_attribute(metadata, "selection_anchor")
        .zip(usize_attribute(metadata, "selection_focus"))
        .map(|(anchor, focus)| UiTextSelection {
            anchor: clamp_grapheme_boundary(&text, anchor),
            focus: clamp_grapheme_boundary(&text, focus),
        });
    let composition = usize_attribute(metadata, "composition_start")
        .zip(usize_attribute(metadata, "composition_end"))
        .zip(string_attribute(metadata, "composition_text"))
        .map(|((start, end), composition_text)| UiTextComposition {
            range: UiTextRange {
                start: clamp_grapheme_boundary(&text, start),
                end: clamp_grapheme_boundary(&text, end),
            },
            preedit_clauses: composition_clauses_from_metadata(metadata, &composition_text),
            text: composition_text,
            restore_text: string_attribute(metadata, "composition_restore_text"),
        });

    Some(UiEditableTextState {
        caret: UiTextCaret {
            offset: clamp_grapheme_boundary(&text, caret_offset),
            affinity: caret_affinity_from_metadata(metadata),
        },
        selection,
        composition,
        read_only: bool_attribute_any(
            metadata,
            &["read_only", "readOnly", "input_read_only", "inputReadOnly"],
        )
        .unwrap_or(false),
        text,
    })
}

#[test]
#[ignore = "managed Windows Release Runtime82 full text state comparison"]
fn runtime82_text_state_repeated_boundary_release_profile() {
    assert!(!cfg!(debug_assertions), "run with --release");
    const WARMUPS: usize = 5;
    const SAMPLES: usize = 31;
    eprintln!("{MARKER} os={} arch={} crate={} processor={:?} profile=release warmups={WARMUPS} samples={SAMPLES}", std::env::consts::OS, std::env::consts::ARCH, env!("CARGO_PKG_VERSION"), std::env::var("PROCESSOR_IDENTIFIER").ok());
    for characters in [1_000, 10_000, 1_000_000] {
        let text = "é".repeat(characters);
        let bytes = text.len();
        let repeated_raw_offset = bytes - "é".len();
        // Prime the ten retained metadata fields through the real property
        // transaction before timing either materializer. The document clone
        // inside each materializer remains part of both measured calls.
        let surface = surface_with_committed_offsets(&text, repeated_raw_offset);
        let mut old_ns = Vec::with_capacity(SAMPLES);
        let mut new_ns = Vec::with_capacity(SAMPLES);
        for sample in 0..WARMUPS + SAMPLES {
            let (old, old_elapsed, new, new_elapsed) = if sample % 2 == 0 {
                let started = Instant::now();
                let old = black_box(legacy_editable_text_state_for_node(
                    black_box(&surface),
                    OWNER,
                ));
                let old_elapsed = started.elapsed().as_nanos();
                let started = Instant::now();
                let new = black_box(editable_text_state_for_node(black_box(&surface), OWNER));
                (old, old_elapsed, new, started.elapsed().as_nanos())
            } else {
                let started = Instant::now();
                let new = black_box(editable_text_state_for_node(black_box(&surface), OWNER));
                let new_elapsed = started.elapsed().as_nanos();
                let started = Instant::now();
                let old = black_box(legacy_editable_text_state_for_node(
                    black_box(&surface),
                    OWNER,
                ));
                (old, started.elapsed().as_nanos(), new, new_elapsed)
            };
            assert_eq!(old, new);
            assert_eq!(old.as_ref().unwrap().text.len(), bytes);
            assert_eq!(old.as_ref().unwrap().caret.offset, repeated_raw_offset);
            if sample >= WARMUPS {
                old_ns.push(old_elapsed);
                new_ns.push(new_elapsed);
            }
        }
        let old_summary = summary(&old_ns);
        let new_summary = summary(&new_ns);
        eprintln!("{MARKER} characters={characters} bytes={bytes} offset={repeated_raw_offset} old_ns={old_ns:?} new_ns={new_ns:?} old_p50_p95_p99={old_summary:?} new_p50_p95_p99={new_summary:?}");
        if characters == 1_000_000 {
            assert!(
                new_summary.1.saturating_mul(100) <= old_summary.1.saturating_mul(80),
                "1M full state materialization p95 must improve by at least 20%"
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
