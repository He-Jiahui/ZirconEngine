use super::super::super::super::sources::*;

#[test]
fn optional_feature_string_facade_reexports_semantic_helpers() {
    assert!(
        STRING_ROOT.contains("use dependency::{")
            && STRING_ROOT.contains("use feature::{")
            && STRING_ROOT.contains("use module::{"),
        "string parent should expose semantic field helpers through child re-exports"
    );
}
