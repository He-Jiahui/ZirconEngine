//! Descriptor for one plugin-contributed settings page.

use serde::{Deserialize, Deserializer, Serialize};

use crate::core::i18n::{EditorLocalizationBundleId, EditorLocalizationKey};

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct SettingsPageDescriptor {
    id: String,
    localization_bundle_id: EditorLocalizationBundleId,
    label_key: EditorLocalizationKey,
    description_key: EditorLocalizationKey,
    category_keys: Vec<EditorLocalizationKey>,
}

impl SettingsPageDescriptor {
    pub fn new<I, S>(
        id: impl Into<String>,
        localization_bundle_id: impl Into<String>,
        label_key: impl Into<String>,
        description_key: impl Into<String>,
        category_keys: I,
    ) -> Result<Self, String>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let category_keys = category_keys
            .into_iter()
            .map(|key| EditorLocalizationKey::parse(key.into()))
            .collect::<Result<Vec<_>, _>>()?;
        if category_keys.is_empty() {
            return Err("editor settings page must include at least one category key".into());
        }
        Ok(Self {
            id: id.into(),
            localization_bundle_id: EditorLocalizationBundleId::parse(localization_bundle_id)?,
            label_key: EditorLocalizationKey::parse(label_key)?,
            description_key: EditorLocalizationKey::parse(description_key)?,
            category_keys,
        })
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn localization_bundle_id(&self) -> &str {
        self.localization_bundle_id.as_str()
    }

    pub fn label_key(&self) -> &str {
        self.label_key.as_str()
    }

    pub fn description_key(&self) -> &str {
        self.description_key.as_str()
    }

    pub fn category_keys(&self) -> impl ExactSizeIterator<Item = &str> {
        self.category_keys.iter().map(EditorLocalizationKey::as_str)
    }

    pub fn localization_keys(&self) -> impl Iterator<Item = &str> {
        std::iter::once(self.label_key())
            .chain(std::iter::once(self.description_key()))
            .chain(self.category_keys())
    }

    pub(crate) fn canonical_category_keys(&self) -> &[EditorLocalizationKey] {
        &self.category_keys
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawSettingsPageDescriptor {
    id: String,
    localization_bundle_id: String,
    label_key: String,
    description_key: String,
    category_keys: Vec<String>,
}

impl<'de> Deserialize<'de> for SettingsPageDescriptor {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = RawSettingsPageDescriptor::deserialize(deserializer)?;
        Self::new(
            raw.id,
            raw.localization_bundle_id,
            raw.label_key,
            raw.description_key,
            raw.category_keys,
        )
        .map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::SettingsPageDescriptor;

    #[test]
    fn settings_page_accepts_only_locale_neutral_presentation() {
        let descriptor = SettingsPageDescriptor::new(
            "plugin.fixture.settings",
            "fixture.editor",
            "plugin.fixture.label",
            "plugin.fixture.description",
            ["plugin.category.plugins", "plugin.category.fixture"],
        )
        .expect("locale-neutral page presentation should be accepted");

        assert_eq!(descriptor.localization_bundle_id(), "fixture.editor");
        assert_eq!(
            descriptor.category_keys().collect::<Vec<_>>(),
            ["plugin.category.plugins", "plugin.category.fixture"]
        );
        assert!(
            SettingsPageDescriptor::new(
                "plugin.fixture.legacy",
                "fixture.editor",
                "Legacy settings",
                "plugin.fixture.description",
                ["Plugins/Fixture"],
            )
            .is_err()
        );
    }
}
