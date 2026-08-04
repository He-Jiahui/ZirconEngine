const COMPONENT_SHOWCASE_TEMPLATE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/ui/editor/component_showcase.zui"
));

const COMPONENT_SHOWCASE_CHILD_TEMPLATES: &[(&str, &str)] = &[
    (
        "bottom log",
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/ui/editor/components/showcase/showcase_bottom_log.zui"
        )),
    ),
    (
        "category navigation",
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/ui/editor/components/showcase/showcase_category_nav.zui"
        )),
    ),
    (
        "collections section",
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/ui/editor/components/showcase/showcase_collections_section.zui"
        )),
    ),
    (
        "command toolbar",
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/ui/editor/components/showcase/showcase_command_toolbar.zui"
        )),
    ),
    (
        "input section",
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/ui/editor/components/showcase/showcase_input_section.zui"
        )),
    ),
    (
        "selection section",
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/ui/editor/components/showcase/showcase_selection_section.zui"
        )),
    ),
    (
        "state panel",
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/ui/editor/components/showcase/showcase_state_panel.zui"
        )),
    ),
    (
        "visual section",
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/ui/editor/components/showcase/showcase_visual_section.zui"
        )),
    ),
];

#[test]
fn component_showcase_root_uses_the_workbench_theme_tokens() {
    for token in [
        "res://ui/theme/editor_base.zui",
        "res://ui/editor/theme/editor_tokens.zui",
        "$editor.surface.0",
        "$editor.surface.2",
        "$editor.text.primary",
        "$editor.border",
        "$editor.control.border_width",
        "$editor.control.radius.control",
        "$editor.control.radius.panel",
        "$editor.typography.body.size",
        "$editor.typography.emphasis.weight",
        "$editor.density.gap.medium",
    ] {
        assert!(
            COMPONENT_SHOWCASE_TEMPLATE.contains(token),
            "component showcase root must consume `{token}`"
        );
    }

    for local_value in [
        "surface = \"#101418\"",
        "surface_panel = \"#202830\"",
        "surface_inset = \"#12181e\"",
        "outline = \"#4b626d\"",
        "text_primary = \"#e6f1f4\"",
        "primary = \"#35c7d0\"",
        "radius = 10.0",
        "radius = 12.0",
        "border_width = 1.0",
        "font_size = 13.0",
        "font_weight = 700",
        "gap = 10.0",
    ] {
        assert!(
            !COMPONENT_SHOWCASE_TEMPLATE.contains(local_value),
            "component showcase root must not retain local theme value `{local_value}`"
        );
    }
}

#[test]
fn component_showcase_children_share_the_editor_control_scale() {
    for (name, template) in COMPONENT_SHOWCASE_CHILD_TEMPLATES {
        assert!(
            template.contains("res://ui/editor/theme/editor_tokens.zui"),
            "component showcase {name} must import editor tokens"
        );

        for legacy_value in [
            "styles = []",
            "font_size = 11.0",
            "font_size = 13.0",
            "font_weight = 700",
            "corner_radius = 10.0",
            "border_width = 1.0",
            "layout_padding_left = 10.0",
            "layout_padding_right = 10.0",
            "layout_padding_left = 12.0",
            "layout_padding_right = 12.0",
            "layout_padding_top = 3.0",
            "layout_padding_bottom = 3.0",
            "layout_padding_top = 5.0",
            "layout_padding_bottom = 5.0",
            "layout_padding_top = 6.0",
            "layout_padding_bottom = 6.0",
            "layout_spacing = 6.0",
            "layout_min_width = 32.0",
            "layout_min_height = 32.0",
            "gap = 6.0",
            "gap = 8.0",
        ] {
            assert!(
                !template.contains(legacy_value),
                "component showcase {name} must not retain local primitive `{legacy_value}`"
            );
        }
    }

    for (name, required_tokens) in [
        (
            "bottom log",
            &[
                "$editor.typography.caption.size",
                "$editor.density.gap.medium",
                "$editor.density.gap.xsmall",
            ][..],
        ),
        (
            "category navigation",
            &[
                "$editor.density.gap.large",
                "$editor.density.gap.small",
                "$editor.control.height.default",
                "$editor.control.height.dense",
            ][..],
        ),
        (
            "command toolbar",
            &[
                "$editor.typography.caption.size",
                "$editor.density.toolbar_action_width",
                "$editor.control.height.dense",
            ][..],
        ),
        (
            "input section",
            &[
                "$editor.typography.body.size",
                "$editor.typography.emphasis.weight",
                "$editor.control.radius.control",
                "$editor.control.border_width",
            ][..],
        ),
        (
            "collections section",
            &[
                "$editor.typography.body.size",
                "$editor.typography.emphasis.weight",
                "$editor.density.gap.medium",
            ][..],
        ),
        (
            "selection section",
            &[
                "$editor.typography.body.size",
                "$editor.typography.emphasis.weight",
                "$editor.density.gap.xsmall",
            ][..],
        ),
        ("state panel", &["$editor.density.gap.small"][..]),
        (
            "visual section",
            &[
                "$editor.typography.body.size",
                "$editor.typography.emphasis.weight",
            ][..],
        ),
    ] {
        let (_, template) = COMPONENT_SHOWCASE_CHILD_TEMPLATES
            .iter()
            .find(|(child_name, _)| *child_name == name)
            .expect("showcase component contract must name a known child");

        for token in required_tokens {
            assert!(
                template.contains(token),
                "component showcase {name} must consume `{token}`"
            );
        }
    }
}

