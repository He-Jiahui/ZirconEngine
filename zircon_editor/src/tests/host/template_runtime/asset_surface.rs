use super::support::*;

#[test]
fn editor_ui_host_runtime_projects_builtin_asset_surface_template_into_retained_projection() {
    let mut runtime = EditorUiHostRuntime::default();
    runtime.load_builtin_host_templates().unwrap();

    assert_eq!(
        runtime
            .component_descriptor("AssetSurfaceControls")
            .unwrap()
            .binding_namespace,
        "AssetSurface"
    );

    let projection = runtime
        .project_document("res://ui/editor/host/asset_surface_controls.zui")
        .unwrap();

    assert_eq!(
        projection.document_id,
        "res://ui/editor/host/asset_surface_controls.zui"
    );
    assert_eq!(projection.root.component, "VerticalGroup");
    assert_eq!(
        projection
            .root
            .children
            .iter()
            .map(|node| node.control_id.as_deref().unwrap_or_default())
            .collect::<Vec<_>>(),
        vec![
            "AssetFilterRow",
            "AssetDisplayRow",
            "AssetNavigationRow",
            "AssetPointerRoutes",
        ]
    );

    let search = projection
        .bindings
        .iter()
        .find(|binding| binding.binding_id == "AssetSurface/SearchEdited")
        .unwrap();
    assert_eq!(search.binding.path().event_kind, UiEventKind::Change);
    assert_eq!(search.binding.path().view_id, "AssetSurface");
    assert_eq!(search.binding.path().control_id, "SearchEdited");

    let open_browser = projection
        .bindings
        .iter()
        .find(|binding| binding.binding_id == "AssetSurface/OpenAssetBrowser")
        .unwrap();
    assert_eq!(open_browser.binding.path().event_kind, UiEventKind::Click);
    assert_eq!(open_browser.binding.path().view_id, "AssetSurface");
    assert_eq!(open_browser.binding.path().control_id, "OpenAssetBrowser");

    for (binding_id, event_kind) in [
        ("AssetSurface/SelectFolder", UiEventKind::Change),
        ("AssetSurface/SelectItem", UiEventKind::Change),
        ("AssetSurface/ActivateReference", UiEventKind::Click),
    ] {
        assert!(
            projection.bindings.iter().any(|binding| {
                binding.binding_id == binding_id && binding.binding.path().event_kind == event_kind
            }),
            "collapsed pointer route `{binding_id}` must retain its template binding"
        );
    }
}

#[test]
fn asset_surface_uses_responsive_drawer_rows_for_its_control_groups() {
    let source = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("assets/ui/editor/host/asset_surface_controls.zui"),
    )
    .expect("asset surface controls template should be readable");

    assert!(
        source.contains(
            "component = \"VerticalGroup\"\ncontrol_id = \"AssetSurfaceControls\"\nclasses = [\"editor-surface-controls\", \"asset-surface-controls\"]\nlayout = { container = { kind = \"VerticalBox\", gap = \"$editor.density.gap.small\" }, width = { stretch = \"Stretch\" }, height = { stretch = \"Stretch\" } }"
        ),
        "the asset surface root must grow vertically inside its drawer rather than force one toolbar row"
    );

    for row in ["AssetFilterRow", "AssetDisplayRow", "AssetNavigationRow"] {
        assert!(
            source.contains(&format!(
                "control_id = \"{row}\"\nlayout = {{ container = {{ kind = \"HorizontalBox\", gap = \"$editor.density.gap.small\" }}, width = {{ stretch = \"Stretch\" }}, height = {{ min = \"$editor.control.height.dense\", preferred = \"$editor.control.height.dense\", max = \"$editor.control.height.dense\", stretch = \"Fixed\" }} }}"
            )),
            "{row} must form one dense, width-responsive drawer row"
        );
    }

    assert!(
        !source.contains("preferred = 280.0"),
        "the asset surface must not cap its search row at a fixed toolbar-width maximum"
    );
    assert!(
        source.contains(
            "component = \"VerticalGroup\"\ncontrol_id = \"AssetPointerRoutes\"\nprops = { visibility = \"collapsed\", visible = false }"
        ),
        "pointer-only asset bindings must remain mounted without rendering contextless actions"
    );
    for selector in [
        "[nodes.view_mode]\ncomponent = \"IconButton\"\ncontrol_id = \"SetViewMode\"",
        "[nodes.utility_tab]\ncomponent = \"IconButton\"\ncontrol_id = \"SetUtilityTab\"",
    ] {
        assert!(
            source.contains(selector),
            "asset display controls must retain direct asset-routing semantics until the shared popup owner supports asset option dispatch"
        );
    }
    assert!(
        !source.contains("component = \"ComboBox\"\ncontrol_id = \"SetViewMode\""),
        "the asset view action must not enter the component-showcase-only dropdown route"
    );
    assert!(
        !source.contains("component = \"ComboBox\"\ncontrol_id = \"SetUtilityTab\""),
        "the asset utility action must not enter the component-showcase-only dropdown route"
    );
}
