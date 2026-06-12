use super::super::super::super::sources::*;

#[test]
fn optional_feature_parser_pending_module_facade_stays_structural() {
    assert!(
        PARSER_PENDING_MODULE.contains("mod append;")
            && PARSER_PENDING_MODULE.contains("mod entry;")
            && PARSER_PENDING_MODULE.contains("mod signature;"),
        "pending module parent must remain a structural child-module owner"
    );
    assert!(
        !PARSER_PENDING_MODULE.contains("fn push_optional_feature_module")
            && !PARSER_PENDING_MODULE.contains("take_optional_feature_module")
            && !PARSER_PENDING_MODULE.contains("append_optional_feature_module")
            && !PARSER_PENDING_MODULE.contains("RuntimeTargetMode")
            && !PARSER_PENDING_MODULE.contains("PluginModuleKind"),
        "pending module parent must not own finalizer composition or module state signature"
    );
    assert!(
        PARSER_PENDING_MODULE.contains("use entry::push_optional_feature_module"),
        "pending module parent should expose the entry child through a re-export"
    );
}
