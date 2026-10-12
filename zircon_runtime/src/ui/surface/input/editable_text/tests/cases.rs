use std::{hint::black_box, time::Instant};

use super::*;
use crate::ui::dispatch::UiInputManager;
use zircon_runtime_interface::ui::{
    dispatch::{
        UiInputEvent, UiInputEventMetadata, UiInputSequence, UiInputTimestamp,
        UiKeyboardInputEvent, UiKeyboardInputState,
    },
    event_ui::{UiNodePath, UiStateFlags, UiTreeId},
    layout::UiFrame,
    surface::{UiTextCaret, UiTextCaretAffinity},
    tree::{UiInputPolicy, UiTemplateNodeMetadata, UiTreeNode},
    widget::{UiWidgetBehavior, UiWidgetContract, UiWidgetEvent},
};

const OWNER: UiNodeId = UiNodeId::new(2);
const PROFILE_MARKER: &str = "RUNTIME82_PROPERTY_PREPARE_BORROWED_V1";

fn text_state(text: &str) -> UiEditableTextState {
    UiEditableTextState {
        text: text.to_owned(),
        caret: UiTextCaret {
            offset: text.len(),
            affinity: UiTextCaretAffinity::Downstream,
        },
        ..Default::default()
    }
}

fn text_surface(text: &str) -> UiSurface {
    let mut surface = UiSurface::new(UiTreeId::new("runtime82.property.prepare"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 320.0, 80.0)),
    );
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(OWNER, UiNodePath::new("root/input"))
                .with_frame(UiFrame::new(8.0, 8.0, 280.0, 30.0))
                .with_input_policy(UiInputPolicy::Receive)
                .with_state_flags(UiStateFlags {
                    visible: true,
                    enabled: true,
                    clickable: true,
                    hoverable: true,
                    focusable: true,
                    ..Default::default()
                })
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "InputField".to_owned(),
                    attributes: [
                        ("content".to_owned(), toml::Value::String(text.to_owned())),
                        (
                            "caret_offset".to_owned(),
                            toml::Value::Integer(text.len() as i64),
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
        )
        .expect("input child");
    surface
}

fn attributes(surface: &UiSurface) -> &std::collections::BTreeMap<String, toml::Value> {
    &surface
        .tree
        .node(OWNER)
        .expect("input node")
        .template_metadata
        .as_ref()
        .expect("input metadata")
        .attributes
}

#[test]
fn provisional_text_keeps_source_epoch_while_ordinary_reset_advances_it() {
    let mut surface = text_surface("abcd");
    let visible = text_state("ad");
    let epoch = surface.input.text_document_epoch(OWNER);
    let receipt = prepare_editable_text_properties_with_edit(
        &mut surface,
        OWNER,
        "content",
        &visible,
        UiBindingSourceKind::WidgetBehavior,
        None,
    )
    .expect("prepare provisional display")
    .preserving_committed_source()
    .commit()
    .expect("publish provisional display");
    assert!(receipt.text_changed);
    assert!(receipt.dirty.layout && receipt.dirty.render && receipt.dirty.text);
    assert_eq!(attributes(&surface)["content"].as_str(), Some("ad"));
    assert_eq!(surface.input.text_document_epoch(OWNER), epoch);

    let receipt = prepare_editable_text_properties_with_edit(
        &mut surface,
        OWNER,
        "content",
        &text_state("xyz"),
        UiBindingSourceKind::RuntimeState,
        None,
    )
    .expect("prepare ordinary source reset")
    .commit()
    .expect("publish ordinary source reset");
    assert!(receipt.text_changed);
    assert_eq!(surface.input.text_document_epoch(OWNER), Some(1));
}

#[test]
fn committed_intent_advances_source_epoch_when_visible_properties_already_match() {
    let mut surface = text_surface("ad");
    let state = text_state("ad");
    commit_editable_text_properties(
        &mut surface,
        OWNER,
        "content",
        &state,
        UiBindingSourceKind::WidgetBehavior,
    )
    .expect("prime every inactive property");
    let attributes_before = attributes(&surface).clone();
    let epoch_before = surface.input.text_document_epoch(OWNER).unwrap();

    let receipt = prepare_editable_text_properties_with_edit(
        &mut surface,
        OWNER,
        "content",
        &state,
        UiBindingSourceKind::WidgetBehavior,
        Some(CommittedTextEditIntent::for_replacement(1..3, 0)),
    )
    .expect("prepare commit against the separately retained source")
    .preserving_committed_source()
    .commit()
    .expect("publish committed intent without another display change");
    assert!(!receipt.text_changed);
    assert!(receipt.changed_properties.is_empty());
    assert!(receipt.binding_report.is_none());
    assert!(receipt.committed_edit.is_some());
    assert_eq!(attributes(&surface), &attributes_before);
    assert_eq!(
        surface.input.text_document_epoch(OWNER),
        Some(epoch_before + 1)
    );
}

