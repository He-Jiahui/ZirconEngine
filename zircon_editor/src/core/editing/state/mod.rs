//! Mutable editor session: scene, viewport, undo, and project I/O.

mod editor_state;
mod editor_state_apply_intent;
mod editor_state_asset_workspace;
mod editor_state_construction;
mod editor_state_field_updates;
mod editor_state_project;
mod editor_state_selection;
mod editor_state_snapshot;
mod editor_state_viewport;
mod editor_world_slot;
mod hierarchy_depth;
mod no_project_open;
mod parse_parent_field;

pub use editor_state::EditorState;