#[test]
fn component_showcase_owns_tokenized_control_state_layers() {
    for selector in [
        ".showcase-field",
        ".showcase-title",
        ".showcase-muted",
        ".showcase-toolbar",
        ".showcase-bottom-log",
        ".unreal-panel",
        ".unreal-command",
        ".unreal-accent",
        ".material-control:hover",
        ".material-control:pressed",
        ".material-control:focus",
        ".material-control:selected",
        ".material-control:disabled",
    ] {
        assert!(
            COMPONENT_SHOWCASE_TEMPLATE.contains(&format!("selector = \"{selector}\"")),
            "component showcase must define the `{selector}` state layer"
        );
    }

    for token in [
        "$editor.surface.1",
        "$editor.surface.3",
        "$editor.surface.recessed",
        "$editor.surface.hover",
        "$editor.surface.selected",
        "$editor.surface.disabled",
        "$editor.text.secondary",
        "$editor.text.disabled",
        "$editor.border.disabled",
        "$editor.focus.ring",
        "$editor.control.radius.small",
        "$editor.typography.caption.size",
        "$editor.typography.title.size",
    ] {
        assert!(
            COMPONENT_SHOWCASE_TEMPLATE.contains(token),
            "component showcase control state layers must consume `{token}`"
        );
    }
}

#[test]
fn component_showcase_body_uses_weighted_responsive_slots() {
    for slot in [
        "node = \"component_showcase_nav\", slot = { layout = { width = { min = 120.0, preferred = 180.0, max = 220.0, weight = 1.0, stretch = \"Stretch\" } } }",
        "node = \"component_showcase_content\", slot = { layout = { width = { min = \"$editor.control.height.large\", preferred = \"$editor.control.height.default\", weight = 4.0, stretch = \"Stretch\" } } }",
        "node = \"component_showcase_state_panel\", slot = { layout = { width = { min = 160.0, preferred = 260.0, max = 320.0, weight = 1.0, stretch = \"Stretch\" } } }",
    ] {
        assert!(
            COMPONENT_SHOWCASE_TEMPLATE.contains(slot),
            "component showcase body must declare responsive slot `{slot}`"
        );
    }

    for fixed_slot in [
        "min = 180.0, preferred = 180.0, max = 180.0, stretch = \"Fixed\"",
        "min = 260.0, preferred = 260.0, max = 260.0, stretch = \"Fixed\"",
    ] {
        assert!(
            !COMPONENT_SHOWCASE_TEMPLATE.contains(fixed_slot),
            "component showcase body must not retain fixed side slot `{fixed_slot}`"
        );
    }
}
