use super::super::build_export_actions;

pub(super) fn is_build_export_surface_action(control_id: &str, action_id: &str) -> bool {
    control_id == build_export_actions::BUILD_EXPORT_ACTION_CONTROL_ID
        || build_export_actions::parse_build_export_action(action_id).is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plugin_export_wizard_action_id_routes_to_build_export_dispatch() {
        assert!(is_build_export_surface_action(
            "DesktopExportStartButton",
            "workbench.build_export.execute.desktop_windows"
        ));
        assert!(is_build_export_surface_action(
            "DesktopExportGeneratePlanButton",
            "workbench.build_export.plan.desktop_windows"
        ));
        assert!(is_build_export_surface_action(
            build_export_actions::BUILD_EXPORT_ACTION_CONTROL_ID,
            "workbench.build_export.unknown.desktop_windows"
        ));
        assert!(!is_build_export_surface_action(
            "DesktopExportStartButton",
            "DesktopExportWizard/Start"
        ));
    }
}
