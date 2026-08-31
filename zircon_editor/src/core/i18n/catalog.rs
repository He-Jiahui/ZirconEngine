use std::collections::BTreeMap;
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

use serde::Deserialize;

use super::{EditorI18nError, EditorLocale, bundle::validate_translation_key};

const ENGLISH_BUNDLE: &str = include_str!("../../../assets/i18n/en.toml");
const SIMPLIFIED_CHINESE_BUNDLE: &str = include_str!("../../../assets/i18n/zh-CN.toml");

#[derive(Deserialize)]
struct BundleDocument {
    locale: String,
    translations: BTreeMap<String, String>,
}

pub struct EditorI18nCatalog {
    active_locale: RwLock<EditorLocale>,
    bundles: BTreeMap<EditorLocale, BTreeMap<String, Arc<str>>>,
}

impl EditorI18nCatalog {
    pub fn embedded() -> Result<Self, EditorI18nError> {
        Self::from_toml_bundles(&[ENGLISH_BUNDLE, SIMPLIFIED_CHINESE_BUNDLE])
    }

    pub fn from_toml_bundles(bundles: &[&str]) -> Result<Self, EditorI18nError> {
        let mut parsed_bundles = BTreeMap::new();
        for bundle in bundles {
            let document = toml::from_str::<BundleDocument>(bundle)
                .map_err(|error| EditorI18nError::InvalidBundle(error.to_string()))?;
            let locale = EditorLocale::parse(document.locale)?;
            if parsed_bundles.contains_key(&locale) {
                return Err(EditorI18nError::DuplicateLocale(locale.to_string()));
            }

            let mut translations = BTreeMap::new();
            for (key, value) in document.translations {
                validate_translation_key(&key)?;
                if value.trim().is_empty() {
                    return Err(EditorI18nError::EmptyTranslation(key));
                }
                translations.insert(key, Arc::from(value));
            }
            parsed_bundles.insert(locale, translations);
        }

        let english = EditorLocale::english();
        if !parsed_bundles.contains_key(&english) {
            return Err(EditorI18nError::MissingEnglishFallback);
        }
        Ok(Self {
            active_locale: RwLock::new(english),
            bundles: parsed_bundles,
        })
    }

    pub fn active_locale(&self) -> EditorLocale {
        self.read_active_locale().clone()
    }

    pub fn available_locales(&self) -> Vec<EditorLocale> {
        self.bundles.keys().cloned().collect()
    }

    pub fn set_active_locale(&self, locale: EditorLocale) -> Result<bool, EditorI18nError> {
        if !self.bundles.contains_key(&locale) {
            return Err(EditorI18nError::UnavailableLocale(locale.to_string()));
        }
        let mut active_locale = self.write_active_locale();
        if *active_locale == locale {
            return Ok(false);
        }
        *active_locale = locale;
        Ok(true)
    }

    pub fn translate(&self, key: &str) -> Arc<str> {
        let active_locale = self.active_locale();
        self.translate_for_locale(&active_locale, key)
    }

    /// Resolves against one captured locale so a compound display projection cannot mix
    /// translations from separate locale generations.
    pub fn translate_for_locale(&self, locale: &EditorLocale, key: &str) -> Arc<str> {
        self.bundles
            .get(locale)
            .and_then(|translations| translations.get(key))
            .or_else(|| {
                self.bundles
                    .get(EditorLocale::english_tag())
                    .and_then(|translations| translations.get(key))
            })
            .cloned()
            .unwrap_or_else(|| Arc::from(key))
    }

    pub(super) fn english_fallback() -> Self {
        let english = EditorLocale::english();
        let mut bundles = BTreeMap::new();
        bundles.insert(english.clone(), BTreeMap::new());
        Self {
            active_locale: RwLock::new(english),
            bundles,
        }
    }

    fn read_active_locale(&self) -> RwLockReadGuard<'_, EditorLocale> {
        self.active_locale
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn write_active_locale(&self) -> RwLockWriteGuard<'_, EditorLocale> {
        self.active_locale
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}
