use std::collections::BTreeMap;
use std::fs;

use crate::core::plugin::EditorPluginState;
use crate::core::project::{
    NewProjectDraft, NewProjectTemplate, ProjectAuthority, RecentProjectValidation,
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

mod global_layout;
mod session_startup;
mod window_topology;
mod workspace_restore;
