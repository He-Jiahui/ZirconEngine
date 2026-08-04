const PROJECT_OVERVIEW_TEMPLATE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/ui/editor/project_overview.zui"
));
const PROJECT_OVERVIEW_PROJECTION: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/ui/layouts/views/project_overview.rs"
));

#[test]
fn project_overview_uses_the_shared_editor_surface_and_control_tokens() {
    for required in [
        "res://ui/editor/theme/editor_tokens.zui",
        "$editor.control.border_width",
        "$editor.control.radius.panel",
        "$editor.control.radius.small",
        "$editor.typography.strong.weight",
        "$editor.density.gap.small",
        "$editor.density.gap.medium",
        "$editor.density.gap.large",
        "$editor.density.row_height",
    ] {
        assert!(
            PROJECT_OVERVIEW_TEMPLATE.contains(required),
            "Project Overview must consume `{required}`"
        );
    }

    for local_primitive in [
        "radius = 4.0",
        "border_width = 1.0",
        "font_weight = 600",
        "gap = 4.0",
        "gap = 8.0",
        "min = 28.0, preferred = 28.0, max = 28.0",
        "min = 12.0, preferred = 12.0, max = 12.0",
    ] {
        assert!(
            !PROJECT_OVERVIEW_TEMPLATE.contains(local_primitive),
            "Project Overview must not retain local primitive `{local_primitive}`"
        );
    }
}

#[test]
fn project_overview_projection_loads_its_declared_theme_assets() {
    for required in [
        "PROJECT_OVERVIEW_STYLE_ASSET_PATH",
        "PROJECT_OVERVIEW_STYLE_ASSET_ID",
        "PROJECT_OVERVIEW_MATERIAL_STYLE_ASSET_PATH",
        "PROJECT_OVERVIEW_MATERIAL_STYLE_ASSET_ID",
        "PROJECT_OVERVIEW_TOKENS_STYLE_ASSET_PATH",
        "PROJECT_OVERVIEW_TOKENS_STYLE_ASSET_ID",
        "/assets/ui/theme/editor_base.zui",
        "/assets/ui/theme/editor_material.zui",
        "/assets/ui/editor/theme/editor_tokens.zui",
    ] {
        assert!(
            PROJECT_OVERVIEW_PROJECTION.contains(required),
            "Project Overview projection must resolve `{required}`"
        );
    }
}
