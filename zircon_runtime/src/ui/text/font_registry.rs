use std::path::Path;

use crate::asset::assets::FontAsset;
use crate::graphics::text::font::FontDatabase;
use zircon_runtime_interface::ui::surface::UiTextRenderMode;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct UiFontId(u32);

impl UiFontId {
    pub(crate) const fn new(value: u32) -> Self {
        Self(value)
    }

    pub(crate) const fn value(self) -> u32 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub(crate) enum UiFontStyle {
    #[default]
    Normal,
    Italic,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum UiFontSource {
    Asset { source: String },
    System { family: String },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct UiFontFamilyRecord {
    pub id: UiFontId,
    pub family: String,
    pub weight: u16,
    pub style: UiFontStyle,
    pub render_mode: Option<UiTextRenderMode>,
    pub source: UiFontSource,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum UiFontRegistryError {
    EmptySource,
    EmptyFamily,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct UiFontRegistry {
    families: Vec<UiFontFamilyRecord>,
    fallback_chain: Vec<String>,
    next_id: u32,
}

impl Default for UiFontRegistry {
    fn default() -> Self {
        Self {
            families: Vec::new(),
            fallback_chain: FontDatabase::with_default_fallbacks()
                .fallback_families()
                .iter()
                .map(|family| family.as_str().to_string())
                .collect(),
            next_id: 1,
        }
    }
}

impl UiFontRegistry {
    pub(crate) fn families(&self) -> &[UiFontFamilyRecord] {
        &self.families
    }

    pub(crate) fn fallback_chain(&self) -> &[String] {
        &self.fallback_chain
    }

    pub(crate) fn set_fallback_chain(&mut self, chain: Vec<String>) {
        self.fallback_chain = chain
            .into_iter()
            .filter(|family| !family.trim().is_empty())
            .collect();
    }

    pub(crate) fn register_font_asset(
        &mut self,
        asset: &FontAsset,
    ) -> Result<UiFontId, UiFontRegistryError> {
        let source = asset.source.trim();
        if source.is_empty() {
            return Err(UiFontRegistryError::EmptySource);
        }

        let family = asset
            .family
            .as_deref()
            .map(str::trim)
            .filter(|family| !family.is_empty())
            .map(str::to_string)
            .unwrap_or_else(|| family_from_source(source));
        if family.trim().is_empty() {
            return Err(UiFontRegistryError::EmptyFamily);
        }

        let id = self.allocate_id();
        self.families.push(UiFontFamilyRecord {
            id,
            family: family.clone(),
            weight: 400,
            style: UiFontStyle::Normal,
            render_mode: asset.effective_render_mode(),
            source: UiFontSource::Asset {
                source: source.to_string(),
            },
        });
        let mut fallback_families = vec![family.as_str()];
        fallback_families.extend(asset.fallback_families.iter().map(String::as_str));
        if let Some(composite) = &asset.composite_font {
            fallback_families.push(composite.default_family.as_str());
            fallback_families.extend(
                composite
                    .sub_fonts
                    .iter()
                    .map(|sub_font| sub_font.family.as_str()),
            );
        }
        self.extend_fallback_chain(fallback_families);
        Ok(id)
    }

    pub(crate) fn register_system_family(&mut self, family: impl Into<String>) -> UiFontId {
        let family = family.into();
        let id = self.allocate_id();
        self.families.push(UiFontFamilyRecord {
            id,
            family: family.clone(),
            weight: 400,
            style: UiFontStyle::Normal,
            render_mode: None,
            source: UiFontSource::System { family },
        });
        id
    }

    fn allocate_id(&mut self) -> UiFontId {
        let id = UiFontId::new(self.next_id);
        self.next_id = self.next_id.saturating_add(1).max(1);
        id
    }

    fn extend_fallback_chain<'a>(&mut self, families: impl IntoIterator<Item = &'a str>) {
        for family in families {
            let family = family.trim();
            if family.is_empty() {
                continue;
            }
            let key = normalized_family_key(family);
            if self
                .fallback_chain
                .iter()
                .any(|existing| normalized_family_key(existing) == key)
            {
                continue;
            }
            self.fallback_chain.push(family.to_string());
        }
    }
}

fn family_from_source(source: &str) -> String {
    Path::new(source)
        .file_stem()
        .and_then(|stem| stem.to_str())
        .map(|stem| stem.replace(['_', '-'], " "))
        .unwrap_or_else(|| source.to_string())
}

fn normalized_family_key(family: &str) -> String {
    family.trim().to_ascii_lowercase()
}
