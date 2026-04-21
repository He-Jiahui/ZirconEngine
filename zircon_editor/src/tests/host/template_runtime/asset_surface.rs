use super::support::*;

#[test]
fn editor_ui_host_runtime_projects_builtin_asset_surface_template_into_slint_projection() {
    let mut runtime = EditorUiHostRuntime::default();
    runtime.load_builtin_workbench_shell().unwrap();

    assert_eq!(
        runtime
            .component_descriptor("AssetSurfaceControls")
            .unwrap()
            .binding_namespace,
        "AssetSurface"
    );

    let projection = runtime.project_document("asset.surface_controls").unwrap();

    assert_eq!(projection.document_id, "asset.surface_controls");
    assert_eq!(projection.root.component, "AssetSurfaceControls");
    assert_eq!(
        projection
            .root
            .children
            .iter()
            .map(|node| node.control_id.as_deref().unwrap_or_default())
            .collect::<Vec<_>>(),
        vec![
            "SelectFolder",
            "SelectItem",
            "SearchEdited",
            "SetKindFilter",
            "SetViewMode",
            "SetUtilityTab",
            "ActivateReference",
            "OpenAssetBrowser",
            "LocateSelectedAsset",
            "ImportModel",
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
}
