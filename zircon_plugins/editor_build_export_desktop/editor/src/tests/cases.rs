use super::*;

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::path::{Path, PathBuf};

    use zircon_editor::core::asset::AssetTypeId;
    use zircon_runtime::core::framework::platform::RuntimeTargetMode;
    use zircon_runtime::core::framework::project::ExportPackagingStrategy;
    use zircon_runtime::plugin::PluginModuleKind;
    use zircon_runtime::ui::v2::UiZuiAssetLoader;
    use zircon_runtime_interface::ui::v2::UiV2AssetKind;

    use super::*;

    #[test]
    fn desktop_export_plugin_contributes_panel_operations_and_reports() {
        let registration = plugin_registration();

        assert!(registration.is_success(), "{:?}", registration.diagnostics);
        assert_eq!(
            registration.capabilities,
            vec![
                CAPABILITY.to_string(),
                DIAGNOSTICS_CAPABILITY.to_string(),
                NATIVE_DYNAMIC_REPORT_CAPABILITY.to_string()
            ]
        );
        assert!(registration
            .extensions
            .views()
            .iter()
            .any(|view| view.id() == EXPORT_VIEW_ID));
        assert!(registration
            .extensions
            .ui_templates()
            .iter()
            .any(|template| template.id() == SOURCE_TEMPLATE_REPORT_ID));
        assert!(registration
            .extensions
            .ui_templates()
            .iter()
            .any(|template| template.id() == LIBRARY_EMBED_REPORT_ID));
        assert!(registration
            .extensions
            .ui_templates()
            .iter()
            .any(|template| template.id() == NATIVE_DYNAMIC_REPORT_ID));
        assert!(registration
            .extensions
            .commands()
            .commands()
            .any(|operation| operation.id().as_str() == EXPORT_OPERATION_NATIVE_DYNAMIC));
        assert!(registration
            .extensions
            .menu_items()
            .iter()
            .any(|menu| menu.path() == "Project/Export/Desktop/Native Dynamic"));
        assert!(registration
            .extensions
            .asset_type_contributions()
            .iter()
            .any(|contribution| contribution.asset_type().as_str() == EXPORT_PROFILE_ASSET_KIND));
        assert!(registration
            .extensions
            .inspector_customizations()
            .iter()
            .any(|customization| customization.target_type() == EXPORT_PROFILE_COMPONENT));
    }

    #[test]
    fn desktop_export_package_manifest_declares_editor_only_metadata() {
        let manifest = package_manifest();

        assert_eq!(manifest.category, "platform");
        assert_eq!(
            manifest.supported_targets,
            vec![zircon_runtime::core::framework::platform::RuntimeTargetMode::EditorHost]
        );
        assert!(manifest.capabilities.contains(&CAPABILITY.to_string()));
        assert!(manifest
            .modules
            .iter()
            .any(|module| module.kind == PluginModuleKind::Editor
                && module.crate_name == "zircon_plugin_editor_build_export_desktop_editor"));
        assert!(manifest
            .default_packaging
            .contains(&ExportPackagingStrategy::SourceTemplate));
    }

    #[test]
    fn desktop_export_package_manifest_declares_editor_dist_contract() {
        let manifest = package_manifest();

        assert!(manifest
            .default_packaging
            .contains(&ExportPackagingStrategy::NativeDynamic));
        assert!(manifest.modules.iter().any(|module| {
            module.name == "editor_build_export_desktop.dist"
                && module.kind == PluginModuleKind::Native
                && module.crate_name == EDITOR_BUILD_EXPORT_DESKTOP_DIST_CRATE_NAME
                && module.target_modes == vec![RuntimeTargetMode::EditorHost]
                && module
                    .capabilities
                    .contains(&NATIVE_DYNAMIC_REPORT_CAPABILITY.to_string())
        }));

        let distribution = manifest
            .distribution
            .as_ref()
            .expect("desktop export should declare native dynamic distribution");
        assert_eq!(distribution.forms, vec!["dist".to_string()]);
        assert_eq!(
            distribution.default_packaging,
            vec![ExportPackagingStrategy::NativeDynamic]
        );
        assert_eq!(
            distribution.dist_crate,
            EDITOR_BUILD_EXPORT_DESKTOP_DIST_CRATE_NAME
        );
        assert_eq!(
            distribution.editor_entry,
            EDITOR_BUILD_EXPORT_DESKTOP_DIST_EDITOR_ENTRY
        );
        assert!(distribution.runtime_entry.is_empty());
    }

    #[test]
    fn export_wizard_descriptor_covers_build_layout_stages_and_reports() {
        let wizard = export_wizard_descriptor();

        assert_eq!(wizard.template_id, EXPORT_TEMPLATE_ID);
        assert_eq!(
            wizard.layout_reference,
            "docs/ui-and-layout/ai-workbench-style/ai-build-export-layout.png"
        );
        assert_eq!(wizard.regions.len(), 3);
        assert!(wizard
            .regions
            .iter()
            .any(|region| region.region == ExportWizardRegion::ProfileTree));
        assert_eq!(
            wizard
                .stages
                .iter()
                .map(|stage| stage.stage)
                .collect::<Vec<_>>(),
            vec![
                zircon_runtime_interface::export::ExportStage::Validate,
                zircon_runtime_interface::export::ExportStage::SourceTemplate,
                zircon_runtime_interface::export::ExportStage::NativeDynamic,
                zircon_runtime_interface::export::ExportStage::CompileHost,
                zircon_runtime_interface::export::ExportStage::CookAssets,
                zircon_runtime_interface::export::ExportStage::Pack,
                zircon_runtime_interface::export::ExportStage::PlatformBundle,
                zircon_runtime_interface::export::ExportStage::Report,
            ]
        );
        assert_eq!(
            wizard
                .stage(zircon_runtime_interface::export::ExportStage::Pack)
                .expect("pack stage descriptor")
                .report_path,
            "report.json"
        );
        let source_template_report = wizard
            .report_view("source_template")
            .expect("source template report");
        assert_eq!(
            source_template_report.required_stage,
            zircon_runtime_interface::export::ExportStage::SourceTemplate
        );
        assert_eq!(
            source_template_report.template_document,
            SOURCE_TEMPLATE_REPORT_DOCUMENT
        );
        assert_eq!(
            source_template_report.summary_entry_keys,
            SOURCE_TEMPLATE_REPORT_SUMMARY_ENTRY_KEYS
        );
        assert_eq!(
            source_template_report.template_control_ids,
            SOURCE_TEMPLATE_REPORT_TEMPLATE_CONTROL_IDS
        );
        assert!(source_template_report
            .summary_entry_keys
            .contains(&REPORT_EXPORT_PLAN_STRATEGIES_ENTRY_KEY));

        let library_embed_report = wizard
            .report_view("library_embed")
            .expect("library embed report");
        assert_eq!(
            library_embed_report.required_stage,
            zircon_runtime_interface::export::ExportStage::CompileHost
        );
        assert_eq!(
            library_embed_report.template_document,
            LIBRARY_EMBED_REPORT_DOCUMENT
        );
        assert_eq!(
            library_embed_report.summary_entry_keys,
            LIBRARY_EMBED_REPORT_SUMMARY_ENTRY_KEYS
        );
        assert_eq!(
            library_embed_report.template_control_ids,
            LIBRARY_EMBED_REPORT_TEMPLATE_CONTROL_IDS
        );
        assert!(library_embed_report
            .summary_entry_keys
            .contains(&REPORT_PIPELINE_REPORT_ENTRY_KEY));

        let native_dynamic_report = wizard
            .report_view("native_dynamic")
            .expect("native dynamic report");
        assert_eq!(native_dynamic_report.template_id, NATIVE_DYNAMIC_REPORT_ID);
        assert_eq!(
            native_dynamic_report.required_stage,
            zircon_runtime_interface::export::ExportStage::NativeDynamic
        );
        assert_eq!(
            native_dynamic_report.template_document,
            NATIVE_DYNAMIC_REPORT_DOCUMENT
        );
        assert_eq!(
            native_dynamic_report.summary_entry_keys,
            NATIVE_DYNAMIC_REPORT_SUMMARY_ENTRY_KEYS
        );
        assert_eq!(
            native_dynamic_report.template_control_ids,
            NATIVE_DYNAMIC_REPORT_TEMPLATE_CONTROL_IDS
        );
        assert!(native_dynamic_report
            .summary_entry_keys
            .contains(&REPORT_NATIVE_PLUGINS_PAYLOAD_PACKAGE_IDS_ENTRY_KEY));

        for report in &wizard.report_views {
            assert_report_template_contains_controls(report);
        }
        assert_eq!(stage_progress_kinds().len(), 4);
    }

    #[test]
    fn desktop_export_private_template_assets_match_registered_documents() {
        let registration = plugin_registration();

        assert!(registration.is_success(), "{:?}", registration.diagnostics);
        let template_documents = registration
            .extensions
            .ui_templates()
            .into_iter()
            .map(|template| {
                (
                    template.id().to_string(),
                    template.ui_document().to_string(),
                )
            })
            .collect::<BTreeMap<_, _>>();
        for (template_id, document) in EXPORT_UI_TEMPLATE_DOCUMENTS {
            assert_eq!(
                template_documents.get(*template_id).map(String::as_str),
                Some(*document),
                "registered template {template_id} should point at its private asset"
            );
            assert_view_template_asset(document, template_id);
        }

        let profile_type = AssetTypeId::parse(EXPORT_PROFILE_ASSET_KIND).unwrap();
        let profile_template = registration
            .extensions
            .asset_type_contributions()
            .into_iter()
            .find(|contribution| contribution.asset_type() == &profile_type)
            .and_then(|contribution| contribution.creation_templates().first())
            .expect("desktop export profile template");
        assert_eq!(
            profile_template.default_document(),
            Some(EXPORT_PROFILE_TEMPLATE_DOCUMENT)
        );
        assert_profile_template_asset(EXPORT_PROFILE_TEMPLATE_DOCUMENT);

        let profile_drawer = registration
            .extensions
            .inspector_customizations()
            .into_iter()
            .find(|customization| customization.target_type() == EXPORT_PROFILE_COMPONENT)
            .expect("desktop export profile customization");
        assert_eq!(
            profile_drawer.surface().ui_document(),
            EXPORT_PROFILE_DRAWER_DOCUMENT
        );
        assert_component_template_asset(EXPORT_PROFILE_DRAWER_DOCUMENT, "ExportProfileDrawer");
    }

    fn assert_view_template_asset(document: &str, expected_id: &str) {
        let path = plugin_asset_path(document);
        let asset = UiZuiAssetLoader::load_zui_file(&path)
            .unwrap_or_else(|error| panic!("{} should parse: {error}", path.display()));

        assert_eq!(asset.asset.kind, UiV2AssetKind::View);
        assert_eq!(asset.asset.id, expected_id);
        assert!(
            asset.root_node_id().is_some(),
            "view template {expected_id} should declare a root node"
        );
    }

    fn assert_report_template_contains_controls(report: &ExportWizardReportViewDescriptor) {
        assert!(
            EXPORT_REPORT_TEMPLATE_DOCUMENTS
                .iter()
                .any(|(template_id, document)| {
                    *template_id == report.template_id && *document == report.template_document
                }),
            "report template {} should register {}",
            report.template_id,
            report.template_document
        );
        let path = plugin_asset_path(report.template_document);
        let asset = UiZuiAssetLoader::load_zui_file(&path)
            .unwrap_or_else(|error| panic!("{} should parse: {error}", path.display()));

        for control_id in report.template_control_ids {
            assert!(
                asset
                    .nodes
                    .values()
                    .any(|node| node.control_id.as_deref() == Some(*control_id)),
                "report template {} should define control id {control_id}",
                report.template_id
            );
        }
    }

    fn assert_component_template_asset(document: &str, component_name: &str) {
        let path = plugin_asset_path(document);
        let asset = UiZuiAssetLoader::load_zui_file(&path)
            .unwrap_or_else(|error| panic!("{} should parse: {error}", path.display()));

        assert_eq!(asset.asset.kind, UiV2AssetKind::Component);
        assert!(
            asset.components.contains_key(component_name),
            "component template {} should define {component_name}",
            path.display()
        );
    }

    fn assert_profile_template_asset(document: &str) {
        let path = plugin_asset_path(document);
        let source = std::fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("{} should be readable: {error}", path.display()));

        assert!(source.contains("[export_profile]"));
        assert!(source.contains("platform = \"windows-x86_64\""));
        assert!(source.contains("path = \"library_embed\""));
        assert!(source.contains("asset_filter = \"shipping\""));
    }

    fn plugin_asset_path(document: &str) -> PathBuf {
        const PREFIX: &str = "asset://editor_build_export_desktop/";

        let relative = document
            .strip_prefix(PREFIX)
            .unwrap_or_else(|| panic!("document {document} should use {PREFIX}"));
        let plugin_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("editor crate should be inside plugin root");
        let mut path = plugin_root.to_path_buf();
        for segment in relative.split('/') {
            path.push(segment);
        }
        path
    }
}
