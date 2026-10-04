use std::collections::BTreeMap;

use toml::Value;

use super::super::{
    projected_settings_window_data, CATEGORIES, PLUGIN_PAGES, SELECTED_CATEGORY_ID, SETTINGS,
    SETTINGS_VALUES,
};

#[test]
#[ignore = "Editor265 selected-category authority and retained-projection boundary evidence; run in the grouped Windows Release batch"]
fn editor265_selected_category_authority_and_retained_projection_release_benchmark() {
    use crate::core::settings::{
        SettingDefinition, SettingSchema, SettingValue, SettingsAuthority, SettingsPresentation,
        SettingsRegistry, SettingsScope,
    };

    const DEFINITION_COUNT: usize = 10_000;
    const CATEGORY_COUNT: usize = 1_000;
    const SETTINGS_PER_CATEGORY: usize = DEFINITION_COUNT / CATEGORY_COUNT;
    const CHILD_CATEGORY_COUNT: usize = CATEGORY_COUNT - 1;
    const PLUGIN_OWNER_COUNT: usize = 100;
    const PLUGIN_CATEGORY_COUNT: usize = PLUGIN_OWNER_COUNT;
    const SELECTED_PLUGIN_OWNER: usize = 50;
    const SAMPLE_COUNT: usize = 31;
    const WARMUP_COUNT: usize = 3;
    const ROOT_CATEGORY: &str = "settings.category.performance";

    struct RefreshSample {
        authority_ns: u128,
        payload_ns: u128,
        projection_ns: u128,
        total_ns: u128,
        projected_rows: usize,
        batch_rows: usize,
        matched_plugin_pages: usize,
    }

    fn value_table(entries: impl IntoIterator<Item = (&'static str, Value)>) -> Value {
        Value::Table(
            entries
                .into_iter()
                .map(|(key, value)| (key.to_owned(), value))
                .collect(),
        )
    }

    fn refresh_sample(
        authority: &SettingsAuthority,
        attributes: &mut BTreeMap<String, Value>,
        category_id: &str,
        expected_batch_rows: usize,
        expected_rows: usize,
        expected_plugin_pages: usize,
    ) -> RefreshSample {
        let total_started = std::time::Instant::now();
        let authority_started = std::time::Instant::now();
        let snapshot = authority.snapshot();
        let batch = if let Some(category_path) = category_id
            .strip_prefix("builtin|")
            .filter(|category_path| !category_path.is_empty())
        {
            authority
                .resolved_settings_from_iter(
                    snapshot.catalog().keys_for_category_subtree(category_path),
                )
                .expect("the performance fixture contains registered keys only")
        } else {
            authority
                .resolved_settings(&[])
                .expect("an empty selected-category batch should resolve")
        };
        drop(snapshot);
        let authority_ns = authority_started.elapsed().as_nanos();
        let batch_rows = batch.values().len();
        assert_eq!(batch_rows, expected_batch_rows);

        let payload_started = std::time::Instant::now();
        let values = Value::Array(
            batch
                .values()
                .iter()
                .map(|resolved| {
                    let value_text = match resolved.value() {
                        SettingValue::Bool(value) => {
                            if *value {
                                "true"
                            } else {
                                "false"
                            }
                        }
                        _ => unreachable!("the performance fixture uses boolean settings"),
                    };
                    value_table([
                        ("key", Value::String(resolved.key().as_str().to_owned())),
                        ("value_text", Value::String(value_text.to_owned())),
                        ("color_channels", Value::Array(Vec::new())),
                        ("value_source", Value::String("default".to_owned())),
                    ])
                })
                .collect(),
        );
        attributes.insert(SETTINGS_VALUES.to_owned(), values);
        attributes.insert(
            SELECTED_CATEGORY_ID.to_owned(),
            Value::String(category_id.to_owned()),
        );
        let payload_ns = payload_started.elapsed().as_nanos();

        let projection_started = std::time::Instant::now();
        let projected = projected_settings_window_data("settings-window", attributes);
        let projected_rows = projected.entries.len();
        let matched_plugin_pages = projected
            .entries
            .iter()
            .filter(|entry| entry.plugin_page)
            .count();
        assert_eq!(projected_rows, expected_rows);
        assert_eq!(matched_plugin_pages, expected_plugin_pages);
        let projected = std::hint::black_box(projected);
        let projection_ns = projection_started.elapsed().as_nanos();
        drop(batch);
        let total_ns = total_started.elapsed().as_nanos();
        drop(projected);

        RefreshSample {
            authority_ns,
            payload_ns,
            projection_ns,
            total_ns,
            projected_rows,
            batch_rows,
            matched_plugin_pages,
        }
    }

    fn percentile<F>(samples: &[RefreshSample], percentile: usize, select: F) -> u128
    where
        F: Fn(&RefreshSample) -> u128,
    {
        let mut values = samples.iter().map(select).collect::<Vec<_>>();
        values.sort_unstable();
        let rank = ((values.len() * percentile + 99) / 100).max(1);
        values[rank - 1]
    }

    fn category_row(domain: String, path: String, label_path: String, label: String) -> Value {
        value_table([
            ("domain", Value::String(domain)),
            ("key_path", Value::String(path)),
            ("label_path", Value::String(label_path)),
            ("label", Value::String(label)),
        ])
    }

    let mut registry = SettingsRegistry::default();
    let mut categories = Vec::with_capacity(CATEGORY_COUNT + PLUGIN_CATEGORY_COUNT);
    let mut settings = Vec::with_capacity(DEFINITION_COUNT);
    let root_label_path = "Settings/Performance".to_owned();
    categories.push(category_row(
        "builtin".to_owned(),
        ROOT_CATEGORY.to_owned(),
        root_label_path.clone(),
        "Performance".to_owned(),
    ));

    for category in 0..CHILD_CATEGORY_COUNT {
        let category_segment = format!("{ROOT_CATEGORY}.group_{category:04}");
        let category_path = format!("{ROOT_CATEGORY}/{category_segment}");
        let category_label = format!("Group {category:04}");
        let category_label_path = format!("{root_label_path}/{category_label}");
        categories.push(category_row(
            "builtin".to_owned(),
            category_path.clone(),
            category_label_path.clone(),
            category_label,
        ));
        for setting in 0..SETTINGS_PER_CATEGORY {
            let key = format!("editor.performance.group_{category:04}.setting_{setting:02}");
            let definition = SettingDefinition::new(
                crate::core::settings::SettingsKey::parse(&key).expect("the fixture key is valid"),
                SettingsScope::User,
                SettingSchema::Bool,
                SettingValue::Bool(false),
                false,
                SettingsPresentation::new(
                    "settings.performance.label",
                    "settings.performance.description",
                    [ROOT_CATEGORY.to_owned(), category_segment.clone()],
                )
                .expect("the fixture presentation is valid"),
            )
            .expect("the fixture definition is valid");
            registry
                .register(definition)
                .expect("the fixture key is unique");
            settings.push(value_table([
                ("key", Value::String(key)),
                ("label", Value::String("Performance setting".to_owned())),
                (
                    "description",
                    Value::String("Performance fixture setting".to_owned()),
                ),
                ("category_key_path", Value::String(category_path.clone())),
                (
                    "category_label_path",
                    Value::String(category_label_path.clone()),
                ),
                ("scope", Value::String("user".to_owned())),
                ("schema", Value::String("bool".to_owned())),
                ("options", Value::Array(Vec::new())),
                ("requires_restart", Value::Boolean(false)),
            ]));
        }
    }

    for setting in 0..SETTINGS_PER_CATEGORY {
        let key = format!("editor.performance.root.setting_{setting:02}");
        let definition = SettingDefinition::new(
            crate::core::settings::SettingsKey::parse(&key).expect("the fixture key is valid"),
            SettingsScope::User,
            SettingSchema::Bool,
            SettingValue::Bool(false),
            false,
            SettingsPresentation::new(
                "settings.performance.label",
                "settings.performance.description",
                [ROOT_CATEGORY.to_owned()],
            )
            .expect("the fixture presentation is valid"),
        )
        .expect("the fixture definition is valid");
        registry
            .register(definition)
            .expect("the fixture key is unique");
        settings.push(value_table([
            ("key", Value::String(key)),
            (
                "label",
                Value::String("Performance root setting".to_owned()),
            ),
            (
                "description",
                Value::String("Performance fixture setting".to_owned()),
            ),
            ("category_key_path", Value::String(ROOT_CATEGORY.to_owned())),
            (
                "category_label_path",
                Value::String(root_label_path.clone()),
            ),
            ("scope", Value::String("user".to_owned())),
            ("schema", Value::String("bool".to_owned())),
            ("options", Value::Array(Vec::new())),
            ("requires_restart", Value::Boolean(false)),
        ]));
    }

    let authority = SettingsAuthority::from_registry_for_performance_benchmark(registry);
    assert_eq!(categories.len(), CATEGORY_COUNT);
    assert_eq!(settings.len(), DEFINITION_COUNT);
    assert_eq!(
        authority.snapshot().catalog().definitions().len(),
        DEFINITION_COUNT
    );
    let mut plugin_pages = Vec::with_capacity(PLUGIN_OWNER_COUNT);
    for owner in 0..PLUGIN_OWNER_COUNT {
        let bundle_id = format!("plugin.performance.owner_{owner:03}");
        let category_path = format!("settings.category.plugin.owner_{owner:03}");
        let category_label = format!("Owner {owner:03}");
        let category_label_path = format!("Plugins/Performance/{category_label}");
        categories.push(category_row(
            format!("plugin:{bundle_id}"),
            category_path.clone(),
            category_label_path.clone(),
            category_label,
        ));
        plugin_pages.push(value_table([
            (
                "id",
                Value::String(format!("plugin.performance.page_{owner:03}")),
            ),
            ("localization_bundle_id", Value::String(bundle_id)),
            (
                "label",
                Value::String(format!("Plugin settings {owner:03}")),
            ),
            (
                "description",
                Value::String("Performance fixture plugin page".to_owned()),
            ),
            ("category_key_path", Value::String(category_path)),
            ("category_label_path", Value::String(category_label_path)),
        ]));
    }
    assert_eq!(categories.len(), CATEGORY_COUNT + PLUGIN_CATEGORY_COUNT);

    let mut attributes = BTreeMap::from([
        (CATEGORIES.to_owned(), Value::Array(categories)),
        (SETTINGS.to_owned(), Value::Array(settings)),
        (PLUGIN_PAGES.to_owned(), Value::Array(plugin_pages)),
    ]);
    let middle_path = format!("{ROOT_CATEGORY}/{ROOT_CATEGORY}.group_{:04}", 500);
    let middle_id = format!("builtin|{middle_path}");
    let root_id = format!("builtin|{ROOT_CATEGORY}");
    let selected_plugin_bundle = format!("plugin.performance.owner_{SELECTED_PLUGIN_OWNER:03}");
    let selected_plugin_path = format!("settings.category.plugin.owner_{SELECTED_PLUGIN_OWNER:03}");
    let selected_plugin_id = format!("plugin:{selected_plugin_bundle}|{selected_plugin_path}");
    let scenarios = [
        (
            "middle_parent",
            middle_id.as_str(),
            SETTINGS_PER_CATEGORY,
            SETTINGS_PER_CATEGORY,
            0,
        ),
        (
            "high_fanout_root_parent",
            root_id.as_str(),
            DEFINITION_COUNT,
            DEFINITION_COUNT,
            0,
        ),
        (
            "selected_plugin_owner",
            selected_plugin_id.as_str(),
            0,
            1,
            1,
        ),
    ];

    for (scenario, category_id, expected_batch_rows, expected_rows, expected_plugin_pages) in
        scenarios
    {
        for _ in 0..WARMUP_COUNT {
            std::hint::black_box(refresh_sample(
                &authority,
                &mut attributes,
                category_id,
                expected_batch_rows,
                expected_rows,
                expected_plugin_pages,
            ));
        }

        let samples = (0..SAMPLE_COUNT)
            .map(|_| {
                refresh_sample(
                    &authority,
                    &mut attributes,
                    category_id,
                    expected_batch_rows,
                    expected_rows,
                    expected_plugin_pages,
                )
            })
            .collect::<Vec<_>>();

        println!(
            "EDITOR265_SETTINGS_REFRESH_BOUNDARY_BENCH_V2 scenario={scenario} \
             samples={SAMPLE_COUNT} definitions={DEFINITION_COUNT} \
             builtin_categories={CATEGORY_COUNT} plugin_categories={PLUGIN_CATEGORY_COUNT} \
             plugin_owner_pages={PLUGIN_OWNER_COUNT} full_settings_rows={} batch_rows={} \
             projected_rows={} matched_plugin_pages={} total_p50_ns={} total_p95_ns={} \
             authority_p50_ns={} authority_p95_ns={} payload_p50_ns={} payload_p95_ns={} \
             projection_p50_ns={} projection_p95_ns={} \
             allocations=external_managed_capture_required virtualization=downstream_not_measured",
            DEFINITION_COUNT,
            samples[0].batch_rows,
            samples[0].projected_rows,
            samples[0].matched_plugin_pages,
            percentile(&samples, 50, |sample| sample.total_ns),
            percentile(&samples, 95, |sample| sample.total_ns),
            percentile(&samples, 50, |sample| sample.authority_ns),
            percentile(&samples, 95, |sample| sample.authority_ns),
            percentile(&samples, 50, |sample| sample.payload_ns),
            percentile(&samples, 95, |sample| sample.payload_ns),
            percentile(&samples, 50, |sample| sample.projection_ns),
            percentile(&samples, 95, |sample| sample.projection_ns),
        );
    }
}
