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

mod animation_assets;
mod console;
mod error_propagation;
mod extensions_registration;
mod extensions_validation;
mod integration;
mod keymap_settings;
mod listeners;
mod registry;
mod stack_play;
mod when_evaluation;
