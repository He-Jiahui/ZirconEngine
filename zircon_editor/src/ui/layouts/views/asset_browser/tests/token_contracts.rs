const ASSET_BROWSER_LAYOUT_TOML: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/ui/editor/asset_browser.zui"
));

#[test]
fn asset_browser_standard_primitives_use_central_design_tokens() {
    for token in [
        "$editor.control.border_width",
        "$editor.control.height.compact",
        "$editor.control.height.dense",
        "$editor.control.radius.large",
        "$editor.control.radius.small",
        "$editor.density.gap.small",
        "$editor.density.gap.medium",
        "$editor.density.row_height",
        "$editor.typography.emphasis.weight",
        "$editor.typography.strong.weight",
    ] {
        assert!(
            ASSET_BROWSER_LAYOUT_TOML.contains(token),
            "asset browser should reference the central token `{token}`"
        );
    }
    for raw_value in [
        "font_weight = 600",
        "font_weight = 700",
        "radius = 4.0",
        "radius = 6.0",
        "radius = 8.0",
        "border_width = 1.0",
        "gap = 4.0",
        "gap = 6.0",
        "gap = 8.0",
        "gap = 10.0",
        "width = { min = 30.0, preferred = 30.0, max = 30.0",
        "height = { min = 28.0, preferred = 28.0, max = 28.0",
        "height = { min = 30.0, preferred = 30.0, max = 30.0",
    ] {
        assert!(
            !ASSET_BROWSER_LAYOUT_TOML.contains(raw_value),
            "asset browser must not retain a local primitive metric `{raw_value}`"
        );
    }
}
