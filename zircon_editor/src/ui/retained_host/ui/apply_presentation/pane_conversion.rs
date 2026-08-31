use super::*;

pub(super) fn to_host_contract_pane(
    data: host_window::PaneData,
    pane_size: host_window::PaneContentSize,
    component_showcase_runtime: Option<&EditorUiHostRuntime>,
    welcome: Option<&view_data::WelcomePresentation>,
    hierarchy_filter_query: &str,
    console_projection_cache: &mut pane_data_conversion::ConsolePaneProjectionCache,
    module_plugins_projection_cache: &mut pane_data_conversion::ModulePluginsPaneProjectionCache,
) -> host_contract::PaneData {
    let pane_kind = data.kind.to_string();
    let pane_id = data.id.to_string();
    let has_hierarchy_payload =
        pane_kind == "Hierarchy" || data.native_body.hierarchy.nodes.row_count() > 0;
    let has_inspector_payload =
        pane_kind == "Inspector" || data.native_body.inspector.nodes.row_count() > 0;
    let has_console_payload =
        pane_kind == "Console" || data.native_body.console.nodes.row_count() > 0;
    let has_assets_activity_payload =
        pane_kind == "Assets" || data.native_body.assets_activity.nodes.row_count() > 0;
    let has_asset_browser_payload =
        pane_kind == "AssetBrowser" || data.native_body.asset_browser.nodes.row_count() > 0;
    let has_project_overview_payload =
        pane_kind == "Project" || data.native_body.project_overview.nodes.row_count() > 0;
    let has_component_showcase_payload = pane_kind == "UiComponentShowcase"
        || data.pane_presentation.as_ref().is_some_and(|presentation| {
            matches!(
                &presentation.body.payload,
                host_window::PanePayload::UiComponentShowcaseV1(_)
            )
        });
    let has_template_v2_payload = data.pane_presentation.as_ref().is_some_and(|presentation| {
        matches!(
            &presentation.body.payload,
            host_window::PanePayload::TemplateV2(_)
        )
    });
    let has_runtime_diagnostics_payload = pane_kind == "RuntimeDiagnostics"
        || data.pane_presentation.as_ref().is_some_and(|presentation| {
            matches!(
                &presentation.body.payload,
                host_window::PanePayload::RuntimeDiagnosticsV1(_)
            )
        });
    let has_performance_timeline_payload = pane_kind == "PerformanceTimeline"
        || data.pane_presentation.as_ref().is_some_and(|presentation| {
            matches!(
                &presentation.body.payload,
                host_window::PanePayload::PerformanceTimelineV1(_)
            )
        });
    let has_module_plugins_payload = pane_kind == "ModulePlugins"
        || data.pane_presentation.as_ref().is_some_and(|presentation| {
            matches!(
                &presentation.body.payload,
                host_window::PanePayload::ModulePluginsV1(_)
            )
        });
    let has_build_export_payload = pane_kind == "BuildExport"
        || data.pane_presentation.as_ref().is_some_and(|presentation| {
            matches!(
                &presentation.body.payload,
                host_window::PanePayload::BuildExportV1(_)
            )
        });
    let has_generated_bottom_payload = pane_kind == "GeneratedBottom"
        || data.pane_presentation.as_ref().is_some_and(|presentation| {
            matches!(
                &presentation.body.payload,
                host_window::PanePayload::GeneratedBottomV1(_)
            )
        });
    let has_ui_asset_payload = pane_kind == "UiAssetEditor"
        || data.native_body.ui_asset
            != crate::ui::asset_editor::UiAssetEditorPanePresentation::default();
    let has_animation_payload = matches!(
        pane_kind.as_str(),
        "AnimationSequenceEditor" | "AnimationGraphEditor"
    ) || data.native_body.animation.nodes.row_count() > 0;
    let welcome_pane = if pane_kind == "Welcome" {
        zircon_runtime::profile_scope!("editor", "retained_host", "convert_pane_welcome");
        welcome.map_or_else(host_contract::WelcomePaneData::default, |welcome| {
            let pane = project_welcome_pane_for_size(&welcome.pane, pane_size);
            to_host_contract_welcome_pane(&pane, &welcome.recent_projects)
        })
    } else {
        host_contract::WelcomePaneData::default()
    };
    let hierarchy = if has_hierarchy_payload {
        zircon_runtime::profile_scope!("editor", "retained_host", "convert_pane_hierarchy");
        to_host_contract_hierarchy_pane(
            &data,
            pane_size,
            component_showcase_runtime,
            hierarchy_filter_query,
        )
    } else {
        host_contract::HierarchyPaneData::default()
    };
    let inspector = if has_inspector_payload {
        zircon_runtime::profile_scope!("editor", "retained_host", "convert_pane_inspector");
        to_host_contract_inspector_pane(&data, pane_size, component_showcase_runtime)
    } else {
        host_contract::InspectorPaneData::default()
    };
    let console = if has_console_payload {
        zircon_runtime::profile_scope!("editor", "retained_host", "convert_pane_console");
        component_showcase_runtime.map_or_else(
            || to_host_contract_console_pane(&data, pane_size, None),
            |runtime| {
                pane_data_conversion::to_host_contract_console_pane_from_host_pane_with_runtime_and_cache(
                    &data,
                    pane_size,
                    runtime,
                    console_projection_cache,
                )
            },
        )
    } else {
        host_contract::ConsolePaneData::default()
    };
    let animation = if has_animation_payload {
        zircon_runtime::profile_scope!("editor", "retained_host", "convert_pane_animation");
        to_host_contract_animation_editor_pane(&data, pane_size, component_showcase_runtime)
    } else {
        host_contract::AnimationEditorPaneData::default()
    };
    let module_plugins = if has_module_plugins_payload {
        zircon_runtime::profile_scope!("editor", "retained_host", "convert_pane_module_plugins");
        pane_data_conversion::to_host_contract_module_plugins_pane_from_host_pane_with_cache(
            &data,
            pane_size,
            module_plugins_projection_cache,
        )
    } else {
        host_contract::ModulePluginsPaneData::default()
    };
    let runtime_diagnostics = if has_runtime_diagnostics_payload {
        zircon_runtime::profile_scope!(
            "editor",
            "retained_host",
            "convert_pane_runtime_diagnostics"
        );
        to_host_contract_runtime_diagnostics_pane(&data, pane_size)
    } else {
        host_contract::RuntimeDiagnosticsPaneData::default()
    };
    let performance_timeline = if has_performance_timeline_payload {
        zircon_runtime::profile_scope!(
            "editor",
            "retained_host",
            "convert_pane_performance_timeline"
        );
        to_host_contract_performance_timeline_pane(&data, pane_size)
    } else {
        host_contract::PerformanceTimelinePaneData::default()
    };
    let build_export = if has_build_export_payload {
        zircon_runtime::profile_scope!("editor", "retained_host", "convert_pane_build_export");
        to_host_contract_build_export_pane(&data, pane_size)
    } else {
        host_contract::BuildExportPaneData::default()
    };
    let generated_bottom = if has_generated_bottom_payload {
        zircon_runtime::profile_scope!("editor", "retained_host", "convert_pane_generated_bottom");
        to_host_contract_generated_bottom_pane(&data, pane_size)
    } else {
        host_contract::GeneratedBottomPaneData::default()
    };
    let template_v2 = if has_template_v2_payload {
        zircon_runtime::profile_scope!("editor", "retained_host", "convert_pane_template_v2");
        pane_data_conversion::to_host_contract_template_v2_pane_from_host_pane_with_runtime(
            &data,
            pane_size,
            component_showcase_runtime,
        )
    } else {
        if let Some(runtime) = component_showcase_runtime {
            runtime.remove_template_actions_for_pane(&pane_id);
        }
        host_contract::TemplateV2PaneData::default()
    };
    let project_overview = if has_component_showcase_payload {
        zircon_runtime::profile_scope!(
            "editor",
            "retained_host",
            "convert_pane_component_showcase"
        );
        component_showcase_runtime.map_or_else(
            || {
                pane_data_conversion::to_host_contract_component_showcase_pane_from_host_pane(
                    &data, pane_size,
                )
            },
            |runtime| {
                pane_data_conversion::to_host_contract_component_showcase_pane_from_host_pane_with_runtime(
                    &data, pane_size, runtime,
                )
            },
        )
    } else if has_project_overview_payload {
        zircon_runtime::profile_scope!("editor", "retained_host", "convert_pane_project");
        to_host_contract_project_overview_pane(data.native_body.project_overview.clone())
    } else {
        host_contract::ProjectOverviewPaneData::default()
    };

    let mut pane = host_contract::PaneData {
        id: data.id,
        slot: data.slot,
        kind: data.kind,
        title: data.title,
        icon_key: data.icon_key,
        subtitle: data.subtitle,
        info: data.info,
        show_empty: data.show_empty,
        empty_title: data.empty_title,
        empty_body: data.empty_body,
        primary_action_label: data.primary_action_label,
        primary_action_id: data.primary_action_id,
        secondary_action_label: data.secondary_action_label,
        secondary_action_id: data.secondary_action_id,
        secondary_hint: data.secondary_hint,
        show_toolbar: data.show_toolbar,
        welcome: welcome_pane,
        viewport: to_host_contract_scene_viewport_chrome(&data.viewport),
        hierarchy,
        inspector,
        console,
        assets_activity: if has_assets_activity_payload {
            to_host_contract_assets_activity_pane(data.native_body.assets_activity)
        } else {
            host_contract::AssetsActivityPaneData::default()
        },
        asset_browser: pane_data_conversion::to_host_contract_asset_browser_pane(
            if has_asset_browser_payload {
                data.native_body.asset_browser
            } else {
                host_window::AssetBrowserPaneViewData::default()
            },
            pane_size,
        ),
        project_overview,
        template_v2,
        runtime_diagnostics,
        performance_timeline,
        module_plugins,
        build_export,
        generated_bottom,
        ui_asset: if has_ui_asset_payload {
            to_host_contract_ui_asset_pane(data.native_body.ui_asset, &pane_id)
        } else {
            host_contract::UiAssetEditorPaneData::default()
        },
        animation,
        ..host_contract::PaneData::default()
    };
    host_contract::rebuild_pane_template_hit_artifacts(
        &mut pane,
        UiSize::new(pane_size.width.max(1.0), pane_size.height.max(1.0)),
    );
    if pane_data_conversion::refresh_runtime_diagnostics_debug_reflector_from_body_surface(
        &mut pane, pane_size,
    ) {
        host_contract::rebuild_pane_template_hit_artifacts(
            &mut pane,
            UiSize::new(pane_size.width.max(1.0), pane_size.height.max(1.0)),
        );
    }
    pane
}
