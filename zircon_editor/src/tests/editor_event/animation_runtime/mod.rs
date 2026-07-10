use std::fs;

use crate::core::editor_event::{EditorAssetEvent, EditorEvent, EditorEventSource};
use crate::ui::binding::{
    AnimationCommand, EditorUiBinding, EditorUiBindingPayload, EditorUiEventKind,
};
use crate::ui::host::module::EDITOR_MANAGER_NAME;
use crate::ui::host::EditorManager;
use crate::ui::workbench::view::ViewDescriptorId;

use self::support::*;
use super::support::{env_lock, EventRuntimeHarness};

mod graph;
mod rebind;
mod sequence;
mod state_machine;
mod support;
