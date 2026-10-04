use std::collections::BTreeMap;
use std::hint::black_box;
use std::sync::Arc;
use std::time::Instant;

use super::{
    project_categories, LocalizedSettingsCategory, LocalizedSettingsPage, SettingsPageProjection,
};
use crate::core::extension::{
    CapabilitySet, ContributionBatch, ContributionSource, ContributionStore, PluginContributionId,
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
    assert!(english
        .categories()
        .iter()
        .all(|category| category.localization_bundle_id() == "sample"));
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

#[test]
fn optimization_batch_hy_editor609_category_projection_borrows_duplicate_prefixes() {
    let source = include_str!("../settings_page_projection.rs");
    let implementation = source
        .split("#[cfg(test)]")
        .next()
        .expect("settings projection implementation");
    let category_projection = implementation
        .split("fn project_categories")
        .nth(1)
        .and_then(|source| source.split("fn localize_page").next())
        .expect("category projection");
    let page_localization = implementation
        .split("fn localize_page")
        .nth(1)
        .expect("page localization");

    assert!(
        category_projection.contains("BTreeMap::<(&[Arc<str>], &Arc<str>), &[Arc<str>]>::new()")
    );
    assert!(!category_projection.contains("[..depth].to_vec()"));
    assert!(page_localization.contains("for key in page.category_keys()"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_hy_editor609_borrowed_category_prefix_benchmark() {
    const PAGE_COUNT: usize = 4_096;
    const SAMPLE_PAIRS: usize = 17;
    let pages = (0..PAGE_COUNT).map(projected_page).collect::<Vec<_>>();
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut borrowed_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for sample_index in 0..SAMPLE_PAIRS {
        if sample_index % 2 == 0 {
            legacy_samples.push(measure_legacy_categories(&pages));
            borrowed_samples.push(measure_borrowed_categories(&pages));
        } else {
            borrowed_samples.push(measure_borrowed_categories(&pages));
            legacy_samples.push(measure_legacy_categories(&pages));
        }
    }
    legacy_samples.sort_unstable();
    borrowed_samples.sort_unstable();
    let legacy_p95_ns = legacy_samples[15];
    let borrowed_p95_ns = borrowed_samples[15];
    println!(
        "EDITOR609_SETTINGS_CATEGORY_BORROWED_PREFIX_BENCH_V1 pages={} unique_categories={} legacy_p95_ns={} borrowed_p95_ns={} target_ratio_bp=4500",
        PAGE_COUNT,
        project_categories(&pages).len(),
        legacy_p95_ns,
        borrowed_p95_ns,
    );
    assert!(
        borrowed_p95_ns.saturating_mul(10_000) <= legacy_p95_ns.saturating_mul(4_500),
        "borrowed category projection P95 {borrowed_p95_ns} ns exceeded 45% of legacy {legacy_p95_ns} ns"
    );
}

fn projected_page(index: usize) -> LocalizedSettingsPage {
    let category = Arc::<str>::from(format!("settings.category.group.{}", index % 4));
    LocalizedSettingsPage {
        id: Arc::from(format!("plugin.sample.page.{index}")),
        localization_bundle_id: Arc::from("sample"),
        label: Arc::from("Page"),
        description: Arc::from("Description"),
        category_keys: vec![
            Arc::from("settings.category.root"),
            Arc::from("settings.category.shared"),
            Arc::clone(&category),
        ]
        .into(),
        category_labels: vec![Arc::from("Settings"), Arc::from("Shared"), category].into(),
    }
}

fn measure_legacy_categories(pages: &[LocalizedSettingsPage]) -> u128 {
    let started = Instant::now();
    let categories = legacy_project_categories(black_box(pages));
    black_box(categories);
    started.elapsed().as_nanos().max(1)
}

fn measure_borrowed_categories(pages: &[LocalizedSettingsPage]) -> u128 {
    let started = Instant::now();
    let categories = project_categories(black_box(pages));
    black_box(categories);
    started.elapsed().as_nanos().max(1)
}

fn legacy_project_categories(pages: &[LocalizedSettingsPage]) -> Arc<[LocalizedSettingsCategory]> {
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
