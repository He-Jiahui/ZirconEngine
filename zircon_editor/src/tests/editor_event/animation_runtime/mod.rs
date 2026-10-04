//! 动画作者事件通过真实资产目录与工具包会话验证，测试从输入事件一直观察到文档状态。
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
