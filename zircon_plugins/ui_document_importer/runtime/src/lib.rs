use zircon_runtime::asset::{
    AssetImportContext, AssetImportError, AssetImportOutcome, ImportedAsset, UiV2ComponentAsset,
    UiV2StyleAsset, UiV2ViewAsset,
};
use zircon_runtime::ui::v2::UiZuiAssetLoader;
use zircon_runtime_interface::ui::v2::UiV2AssetKind;

mod capability;
mod plugin;

pub use capability::{
    IMPORTER_CAPABILITY, MODULE_NAME, NATIVE_PLUGIN_ID, NATIVE_REQUESTED_CAPABILITIES,
    NATIVE_RUNTIME_ENTRY, NATIVE_RUNTIME_REGISTRATION_MANIFEST, PLUGIN_ID, RUNTIME_CAPABILITY,
    RUNTIME_CRATE_NAME, UI_DOCUMENT_IMPORTER_DECLARATION,
};
pub use plugin::{
    asset_importer_descriptors, dist_module_manifest, module_descriptor, package_manifest,
    plugin_registration, runtime_capabilities, runtime_module_manifest, runtime_plugin,
    runtime_plugin_descriptor, runtime_selection, supported_platforms, supported_targets,
    UiDocumentImporterRuntimePlugin, UI_DOCUMENT_IMPORTER_DIST_CRATE_NAME,
    UI_DOCUMENT_IMPORTER_DIST_RUNTIME_ENTRY,
};

pub fn import_ui_zui_document(
    context: &AssetImportContext,
) -> Result<AssetImportOutcome, AssetImportError> {
    let document = context.source_str()?;
    let parsed = UiZuiAssetLoader::load_zui_str(document).map_err(|source| {
        AssetImportError::UiV2Document {
            context: "parse .zui ui asset",
            source: source.into(),
        }
    })?;
    let imported = match parsed.asset.kind {
        UiV2AssetKind::View => ImportedAsset::UiV2View(UiV2ViewAsset { document: parsed }),
        UiV2AssetKind::Style | UiV2AssetKind::ThemeTokens => {
            ImportedAsset::UiV2Style(UiV2StyleAsset { document: parsed })
        }
        UiV2AssetKind::Component => {
            ImportedAsset::UiV2Component(UiV2ComponentAsset { document: parsed })
        }
    };
    Ok(AssetImportOutcome::new(context.uri.clone(), imported))
}

#[cfg(test)]
mod tests {
    use super::*;
    use zircon_runtime::asset::AssetKind;
    use zircon_runtime::core::framework::project::ExportPackagingStrategy;

    #[test]
    fn package_declares_zui_document_importer() {
        let manifest = package_manifest();

        assert_eq!(manifest.id, PLUGIN_ID);
        assert!(manifest
            .capabilities
            .contains(&RUNTIME_CAPABILITY.to_string()));
        assert_eq!(manifest.asset_importers.len(), 1);
        assert!(manifest.asset_importers.iter().any(|importer| {
            importer.id == "ui_document_importer.zui_document"
                && importer.full_suffixes.contains(&".zui".to_string())
                && importer.importer_version == 2
                && importer.allows_output_kind(AssetKind::UiWidget)
                && importer.allows_output_kind(AssetKind::UiLayout)
                && importer.allows_output_kind(AssetKind::UiStyle)
        }));
        assert!(manifest.asset_importers.iter().all(|importer| {
            importer.full_suffixes.contains(&".zui".to_string())
                && importer.importer_version == 2
                && !importer.full_suffixes.contains(&".ui.json".to_string())
                && !importer.full_suffixes.contains(&".v2.ui.toml".to_string())
                && !importer.source_extensions.contains(&"uidoc".to_string())
        }));
    }

    #[test]
    fn declaration_projects_ui_document_package_metadata() {
        let descriptor = runtime_plugin_descriptor();
        let manifest = package_manifest();

        assert_eq!(
            descriptor.package_id(),
            UI_DOCUMENT_IMPORTER_DECLARATION.id()
        );
        assert_eq!(
            descriptor.category(),
            UI_DOCUMENT_IMPORTER_DECLARATION.category()
        );
        assert_eq!(
            descriptor.target_modes(),
            UI_DOCUMENT_IMPORTER_DECLARATION.target_modes()
        );
        assert_eq!(
            descriptor.capabilities(),
            runtime_capabilities()
                .iter()
                .map(|capability| capability.to_string())
                .collect::<Vec<_>>()
        );
        assert_eq!(
            manifest.supported_platforms.as_slice(),
            UI_DOCUMENT_IMPORTER_DECLARATION.supported_platforms()
        );
        assert_eq!(
            manifest.default_packaging.as_slice(),
            UI_DOCUMENT_IMPORTER_DECLARATION.default_packaging()
        );
    }

