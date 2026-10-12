//! 从 UI binding 到 EditorHostEventController 的运行时事件测试，共用真实宿主 harness。
use crate::ui::binding::{
    AnimationCommand, DockCommand, EditorUiBinding, EditorUiBindingPayload, EditorUiEventKind,
};
use serde_json::json;
use std::fs;
use zircon_runtime::core::framework::animation::AnimationTrackPath;
use zircon_runtime::scene::components::NodeKind;
use zircon_runtime_interface::resource::ResourceKind;
use zircon_runtime_interface::ui::{
    binding::{UiBindingCall, UiBindingValue},
    event_ui::UiControlRequest,
    event_ui::UiControlResponse,
    event_ui::UiNodePath,
};

use crate::core::editor_event::{
    EditorAnimationEvent, EditorAssetEvent, EditorEvent, EditorEventEffect, EditorEventReplay,
    EditorEventSource, EditorEventTransient, EditorInspectorEvent, InspectorFieldChange,
    LayoutCommand, MenuAction, ViewDescriptorId as EventViewDescriptorId,
    ViewInstanceId as EventViewInstanceId,
};
use crate::ui::retained_host::{
    callback_dispatch::{dispatch_builtin_template_binding, retained_menu_action},
    HostInvalidationMask,
};
use crate::ui::workbench::event::menu_action_binding;
use crate::ui::workbench::layout::WorkbenchLayout;
use crate::ui::workbench::view::ViewDescriptorId;

use super::support::{env_lock, EventRuntimeHarness};

#[path = "animation_assets.rs"]
mod animation_assets;
#[path = "console.rs"]
mod console;
#[path = "error_propagation.rs"]
mod error_propagation;
#[path = "extensions_registration.rs"]
mod extensions_registration;
#[path = "extensions_validation.rs"]
mod extensions_validation;
#[path = "integration/mod.rs"]
mod integration;
#[path = "keymap_settings.rs"]
mod keymap_settings;
#[path = "listeners.rs"]
mod listeners;
#[path = "registry.rs"]
mod registry;
#[path = "stack_play.rs"]
mod stack_play;
#[path = "when_evaluation.rs"]
mod when_evaluation;
