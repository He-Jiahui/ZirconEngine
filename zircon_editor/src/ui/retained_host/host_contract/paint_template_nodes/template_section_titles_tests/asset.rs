use std::path::Path;

use zircon_runtime_interface::ui::design_tokens::EditorDesignTokens;

#[test]
fn section_title_asset_projects_runtime_text_and_unreal_header_contract() {
    let tokens = EditorDesignTokens::workbench_dark();
    assert!((tokens.typography.body_size - 13.333_333).abs() <= 0.000_01);
    assert_eq!(tokens.typography.strong_weight, 600);

    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let component = std::fs::read_to_string(manifest.join(
        "assets/ui/editor/components/workbench/primitives/chrome/workbench_section_title.zui",
    ))
    .expect("Workbench section-title primitive should be readable");
    for required in [
        "component_variant = \"section-title\"",
        "font_size = 13.333333",
        "font_weight = 600",
        "height = { min = 28.0, preferred = 28.0, max = 30.0, stretch = \"Fixed\" }",
    ] {
        assert!(
            component.contains(required),
            "missing compact section-title contract: {required}"
        );
    }
    assert!(!component.contains("background_color ="));
    assert!(!component.contains("border_color ="));

    let theme =
        std::fs::read_to_string(manifest.join("assets/ui/theme/editor_workbench_strict.zui"))
            .expect("Workbench strict theme should be readable");
    let section_rule = theme
        .split("[[stylesheets.rules]]")
        .find(|rule| rule.contains("selector = \".workbench-section-title\""))
        .expect("strict theme should own the shared section-title rule");
    for required in [
        "background_color = \"$workbench_panel_raised\"",
        "border_color = \"$workbench_border_soft\"",
        "radius = 0.0",
        "font_size = 13.333333",
        "font_weight = 600",
    ] {
        assert!(
            section_rule.contains(required),
            "missing Unreal-flat themed title contract: {required}"
        );
    }
}
