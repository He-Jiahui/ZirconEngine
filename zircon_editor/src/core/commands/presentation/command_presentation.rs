use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::core::editor_operation::EditorOperationPath;
use crate::core::i18n::{
    EditorI18nService, EditorLocale, EditorLocalizationBundle, EditorLocalizationBundleId,
    EditorLocalizationKey,
};

use super::EditorCommandLocalizationSource;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EditorCommandPresentation {
    source: EditorCommandLocalizationSource,
    label_key: EditorLocalizationKey,
    description_key: EditorLocalizationKey,
    #[serde(skip)]
    bound_bundle: Option<EditorLocalizationBundle>,
}

impl EditorCommandPresentation {
    pub fn builtin(command_id: &EditorOperationPath) -> Self {
        let label_key =
            EditorLocalizationKey::parse(format!("command.{}.label", command_id.as_str()))
                .expect("validated editor operation paths form valid localization keys");
        let description_key =
            EditorLocalizationKey::parse(format!("command.{}.description", command_id.as_str()))
                .expect("validated editor operation paths form valid localization keys");
        Self {
            source: EditorCommandLocalizationSource::Builtin,
            label_key,
            description_key,
            bound_bundle: None,
        }
    }

    pub fn localized(
        bundle_id: impl Into<String>,
        label_key: impl Into<String>,
        description_key: impl Into<String>,
    ) -> Result<Self, String> {
        Ok(Self {
            source: EditorCommandLocalizationSource::Bundle(EditorLocalizationBundleId::parse(
                bundle_id,
            )?),
            label_key: EditorLocalizationKey::parse(label_key)?,
            description_key: EditorLocalizationKey::parse(description_key)?,
            bound_bundle: None,
        })
    }

    pub fn source(&self) -> &EditorCommandLocalizationSource {
        &self.source
    }

    pub fn label_key(&self) -> &str {
        self.label_key.as_str()
    }

    pub fn description_key(&self) -> &str {
        self.description_key.as_str()
    }

    pub(crate) fn bind_bundle(&mut self, bundle: &EditorLocalizationBundle) -> Result<(), String> {
        let Some(expected_id) = self.source.bundle_id() else {
            return Err("built-in command presentation cannot bind a plugin bundle".to_string());
        };
        if expected_id.as_str() != bundle.id() {
            return Err(format!(
                "command presentation expects localization bundle `{expected_id}` but received `{}`",
                bundle.id()
            ));
        }
        for key in [&self.label_key, &self.description_key] {
            if !bundle.contains_key(key.as_str()) {
                return Err(format!(
                    "localization bundle `{expected_id}` does not define command key `{key}`"
                ));
            }
        }
        self.bound_bundle = Some(bundle.clone());
        Ok(())
    }

    pub fn resolve_label(&self, i18n: &EditorI18nService, locale: &EditorLocale) -> Arc<str> {
        self.resolve_key(i18n, locale, self.label_key())
    }

    pub fn resolve_description(&self, i18n: &EditorI18nService, locale: &EditorLocale) -> Arc<str> {
        self.resolve_key(i18n, locale, self.description_key())
    }

    pub fn resolve_key(
        &self,
        i18n: &EditorI18nService,
        locale: &EditorLocale,
        key: &str,
    ) -> Arc<str> {
        match (&self.source, &self.bound_bundle) {
            (EditorCommandLocalizationSource::Builtin, _) => i18n.translate_for_locale(locale, key),
            (EditorCommandLocalizationSource::Bundle(_), Some(bundle)) => {
                i18n.translate_bundle_for_locale(bundle, locale, key)
            }
            (EditorCommandLocalizationSource::Bundle(_), None) => Arc::from(key),
        }
    }
}

impl PartialEq for EditorCommandPresentation {
    fn eq(&self, other: &Self) -> bool {
        self.source == other.source
            && self.label_key == other.label_key
            && self.description_key == other.description_key
    }
}

impl Eq for EditorCommandPresentation {}
