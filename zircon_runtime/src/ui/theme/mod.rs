use std::hash::{Hash, Hasher};

use zircon_runtime_interface::ui::style::{UiStyleColor, UiThemeDocument, UiThemeTokenRef};

#[derive(Clone, Debug, PartialEq)]
pub struct UiThemeRegistry {
    active: UiThemeDocument,
    fingerprint: u64,
}

impl Default for UiThemeRegistry {
    fn default() -> Self {
        Self::new(UiThemeDocument::dark())
    }
}

impl UiThemeRegistry {
    pub fn new(active: UiThemeDocument) -> Self {
        let fingerprint = theme_fingerprint(&active);
        Self {
            active,
            fingerprint,
        }
    }

    pub fn active(&self) -> &UiThemeDocument {
        &self.active
    }

    pub fn fingerprint(&self) -> u64 {
        self.fingerprint
    }

    pub fn resolve_token(&self, token: &UiThemeTokenRef) -> Option<UiStyleColor> {
        let color = match token.as_str() {
            "palette.surface.0" => self.active.palette.surface[0],
            "palette.surface.1" => self.active.palette.surface[1],
            "palette.surface.2" => self.active.palette.surface[2],
            "palette.surface.3" => self.active.palette.surface[3],
            "palette.text.primary" => self.active.palette.text_primary,
            "palette.text.secondary" => self.active.palette.text_secondary,
            "palette.text.disabled" => self.active.palette.text_disabled,
            "palette.accent" => self.active.palette.accent,
            "palette.success" => self.active.palette.success,
            "palette.info" => self.active.palette.info,
            "palette.warning" => self.active.palette.warning,
            "palette.error" => self.active.palette.error,
            "palette.separator" => self.active.palette.separator,
            _ => return None,
        };
        Some(UiStyleColor::Rgba(color))
    }

    pub fn resolve_role(&self, role: &str) -> Option<UiStyleColor> {
        let token = normalized_theme_role(role)?;
        self.resolve_token(&UiThemeTokenRef::new(token))
    }

    pub fn resolve_style_color(&self, color: &UiStyleColor) -> UiStyleColor {
        match color {
            UiStyleColor::Role(role) => self
                .resolve_role(role)
                .unwrap_or_else(|| UiStyleColor::Role(role.clone())),
            value => value.clone(),
        }
    }

    pub fn apply_document(&mut self, document: UiThemeDocument) -> UiThemeReloadOutcome {
        let previous_fingerprint = self.fingerprint;
        let new_fingerprint = theme_fingerprint(&document);
        self.active = document;
        self.fingerprint = new_fingerprint;
        UiThemeReloadOutcome {
            previous_fingerprint,
            new_fingerprint,
            changed: previous_fingerprint != new_fingerprint,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UiThemeReloadOutcome {
    pub previous_fingerprint: u64,
    pub new_fingerprint: u64,
    pub changed: bool,
}

fn normalized_theme_role(role: &str) -> Option<&str> {
    let trimmed = role.trim();
    if trimmed.is_empty() {
        return None;
    }
    let trimmed = trimmed.strip_prefix('$').unwrap_or(trimmed);
    Some(
        trimmed
            .strip_prefix("theme.")
            .or_else(|| trimmed.strip_prefix("theme:"))
            .unwrap_or(trimmed),
    )
}

fn theme_fingerprint(document: &UiThemeDocument) -> u64 {
    let serialized = serde_json::to_string(document).unwrap_or_else(|_| document.id.clone());
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    serialized.hash(&mut hasher);
    hasher.finish()
}
