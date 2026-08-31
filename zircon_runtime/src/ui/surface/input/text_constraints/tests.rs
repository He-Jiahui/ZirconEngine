use zircon_runtime_interface::ui::{
    dispatch::{UiImePreeditClause, UiImePreeditClauseKind, UiTextByteRange},
    surface::UiTextRange,
    tree::UiTemplateNodeMetadata,
};

use std::collections::HashSet;

use super::{
    TEXT_INPUT_GRAPHEME_AUTHORITY_COUNTER_NAMES, TextInputConstraints, TextInputFilter,
    TextInputRetainedGraphemeCount, retained_grapheme_count_from_source,
};

#[test]
fn constrained_preedit_maps_each_crlf_boundary_and_counts_one_separator() {
    let constraints = TextInputConstraints {
        max_graphemes: None,
        filter: TextInputFilter::Any,
        multiline: false,
    };
    let clauses = [
        UiImePreeditClause::new(UiTextByteRange::new(0, 1), UiImePreeditClauseKind::Input),
        UiImePreeditClause::new(
            UiTextByteRange::new(1, 3),
            UiImePreeditClauseKind::Converted,
        ),
        UiImePreeditClause::new(
            UiTextByteRange::new(3, 4),
            UiImePreeditClauseKind::TargetConverted,
        ),
    ];

    let sanitized = constraints.sanitize_preedit_replacement(
        "",
        UiTextRange { start: 0, end: 0 },
        "a\r\nb",
        Some(UiTextByteRange::new(2, 2)),
        &clauses,
    );

    assert_eq!(sanitized.text, "ab");
    assert_eq!(sanitized.cursor_range, Some(UiTextByteRange::new(1, 1)));
    assert_eq!(sanitized.receipt.removed_hard_line_count, 1);
    assert!(sanitized.receipt.preedit_cursor_range_adjusted);
    assert_eq!(sanitized.receipt.preedit_clause_range_adjusted_count, 1);
    assert_eq!(sanitized.receipt.preedit_clause_dropped_count, 1);
    assert_eq!(
        sanitized.preedit_clauses,
        [
            UiImePreeditClause::new(UiTextByteRange::new(0, 1), UiImePreeditClauseKind::Input,),
            UiImePreeditClause::new(
                UiTextByteRange::new(1, 2),
                UiImePreeditClauseKind::TargetConverted,
            ),
        ]
    );
}

#[test]
fn empty_preedit_clause_is_preserved_when_constraints_do_not_move_it() {
    let clauses = [UiImePreeditClause::new(
        UiTextByteRange::new(1, 1),
        UiImePreeditClauseKind::Input,
    )];

    let sanitized = TextInputConstraints::default().sanitize_preedit_replacement(
        "",
        UiTextRange { start: 0, end: 0 },
        "ab",
        None,
        &clauses,
    );

    assert_eq!(sanitized.preedit_clauses, clauses);
    assert!(sanitized.receipt.is_empty());
}

#[test]
fn max_length_uses_the_retained_document_grapheme_count_when_supplied() {
    let constraints = TextInputConstraints {
        max_graphemes: Some(2),
        filter: TextInputFilter::Any,
        multiline: true,
    };

    let sanitized = constraints.sanitize_replacement_with_retained_grapheme_count(
        "abcdef",
        UiTextRange { start: 0, end: 0 },
        "xy",
        TextInputRetainedGraphemeCount::DocumentIndex(1),
    );

    assert_eq!(sanitized.text, "x");
    assert!(sanitized.receipt.max_graphemes_truncated);
}

#[test]
fn grapheme_authority_profile_uses_only_fixed_names() {
    let unique = TEXT_INPUT_GRAPHEME_AUTHORITY_COUNTER_NAMES
        .into_iter()
        .collect::<HashSet<_>>();
    assert_eq!(unique.len(), 3);
    assert!(
        unique
            .iter()
            .all(|name| name.starts_with("text_input_grapheme_"))
    );
}

#[test]
fn source_scan_receipt_counts_only_bytes_outside_the_replaced_range() {
    assert_eq!(
        retained_grapheme_count_from_source(
            "a\u{0301}bc\u{1f469}\u{200d}\u{1f4bb}",
            UiTextRange { start: 3, end: 5 },
        ),
        (2, 14)
    );
}

#[test]
fn number_field_constraint_never_exceeds_the_invariant_edit_buffer_budget() {
    let mut surface = crate::ui::surface::UiSurface::new(
        zircon_runtime_interface::ui::event_ui::UiTreeId::new("number.limit"),
    );
    let root = zircon_runtime_interface::ui::event_ui::UiNodeId::new(1);
    surface.tree.insert_root(
        zircon_runtime_interface::ui::tree::UiTreeNode::new(
            root,
            zircon_runtime_interface::ui::event_ui::UiNodePath::new("root"),
        )
        .with_template_metadata(UiTemplateNodeMetadata {
            component: "NumberField".to_string(),
            attributes: toml::from_str("max_graphemes = 4096").unwrap(),
            ..UiTemplateNodeMetadata::default()
        }),
    );

    let constraints = super::text_input_constraints_for_node(&surface, root);
    let sanitized = constraints.sanitize_replacement(
        "",
        UiTextRange { start: 0, end: 0 },
        &"1".repeat(super::super::number_field::MVP_MAX_NUMBER_FIELD_EDIT_BYTES + 1),
    );

    assert_eq!(
        sanitized.text.len(),
        super::super::number_field::MVP_MAX_NUMBER_FIELD_EDIT_BYTES
    );
    assert!(sanitized.receipt.max_graphemes_truncated);
}
