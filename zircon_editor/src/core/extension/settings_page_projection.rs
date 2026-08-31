use std::collections::BTreeMap;
use std::sync::Arc;

use crate::core::i18n::{EditorI18nService, EditorLocale, EditorLocalizationBundle};
use crate::core::settings::SettingsPageDescriptor;

use super::{CapabilitySet, ContributionSnapshot};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LocalizedSettingsPage {
    id: Arc<str>,
    localization_bundle_id: Arc<str>,
    label: Arc<str>,
    description: Arc<str>,
    category_keys: Arc<[Arc<str>]>,
    category_labels: Arc<[Arc<str>]>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LocalizedSettingsCategory {
    localization_bundle_id: Arc<str>,
    keys: Arc<[Arc<str>]>,
    labels: Arc<[Arc<str>]>,
}

impl LocalizedSettingsCategory {
    pub fn localization_bundle_id(&self) -> &str {
        &self.localization_bundle_id
    }

    pub fn keys(&self) -> &[Arc<str>] {
        &self.keys
    }

    pub fn labels(&self) -> &[Arc<str>] {
        &self.labels
    }
}

impl LocalizedSettingsPage {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn localization_bundle_id(&self) -> &str {
        &self.localization_bundle_id
    }

    pub fn label(&self) -> &str {
        &self.label
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn category_keys(&self) -> &[Arc<str>] {
        &self.category_keys
    }

    pub fn category_labels(&self) -> &[Arc<str>] {
        &self.category_labels
    }
}

/// One immutable settings-page view of a contribution generation and a captured locale.
///
/// Ordering is decided from canonical category keys before any text is translated. Consumers
/// rebuild after either the contribution generation or locale changes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SettingsPageProjection {
    contribution_generation: u64,
    locale: EditorLocale,
    categories: Arc<[LocalizedSettingsCategory]>,
    pages: Arc<[LocalizedSettingsPage]>,
}

impl SettingsPageProjection {
    pub fn capture(
        snapshot: &ContributionSnapshot,
        capabilities: &CapabilitySet,
        i18n: &EditorI18nService,
    ) -> Self {
        let locale = i18n.active_locale();
        Self::capture_for_locale(snapshot, capabilities, i18n, locale)
    }

    pub fn capture_for_locale(
        snapshot: &ContributionSnapshot,
        capabilities: &CapabilitySet,
        i18n: &EditorI18nService,
        locale: EditorLocale,
    ) -> Self {
        let bundles = snapshot
            .localization_bundles(capabilities)
            .map(|bundle| (bundle.id(), bundle))
            .collect::<BTreeMap<_, _>>();
        let mut pages = snapshot.settings_pages(capabilities).collect::<Vec<_>>();
        pages.sort_unstable_by(|left, right| {
            left.canonical_category_keys()
                .cmp(right.canonical_category_keys())
                .then_with(|| {
                    left.localization_bundle_id()
                        .cmp(right.localization_bundle_id())
                })
                .then_with(|| left.id().cmp(right.id()))
        });
        let pages = pages
            .into_iter()
            .map(|page| localize_page(page, &bundles, &locale, i18n))
            .collect::<Vec<_>>();
        let categories = project_categories(&pages);
        Self {
            contribution_generation: snapshot.generation(),
            locale,
            categories,
            pages: pages.into(),
        }
    }

    pub fn contribution_generation(&self) -> u64 {
        self.contribution_generation
    }

    pub fn locale(&self) -> &EditorLocale {
        &self.locale
    }

    pub fn pages(&self) -> &[LocalizedSettingsPage] {
        &self.pages
    }

    pub fn categories(&self) -> &[LocalizedSettingsCategory] {
        &self.categories
    }

    pub fn is_current(&self, snapshot: &ContributionSnapshot, i18n: &EditorI18nService) -> bool {
        self.contribution_generation == snapshot.generation() && self.locale == i18n.active_locale()
    }
}

fn project_categories(pages: &[LocalizedSettingsPage]) -> Arc<[LocalizedSettingsCategory]> {
    let mut categories = BTreeMap::<(Vec<Arc<str>>, Arc<str>), Vec<Arc<str>>>::new();
    for page in pages {
        for depth in 1..=page.category_keys.len() {
            categories
                .entry((
                    page.category_keys[..depth].to_vec(),
                    Arc::clone(&page.localization_bundle_id),
                ))
                .or_insert_with(|| page.category_labels[..depth].to_vec());
        }
    }
    categories
        .into_iter()
        .map(
            |((keys, localization_bundle_id), labels)| LocalizedSettingsCategory {
                localization_bundle_id,
                keys: keys.into(),
                labels: labels.into(),
            },
        )
        .collect::<Vec<_>>()
        .into()
}

fn localize_page(
    page: &SettingsPageDescriptor,
    bundles: &BTreeMap<&str, &EditorLocalizationBundle>,
    locale: &EditorLocale,
    i18n: &EditorI18nService,
) -> LocalizedSettingsPage {
    let bundle = bundles
        .get(page.localization_bundle_id())
        .copied()
        .expect("published settings pages retain their ticket-owned localization bundle");
    let translate = |key: &str| i18n.translate_bundle_for_locale(bundle, locale, key);
    LocalizedSettingsPage {
        id: Arc::from(page.id()),
        localization_bundle_id: Arc::from(page.localization_bundle_id()),
        label: translate(page.label_key()),
        description: translate(page.description_key()),
        category_keys: page
            .category_keys()
            .map(Arc::from)
            .collect::<Vec<_>>()
            .into(),
        category_labels: page
            .category_keys()
            .map(|key| translate(key))
            .collect::<Vec<_>>()
            .into(),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::SettingsPageProjection;
    use crate::core::extension::{
        CapabilitySet, ContributionBatch, ContributionSource, ContributionStore,
        PluginContributionId,
    };
    use crate::core::i18n::{EditorI18nService, EditorLocale, EditorLocalizationBundle};
    use crate::core::settings::SettingsPageDescriptor;

    fn plugin_source() -> ContributionSource {
        ContributionSource::Plugin(PluginContributionId::parse("sample").unwrap())
    }

    fn localized_batch() -> ContributionBatch {
        let bundle = EditorLocalizationBundle::from_locale_maps(
            "sample",
            BTreeMap::from([
                (
                    "en".to_string(),
                    BTreeMap::from([
                        ("settings.alpha.label".to_string(), "Alpha".to_string()),
                        ("settings.zulu.label".to_string(), "Zulu".to_string()),
                        ("settings.category.alpha".to_string(), "Zulu".to_string()),
                        ("settings.category.zulu".to_string(), "Alpha".to_string()),
                        ("settings.category.root".to_string(), "Settings".to_string()),
                    ]),
                ),
                (
                    "zh-CN".to_string(),
                    BTreeMap::from([
                        ("settings.alpha.label".to_string(), "甲".to_string()),
                        ("settings.zulu.label".to_string(), "乙".to_string()),
                        ("settings.category.alpha".to_string(), "乙类".to_string()),
                        ("settings.category.zulu".to_string(), "甲类".to_string()),
                        ("settings.category.root".to_string(), "设置".to_string()),
                        (
                            "settings.alpha.missing_description".to_string(),
                            "甲描述".to_string(),
                        ),
                        (
                            "settings.zulu.missing_description".to_string(),
                            "乙描述".to_string(),
                        ),
                    ]),
                ),
            ]),
        )
        .unwrap();
        let mut batch = ContributionBatch::default();
        batch.register_localization_bundle(bundle).unwrap();
        batch
            .register_settings_page(
                SettingsPageDescriptor::new(
                    "plugin.sample.zulu",
                    "sample",
                    "settings.zulu.label",
                    "settings.zulu.missing_description",
                    ["settings.category.root", "settings.category.zulu"],
                )
                .unwrap(),
            )
            .unwrap();
        batch
            .register_settings_page(
                SettingsPageDescriptor::new(
                    "plugin.sample.alpha",
                    "sample",
                    "settings.alpha.label",
                    "settings.alpha.missing_description",
                    ["settings.category.root", "settings.category.alpha"],
                )
                .unwrap(),
            )
            .unwrap();
        batch
    }

    #[test]
    fn projection_is_locale_bound_key_ordered_and_invalidated_by_revoke() {
        let mut store = ContributionStore::default();
        let ticket = store
            .contribute(plugin_source(), localized_batch())
            .unwrap();
        let capabilities = CapabilitySet::default();
        let i18n = EditorI18nService::default();
        let english = SettingsPageProjection::capture(&store.snapshot(), &capabilities, &i18n);

        assert_eq!(
            english
                .pages()
                .iter()
                .map(|page| page.id())
                .collect::<Vec<_>>(),
            ["plugin.sample.alpha", "plugin.sample.zulu"]
        );
        assert_eq!(english.pages()[0].category_labels()[1].as_ref(), "Zulu");
        assert_eq!(english.categories().len(), 3);
        assert!(
            english
                .categories()
                .iter()
                .all(|category| category.localization_bundle_id() == "sample")
        );
        assert_eq!(english.categories()[0].keys().len(), 1);
        assert_eq!(
            english.pages()[0].description(),
            "settings.alpha.missing_description",
            "missing plugin translations must use the canonical raw-key fallback"
        );

        i18n.set_active_locale(EditorLocale::parse("zh-CN").unwrap())
            .unwrap();
        assert!(!english.is_current(&store.snapshot(), &i18n));
        let chinese = SettingsPageProjection::capture(&store.snapshot(), &capabilities, &i18n);
        assert_eq!(chinese.pages()[0].label(), "甲");
        assert_eq!(chinese.pages()[0].category_labels()[1].as_ref(), "乙类");
        assert_eq!(
            chinese
                .pages()
                .iter()
                .map(|page| page.id())
                .collect::<Vec<_>>(),
            ["plugin.sample.alpha", "plugin.sample.zulu"],
            "translated collation must not affect page order"
        );

        let report = store.revoke(ticket);
        assert_eq!(report.removed().localization_bundles(), 1);
        assert_eq!(report.removed().settings_pages(), 2);
        assert!(!chinese.is_current(&store.snapshot(), &i18n));
        assert!(
            SettingsPageProjection::capture(&store.snapshot(), &capabilities, &i18n)
                .pages()
                .is_empty()
        );
    }
}
