use super::super::super::super::sources::*;

#[test]
fn optional_feature_parser_pending_feature_entry_owner_composes_finalize_steps() {
    assert!(
        PARSER_PENDING_FEATURE_ENTRY
            .contains("pub(in super::super::super) fn push_optional_feature")
            && PARSER_PENDING_FEATURE_ENTRY.contains("let Some(mut feature) = feature.take()")
            && PARSER_PENDING_FEATURE_ENTRY
                .contains("super::normalize::normalize_optional_feature(&mut feature)")
            && PARSER_PENDING_FEATURE_ENTRY
                .contains("super::output::push_static_optional_feature_manifest")
            && PARSER_PENDING_FEATURE_ENTRY
                .contains("super::static_manifest::static_optional_feature_manifest(feature)"),
        "pending feature entry child should own normalize/assemble/output composition"
    );
}
