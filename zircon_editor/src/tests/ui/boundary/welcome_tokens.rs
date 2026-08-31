const WELCOME_TEMPLATE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/ui/editor/welcome.zui"
));
const WELCOME_PROJECTION: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/ui/layouts/views/welcome.rs"
));

#[test]
fn welcome_project_entry_uses_the_shared_editor_control_scale() {
    for required in [
        "res://ui/editor/theme/editor_tokens.zui",
        "$editor.control.border_width",
        "$editor.control.radius.control",
        "$editor.control.height.compact",
        "$editor.control.height.default",
        "$editor.control.height.large",
        "$editor.typography.caption.size",
        "$editor.density.gap.medium",
        "$editor.density.panel_padding",
    ] {
        assert!(
            WELCOME_TEMPLATE.contains(required),
            "Welcome project entry must consume `{required}`"
        );
    }

    for local_primitive in [
        "border_width = 1.0",
        "corner_radius = 4.0",
        "font_size = 10.666667",
        "height = 30.0",
        "height = 32.0",
        "height = 44.0",
        "gap = 10.0",
        "left = 28.0, right = 28.0",
    ] {
        assert!(
            !WELCOME_TEMPLATE.contains(local_primitive),
            "Welcome project entry must not retain local control primitive `{local_primitive}`"
        );
    }
}

#[test]
fn welcome_projection_uses_the_document_import_graph() {
    for required in ["WELCOME_LAYOUT_ASSET_PATH", "&[]"] {
        assert!(
            WELCOME_PROJECTION.contains(required),
            "Welcome projection must retain `{required}`"
        );
    }
    for legacy_source in [
        "editor_base.zui",
        "editor_material.zui",
        "editor_tokens.zui",
    ] {
        assert!(
            !WELCOME_PROJECTION.contains(legacy_source),
            "Welcome projection must not inject `{legacy_source}`"
        );
    }
}

#[test]
fn welcome_project_entry_uses_weighted_responsive_columns() {
    for layout in [
        "width = { min = 136.0, preferred = 184.0, max = 240.0, weight = 1.0, stretch = \"Stretch\" }",
        "width = { min = 280.0, preferred = 560.0, max = 760.0, weight = 4.0, stretch = \"Stretch\" }",
    ] {
        assert!(
            WELCOME_TEMPLATE.contains(layout),
            "Welcome project entry must declare responsive column `{layout}`"
        );
    }

    for fixed_layout in [
        "width = { min = 220.0, preferred = 320.0, max = 320.0, stretch = \"Stretch\" }",
        "width = { min = 280.0, stretch = \"Stretch\" }",
    ] {
        assert!(
            !WELCOME_TEMPLATE.contains(fixed_layout),
            "Welcome project entry must not retain fixed column `{fixed_layout}`"
        );
    }
}