#[test]
fn single_property_prepare_is_atomic_and_keyboard_edit_reaches_the_surface() {
    let mut surface = text_surface("hello");
    let state = text_state("hell");
    let before = surface.clone();
    let prepared = prepare_editable_text_properties_with_edit(
        &mut surface,
        OWNER,
        "content",
        &state,
        UiBindingSourceKind::WidgetBehavior,
        None,
    )
    .expect("valid same-property prepare");
    assert!(prepared.supplemental_properties.iter().all(Option::is_none));
    drop(prepared);
    assert_eq!(
        surface, before,
        "prepare and discard must not mutate the surface"
    );

    surface.rebuild();
    surface.focus_node(OWNER).expect("focus input");
    let mut manager = UiInputManager::default();
    let result = manager
        .dispatch_input_event(
            &mut surface,
            UiInputEvent::Keyboard(UiKeyboardInputEvent {
                metadata: UiInputEventMetadata::new(
                    UiInputTimestamp::from_micros(30),
                    UiInputSequence::new(3),
                ),
                state: UiKeyboardInputState::Pressed,
                key_code: 8,
                scan_code: None,
                physical_key: "Backspace".to_owned(),
                logical_key: "Backspace".to_owned(),
                text: None,
            }),
        )
        .expect("keyboard dispatch");
    assert_eq!(attributes(&surface)["content"].as_str(), Some("hell"));
    assert_eq!(attributes(&surface)["caret_offset"].as_integer(), Some(4));
    assert_eq!(
        attributes(&surface)["selection_anchor"].as_integer(),
        Some(4)
    );
    assert_eq!(
        attributes(&surface)["selection_focus"].as_integer(),
        Some(4)
    );
    assert!(!result.binding_reports.is_empty());
    assert!(result.widget_events.iter().any(|event| {
        matches!(event, UiWidgetEvent::TextEditChange { receipt } if receipt.revision.get() > receipt.previous_revision.get())
    }));
}

#[test]
fn different_numeric_value_and_text_buffer_commit_independently() {
    let mut surface = text_surface("42");
    let metadata = surface
        .tree
        .node_mut(OWNER)
        .unwrap()
        .template_metadata
        .as_mut()
        .unwrap();
    metadata.component = "NumberField".to_owned();
    metadata.attributes.remove("content");
    metadata
        .attributes
        .insert("value".to_owned(), toml::Value::Float(42.0));
    metadata.attributes.insert(
        "value_text".to_owned(),
        toml::Value::String("42".to_owned()),
    );
    metadata.widget.value_property = Some("value".to_owned());
    let state = text_state("420");
    let prepared = prepare_editable_text_properties_with_value(
        &mut surface,
        OWNER,
        "value",
        UiValue::Float(42.0),
        &state,
        UiBindingSourceKind::WidgetBehavior,
    )
    .expect("valid two-property prepare");
    let receipt = prepared.commit().expect("two-property commit");
    assert!(!receipt.value_changed);
    assert!(receipt.text_changed);
    assert_eq!(attributes(&surface)["value"].as_float(), Some(42.0));
    assert_eq!(attributes(&surface)["value_text"].as_str(), Some("420"));
    assert_eq!(surface.input.text_document_epoch(OWNER), Some(1));

    let published = prepare_editable_text_properties_with_value(
        &mut surface,
        OWNER,
        "value",
        UiValue::Float(43.0),
        &text_state("43"),
        UiBindingSourceKind::WidgetBehavior,
    )
    .expect("changed numeric value and text")
    .commit()
    .expect("changed numeric commit");
    assert!(published.value_changed && published.text_changed);
    assert_eq!(attributes(&surface)["value"].as_float(), Some(43.0));
    assert_eq!(attributes(&surface)["value_text"].as_str(), Some("43"));
    assert_eq!(
        attributes(&surface)["number_value_revision"].as_integer(),
        Some(1)
    );
    assert_eq!(surface.input.text_document_epoch(OWNER), Some(2));
}

