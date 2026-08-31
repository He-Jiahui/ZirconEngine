use std::collections::BTreeMap;
use std::sync::Arc;

use crate::core::extension::{CapabilitySet, ContributionSnapshot, SettingsPageProjection};
use crate::core::i18n::{EditorI18nService, EditorLocale};
use crate::core::settings::{SettingDefinition, SettingsSnapshot};

use super::super::{LocalizedSetting, SettingsLocalizationDomain, SettingsNavigationCategory};
use super::SettingsWindowProjection;

impl SettingsWindowProjection {
    pub fn capture(
        settings: &SettingsSnapshot,
        contributions: &ContributionSnapshot,
        capabilities: &CapabilitySet,
        i18n: &EditorI18nService,
    ) -> Self {
        let locale = i18n.active_locale();
        let mut localized_settings = settings
            .catalog()
            .definitions()
            .iter()
            .map(|definition| localize_definition(definition, &locale, i18n))
            .collect::<Vec<_>>();
        localized_settings.sort_unstable_by(|left, right| {
            left.category_keys
                .cmp(&right.category_keys)
                .then_with(|| left.key.cmp(&right.key))
        });
        let plugin_pages = SettingsPageProjection::capture_for_locale(
            contributions,
            capabilities,
            i18n,
            locale.clone(),
        );
        let categories = project_categories(&localized_settings, &plugin_pages);
        Self {
            settings_generation: settings.generation(),
            settings_catalog: settings.catalog_handle(),
            title: i18n.translate_for_locale(&locale, "settings.window.title"),
            locale,
            enabled_capabilities: capabilities.clone(),
            categories,
            settings: localized_settings.into(),
            plugin_pages,
        }
    }

    pub fn is_current(
        &self,
        settings: &SettingsSnapshot,
        contributions: &ContributionSnapshot,
        capabilities: &CapabilitySet,
        i18n: &EditorI18nService,
    ) -> bool {
        self.settings_snapshot_is_current(settings)
            && self.plugin_pages.is_current(contributions, i18n)
            && self.enabled_capabilities == *capabilities
            && self.locale == i18n.active_locale()
    }
}

fn localize_definition(
    definition: &SettingDefinition,
    locale: &EditorLocale,
    i18n: &EditorI18nService,
) -> LocalizedSetting {
    let presentation = definition.presentation();
    let category_keys = presentation
        .category_path()
        .map(Arc::from)
        .collect::<Vec<_>>();
    let category_labels = presentation
        .category_path()
        .map(|key| i18n.translate_for_locale(locale, key))
        .collect::<Vec<_>>();
    LocalizedSetting {
        key: Arc::from(definition.key.as_str()),
        label: i18n.translate_for_locale(locale, presentation.label_key()),
        description: i18n.translate_for_locale(locale, presentation.description_key()),
        category_keys: category_keys.into(),
        category_labels: category_labels.into(),
        scope: definition.scope,
        schema: definition.schema.clone(),
        requires_restart: definition.requires_restart,
    }
}

fn project_categories(
    settings: &[LocalizedSetting],
    plugin_pages: &SettingsPageProjection,
) -> Arc<[SettingsNavigationCategory]> {
    let mut categories =
        BTreeMap::<(Vec<Arc<str>>, SettingsLocalizationDomain), Vec<Arc<str>>>::new();
    for setting in settings {
        for depth in 1..=setting.category_keys.len() {
            categories
                .entry((
                    setting.category_keys[..depth].to_vec(),
                    SettingsLocalizationDomain::BuiltIn,
                ))
                .or_insert_with(|| setting.category_labels[..depth].to_vec());
        }
    }
    for category in plugin_pages.categories() {
        categories
            .entry((
                category.keys().to_vec(),
                SettingsLocalizationDomain::Plugin(Arc::from(category.localization_bundle_id())),
            ))
            .or_insert_with(|| category.labels().to_vec());
    }
    categories
        .into_iter()
        .map(
            |((keys, localization_domain), labels)| SettingsNavigationCategory {
                localization_domain,
                keys: keys.into(),
                labels: labels.into(),
            },
        )
        .collect::<Vec<_>>()
        .into()
}
