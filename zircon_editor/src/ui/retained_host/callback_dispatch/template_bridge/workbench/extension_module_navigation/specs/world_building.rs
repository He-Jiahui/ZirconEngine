use super::types::{ActionControl, ExtensionNavigationSpec, action, spec};

mod level_tools;
mod prefab_and_scatter;
mod terrain_and_foliage;
mod volume_and_weather;

pub(super) use level_tools::{LEVEL_STREAMING_NAVIGATION_SPEC, LEVEL_VARIANT_NAVIGATION_SPEC};
pub(super) use prefab_and_scatter::{
    PREFAB_EDITOR_NAVIGATION_SPEC, SCATTER_EDITOR_NAVIGATION_SPEC,
};
pub(super) use terrain_and_foliage::{
    FOLIAGE_EDITOR_NAVIGATION_SPEC, TERRAIN_EDITOR_NAVIGATION_SPEC,
};
pub(super) use volume_and_weather::{
    VOLUME_EDITOR_NAVIGATION_SPEC, WEATHER_EDITOR_NAVIGATION_SPEC,
};

#[cfg(test)]
mod tests {
    use super::*;

    const WORLD_BUILDING_NAVIGATION_SPECS: &[ExtensionNavigationSpec] = &[
        TERRAIN_EDITOR_NAVIGATION_SPEC,
        FOLIAGE_EDITOR_NAVIGATION_SPEC,
        LEVEL_STREAMING_NAVIGATION_SPEC,
        LEVEL_VARIANT_NAVIGATION_SPEC,
        PREFAB_EDITOR_NAVIGATION_SPEC,
        SCATTER_EDITOR_NAVIGATION_SPEC,
        VOLUME_EDITOR_NAVIGATION_SPEC,
        WEATHER_EDITOR_NAVIGATION_SPEC,
    ];

    #[test]
    fn world_building_specs_bind_each_declared_action_to_a_control() {
        for navigation in WORLD_BUILDING_NAVIGATION_SPECS {
            assert_eq!(navigation.tab_controls.len(), navigation.tab_actions.len());
            assert_eq!(navigation.row_controls.len(), navigation.row_actions.len());
            assert_eq!(
                navigation.command_controls.len(),
                navigation.command_actions.len()
            );
            assert!(
                navigation
                    .open_action_id
                    .starts_with("workbench.extension.")
            );
            assert!(
                navigation
                    .workspace_control_id
                    .starts_with("WorkbenchExtension")
            );
            assert!(!navigation.field_actions.is_empty());
            let namespace = navigation.open_action_id.trim_end_matches(".open");
            assert!(namespace.len() < navigation.open_action_id.len());

            for (control_id, action) in navigation.tab_controls.iter().zip(navigation.tab_actions) {
                assert_eq!(*control_id, action.control_id);
                assert!(action.action_id.starts_with(namespace));
            }
            for (control_id, action) in navigation.row_controls.iter().zip(navigation.row_actions) {
                assert_eq!(*control_id, action.control_id);
                assert!(action.action_id.starts_with(namespace));
            }
            for (control_id, action) in navigation
                .command_controls
                .iter()
                .zip(navigation.command_actions)
            {
                assert_eq!(*control_id, action.control_id);
                assert!(action.action_id.starts_with(namespace));
            }
            for action_id in navigation.field_actions {
                assert!(action_id.starts_with(namespace));
            }
            for action in navigation
                .tab_actions
                .iter()
                .chain(navigation.row_actions)
                .chain(navigation.command_actions)
            {
                assert!(action.action_id.starts_with("workbench.extension."));
                assert!(action.control_id.starts_with("Workbench"));
            }
        }
    }
}
