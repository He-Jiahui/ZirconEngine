//! Shared preview fixtures used by browser and desktop workbench hosts.

mod constants;
mod default_preview_fixture;
mod ensure_ui_asset_descriptor;
mod preview_editor_data;
mod preview_editor_data_into_snapshot;
mod preview_fixture;
mod preview_fixture_build_chrome;
mod preview_gizmo_axis;
mod preview_gizmo_axis_into_gizmo_axis;
mod preview_inspector;
mod preview_inspector_into_snapshot;
mod preview_scene_entry;
mod preview_scene_entry_into_snapshot;

pub use default_preview_fixture::default_preview_fixture;
pub use preview_editor_data::PreviewEditorData;
pub use preview_fixture::PreviewFixture;
pub use preview_gizmo_axis::PreviewGizmoAxis;
pub use preview_inspector::PreviewInspector;
pub use preview_scene_entry::PreviewSceneEntry;