#[test]
fn mismatched_kind_and_invalid_grapheme_reject_without_partial_mutation() {
    let mut surface = text_surface("e\u{301}x");
    let before = surface.clone();
    let state = text_state("e\u{301}x");
    let mismatch = prepare_editable_text_properties_with_value(
        &mut surface,
        OWNER,
        "content",
        UiValue::Int(7),
        &state,
        UiBindingSourceKind::RuntimeState,
    );
    assert_eq!(
        mismatch.err(),
        Some(UiEditableTextPropertyTransactionError::ValueKindMismatch)
    );
    assert_eq!(surface, before);

    let mut invalid = state.clone();
    invalid.caret.offset = 1;
    let invalid_result = prepare_editable_text_properties_with_edit(
        &mut surface,
        OWNER,
        "content",
        &invalid,
        UiBindingSourceKind::WidgetBehavior,
        None,
    );
    assert_eq!(
        invalid_result.err(),
        Some(UiEditableTextPropertyTransactionError::InvalidState)
    );
    assert_eq!(surface, before);
}

#[test]
fn explicit_editable_component_aliases_keep_their_canonical_sibling_value() {
    for (component, property) in [("RichText", "content"), ("ComboBox", "value_text")] {
        let mut surface = text_surface("hello");
        let metadata = surface
            .tree
            .node_mut(OWNER)
            .unwrap()
            .template_metadata
            .as_mut()
            .unwrap();
        metadata.component = component.to_owned();
        metadata.widget.value_property = Some(property.to_owned());
        metadata.attributes.remove("content");
        metadata
            .attributes
            .insert(property.to_owned(), toml::Value::String("hello".to_owned()));
        metadata.attributes.insert(
            "value".to_owned(),
            toml::Value::String("canonical".to_owned()),
        );
        let receipt = prepare_editable_text_properties_with_edit(
            &mut surface,
            OWNER,
            property,
            &text_state("hell"),
            UiBindingSourceKind::WidgetBehavior,
            None,
        )
        .expect("explicit editable property")
        .commit()
        .expect("alias commit");
        assert!(receipt.text_changed && receipt.value_changed);
        assert_eq!(attributes(&surface)[property].as_str(), Some("hell"));
        assert_eq!(attributes(&surface)["value"].as_str(), Some("canonical"));
    }
}

#[test]
fn reserved_property_and_inconsistent_text_reject_the_whole_surface_transaction() {
    let mut surface = text_surface("hello");
    let before = surface.clone();
    let state = text_state("hello");
    let reserved = prepare_editable_text_properties_with_edit(
        &mut surface,
        OWNER,
        "caret_offset",
        &state,
        UiBindingSourceKind::WidgetBehavior,
        None,
    );
    assert_eq!(
        reserved.err(),
        Some(UiEditableTextPropertyTransactionError::ReservedValueProperty)
    );
    assert_eq!(surface, before);
    let inconsistent = prepare_editable_text_properties_with_value(
        &mut surface,
        OWNER,
        "content",
        UiValue::String("other".to_owned()),
        &state,
        UiBindingSourceKind::RuntimeState,
    );
    assert_eq!(
        inconsistent.err(),
        Some(UiEditableTextPropertyTransactionError::InvalidState)
    );
    assert_eq!(surface, before);
    let conflicting_alias = prepare_editable_text_properties_with_values_and_edit(
        &mut surface,
        OWNER,
        "content",
        Some(UiValue::Int(7)),
        "content",
        UiValue::String(state.text.clone()),
        [None, None, None],
        None,
        &state,
        UiBindingSourceKind::RuntimeState,
        None,
    );
    assert_eq!(
        conflicting_alias.err(),
        Some(UiEditableTextPropertyTransactionError::ValueKindMismatch)
    );
    assert_eq!(
        surface, before,
        "same property names must still validate explicitly distinct values"
    );
}

#[test]
fn invalid_edit_intent_rejects_before_the_surface_changes() {
    let mut surface = text_surface("hello");
    let before = surface.clone();
    let state = text_state("hello");
    let intent = CommittedTextEditIntent::for_replacement(0..0, 0);
    let result = prepare_editable_text_properties_with_edit(
        &mut surface,
        OWNER,
        "content",
        &state,
        UiBindingSourceKind::WidgetBehavior,
        Some(intent),
    );
    assert_eq!(
        result.err(),
        Some(UiEditableTextPropertyTransactionError::InvalidEditIntent)
    );
    assert_eq!(surface, before);
}

