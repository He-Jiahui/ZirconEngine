use std::borrow::Borrow;
use std::collections::BTreeMap;
use std::fmt::{Display, Formatter};
use std::sync::Arc;

use serde::{Deserialize, Deserializer, Serialize};

use super::{EditorI18nError, EditorLocale};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(transparent)]
pub struct EditorLocalizationKey(String);

impl EditorLocalizationKey {
    pub fn parse(value: impl Into<String>) -> Result<Self, String> {
        let value = value.into();
        validate_translation_key(&value).map_err(|error| error.to_string())?;
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl<'de> Deserialize<'de> for EditorLocalizationKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::parse(value).map_err(serde::de::Error::custom)
    }
}

impl Borrow<str> for EditorLocalizationKey {
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl Display for EditorLocalizationKey {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(transparent)]
pub struct EditorLocalizationBundleId(String);

impl EditorLocalizationBundleId {
    pub fn parse(value: impl Into<String>) -> Result<Self, String> {
        let value = value.into();
        let valid = !value.is_empty()
            && value.trim() == value
            && value.split('.').all(|segment| {
                !segment.is_empty()
                    && segment
                        .bytes()
                        .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
            });
        if !valid {
            return Err(format!(
                "editor localization bundle id `{value}` must use non-empty dot-separated identifier segments"
            ));
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl<'de> Deserialize<'de> for EditorLocalizationBundleId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::parse(value).map_err(serde::de::Error::custom)
    }
}

impl Display for EditorLocalizationBundleId {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

type LocaleTranslations = BTreeMap<EditorLocalizationKey, Arc<str>>;

/// Immutable locale resources contributed by one editor plugin package.
///
/// The extension contribution ticket owns this value. Page projections borrow it from the same
/// immutable contribution snapshot as their page descriptors, so revoke cannot leave a second
/// mutable plugin-localization registry behind.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditorLocalizationBundle {
    id: EditorLocalizationBundleId,
    locales: Arc<BTreeMap<EditorLocale, LocaleTranslations>>,
}

impl EditorLocalizationBundle {
    pub fn from_locale_maps(
        id: impl Into<String>,
        locale_maps: BTreeMap<String, BTreeMap<String, String>>,
    ) -> Result<Self, String> {
        let id = EditorLocalizationBundleId::parse(id)?;
        if locale_maps.is_empty() {
            return Err(format!(
                "editor localization bundle `{id}` must provide at least one locale"
            ));
        }

        let mut locales = BTreeMap::new();
        for (locale, translations) in locale_maps {
            let locale = EditorLocale::parse(locale).map_err(|error| error.to_string())?;
            if locales.contains_key(&locale) {
                return Err(EditorI18nError::DuplicateLocale(locale.to_string()).to_string());
            }
            if translations.is_empty() {
                return Err(format!(
                    "editor localization bundle `{id}` locale `{locale}` must provide at least one translation"
                ));
            }
            let mut validated = BTreeMap::new();
            for (key, value) in translations {
                let key = EditorLocalizationKey::parse(key)?;
                if value.trim().is_empty() {
                    return Err(EditorI18nError::EmptyTranslation(key.to_string()).to_string());
                }
                validated.insert(key, Arc::from(value));
            }
            locales.insert(locale, validated);
        }
        Ok(Self {
            id,
            locales: Arc::new(locales),
        })
    }

    pub fn id(&self) -> &str {
        self.id.as_str()
    }

    pub(crate) fn translation(&self, locale: &EditorLocale, key: &str) -> Option<Arc<str>> {
        self.translation_for_locale_tag(locale.as_str(), key)
    }

    pub(crate) fn translation_for_locale_tag(&self, locale: &str, key: &str) -> Option<Arc<str>> {
        self.locales
            .get(locale)
            .and_then(|translations| translations.get(key))
            .cloned()
    }

    pub fn contains_key(&self, key: &str) -> bool {
        self.locales
            .values()
            .any(|translations| translations.contains_key(key))
    }
}

pub(super) fn validate_translation_key(key: &str) -> Result<(), EditorI18nError> {
    if key.is_empty()
        || key.starts_with('.')
        || key.ends_with('.')
        || key.split('.').any(str::is_empty)
        || !key
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
    {
        return Err(EditorI18nError::InvalidTranslationKey(key.to_owned()));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::EditorLocalizationBundle;
    use crate::core::i18n::EditorLocale;

    #[test]
    fn bundle_normalizes_locales_and_rejects_invalid_resources() {
        let bundle = EditorLocalizationBundle::from_locale_maps(
            "fixture.editor",
            BTreeMap::from([(
                "zh-cn".to_string(),
                BTreeMap::from([("settings.fixture.label".to_string(), "示例".to_string())]),
            )]),
        )
        .expect("valid plugin bundle should be accepted");

        assert_eq!(
            bundle
                .translation(
                    &EditorLocale::parse("zh-CN").unwrap(),
                    "settings.fixture.label"
                )
                .as_deref(),
            Some("示例")
        );
        assert!(
            EditorLocalizationBundle::from_locale_maps(
                "fixture.editor",
                BTreeMap::from([("en".to_string(), BTreeMap::new())]),
            )
            .is_err()
        );
        assert!(
            EditorLocalizationBundle::from_locale_maps(
                "fixture.editor",
                BTreeMap::from([
                    (
                        "zh-CN".to_string(),
                        BTreeMap::from([("plugin.fixture.label".to_string(), "甲".to_string())]),
                    ),
                    (
                        "zh-cn".to_string(),
                        BTreeMap::from([("plugin.fixture.label".to_string(), "乙".to_string())]),
                    ),
                ]),
            )
            .is_err(),
            "locale aliases that normalize to one identity must not overwrite each other"
        );
    }
}
