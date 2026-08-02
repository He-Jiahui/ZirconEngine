use crate::ui::skin::{
    UiColorScheme, UiComponentVisualState, UiDesignPresetDescriptor, UiDesignPresetKind,
    UiDesignReference, UiSemanticTokenFamily, FYROX_PANEL_PRESET_ID, JETBRAINS_SHELL_PRESET_ID,
    MATERIAL_DARK_SKIN_ID, UNREAL_WINDOW_MODEL_PRESET_ID,
};

#[test]
fn material_dark_skin_exposes_required_token_families_and_states() {
    let preset = UiDesignPresetDescriptor::material_dark();

    assert_eq!(preset.id, MATERIAL_DARK_SKIN_ID);
    assert_eq!(preset.kind, UiDesignPresetKind::Skin);
    assert_eq!(preset.default_color_scheme, Some(UiColorScheme::Dark));
    assert!(preset.has_reference(UiDesignReference::MaterialUi));
    assert!(preset.has_reference(UiDesignReference::MaterialComponents));
    assert!(preset.has_reference(UiDesignReference::SlintMaterial));

    for state in [
        UiComponentVisualState::Normal,
        UiComponentVisualState::Hover,
        UiComponentVisualState::Pressed,
        UiComponentVisualState::Selected,
        UiComponentVisualState::Disabled,
        UiComponentVisualState::Focused,
        UiComponentVisualState::Error,
        UiComponentVisualState::Warning,
    ] {
        assert!(preset.has_visual_state(state), "missing {state:?}");
    }

    for (name, family) in [
        ("palette.primary.main", UiSemanticTokenFamily::Palette),
        ("text.primary", UiSemanticTokenFamily::Text),
        ("action.hover", UiSemanticTokenFamily::Action),
        ("surface.panel", UiSemanticTokenFamily::Surface),
        ("divider.default", UiSemanticTokenFamily::Divider),
        ("focus.ring", UiSemanticTokenFamily::Focus),
        ("elevation.1", UiSemanticTokenFamily::Elevation),
        ("radius.sm", UiSemanticTokenFamily::Radius),
        ("spacing.2", UiSemanticTokenFamily::Spacing),
        ("typography.body.size", UiSemanticTokenFamily::Typography),
        ("icon.size.md", UiSemanticTokenFamily::IconSize),
    ] {
        let token = preset
            .token(name)
            .unwrap_or_else(|| panic!("missing {name}"));
        assert_eq!(token.family, family, "token family mismatch for {name}");
    }
}

#[test]
fn editor_ui_reference_presets_have_stable_ids_and_roles() {
    let fyrox = UiDesignPresetDescriptor::fyrox_panel();
    let jetbrains = UiDesignPresetDescriptor::jetbrains_shell();
    let unreal = UiDesignPresetDescriptor::unreal_window_model();

    assert_eq!(fyrox.id, FYROX_PANEL_PRESET_ID);
    assert_eq!(fyrox.kind, UiDesignPresetKind::Panel);
    assert!(fyrox.has_reference(UiDesignReference::FyroxEditor));
    assert!(fyrox.component_roles.iter().any(|role| role == "inspector"));
    assert!(fyrox
        .component_roles
        .iter()
        .any(|role| role == "asset_browser"));

    assert_eq!(jetbrains.id, JETBRAINS_SHELL_PRESET_ID);
    assert_eq!(jetbrains.kind, UiDesignPresetKind::Shell);
    assert!(jetbrains.has_reference(UiDesignReference::JetBrainsIde));
    assert!(jetbrains
        .component_roles
        .iter()
        .any(|role| role == "side_drawer"));

    assert_eq!(unreal.id, UNREAL_WINDOW_MODEL_PRESET_ID);
    assert_eq!(unreal.kind, UiDesignPresetKind::WindowModel);
    assert!(unreal.has_reference(UiDesignReference::UnrealEditor));
    assert!(unreal
        .component_roles
        .iter()
        .any(|role| role == "asset_editor_window"));
}
