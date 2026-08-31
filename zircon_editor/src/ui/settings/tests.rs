use std::collections::BTreeMap;

use crate::core::extension::{
    CapabilitySet, ContributionBatch, ContributionSource, ContributionStore, PluginContributionId,
};
use crate::core::i18n::{EditorI18nService, EditorLocale, EditorLocalizationBundle};
use crate::core::settings::{
    EDITOR_LOCALE_KEY, SettingSchema, SettingValue, SettingsAuthority, SettingsKey,
    SettingsPageDescriptor, SettingsScope,
};

use super::{SettingsLocalizationDomain, SettingsWindowProjection};

fn plugin_batch() -> ContributionBatch {
    let mut batch = ContributionBatch::default();
    batch
        .register_localization_bundle(
            EditorLocalizationBundle::from_locale_maps(
                "fixture.editor",
                BTreeMap::from([
                    (
                        "en".to_string(),
                        BTreeMap::from([
                            ("plugin.fixture.label".to_string(), "Fixture".to_string()),
                            (
                                "plugin.fixture.description".to_string(),
                                "Fixture settings".to_string(),
                            ),
                            (
                                "settings.category.editor".to_string(),
                                "Plugin Editor".to_string(),
                            ),
                        ]),
                    ),
                    (
                        "zh-CN".to_string(),
                        BTreeMap::from([
                            ("plugin.fixture.label".to_string(), "示例".to_string()),
                            (
                                "plugin.fixture.description".to_string(),
                                "示例设置".to_string(),
                            ),
                            (
                                "settings.category.editor".to_string(),
                                "插件编辑器".to_string(),
                            ),
                        ]),
                    ),
                ]),
            )
            .unwrap(),
        )
        .unwrap();
    batch
        .register_settings_page(
            SettingsPageDescriptor::new(
                "plugin.fixture.editor.settings",
                "fixture.editor",
                "plugin.fixture.label",
                "plugin.fixture.description",
                ["settings.category.editor"],
            )
            .unwrap(),
        )
        .unwrap();
    batch
}

#[test]
fn settings_window_combines_builtins_and_plugin_pages_at_one_locale_boundary() {
    let authority = SettingsAuthority::with_defaults();
    let mut contributions = ContributionStore::default();
    contributions
        .contribute(
            ContributionSource::Plugin(PluginContributionId::parse("fixture.editor").unwrap()),
            plugin_batch(),
        )
        .unwrap();
    let capabilities = CapabilitySet::default();
    let i18n = EditorI18nService::default();
    let settings_snapshot = authority.snapshot();
    let contribution_snapshot = contributions.snapshot();
    let english = SettingsWindowProjection::capture(
        &settings_snapshot,
        &contribution_snapshot,
        &capabilities,
        &i18n,
    );

    let locale = english
        .settings()
        .iter()
        .find(|setting| setting.key() == EDITOR_LOCALE_KEY)
        .unwrap();
    assert_eq!(locale.label(), "Editor Language");
    assert!(matches!(locale.schema(), SettingSchema::Enum { .. }));
    assert_eq!(english.plugin_pages()[0].label(), "Fixture");
    let editor_categories = english
        .categories()
        .iter()
        .filter(|category| {
            category
                .keys()
                .iter()
                .map(AsRef::<str>::as_ref)
                .eq(["settings.category.editor"])
        })
        .collect::<Vec<_>>();
    assert_eq!(editor_categories.len(), 2);
    assert!(editor_categories.iter().any(|category| matches!(
        category.localization_domain(),
        SettingsLocalizationDomain::BuiltIn
    )));
    assert!(editor_categories.iter().any(|category| matches!(
        category.localization_domain(),
        SettingsLocalizationDomain::Plugin(bundle) if bundle.as_ref() == "fixture.editor"
    )));

    i18n.set_active_locale(EditorLocale::parse("zh-CN").unwrap())
        .unwrap();
    assert!(!english.is_current(
        &authority.snapshot(),
        &contributions.snapshot(),
        &capabilities,
        &i18n
    ));
    let chinese = SettingsWindowProjection::capture(
        &authority.snapshot(),
        &contributions.snapshot(),
        &capabilities,
        &i18n,
    );
    assert_eq!(
        chinese
            .settings()
            .iter()
            .find(|setting| setting.key() == EDITOR_LOCALE_KEY)
            .unwrap()
            .label(),
        "编辑器语言"
    );
    assert_eq!(chinese.plugin_pages()[0].label(), "示例");
}

#[test]
fn settings_window_keeps_its_catalog_projection_across_single_value_generations() {
    let authority = SettingsAuthority::with_defaults();
    let contributions = ContributionStore::default();
    let capabilities = CapabilitySet::default();
    let i18n = EditorI18nService::default();
    let before = authority.snapshot();
    let projection =
        SettingsWindowProjection::capture(&before, &contributions.snapshot(), &capabilities, &i18n);
    let key = SettingsKey::parse(EDITOR_LOCALE_KEY).unwrap();

    authority
        .set(
            SettingsScope::User,
            &key,
            SettingValue::Enum("zh-CN".to_string()),
        )
        .unwrap();
    let after = authority.snapshot();

    assert!(before.shares_catalog_with(&after));
    assert!(projection.is_current(&after, &contributions.snapshot(), &capabilities, &i18n));
    assert!(!projection.is_current(
        &after,
        &contributions.snapshot(),
        &CapabilitySet::from(["editor.capability.changed"]),
        &i18n
    ));
    assert_eq!(
        authority.resolved_setting(&key).unwrap().value(),
        &SettingValue::Enum("zh-CN".to_string())
    );
}
