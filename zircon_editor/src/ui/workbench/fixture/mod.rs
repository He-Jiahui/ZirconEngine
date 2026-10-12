//! Shared preview fixtures used by browser and desktop workbench hosts.

#[path = "constants.rs"]
mod constants;
#[path = "default_preview_fixture.rs"]
mod default_preview_fixture;
#[path = "ensure_ui_asset_descriptor.rs"]
mod ensure_ui_asset_descriptor;
#[path = "preview_editor_data.rs"]
mod preview_editor_data;
#[path = "preview_editor_data_into_snapshot.rs"]
mod preview_editor_data_into_snapshot;
#[path = "preview_fixture.rs"]
mod preview_fixture;
#[path = "preview_fixture_build_chrome.rs"]
mod preview_fixture_build_chrome;
#[path = "preview_gizmo_axis.rs"]
mod preview_gizmo_axis;
#[path = "preview_gizmo_axis_into_gizmo_axis.rs"]
mod preview_gizmo_axis_into_gizmo_axis;
#[path = "preview_inspector.rs"]
mod preview_inspector;
#[path = "preview_inspector_into_snapshot.rs"]
mod preview_inspector_into_snapshot;
#[path = "preview_scene_entry.rs"]
mod preview_scene_entry;
#[path = "preview_scene_entry_into_snapshot.rs"]
mod preview_scene_entry_into_snapshot;

pub use default_preview_fixture::default_preview_fixture;
pub use preview_editor_data::PreviewEditorData;
pub use preview_fixture::PreviewFixture;
pub use preview_gizmo_axis::PreviewGizmoAxis;
pub use preview_inspector::PreviewInspector;
pub use preview_scene_entry::PreviewSceneEntry;
