const CONSOLE_BODY_TEMPLATE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/ui/editor/host/console_body.zui"
));

#[test]
fn console_body_owns_the_tokens_used_by_its_runtime_text_and_filter_controls() {
    for required in [
        "styles = [\"res://ui/editor/theme/editor_tokens.zui\"]",
        "$editor.typography.caption.size",
        "$editor.density.gap.small",
        "ConsolePaneBody/FilterAll",
        "ConsolePaneBody/FilterError",
        "ConsolePaneBody/FilterWarning",
        "ConsolePaneBody/FilterInfo",
        "ConsolePaneBody/SourceAll",
        "ConsolePaneBody/SourceEditor",
        "ConsolePaneBody/SourceRuntime",
        "ConsolePaneBody/SourcePlay",
        "ConsolePaneBody/SourcePlugin",
        "ConsolePaneBody/SourceImport",
        "ConsolePaneBody/SourceScriptBuild",
        "ConsolePaneBody/ClearConsole",
    ] {
        assert!(
            CONSOLE_BODY_TEMPLATE.contains(required),
            "Console body must preserve `{required}`"
        );
    }
}
