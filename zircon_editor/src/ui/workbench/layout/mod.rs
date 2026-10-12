//! Workbench layout model and mutation logic for the editor shell.

#[path = "activity_drawer_layout.rs"]
mod activity_drawer_layout;
#[path = "activity_drawer_mode.rs"]
mod activity_drawer_mode;
#[path = "activity_drawer_slot.rs"]
mod activity_drawer_slot;
#[path = "activity_window_host_mode.rs"]
mod activity_window_host_mode;
#[path = "activity_window_id.rs"]
mod activity_window_id;
#[path = "activity_window_layout.rs"]
mod activity_window_layout;
#[path = "dock_edge.rs"]
mod dock_edge;
#[cfg(test)]
#[path = "tests/document_identity_tests.rs"]
mod document_identity_tests;
#[path = "document_leaf_layout.rs"]
mod document_leaf_layout;
#[path = "document_node.rs"]
mod document_node;
#[path = "document_node_id.rs"]
mod document_node_id;
#[path = "drag_payload.rs"]
mod drag_payload;
#[path = "drop_target.rs"]
mod drop_target;
#[path = "editor_main_frame_layout.rs"]
mod editor_main_frame_layout;
#[path = "floating_window_layout.rs"]
mod floating_window_layout;
#[path = "hit_target.rs"]
mod hit_target;
#[path = "layout_command.rs"]
mod layout_command;
#[path = "layout_command_error.rs"]
mod layout_command_error;
#[path = "layout_diff.rs"]
mod layout_diff;
#[path = "layout_manager.rs"]
mod layout_manager;
#[path = "layout_normalization_report.rs"]
mod layout_normalization_report;
#[path = "main_host_page_layout.rs"]
mod main_host_page_layout;
#[path = "main_page_id.rs"]
mod main_page_id;
#[path = "manager/mod.rs"]
mod manager;
#[path = "restore_policy.rs"]
mod restore_policy;
#[path = "split_axis.rs"]
mod split_axis;
#[path = "split_placement.rs"]
mod split_placement;
#[path = "tab_insertion_anchor.rs"]
mod tab_insertion_anchor;
#[path = "tab_insertion_side.rs"]
mod tab_insertion_side;
#[path = "tab_stack_layout.rs"]
mod tab_stack_layout;
#[path = "workbench_layout.rs"]
mod workbench_layout;
#[path = "workspace_target.rs"]
mod workspace_target;

pub use activity_drawer_layout::ActivityDrawerLayout;
pub use activity_drawer_mode::ActivityDrawerMode;
pub use activity_drawer_slot::ActivityDrawerSlot;
pub use activity_window_host_mode::ActivityWindowHostMode;
pub use activity_window_id::ActivityWindowId;
pub use activity_window_layout::ActivityWindowLayout;
pub use dock_edge::DockEdge;
pub use document_leaf_layout::DocumentLeafLayout;
pub use document_node::DocumentNode;
pub use document_node_id::DocumentNodeId;
pub use drag_payload::DragPayload;
pub use drop_target::DropTarget;
pub use editor_main_frame_layout::EditorMainFrameLayout;
pub use floating_window_layout::FloatingWindowLayout;
pub use hit_target::HitTarget;
pub use layout_command::LayoutCommand;
pub use layout_command_error::LayoutCommandError;
pub use layout_diff::LayoutDiff;
pub use layout_manager::LayoutManager;
pub use layout_normalization_report::LayoutNormalizationReport;
pub use main_host_page_layout::MainHostPageLayout;
pub use main_page_id::MainPageId;
pub use restore_policy::RestorePolicy;
pub use split_axis::SplitAxis;
pub use split_placement::SplitPlacement;
pub use tab_insertion_anchor::TabInsertionAnchor;
pub use tab_insertion_side::TabInsertionSide;
pub use tab_stack_layout::TabStackLayout;
pub use workbench_layout::WorkbenchLayout;
pub use workspace_target::WorkspaceTarget;
