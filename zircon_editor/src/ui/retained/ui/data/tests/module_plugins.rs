use super::*;
use crate::ui::layouts::views::blank_viewport_chrome;
use crate::ui::layouts::windows::workbench_host_window::{
    ModulePluginStatusViewData, ModulePluginsPaneViewData, PaneNativeBodyData,
};

#[test]
fn module_plugins_pane_projects_visual_rows_and_action_buttons() {
    let pane = module_plugins_pane_fixture();
    let data = to_host_contract_module_plugins_pane_from_host_pane(
        &pane,
        PaneContentSize::new(480.0, 260.0),
    );

    let action_ids = (0..data.nodes.row_count())
        .filter_map(|row| data.nodes.row_data(row))
        .filter(|node| node.control_id.as_str() == "ModulePluginAction")
        .map(|node| node.action_id.to_string())
        .collect::<Vec<_>>();

    assert_eq!(
        action_ids,
        vec![
            "workbench.plugin.disable.physics",
            "workbench.plugin.feature.enable.physics.physics.raycast_queries",
            "workbench.plugin.packaging.next.physics",
            "workbench.plugin.target_modes.next.physics",
            "workbench.plugin.unload.physics",
            "workbench.plugin.hot_reload.physics",
        ]
    );

    let row_node = (0..data.nodes.row_count())
        .filter_map(|row| data.nodes.row_data(row))
        .find(|node| node.control_id.as_str() == "ModulePluginRow.physics")
        .expect("module plugin row should be projected");
    assert_eq!(row_node.actions.row_count(), 6);
    let feature_node = (0..data.nodes.row_count())
        .filter_map(|row| data.nodes.row_data(row))
        .find(|node| node.control_id.as_str() == "ModulePluginFeatures.physics")
        .expect("module plugin optional feature summary should be projected");
    assert_eq!(feature_node.text.to_string(), "Ray Cast Queries [ready]");
}

#[test]
fn module_plugins_row_actions_skip_label_only_action_slots() {
    let mut pane = module_plugins_pane_fixture();
    let mut plugin = module_plugin_status_fixture();
    plugin.feature_action_id = "".into();
    pane.native_body.module_plugins.plugins = model_rc(vec![plugin]);

    let data = to_host_contract_module_plugins_pane_from_host_pane(
        &pane,
        PaneContentSize::new(480.0, 260.0),
    );

    let action_ids = (0..data.nodes.row_count())
        .filter_map(|row| data.nodes.row_data(row))
        .filter(|node| node.control_id.as_str() == "ModulePluginAction")
        .map(|node| node.action_id.to_string())
        .collect::<Vec<_>>();

    assert_eq!(
        action_ids,
        vec![
            "workbench.plugin.disable.physics",
            "workbench.plugin.packaging.next.physics",
            "workbench.plugin.target_modes.next.physics",
            "workbench.plugin.unload.physics",
            "workbench.plugin.hot_reload.physics",
        ]
    );

    let row_node = (0..data.nodes.row_count())
        .filter_map(|row| data.nodes.row_data(row))
        .find(|node| node.control_id.as_str() == "ModulePluginRow.physics")
        .expect("module plugin row should be projected");
    assert_eq!(row_node.actions.row_count(), 5);
}

#[test]
fn module_plugins_host_projection_reuses_models_for_one_source_generation() {
    let pane = module_plugins_pane_fixture();
    let mut cache = ModulePluginsPaneProjectionCache::default();
    let first = to_host_contract_module_plugins_pane_from_host_pane_with_cache(
        &pane,
        PaneContentSize::new(480.0, 260.0),
        &mut cache,
    );
    let second = to_host_contract_module_plugins_pane_from_host_pane_with_cache(
        &pane,
        PaneContentSize::new(480.0, 260.0),
        &mut cache,
    );

    assert!(first.nodes.shares_values_with(&second.nodes));
}

#[test]
fn module_plugins_host_projection_invalidates_for_new_plugin_storage() {
    let mut pane = module_plugins_pane_fixture();
    let mut cache = ModulePluginsPaneProjectionCache::default();
    let first = to_host_contract_module_plugins_pane_from_host_pane_with_cache(
        &pane,
        PaneContentSize::new(480.0, 260.0),
        &mut cache,
    );
    pane.native_body.module_plugins.plugins = model_rc(vec![module_plugin_status_fixture()]);
    let second = to_host_contract_module_plugins_pane_from_host_pane_with_cache(
        &pane,
        PaneContentSize::new(480.0, 260.0),
        &mut cache,
    );

    assert!(!first.nodes.shares_values_with(&second.nodes));
}

