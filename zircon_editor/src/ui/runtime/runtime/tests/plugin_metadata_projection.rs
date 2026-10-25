use super::*;
use crate::ui::host::EditorPluginPackageMetadata;
use crate::ui::layouts::windows::workbench_host_window::{
    ModulePluginStatusPayload, ModulePluginsPanePayload,
};
use zircon_runtime::plugin::PluginMaturity;

fn payload(metadata: Option<EditorPluginPackageMetadata>) -> PanePayload {
    PanePayload::ModulePluginsV1(ModulePluginsPanePayload {
        diagnostics: String::new(),
        plugins: vec![ModulePluginStatusPayload {
            plugin_id: "vendor.plugin".into(),
            display_name: "Actual plugin".into(),
            package_source: "native".into(),
            package_metadata: metadata,
            load_state: "loaded".into(),
            enabled: true,
            required: false,
            target_modes: "editor".into(),
            packaging: "native".into(),
            runtime_crate: String::new(),
            editor_crate: String::new(),
            runtime_capabilities: String::new(),
            editor_capabilities: String::new(),
            optional_features: String::new(),
            feature_action_label: String::new(),
            feature_action_id: String::new(),
            diagnostics: String::new(),
            primary_action_label: "Disable".into(),
            primary_action_id: "existing-qualified-operation".into(),
            packaging_action_label: String::new(),
            packaging_action_id: String::new(),
            target_modes_action_label: String::new(),
            target_modes_action_id: String::new(),
            unload_action_label: String::new(),
            unload_action_id: String::new(),
            hot_reload_action_label: String::new(),
            hot_reload_action_id: String::new(),
        }],
        detail_generation: 7,
        detail_locale: "zh-CN".into(),
        selected_plugin_id: "vendor.plugin".into(),
        focused_control: "package".into(),
        details: None,
    })
}

#[test]
fn package_metadata_projects_typed_maturity_and_empty_manifest_fields() {
    for maturity in [
        PluginMaturity::Core,
        PluginMaturity::Stable,
        PluginMaturity::Beta,
        PluginMaturity::Experimental,
        PluginMaturity::Externalized,
        PluginMaturity::Stub,
        PluginMaturity::Deprecated,
    ] {
        let expected = Value::try_from(maturity).expect("canonical maturity serializer");
        let mut attributes = BTreeMap::new();
        inject_payload_attributes(
            &mut attributes,
            &payload(Some(EditorPluginPackageMetadata {
                version: "1.2.3".into(),
                category: String::new(),
                description: "中文描述 / manifest text\nsecond line".into(),
                maturity,
            })),
        );
        let row = &attributes["payload_plugins"].as_array().unwrap()[0];
        let metadata = &row["package_metadata"];
        assert_eq!(metadata["version"].as_str(), Some("1.2.3"));
        assert_eq!(metadata["category"].as_str(), Some(""));
        assert_eq!(
            metadata["description"].as_str(),
            Some("中文描述 / manifest text\nsecond line")
        );
        assert_eq!(metadata["maturity"], expected);
        assert_eq!(row["plugin_id"].as_str(), Some("vendor.plugin"));
        assert_eq!(row["load_state"].as_str(), Some("loaded"));
    }
}

#[test]
fn package_metadata_removal_rebuilds_attributes_without_stale_manifest_fields() {
    let mut attributes = BTreeMap::new();
    inject_payload_attributes(
        &mut attributes,
        &payload(Some(EditorPluginPackageMetadata {
            version: "previous-catalog-version".into(),
            category: "previous category".into(),
            description: "previous description".into(),
            maturity: PluginMaturity::Stable,
        })),
    );
    let previous = attributes.clone();
    let mut next = payload(None);
    let PanePayload::ModulePluginsV1(data) = &mut next else {
        unreachable!()
    };
    data.plugins[0].load_state = "missing package".into();
    inject_payload_attributes(&mut attributes, &next);

    let row = &attributes["payload_plugins"].as_array().unwrap()[0];
    assert!(row.get("package_metadata").is_none());
    assert_eq!(row["load_state"].as_str(), Some("missing package"));
    assert_eq!(attributes["payload_plugin_count"].as_integer(), Some(1));
    assert_eq!(
        previous["payload_plugins"][0]["package_metadata"]["version"].as_str(),
        Some("previous-catalog-version")
    );
}
