pub(in crate::ui::retained_host) fn asset_dispatch_source(dispatch_kind: &str) -> Option<&str> {
    if dispatch_kind == "asset" {
        return Some("activity");
    }
    dispatch_kind.strip_prefix("asset:")
}

pub(in crate::ui::retained_host) fn asset_surface_binding_control_id(
    action_or_control_id: &str,
) -> Option<&'static str> {
    match action_or_control_id {
        "SearchEdited" | "workbench.asset.search.edit" => Some("SearchEdited"),
        "SetKindFilter" | "workbench.asset.kind_filter.set" => Some("SetKindFilter"),
        "SetViewMode" | "workbench.asset.view_mode.set" => Some("SetViewMode"),
        "SetUtilityTab" | "workbench.asset.utility_tab.set" => Some("SetUtilityTab"),
        "OpenAssetBrowser" | "workbench.asset_browser.open" => Some("OpenAssetBrowser"),
        "LocateSelectedAsset" | "workbench.asset.locate_selected" => Some("LocateSelectedAsset"),
        "ImportModel" | "workbench.asset.model.import" => Some("ImportModel"),
        _ => None,
    }
}
