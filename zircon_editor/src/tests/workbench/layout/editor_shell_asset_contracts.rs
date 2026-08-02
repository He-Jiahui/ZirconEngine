const EDITOR_TOKENS_ASSET: &str =
    include_str!("../../../../assets/ui/editor/theme/editor_tokens.zui");
const WORKBENCH_SKELETON_ASSET: &str =
    include_str!("../../../../assets/ui/editor/components/workbench/shell/workbench_skeleton.zui");
const WORKBENCH_STATUS_BAR_ASSET: &str = include_str!(
    "../../../../assets/ui/editor/components/workbench/shell/workbench_status_bar.zui"
);
const WORKBENCH_TOP_TOOLBAR_ASSET: &str = include_str!(
    "../../../../assets/ui/editor/components/workbench/shell/workbench_top_toolbar.zui"
);
const WORKBENCH_COMPONENT_DRAWER_ASSET: &str = include_str!(
    "../../../../assets/ui/editor/components/workbench/shell/workbench_component_drawer.zui"
);
const WORKBENCH_MAIN_BAND_ASSET: &str =
    include_str!("../../../../assets/ui/editor/components/workbench/shell/workbench_main_band.zui");
const WORKBENCH_SCENE_TREE_PANEL_ASSET: &str = include_str!(
    "../../../../assets/ui/editor/components/workbench/shell/workbench_scene_tree_panel.zui"
);
const WORKBENCH_INSPECTOR_PANEL_ASSET: &str = include_str!(
    "../../../../assets/ui/editor/components/workbench/shell/workbench_inspector_panel.zui"
);
const COMMAND_PALETTE_ASSET: &str = include_str!(
    "../../../../assets/ui/editor/components/workbench/floating/workbench_command_palette.zui"
);
const PREFERENCES_ASSET: &str = include_str!(
    "../../../../assets/ui/editor/components/workbench/floating/workbench_preferences.zui"
);
const WORKBENCH_VIEWPORT_PANEL_ASSET: &str = include_str!(
    "../../../../assets/ui/editor/components/workbench/shell/workbench_viewport_panel.zui"
);

#[test]
fn layout_shell_and_floating_assets_reference_editor_tokens_instead_of_hex_colors() {
    assert!(EDITOR_TOKENS_ASSET.contains("editor.surface.0"));
    assert!(EDITOR_TOKENS_ASSET.contains("editor.control.height.default"));
    assert!(EDITOR_TOKENS_ASSET.contains("editor.control.radius.pill"));
    assert!(
        EDITOR_TOKENS_ASSET.contains("editor.density.toolbar_action_width")
            && EDITOR_TOKENS_ASSET.contains("editor.density.toolbar_wide_action_width"),
        "editor token asset must name both toolbar action width tokens"
    );
    assert!(EDITOR_TOKENS_ASSET.contains("--left-drawer-width"));
    let typography_tokens = EDITOR_TOKENS_ASSET
        .split("[typography]")
        .nth(1)
        .and_then(|section| section.split("[controls]").next())
        .expect("editor token asset must contain typography and controls sections");
    assert!(
        typography_tokens.contains("font_smoothing = \"grayscale\""),
        "the typography asset must mirror the central font smoothing token"
    );

    for (asset_name, asset_source, required_tokens) in [
        ("workbench_skeleton.zui", WORKBENCH_SKELETON_ASSET, &[][..]),
        (
            "workbench_command_palette.zui",
            COMMAND_PALETTE_ASSET,
            &[
                "$editor.control.border_width",
                "$editor.control.radius.large",
            ][..],
        ),
        (
            "workbench_preferences.zui",
            PREFERENCES_ASSET,
            &[
                "$editor.control.border_width",
                "$editor.control.radius.large",
            ][..],
        ),
        (
            "workbench_status_bar.zui",
            WORKBENCH_STATUS_BAR_ASSET,
            &[
                "$editor.text.secondary",
                "$editor.semantic.success",
                "$editor.semantic.warning",
                "$editor.semantic.info",
                "$editor.density.gap.medium",
                "$editor.control.height.default",
                "$editor.control.height.dense",
            ][..],
        ),
        (
            "workbench_top_toolbar.zui",
            WORKBENCH_TOP_TOOLBAR_ASSET,
            &[
                "$editor.density.gap.small",
                "$editor.control.height.compact",
                "$editor.control.height.default",
                "$editor.control.border_width",
            ][..],
        ),
        (
            "workbench_component_drawer.zui",
            WORKBENCH_COMPONENT_DRAWER_ASSET,
            &[
                "$editor.accent",
                "$editor.accent.soft",
                "$editor.surface.recessed",
                "$editor.surface.selected",
                "$editor.text.primary",
                "$editor.text.secondary",
                "$editor.text.disabled",
                "$editor.semantic.error",
                "$editor.separator.soft",
                "$editor.density.gap.small",
                "$editor.density.gap.medium",
                "$editor.density.gap.large",
                "$editor.control.height.default",
                "$editor.control.height.dense",
                "$editor.control.radius.small",
                "$editor.control.border_width",
            ][..],
        ),
        (
            "workbench_inspector_panel.zui",
            WORKBENCH_INSPECTOR_PANEL_ASSET,
            &[
                "$editor.density.right_drawer_width",
                "$editor.density.gap.small",
                "$editor.density.gap.large",
                "$editor.typography.body.size",
                "$editor.typography.caption.size",
                "$editor.text.disabled",
                "$editor.text.secondary",
                "$editor.text.primary",
                "$editor.surface.0",
                "$editor.separator.soft",
                "$editor.control.height.default",
                "$editor.control.height.dense",
            ][..],
        ),
        (
            "workbench_scene_tree_panel.zui",
            WORKBENCH_SCENE_TREE_PANEL_ASSET,
            &[
                "$editor.density.left_drawer_width",
                "$editor.density.gap.medium",
                "$editor.control.height.default",
            ][..],
        ),
        (
            "workbench_main_band.zui",
            WORKBENCH_MAIN_BAND_ASSET,
            &[
                "$editor.density.left_drawer_width",
                "$editor.density.right_drawer_width",
            ][..],
        ),
    ] {
        assert!(
            asset_source.contains("res://ui/editor/theme/editor_tokens.zui"),
            "{asset_name} must import the editor token asset"
        );
        assert!(
            asset_source.contains("$editor.surface.")
                || asset_source.contains("$editor.text.")
                || asset_source.contains("$editor.border")
                || asset_source.contains("$editor.density."),
            "{asset_name} must use the canonical $editor token reference grammar"
        );
        for token in required_tokens {
            assert!(
                asset_source.contains(token),
                "{asset_name} must reference {token} instead of a raw control metric"
            );
        }
        assert!(
            !contains_hex_color(asset_source),
            "{asset_name} must not reintroduce naked hex colors"
        );
    }
}