#[test]
fn old_and_borrowed_prepare_commit_the_same_text_and_caret_only_results() {
    let mut caret_only = text_state("e\u{301}xyz");
    caret_only.caret.offset = "e\u{301}".len();
    for state in [text_state("e\u{301}xy"), caret_only] {
        let mut old_surface = text_surface("e\u{301}xyz");
        let mut new_surface = old_surface.clone();
        let old = legacy_prepare(
            &mut old_surface,
            OWNER,
            "content",
            &state,
            UiBindingSourceKind::WidgetBehavior,
            None,
        )
        .unwrap()
        .commit()
        .unwrap();
        let new = prepare_editable_text_properties_with_edit(
            &mut new_surface,
            OWNER,
            "content",
            &state,
            UiBindingSourceKind::WidgetBehavior,
            None,
        )
        .unwrap()
        .commit()
        .unwrap();
        assert_eq!(
            new, old,
            "binding/dirty/text receipts must remain equivalent"
        );
        assert_eq!(new_surface, old_surface);
    }
}

#[test]
fn borrowed_kind_and_text_checks_match_the_allocating_contract() {
    let datetime: toml::value::Datetime = "2026-09-27".parse().unwrap();
    let cases = [
        (
            toml::Value::String("x".to_owned()),
            UiValue::String("x".to_owned()),
        ),
        (
            toml::Value::String("x".to_owned()),
            UiValue::Color("x".to_owned()),
        ),
        (
            toml::Value::String("x".to_owned()),
            UiValue::Enum("x".to_owned()),
        ),
        (
            toml::Value::Datetime(datetime),
            UiValue::String("x".to_owned()),
        ),
        (toml::Value::Integer(4), UiValue::Int(4)),
        (toml::Value::Float(4.0), UiValue::Float(4.0)),
        (toml::Value::Boolean(true), UiValue::Bool(true)),
        (
            toml::Value::Array(vec![toml::Value::Integer(1)]),
            UiValue::Array(vec![]),
        ),
        (
            toml::Value::Table(Default::default()),
            UiValue::Map(Default::default()),
        ),
        (toml::Value::Integer(4), UiValue::Float(4.0)),
    ];
    for (current, proposed) in cases {
        let expected_kind = std::mem::discriminant(&UiValue::from_toml(&current))
            == std::mem::discriminant(&proposed);
        assert_eq!(
            borrowed_toml_kind_matches(&current, &proposed),
            expected_kind,
            "borrowed kind must preserve from_toml variant semantics: {current:?}"
        );
        let mut surface = text_surface("x");
        surface
            .tree
            .node_mut(OWNER)
            .unwrap()
            .template_metadata
            .as_mut()
            .unwrap()
            .attributes
            .insert("content".to_owned(), current);
        let before = surface.clone();
        let state = text_state(&proposed.display_text());
        let result = prepare_editable_text_properties_with_value(
            &mut surface,
            OWNER,
            "content",
            proposed,
            &state,
            UiBindingSourceKind::RuntimeState,
        );
        if expected_kind {
            drop(result.expect("compatible outer kind"));
        } else {
            assert_eq!(
                result.err(),
                Some(UiEditableTextPropertyTransactionError::ValueKindMismatch)
            );
        }
        assert_eq!(surface, before);
    }
    for proposed in [
        UiValue::String("x".to_owned()),
        UiValue::Color("x".to_owned()),
        UiValue::AssetRef("x".to_owned()),
        UiValue::InstanceRef("x".to_owned()),
        UiValue::Enum("x".to_owned()),
        UiValue::Int(7),
    ] {
        assert_eq!(
            display_text_matches(&proposed, "x"),
            proposed.display_text() == "x"
        );
    }
}

// Frozen HEAD plain-text prepare path. NumberField calls share unchanged helpers;
// this comparison uses only InputField. Allocation-producing admission remains old.
fn legacy_prepare<'surface>(
    surface: &'surface mut UiSurface,
    target: UiNodeId,
    value_property: &str,
    state: &UiEditableTextState,
    source_kind: UiBindingSourceKind,
    committed_edit: Option<CommittedTextEditIntent>,
) -> Result<
    PreparedUiEditableTextPropertyTransaction<'surface>,
    UiEditableTextPropertyTransactionError,