    #[test]
    fn package_manifest_declares_ui_document_importer_dist_contract() {
        let manifest = package_manifest();
        let distribution = manifest
            .distribution
            .as_ref()
            .expect("UI document importer package exposes dist metadata");

        assert!(manifest
            .default_packaging
            .contains(&ExportPackagingStrategy::NativeDynamic));
        assert_eq!(distribution.forms, vec!["dist"]);
        assert_eq!(
            distribution.default_packaging,
            vec![ExportPackagingStrategy::NativeDynamic]
        );
        assert_eq!(distribution.abi_version, Some(3));
        assert_eq!(
            distribution.dist_crate,
            UI_DOCUMENT_IMPORTER_DIST_CRATE_NAME
        );
        assert_eq!(
            distribution.runtime_entry,
            UI_DOCUMENT_IMPORTER_DIST_RUNTIME_ENTRY
        );

        let dist_module = manifest
            .modules
            .iter()
            .find(|module| module.name == "ui_document_importer.dist")
            .expect("UI document importer package includes native dist module");
        assert_eq!(
            dist_module.kind,
            zircon_runtime::plugin::PluginModuleKind::Native
        );
        assert_eq!(dist_module.crate_name, UI_DOCUMENT_IMPORTER_DIST_CRATE_NAME);
        assert!(dist_module
            .capabilities
            .contains(&IMPORTER_CAPABILITY.to_string()));
    }

    #[test]
    fn plugin_toml_declares_zui_document_importer() {
        let manifest = include_str!("../../plugin.toml");

        assert_eq!(manifest.matches("[[asset_importers]]").count(), 1);
        assert!(manifest.contains("id = \"ui_document_importer.zui_document\""));
        assert!(manifest.contains("full_suffixes = [\".zui\"]"));
        assert!(manifest.contains("output_kind = \"UiWidget\""));
        assert!(manifest.contains("additional_output_kinds = [\"UiLayout\", \"UiStyle\"]"));
        assert!(manifest.contains("importer_version = 2"));
        assert!(!manifest.contains("ui_document_importer.serialized_json"));
        assert!(!manifest.contains("ui_document_importer.serialized_binary"));
        assert!(!manifest.contains("ui_document_importer.zui_component"));
        assert!(!manifest.contains("full_suffixes = [\".v2.ui.toml\"]"));
        assert!(!manifest.contains("full_suffixes = [\".ui.toml\"]"));
        assert!(!manifest.contains(".ui.json"));
        assert!(!manifest.contains("source_extensions = [\"uidoc\"]"));
    }

    #[test]
    fn registration_contributes_module_and_importers() {
        let report = plugin_registration();

        assert!(report.is_success(), "{:?}", report.diagnostics);
        assert!(report
            .extensions
            .modules()
            .iter()
            .any(|module| module.name == MODULE_NAME));
        assert_eq!(report.extensions.asset_importers().descriptors().len(), 1);
    }

    #[test]
    fn zui_importer_decodes_single_component_asset() {
        let report = plugin_registration();
        let importer = report
            .extensions
            .asset_importers()
            .select(std::path::Path::new("toolbar.zui"))
            .unwrap();
        let context = zircon_runtime::asset::AssetImportContext::new(
            "toolbar.zui".into(),
            zircon_runtime::asset::AssetUri::parse("res://ui/toolbar.zui").unwrap(),
            br#"
[asset]
kind = "component"
id = "toolbar"
version = 2

[components.Toolbar]
root = "root"

[nodes.root]
component = "Container"
"#
            .to_vec(),
            Default::default(),
        );

        let outcome = importer.import(&context).unwrap();
        let imported = &outcome.root_entry().expect("root UI asset entry").asset;

        assert!(matches!(
            imported,
            zircon_runtime::asset::ImportedAsset::UiV2Component(_)
        ));
    }

    #[test]
    fn zui_importer_decodes_view_and_style_assets() {
        let report = plugin_registration();
        let importer = report
            .extensions
            .asset_importers()
            .select(std::path::Path::new("panel.zui"))
            .unwrap();

        let view = importer
            .import(&zircon_runtime::asset::AssetImportContext::new(
                "panel.zui".into(),
                zircon_runtime::asset::AssetUri::parse("res://ui/panel.zui").unwrap(),
                minimal_view_zui().as_bytes().to_vec(),
                Default::default(),
            ))
            .unwrap();
        let style = importer
            .import(&zircon_runtime::asset::AssetImportContext::new(
                "theme.zui".into(),
                zircon_runtime::asset::AssetUri::parse("res://ui/theme.zui").unwrap(),
                minimal_style_zui().as_bytes().to_vec(),
                Default::default(),
            ))
            .unwrap();

        assert!(matches!(
            view.root_entry().expect("root UI view entry").asset,
            zircon_runtime::asset::ImportedAsset::UiV2View(_)
        ));
        assert!(matches!(
            style.root_entry().expect("root UI style entry").asset,
            zircon_runtime::asset::ImportedAsset::UiV2Style(_)
        ));
    }

