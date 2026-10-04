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
    let mut categories = BTreeMap::<(&[Arc<str>], &Arc<str>), &[Arc<str>]>::new();
    for page in pages {
        for depth in 1..=page.category_keys.len() {
            categories
                .entry((&page.category_keys[..depth], &page.localization_bundle_id))
                .or_insert(&page.category_labels[..depth]);
        }
    }
    categories
        .into_iter()
        .map(
            |((keys, localization_bundle_id), labels)| LocalizedSettingsCategory {
                localization_bundle_id: Arc::clone(localization_bundle_id),
                keys: Arc::from(keys),
                labels: Arc::from(labels),
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
    let category_count = page.category_keys().len();
    let mut category_keys = Vec::with_capacity(category_count);
    let mut category_labels = Vec::with_capacity(category_count);
    for key in page.category_keys() {
        category_keys.push(Arc::from(key));
        category_labels.push(translate(key));
    }
    LocalizedSettingsPage {
        id: Arc::from(page.id()),
        localization_bundle_id: Arc::from(page.localization_bundle_id()),
        label: translate(page.label_key()),
        description: translate(page.description_key()),
        category_keys: category_keys.into(),
        category_labels: category_labels.into(),
    }
}

#[cfg(test)]
#[path = "tests/settings_page_projection.rs"]
mod tests;
