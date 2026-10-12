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
    }
}

fn module_plugin_status_fixture() -> ModulePluginStatusViewData {
    ModulePluginStatusViewData {
        plugin_id: "physics".into(),
        display_name: "Physics".into(),
        package_source: "builtin".into(),
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