    #[test]
    fn registration_does_not_select_legacy_ui_document_formats() {
        let report = plugin_registration();
        let importers = report.extensions.asset_importers();

        assert!(importers
            .select(std::path::Path::new("layout.ui.json"))
            .is_err());
        assert!(importers
            .select(std::path::Path::new("layout.v2.ui.toml"))
            .is_err());
        assert!(importers
            .select(std::path::Path::new("layout.uidoc"))
            .is_err());
    }

    #[test]
    fn plugins07_importer_hotpath_ui_document_uses_borrowed_source_snapshot() {
        let context = zircon_runtime::asset::AssetImportContext::new(
            "panel.zui".into(),
            zircon_runtime::asset::AssetUri::parse("res://ui/panel.zui").unwrap(),
            minimal_view_zui().as_bytes().to_vec(),
            Default::default(),
        );

        let borrowed = context.source_str().unwrap();
        let outcome = import_ui_zui_document(&context).unwrap();

        assert_eq!(borrowed.as_ptr(), context.source_bytes.as_ptr());
        assert!(matches!(
            outcome.root_entry().expect("root UI view entry").asset,
            zircon_runtime::asset::ImportedAsset::UiV2View(_)
        ));
    }

    #[test]
    #[ignore = "release performance gate; run through the Plugins07 coordinator validator"]
    fn plugins07_importer_hotpath_release_ui_document_borrowed_source_p95_gate() {
        use std::hint::black_box;
        use std::time::Instant;

        const SAMPLE_PAIRS: usize = 21;
        const SOURCE_BYTES: usize = 2_097_152;
        const ITERATIONS: usize = 16;
        const THRESHOLD_PERCENT: u128 = 20;
        let context = zircon_runtime::asset::AssetImportContext::new(
            "large.zui".into(),
            zircon_runtime::asset::AssetUri::parse("res://ui/large.zui").unwrap(),
            vec![b'x'; SOURCE_BYTES],
            Default::default(),
        );
        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for pair in 0..SAMPLE_PAIRS {
            let legacy = || {
                let started = Instant::now();
                let mut bytes = 0_usize;
                for _ in 0..ITERATIONS {
                    let source = context.source_text().unwrap();
                    bytes += black_box(source.as_str()).len();
                }
                black_box(bytes);
                started.elapsed().as_nanos()
            };
            let optimized = || {
                let started = Instant::now();
                let mut bytes = 0_usize;
                for _ in 0..ITERATIONS {
                    let source = context.source_str().unwrap();
                    bytes += black_box(source).len();
                }
                black_box(bytes);
                started.elapsed().as_nanos()
            };
            if pair % 2 == 0 {
                legacy_samples.push(legacy());
                optimized_samples.push(optimized());
            } else {
                optimized_samples.push(optimized());
                legacy_samples.push(legacy());
            }
        }

        emit_ui_source_performance_gate(
            &legacy_samples,
            &optimized_samples,
            THRESHOLD_PERCENT,
            &format!(
                "source_bytes={SOURCE_BYTES} iterations_per_sample={ITERATIONS} legacy_cloned_bytes_per_sample={} optimized_cloned_bytes_per_sample=0",
                SOURCE_BYTES * ITERATIONS
            ),
        );
    }

    fn emit_ui_source_performance_gate(
        legacy_samples: &[u128],
        optimized_samples: &[u128],
        threshold_percent: u128,
        workload: &str,
    ) {
        let legacy_p95 = nearest_rank_ui_p95(legacy_samples);
        let optimized_p95 = nearest_rank_ui_p95(optimized_samples);
        let improvement_percent =
            legacy_p95.saturating_sub(optimized_p95).saturating_mul(100) / legacy_p95.max(1);
        println!(
            "PERF_RESULT plugins07_ui_document_borrowed_source sample_pairs=21 order=alternating_legacy_first_even {workload} legacy_ns={} optimized_ns={} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent={threshold_percent}",
            ui_samples_csv(legacy_samples),
            ui_samples_csv(optimized_samples),
        );
        assert!(
            improvement_percent >= threshold_percent,
            "UI document borrowed source must improve P95 by at least {threshold_percent}% (legacy={legacy_p95}ns optimized={optimized_p95}ns improvement={improvement_percent}%)"
        );
    }

    fn nearest_rank_ui_p95(samples: &[u128]) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        sorted[(sorted.len() * 95).div_ceil(100).saturating_sub(1)]
    }

    fn ui_samples_csv(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }

    fn minimal_view_zui() -> &'static str {
        r#"
[asset]
kind = "view"
id = "ui_document_importer.test.panel"
version = 2

[root]
node = "root"

[nodes.root]
component = "Text"
props = { text = "Panel" }
"#
    }

    fn minimal_style_zui() -> &'static str {
        r##"
[asset]
kind = "style"
id = "ui_document_importer.test.style"
version = 2

[[stylesheets]]
id = "test_style"

[[stylesheets.rules]]
selector = "Text"
set = { foreground = { color = "#ffffff" } }
"##
    }
}
