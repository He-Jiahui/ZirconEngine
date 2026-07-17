use zircon_runtime_interface::ui::design_tokens::{
    EditorControlTokens, EditorDensityTokens, EditorDesignTokens, EditorPaletteTokens,
    EditorStateRoleTokens, EditorTypographyTokens,
};

use super::persistence::{EditorAppearancePreferencesDocument, APPEARANCE_PREFERENCES_VERSION};
use super::typography_migration::migrate_legacy_workbench_typography;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct EditorAppearancePreferences {
    design_tokens: EditorDesignTokens,
}

impl Default for EditorAppearancePreferences {
    fn default() -> Self {
        Self::workbench_default()
    }
}

impl EditorAppearancePreferences {
    pub(crate) fn workbench_default() -> Self {
        Self {
            design_tokens: EditorDesignTokens::workbench_dark(),
        }
    }

    pub(crate) fn from_design_tokens(design_tokens: EditorDesignTokens) -> Self {
        Self { design_tokens }
    }

    pub(crate) fn with_typography(mut self, typography: EditorTypographyTokens) -> Self {
        self.design_tokens.typography = typography;
        self
    }

    pub(crate) fn with_palette(mut self, palette: EditorPaletteTokens) -> Self {
        self.design_tokens.palette = palette;
        self
    }

    pub(crate) fn with_controls(mut self, controls: EditorControlTokens) -> Self {
        self.design_tokens.controls = controls;
        self
    }

    pub(crate) fn with_density(mut self, density: EditorDensityTokens) -> Self {
        self.design_tokens.density = density;
        self
    }

    pub(crate) fn with_state_roles(mut self, state_roles: EditorStateRoleTokens) -> Self {
        self.design_tokens.state_roles = state_roles;
        self
    }

    pub(crate) fn design_tokens(&self) -> &EditorDesignTokens {
        &self.design_tokens
    }

    pub(super) fn to_persistence_document(&self) -> EditorAppearancePreferencesDocument {
        EditorAppearancePreferencesDocument {
            version: APPEARANCE_PREFERENCES_VERSION,
            active_profile: self.design_tokens.id.clone(),
            design_tokens: self.design_tokens.clone(),
        }
    }

    pub(super) fn from_persistence_document(document: EditorAppearancePreferencesDocument) -> Self {
        match document.version {
            APPEARANCE_PREFERENCES_VERSION => Self::from_design_tokens(document.design_tokens),
            1 => {
                let mut design_tokens = document.design_tokens;
                migrate_legacy_workbench_typography(&mut design_tokens.typography);
                Self::from_design_tokens(design_tokens)
            }
            _ => Self::default(),
        }
    }
}