fn module_plugins_pane_fixture() -> PaneData {
    let module_plugins = module_plugins_fixture();
    PaneData {
        id: "editor.module_plugins#1".into(),
        slot: "left_bottom".into(),
        kind: "ModulePlugins".into(),
        title: "Plugin Manager".into(),
        icon_key: "plugin".into(),
        subtitle: "Project Plugins".into(),
        info: "Builtin and native plugin packages".into(),
        show_empty: false,
        empty_title: "".into(),
        empty_body: "".into(),
        primary_action_label: "".into(),
        primary_action_id: "".into(),
        secondary_action_label: "".into(),
        secondary_action_id: "".into(),
        secondary_hint: "".into(),
        show_toolbar: false,
        viewport: blank_viewport_chrome(),
        native_body: PaneNativeBodyData {
            module_plugins,
            ..PaneNativeBodyData::default()
        },
        pane_presentation: None,
    }
}

fn module_plugins_fixture() -> ModulePluginsPaneViewData {
    ModulePluginsPaneViewData {
        plugins: model_rc(vec![module_plugin_status_fixture()]),
        diagnostics: "plugin catalog ready".into(),
        ..ModulePluginsPaneViewData::default()
    }
}

fn module_plugin_status_fixture() -> ModulePluginStatusViewData {
    ModulePluginStatusViewData {
        plugin_id: "physics".into(),
        display_name: "Physics".into(),
        package_source: "builtin".into(),
        package_metadata: None,
        load_state: "loaded".into(),
        enabled: true,
        required: false,
        target_modes: "editor, runtime".into(),
        packaging: "linked".into(),
        runtime_crate: "zircon_plugins_physics_runtime".into(),
        editor_crate: "zircon_plugins_physics_editor".into(),
        runtime_capabilities: "simulation".into(),
        editor_capabilities: "inspector".into(),
        optional_features: "Ray Cast Queries [ready]".into(),
        feature_action_label: "Enable Feature".into(),
        feature_action_id: "workbench.plugin.feature.enable.physics.physics.raycast_queries".into(),
        diagnostics: "".into(),
        primary_action_label: "Disable".into(),
        primary_action_id: "workbench.plugin.disable.physics".into(),
        packaging_action_label: "Cycle linked".into(),
        packaging_action_id: "workbench.plugin.packaging.next.physics".into(),
        target_modes_action_label: "Cycle targets".into(),
        target_modes_action_id: "workbench.plugin.target_modes.next.physics".into(),
        unload_action_label: "Unload".into(),
        unload_action_id: "workbench.plugin.unload.physics".into(),
        hot_reload_action_label: "Hot Reload".into(),
        hot_reload_action_id: "workbench.plugin.hot_reload.physics".into(),
    }
}

