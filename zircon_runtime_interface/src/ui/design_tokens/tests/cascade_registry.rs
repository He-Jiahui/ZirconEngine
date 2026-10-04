use super::canonical_cascade_token_values;
use crate::ui::design_tokens::EditorDesignTokens;

#[test]
fn workbench_chrome_metrics_are_registered_as_logical_float_tokens() {
    let values = canonical_cascade_token_values(&EditorDesignTokens::workbench_dark());

    for (name, expected) in [
        ("editor.chrome.top_bar.height", 25.0),
        ("editor.chrome.host_bar.height", 32.0),
        ("editor.chrome.workbench_toolbar.height", 42.0),
        ("editor.chrome.workbench_toolbar.command_row.height", 42.0),
        (
            "editor.chrome.workbench_toolbar.popup.command_offset_y",
            29.0,
        ),
        (
            "editor.chrome.workbench_toolbar.popup.module_offset_y",
            -6.0,
        ),
        ("editor.chrome.status_bar.height", 24.0),
        ("editor.chrome.panel_header.height", 30.0),
        ("editor.chrome.document_header.height", 31.0),
        ("editor.chrome.viewport_toolbar.height", 28.0),
        ("editor.chrome.activity_rail.width", 34.0),
        ("editor.chrome.separator.thickness", 1.0),
        ("editor.chrome.splitter.hit_size", 8.0),
    ] {
        assert_eq!(
            values.get(name).and_then(toml::Value::as_float),
            Some(expected),
            "missing or invalid Workbench chrome token `{name}`"
        );
    }
}

#[test]
fn workbench_dark_palette_matches_approved_editor_theme_baseline() {
    let values = canonical_cascade_token_values(&EditorDesignTokens::workbench_dark());

    for (name, expected) in [
        ("editor.surface.0", "#151515"),
        ("editor.surface.1", "#242424"),
        ("editor.surface.2", "#2f2f2f"),
        ("editor.surface.3", "#383838"),
        ("editor.surface.recessed", "#1a1a1a"),
        ("editor.surface.input", "#0f0f0f"),
        ("editor.surface.tab_hover", "#242424cc"),
        ("editor.text.tab_active", "#ffffff"),
        ("editor.surface.hover", "#575757"),
        ("editor.surface.selected", "#0070e0"),
        ("editor.accent", "#0070e0"),
        ("editor.border", "#383838"),
        ("editor.separator.strong", "#575757"),
        ("editor.separator.soft", "#383838"),
        ("editor.text.primary", "#c0c0c0"),
        ("editor.text.secondary", "#c0c0c0"),
        ("editor.text.disabled", "#808080"),
        ("editor.popup", "#383838"),
        ("editor.track", "#0f0f0f"),
        ("editor.focus.ring", "#0070e0"),
    ] {
        assert_eq!(
            values.get(name).and_then(toml::Value::as_str),
            Some(expected),
            "palette token `{name}` diverged from the approved editor theme baseline"
        );
    }
}
