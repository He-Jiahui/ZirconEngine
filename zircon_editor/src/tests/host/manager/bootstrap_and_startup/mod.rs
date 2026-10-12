use std::collections::BTreeMap;
use std::fs;

use crate::core::plugin::EditorPluginState;
use crate::core::project::{
    NewProjectDraft, ProjectAuthority, ProjectTemplateId, RecentProjectValidation,
};
use crate::ui::host::module::EDITOR_MANAGER_NAME;
use crate::ui::host::EditorManager;
use crate::ui::workbench::layout::{
    ActivityDrawerLayout, ActivityDrawerMode, ActivityDrawerSlot, ActivityWindowHostMode,
    ActivityWindowId, ActivityWindowLayout, DocumentNode, MainHostPageLayout, MainPageId,
    TabStackLayout, WorkbenchLayout,
};
use crate::ui::workbench::project::ProjectEditorWorkspace;
use crate::ui::workbench::startup::EditorSessionMode;
use crate::ui::workbench::view::{ViewDescriptorId, ViewHost, ViewInstance, ViewInstanceId};
use zircon_runtime::core::manager::ManagerResolver;

use super::support::*;

#[path = "global_layout.rs"]
mod global_layout;
#[path = "session_startup.rs"]
mod session_startup;
#[path = "window_topology.rs"]
mod window_topology;
#[path = "workspace_restore.rs"]
mod workspace_restore;
