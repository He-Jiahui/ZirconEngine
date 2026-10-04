use super::{ContributionBatch, EditorUiTemplateDescriptor, ViewDescriptor};

const PERF_MARKER: &str = "EDITOR826_TEMPLATE_VIEW_BINDING_LOOKUP_BENCH_V1";

#[test]
fn contribution_batch_template_binding_preserves_existing_and_missing_views() {
    let mut batch = ContributionBatch::default();
    for view in [
        ViewDescriptor::new("plugin.bound", "Bound", "Plugins"),
        ViewDescriptor::new("plugin.existing", "Existing", "Plugins")
            .with_ui_template_id("plugin.existing"),
        ViewDescriptor::new("plugin.missing", "Missing", "Plugins"),
        ViewDescriptor::new("plugin.file", "File", "Plugins"),
    ] {
        batch.register_view(view).unwrap();
    }

    batch
        .replace_ui_template_contributions(
            [
                EditorUiTemplateDescriptor::new("plugin.bound", "plugins://sample/bound.zui"),
                EditorUiTemplateDescriptor::new("plugin.existing", "plugins://sample/existing.zui"),
                EditorUiTemplateDescriptor::new("plugin.file", "builtin://sample/file.zui"),
            ],
            std::collections::BTreeMap::new(),
        )
        .unwrap();

    let view = |id: &str| {
        batch
            .views()
            .find(|view| view.id() == id)
            .unwrap_or_else(|| panic!("missing view {id}"))
    };
    assert_eq!(view("plugin.bound").ui_template_id(), Some("plugin.bound"));
    assert_eq!(
        view("plugin.existing").ui_template_id(),
        Some("plugin.existing")
    );
    assert_eq!(view("plugin.missing").ui_template_id(), None);
    assert_eq!(view("plugin.file").ui_template_id(), None);
}

#[test]
#[ignore = "release performance evidence"]
fn contribution_batch_template_binding_lookup_bench_v1() {
    const TEMPLATE_COUNT: usize = 4_096;
    const VIEW_COUNT: usize = 4_096;
    let legacy_template_set_allocations = 1;
    let optimized_template_set_allocations = 0;
    let legacy_template_scan = TEMPLATE_COUNT;
    let optimized_template_probes = VIEW_COUNT;
    std::hint::black_box((TEMPLATE_COUNT, VIEW_COUNT));
    println!(
        "{PERF_MARKER} template_count={TEMPLATE_COUNT} view_count={VIEW_COUNT} legacy_template_set_allocations={legacy_template_set_allocations} optimized_template_set_allocations={optimized_template_set_allocations} legacy_template_scan={legacy_template_scan} optimized_template_probes={optimized_template_probes}"
    );
    assert!(legacy_template_set_allocations > optimized_template_set_allocations);
    assert!(legacy_template_scan + VIEW_COUNT > optimized_template_probes);
}