> {
    crate::profile_scope!("runtime", "ui_text.edit", "property_prepare");
    super::super::profile::record_property_value_clone(state.text.len());
    let is_number_field = surface
        .tree
        .node(target)
        .and_then(|node| node.template_metadata.as_ref())
        .is_some_and(is_number_field_metadata);
    let prepared = if is_number_field {
        let number_edit = super::super::super::number_field::number_field_edit_decision(
            surface,
            target,
            &state.text,
        );
        let canonical_value = number_edit
            .and_then(|decision| decision.publish_value)
            .map(UiValue::Float)
            .map_or_else(|| canonical_value(surface, target, value_property), Ok)?;
        prepare_number_field_properties_with_edit(
            surface,
            target,
            value_property,
            canonical_value,
            state,
            true,
            false,
            source_kind,
            number_edit,
            committed_edit,
        )?
    } else {
        legacy_prepare_with_values_and_edit(
            surface,
            target,
            value_property,
            UiValue::String(state.text.clone()),
            value_property,
            UiValue::String(state.text.clone()),
            [None, None, None],
            None,
            state,
            source_kind,
            committed_edit,
        )?
    };
    super::super::profile::record_property_projection(
        state.text.len(),
        prepared.committed_edit.is_some(),
        state.composition.is_some(),
        state
            .composition
            .as_ref()
            .map_or(0, |composition| composition.text.len()),
    );
    Ok(prepared)
}

#[allow(clippy::too_many_arguments)]
fn legacy_prepare_with_values_and_edit<'surface>(
    surface: &'surface mut UiSurface,
    target: UiNodeId,
    value_property: &str,
    value: UiValue,
    text_property: &str,
    text_value: UiValue,
    additional_properties: [Option<(String, UiValue)>; 3],
    number_edit: Option<super::super::super::number_field::NumberFieldEditDecision>,
    state: &UiEditableTextState,
    source_kind: UiBindingSourceKind,
    committed_edit: Option<CommittedTextEditIntent>,
) -> Result<
    PreparedUiEditableTextPropertyTransaction<'surface>,
    UiEditableTextPropertyTransactionError,
> {
    legacy_validate(
        surface,
        target,
        value_property,
        &value,
        text_property,
        &text_value,
        &additional_properties,
        state,
        committed_edit.as_ref(),
    )?;
    Ok(PreparedUiEditableTextPropertyTransaction {
        surface,
        target,
        value_property: value_property.to_string(),
        text_property: text_property.to_string(),
        properties: editable_text_properties(text_property, text_value, state),
        supplemental_properties: [
            (value_property != text_property).then(|| (value_property.to_string(), value)),
            additional_properties[0].clone(),
            additional_properties[1].clone(),
            additional_properties[2].clone(),
        ],
        source_kind,
        committed_edit,
        preserve_committed_source_epoch: false,
        number_input: number_edit.map(|decision| decision.receipt),
        number_publish_value: number_edit.and_then(|decision| decision.publish_value),
    })
}

fn legacy_validate(
    surface: &UiSurface,
    target: UiNodeId,
    value_property: &str,
    value: &UiValue,
    text_property: &str,
    text_value: &UiValue,
    additional_properties: &[Option<(String, UiValue)>; 3],
    state: &UiEditableTextState,
    committed_edit: Option<&CommittedTextEditIntent>,
) -> Result<(), UiEditableTextPropertyTransactionError> {
    let node = surface
        .tree
        .node(target)
        .ok_or(UiEditableTextPropertyTransactionError::MissingNode)?;
    let metadata = node
        .template_metadata
        .as_ref()
        .ok_or(UiEditableTextPropertyTransactionError::MissingMetadata)?;
    if value_property.is_empty() || value_property_is_reserved(value_property) {
        return Err(UiEditableTextPropertyTransactionError::ReservedValueProperty);
    }
    let kind_matches = |property: &str, proposed: &UiValue| {
        metadata
            .attributes
            .get(property)
            .map(UiValue::from_toml)
            .is_none_or(|current| {
                std::mem::discriminant(&current) == std::mem::discriminant(proposed)
            })
    };
    if !kind_matches(value_property, value)
        || !kind_matches(text_property, text_value)
        || additional_properties
            .iter()
            .flatten()
            .any(|(property, value)| !kind_matches(property, value))
    {
        return Err(UiEditableTextPropertyTransactionError::ValueKindMismatch);
    }
    if text_value.display_text() != state.text || !editable_text_state_is_valid(state) {
        return Err(UiEditableTextPropertyTransactionError::InvalidState);
    }
    if committed_edit.is_some_and(|intent| !intent.is_valid_for_state(state)) {
        return Err(UiEditableTextPropertyTransactionError::InvalidEditIntent);
    }
    Ok(())
}

