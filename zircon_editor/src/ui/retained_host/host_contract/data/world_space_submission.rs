mod builder;
mod model;

#[cfg(test)]
use builder::build_world_space_ui_surface_submissions;
pub(crate) use builder::build_world_space_ui_surface_submissions_from_host_scene;
pub(crate) use model::WorldSpaceUiSurfaceSubmission;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod performance_tests {
    #[test]
    fn world_space_collection_borrows_host_rows_and_surface_ids() {
        let node = include_str!("world_space_submission/builder/node.rs");
        let pane = include_str!("world_space_submission/builder/pane.rs");
        let scene = include_str!("world_space_submission/builder/scene.rs");
        let compact_node = node.split_whitespace().collect::<String>();

        assert!(compact_node.contains("letmutsubmissions=nodes.iter()"));
        assert!(!node.contains("row_data("));
        assert!(!pane.contains("pane_surface_id.clone()"));
        assert!(scene.contains("floating_windows.iter()"));
        assert!(!scene.contains("floating_windows.row_data("));
    }
}