#[test]
fn shell_drawer_assets_use_canonical_design_tokens_instead_of_inline_drawer_widths() {
    for (asset_name, asset_source, token_name, removed_width) in [
        (
            "workbench_main_band.zui",
            WORKBENCH_MAIN_BAND_ASSET,
            "$editor.density.left_drawer_width",
            "332.0",
        ),
        (
            "workbench_main_band.zui",
            WORKBENCH_MAIN_BAND_ASSET,
            "$editor.density.right_drawer_width",
            "404.0",
        ),
        (
            "workbench_scene_tree_panel.zui",
            WORKBENCH_SCENE_TREE_PANEL_ASSET,
            "$editor.density.left_drawer_width",
            "332.0",
        ),
        (
            "workbench_inspector_panel.zui",
            WORKBENCH_INSPECTOR_PANEL_ASSET,
            "$editor.density.right_drawer_width",
            "404.0",
        ),
    ] {
        assert!(
            asset_source.contains("res://ui/editor/theme/editor_tokens.zui"),
            "{asset_name} must import the editor token asset"
        );
        assert!(
            asset_source.contains(token_name),
            "{asset_name} must reference {token_name}"
        );
        assert!(
            !asset_source.contains(removed_width),
            "{asset_name} must not keep the old inline drawer width {removed_width}"
        );
    }

    assert!(
        !WORKBENCH_MAIN_BAND_ASSET.contains("$--"),
        "workbench_main_band.zui must not use the retired $-- token grammar"
    );
    assert!(
        !WORKBENCH_SCENE_TREE_PANEL_ASSET.contains("$--"),
        "workbench_scene_tree_panel.zui must not use the retired $-- token grammar"
    );
    assert!(
        !WORKBENCH_INSPECTOR_PANEL_ASSET.contains("$--"),
        "workbench_inspector_panel.zui must not use the retired $-- token grammar"
    );
}

#[test]
fn viewport_overlay_text_uses_central_typography_tokens() {
    assert!(
        WORKBENCH_VIEWPORT_PANEL_ASSET.contains("res://ui/editor/theme/editor_tokens.zui"),
        "workbench_viewport_panel.zui must import the editor token asset"
    );
    for token in [
        "$editor.typography.overlay.size",
        "$editor.typography.emphasis.weight",
    ] {
        assert!(
            WORKBENCH_VIEWPORT_PANEL_ASSET.contains(token),
            "workbench_viewport_panel.zui must reference {token}"
        );
    }
    assert!(
        !WORKBENCH_VIEWPORT_PANEL_ASSET.contains("font_size = 12.0"),
        "viewport overlay labels must not keep a local font size"
    );
    assert!(
        !WORKBENCH_VIEWPORT_PANEL_ASSET.contains("font_weight = 700"),
        "viewport overlay labels must not keep a local font weight"
    );
}

fn contains_hex_color(source: &str) -> bool {
    source
        .as_bytes()
        .windows(7)
        .any(|window| window[0] == b'#' && window[1..].iter().all(u8::is_ascii_hexdigit))
}