#[test]
#[ignore = "managed Windows Release Runtime82 property prepare comparison"]
fn runtime82_property_prepare_legacy_vs_borrowed_release_profile() {
    assert!(!cfg!(debug_assertions), "run only with --release");
    const WARMUPS: usize = 5;
    const SAMPLES: usize = 31;
    eprintln!(
        "{PROFILE_MARKER} os={} arch={} crate={} processor={:?} profile=release lane=prepare_discard warmups={WARMUPS} samples={SAMPLES}",
        std::env::consts::OS, std::env::consts::ARCH, env!("CARGO_PKG_VERSION"),
        std::env::var("PROCESSOR_IDENTIFIER").ok(),
    );
    for bytes in [1_000, 10_000, 1_000_000] {
        let text = "x".repeat(bytes);
        let initial = format!("{text}x");
        let state = text_state(&text);
        let intent = CommittedTextEditIntent::for_replacement(bytes..bytes + 1, 0);
        let mut legacy_surface = text_surface(&initial);
        let mut borrowed_surface = text_surface(&initial);
        let old = legacy_prepare(
            &mut legacy_surface,
            OWNER,
            "content",
            &state,
            UiBindingSourceKind::WidgetBehavior,
            Some(intent.clone()),
        )
        .expect("old prepare");
        let old_properties = old.properties.clone();
        let old_supplemental = old.supplemental_properties.clone();
        let old_intent = old.committed_edit.clone();
        drop(old);
        let new = prepare_editable_text_properties_with_edit(
            &mut borrowed_surface,
            OWNER,
            "content",
            &state,
            UiBindingSourceKind::WidgetBehavior,
            Some(intent.clone()),
        )
        .expect("new prepare");
        assert_eq!(new.properties, old_properties);
        assert_eq!(new.supplemental_properties, old_supplemental);
        assert_eq!(new.committed_edit, old_intent);
        drop(new);

        let mut old_ns = Vec::with_capacity(SAMPLES);
        let mut new_ns = Vec::with_capacity(SAMPLES);
        for sample in 0..(WARMUPS + SAMPLES) {
            let old_first = sample % 2 == 0;
            for old in [old_first, !old_first] {
                let started = Instant::now();
                if old {
                    drop(black_box(
                        legacy_prepare(
                            &mut legacy_surface,
                            OWNER,
                            black_box("content"),
                            black_box(&state),
                            UiBindingSourceKind::WidgetBehavior,
                            Some(black_box(&intent).clone()),
                        )
                        .unwrap(),
                    ));
                } else {
                    drop(black_box(
                        prepare_editable_text_properties_with_edit(
                            &mut borrowed_surface,
                            OWNER,
                            black_box("content"),
                            black_box(&state),
                            UiBindingSourceKind::WidgetBehavior,
                            Some(black_box(&intent).clone()),
                        )
                        .unwrap(),
                    ));
                }
                let ns = started.elapsed().as_nanos();
                if sample >= WARMUPS {
                    if old {
                        old_ns.push(ns);
                    } else {
                        new_ns.push(ns);
                    }
                }
            }
        }
        let old_summary = summary(&old_ns);
        let new_summary = summary(&new_ns);
        eprintln!(
            "{PROFILE_MARKER} bytes={bytes} warmups={WARMUPS} samples={SAMPLES} old_ns={old_ns:?} new_ns={new_ns:?} old_p50_p95_p99={old_summary:?} new_p50_p95_p99={new_summary:?}"
        );
        if bytes == 1_000_000 {
            assert!(
                new_summary.1.saturating_mul(100) <= old_summary.1.saturating_mul(80),
                "million-character prepare p95 must improve by at least 20%"
            );
        }
    }
}

fn summary(samples: &[u128]) -> (u128, u128, u128) {
    let mut ordered = samples.to_vec();
    ordered.sort_unstable();
    let percentile = |numerator: usize| ordered[(ordered.len() * numerator).div_ceil(100) - 1];
    (percentile(50), percentile(95), percentile(99))
}
