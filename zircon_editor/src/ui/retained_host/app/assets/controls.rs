use super::super::*;

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn dispatch_asset_control_changed(
        &mut self,
        source: &str,
        control_id: &str,
        value: &str,
    ) {
        let Some(binding_control_id) = asset_surface_binding_control_id(control_id) else {
            self.set_status_line(format!("Unknown asset change control {control_id}"));
            return;
        };
        let arguments = match binding_control_id {
            "SearchEdited" | "SetKindFilter" => vec![UiBindingValue::string(value)],
            "SetViewMode" | "SetUtilityTab" => vec![
                UiBindingValue::string(source),
                UiBindingValue::string(value),
            ],
            _ => {
                self.set_status_line(format!("Unknown asset change control {control_id}"));
                return;
            }
        };
        self.dispatch_asset_surface_control(binding_control_id, UiEventKind::Change, arguments);
    }

    pub(in crate::ui::retained_host::app) fn dispatch_asset_control_clicked(
        &mut self,
        _source: &str,
        control_id: &str,
    ) {
        let Some(binding_control_id) = asset_surface_binding_control_id(control_id) else {
            self.set_status_line(format!("Unknown asset click control {control_id}"));
            return;
        };
        match binding_control_id {
            "OpenAssetBrowser" | "LocateSelectedAsset" | "ImportModel" => {
                self.dispatch_asset_surface_control(
                    binding_control_id,
                    UiEventKind::Click,
                    Vec::new(),
                );
            }
            _ => {
                self.set_status_line(format!("Unknown asset click control {control_id}"));
            }
        }
    }
}

fn asset_surface_binding_control_id(action_id: &str) -> Option<&'static str> {
    match action_id {
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
