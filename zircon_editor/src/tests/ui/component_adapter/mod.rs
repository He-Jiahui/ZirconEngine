use crate::core::editor_event::{
    EditorEvent, EditorEventEffect, EditorEventSource, EditorEventTransient, MenuAction,
};
use crate::tests::editor_event::support::{env_lock, EventRuntimeHarness};
use crate::ui::host::module::EDITOR_MANAGER_NAME;
use crate::ui::host::EditorManager;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use zircon_runtime_interface::ui::component::{
    UiComponentAdapterError, UiComponentBindingTarget, UiComponentEvent, UiComponentEventEnvelope,
    UiValue, UiValueKind,
};
use zircon_runtime_interface::ui::dispatch::{
    UiDispatchReply, UiInputDispatchResult, UiInputEvent, UiInputEventMetadata, UiInputModifiers,
    UiInputSequence, UiInputTimestamp, UiKeyboardInputEvent, UiKeyboardInputState,
};
use zircon_runtime_interface::ui::template::UiRootClassPolicy;

use crate::ui::template_runtime::component_adapter::registry::EditorUiComponentAdapterRegistry;

fn inspector_value_envelope(field_path: &str, value: UiValue) -> UiComponentEventEnvelope {
    UiComponentEventEnvelope::new(
        "res://ui/editor/host/inspector_surface_controls.zui",
        field_path.replace('.', "_"),
        UiComponentBindingTarget::inspector("entity://selected", field_path),
        UiComponentEvent::ValueChanged {
            property: "value".to_string(),
            value,
        },
    )
    .with_component_id("InspectorField")
}

fn inspector_commit_envelope(field_path: &str, value: UiValue) -> UiComponentEventEnvelope {
    UiComponentEventEnvelope::new(
        "res://ui/editor/host/inspector_surface_controls.zui",
        field_path.replace('.', "_"),
        UiComponentBindingTarget::inspector("entity://selected", field_path),
        UiComponentEvent::Commit {
            property: "value".to_string(),
            value,
        },
    )
    .with_component_id("InspectorField")
}

fn reflection_commit_envelope(field_path: &str, value: UiValue) -> UiComponentEventEnvelope {
    UiComponentEventEnvelope::new(
        "reflection.surface_controls",
        field_path.replace('.', "_"),
        UiComponentBindingTarget::reflection("component://selected", field_path),
        UiComponentEvent::Commit {
            property: "value".to_string(),
            value,
        },
    )
    .with_component_id("ReflectionField")
}

fn component_drawer_press_envelope(
    component_type: &str,
    operation_path: &str,
) -> UiComponentEventEnvelope {
    UiComponentEventEnvelope::new(
        "asset://weather/editor/cloud_layer.inspector.zui",
        "RefreshButton",
        UiComponentBindingTarget::new("component_drawer", operation_path)
            .with_subject(component_type),
        UiComponentEvent::Press { pressed: true },
    )
    .with_component_id("weather.cloud_layer.inspector")
}

fn component_drawer_action_envelope(
    component_type: &str,
    operation_path: &str,
    event: UiComponentEvent,
) -> UiComponentEventEnvelope {
    UiComponentEventEnvelope::new(
        "asset://weather/editor/cloud_layer.inspector.zui",
        "DrawerAction",
        UiComponentBindingTarget::new("component_drawer", operation_path)
            .with_subject(component_type),
        event,
    )
    .with_component_id("weather.cloud_layer.inspector")
}

fn command_commit_envelope(command_id: &str) -> UiComponentEventEnvelope {
    command_commit_envelope_with_value(UiValue::String(command_id.to_string()))
}

fn command_commit_envelope_with_value(value: UiValue) -> UiComponentEventEnvelope {
    UiComponentEventEnvelope::new(
        "res://ui/editor/windows/workbench_window.zui",
        "WorkbenchCommandPalette",
        UiComponentBindingTarget::new("command", "committed_command_id"),
        UiComponentEvent::Commit {
            property: "committed_command_id".to_string(),
            value,
        },
    )
    .with_component_id("CommandPalette")
}

fn keyboard_dispatch_result(
    logical_key: &str,
    key_code: u32,
    modifiers: UiInputModifiers,
    state: UiKeyboardInputState,
    reply: UiDispatchReply,
) -> UiInputDispatchResult {
    let mut metadata =
        UiInputEventMetadata::new(UiInputTimestamp::from_micros(1), UiInputSequence::new(1));
    metadata.modifiers = modifiers;
    UiInputDispatchResult::new(
        UiInputEvent::Keyboard(UiKeyboardInputEvent {
            metadata,
            state,
            key_code,
            scan_code: None,
            physical_key: logical_key.to_string(),
            logical_key: logical_key.to_string(),
            text: None,
        }),
        reply,
    )
}

fn key_modifiers(ctrl: bool, shift: bool, alt: bool, meta: bool) -> UiInputModifiers {
    UiInputModifiers {
        control: ctrl,
        shift,
        alt,
        super_key: meta,
        ..UiInputModifiers::default()
    }
}

mod asset_editor;
mod commands;
mod inspector;

const ASSET_EDITOR_ADAPTER_LAYOUT: &str = r#"
[asset]
kind = "layout"
id = "editor.tests.asset.component_adapter"
version = 1
display_name = "Component Adapter UI Asset"

[root]
node = "root"

[nodes.root]
kind = "native"
type = "VerticalBox"
control_id = "Root"
children = [{ child = "confirm" }]

[nodes.confirm]
kind = "native"
type = "Button"
control_id = "ConfirmButton"
props = { text = "Save" }
"#;

fn unique_asset_adapter_temp_dir(suffix: &str) -> PathBuf {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after unix epoch")
        .as_millis();
    std::env::temp_dir().join(format!(
        "zircon-ui-asset-component-adapter-{suffix}-{millis}"
    ))
}