#[test]
fn details_flow_handles_empty_pages_long_labels_narrow_width_and_cache_changes() {
    use crate::ui::layouts::windows::workbench_host_window::{
        ModulePluginDetailsPayload, ModulePluginDetailsViewData, ModulePluginFeatureRowPayload,
        ModulePluginFeatureRowViewData, ModulePluginsPanePayload, PaneBodyPresentation,
        PanePresentation, PaneShellPresentation,
    };
    use crate::ui::workbench::view::{PaneInteractionMode, PanePayloadKind, PaneRouteNamespace};
    use std::rc::Rc;

    let mut pane = module_plugins_pane_fixture();
    pane.id = "editor.module_plugins#details-flow".into();
    let identity = format!(
        "Owner: vendor.{}\nFeature: shared.{}",
        "long_owner".repeat(12),
        "long_feature".repeat(12)
    );
    let mut detail = ModulePluginDetailsPayload {
        plugin_id: "vendor.owner".into(),
        title: "Selected plugin".into(),
        total_features: 10,
        rows: (0..8)
            .map(|index| ModulePluginFeatureRowPayload {
                feature_id: format!("shared.feature.{index}"),
                label: format!(
                    "Feature {index}\nDesired: No\nEffective: Unknown\nRequired: No\nBlocked: No"
                ),
                effective: "Unknown".into(),
                ..Default::default()
            })
            .collect(),
        selected_feature_id: "shared.feature.7".into(),
        selected_identity: identity.clone(),
        selected_name: "Long feature name ".repeat(16),
        selected_states: "Desired: No\nEffective: Unknown\nRequired: No\nBlocked: No".into(),
        selected_reason: "A reason remains available in the flowing detail region.".repeat(8),
        back_label: "Back".into(),
        previous_label: "Previous page".into(),
        next_label: "Next page".into(),
        page_label: "Page 1/2".into(),
        empty_label: "No optional features".into(),
        action_label: "Enable Feature".into(),
        action_id: "qualified.action".into(),
        intent_id: "intent-1".into(),
        ..Default::default()
    };
    let sync = |pane: &mut PaneData, detail: &ModulePluginDetailsPayload| {
        let native = &mut pane.native_body.module_plugins;
        native.details = Some(Rc::new(ModulePluginDetailsViewData {
            plugin_id: detail.plugin_id.clone(),
            title: detail.title.clone(),
            page: detail.page,
            total_features: detail.total_features,
            rows: detail
                .rows
                .iter()
                .map(|row| ModulePluginFeatureRowViewData {
                    feature_id: row.feature_id.clone(),
                    label: row.label.clone(),
                    desired: row.desired,
                    required: row.required,
                    blocked: row.blocked,
                    effective: row.effective.clone(),
                })
                .collect(),
            selected_feature_id: detail.selected_feature_id.clone(),
            selected_identity: detail.selected_identity.clone(),
            selected_name: detail.selected_name.clone(),
            selected_states: detail.selected_states.clone(),
            selected_reason: detail.selected_reason.clone(),
            action_label: detail.action_label.clone(),
            action_id: detail.action_id.clone(),
            intent_id: detail.intent_id.clone(),
            back_label: detail.back_label.clone(),
            previous_label: detail.previous_label.clone(),
            next_label: detail.next_label.clone(),
            page_label: detail.page_label.clone(),
            empty_label: detail.empty_label.clone(),
        }));
        pane.pane_presentation = Some(PanePresentation::new(
            PaneShellPresentation::new(
                "Plugin Manager",
                "plugin",
                "",
                "",
                None,
                false,
                blank_viewport_chrome(),
            ),
            PaneBodyPresentation {
                document_id: "res://ui/editor/host/module_plugins_body.zui".into(),
                payload_kind: PanePayloadKind::ModulePluginsV1,
                route_namespace: PaneRouteNamespace::Template,
                interaction_mode: PaneInteractionMode::HybridNativeSlot,
                payload: PanePayload::ModulePluginsV1(ModulePluginsPanePayload {
                    diagnostics: native.diagnostics.to_string(),
                    plugins: vec![],
                    detail_generation: native.detail_generation,
                    detail_locale: native.detail_locale.to_string(),
                    selected_plugin_id: detail.plugin_id.clone(),
                    focused_control: "features".into(),
                    details: Some(detail.clone()),
                }),
            },
        ));
    };
    pane.native_body.module_plugins.detail_generation = 1;
    pane.native_body.module_plugins.detail_locale = "en".into();
    sync(&mut pane, &detail);
    let mut cache = ModulePluginsPaneProjectionCache::default();
    let project = |pane: &PaneData, width, cache: &mut ModulePluginsPaneProjectionCache| {
        to_host_contract_module_plugins_pane_from_host_pane_with_cache(
            pane,
            PaneContentSize::new(width, 520.0),
            cache,
        )
    };
    let wide = project(&pane, 640.0, &mut cache);
    let repeated = project(&pane, 640.0, &mut cache);
    assert!(wide.nodes.shares_values_with(&repeated.nodes));
    let find = |data: &host_contract::ModulePluginsPaneData, control: &str| {
        data.nodes
            .iter()
            .find(|node| node.control_id.as_str() == control)
            .cloned()
            .unwrap()
    };
    assert_eq!(
        find(&wide, "ModulePluginFeatureRows")
            .collection_rows
            .row_count(),
        8
    );
    assert!(!wide
        .nodes
        .iter()
        .any(|node| node.control_id.as_str() == "ModulePluginAction"));
    let narrow = project(&pane, 220.0, &mut cache);
    let narrow_identity = find(&narrow, "ModulePluginFeatureIdentity");
    assert_eq!(
        narrow_identity.text.to_string().replace('\n', ""),
        identity.replace('\n', "")
    );
    assert!(narrow_identity.frame.height > find(&wide, "ModulePluginFeatureIdentity").frame.height);
    assert!(narrow_identity.frame.width <= 220.0);

    pane.native_body.module_plugins.detail_generation += 1;
    detail.page = 1;
    detail.rows = (8..10)
        .map(|index| ModulePluginFeatureRowPayload {
            feature_id: format!("shared.feature.{index}"),
            label: format!("Feature {index}"),
            ..Default::default()
        })
        .collect();
    detail.selected_feature_id.clear();
    detail.action_id.clear();
    detail.intent_id.clear();
    sync(&mut pane, &detail);
    let last = project(&pane, 220.0, &mut cache);
    assert!(!narrow.nodes.shares_values_with(&last.nodes));
    assert_eq!(
        find(&last, "ModulePluginFeatureRows")
            .collection_rows
            .row_count(),
        2
    );
    assert!(find(&last, "ModulePluginDetailsNext").disabled);
    assert!(find(&last, "ModulePluginDetailsApply").disabled);
    pane.native_body.module_plugins.detail_locale = "zh-CN".into();
    sync(&mut pane, &detail);
    let localized = project(&pane, 220.0, &mut cache);
    assert!(!last.nodes.shares_values_with(&localized.nodes));
    pane.native_body.module_plugins.detail_generation += 1;
    detail.page = 0;
    detail.total_features = 0;
    detail.rows.clear();
    sync(&mut pane, &detail);
    let empty = project(&pane, 220.0, &mut cache);
    assert!(empty.nodes.iter().any(
        |node| node.control_id.as_str() == "ModulePluginFeatureEmpty"
            && node.text.as_str() == "No optional features"
    ));
    assert!(!empty
        .nodes
        .iter()
        .any(|node| node.control_id.as_str() == "ModulePluginFeatureRows"
            && node.collection_rows.row_count() != 0));
}
