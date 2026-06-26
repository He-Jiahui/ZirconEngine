use crate::core::editor_event::EditorEventRuntime;
use crate::ui::retained_host::event_bridge::UiHostEventEffects;
use crate::ui::retained_host::workbench_popup_actions::WORKBENCH_POPUP_CANCEL_ACTION_ID;
use zircon_runtime_interface::ui::binding::UiEventKind;

use super::super::{
    common::{dispatch_editor_binding, merge_effects},
    BuiltinHostWindowTemplateBridge, BuiltinWorkbenchWindowTemplateSurfaceBridge,
};

pub(crate) fn dispatch_builtin_host_control(
    runtime: &EditorEventRuntime,
    bridge: &BuiltinHostWindowTemplateBridge,
    control_id: &str,
    event_kind: UiEventKind,
) -> Option<Result<UiHostEventEffects, String>> {
    let binding = bridge.binding_for_control(control_id, event_kind)?.clone();
    Some(dispatch_editor_binding(runtime, binding))
}

pub(crate) fn dispatch_builtin_host_menu_action(
    runtime: &EditorEventRuntime,
    bridge: &BuiltinHostWindowTemplateBridge,
    action: &str,
) -> Option<Result<UiHostEventEffects, String>> {
    dispatch_builtin_host_control(runtime, bridge, action, UiEventKind::Click)
}

pub(crate) fn dispatch_componentized_workbench_control(
    runtime: &EditorEventRuntime,
    bridge: &mut BuiltinWorkbenchWindowTemplateSurfaceBridge,
    control_id: &str,
    event_kind: UiEventKind,
) -> Option<Result<UiHostEventEffects, String>> {
    let binding = match bridge.dispatch_control_state(control_id, event_kind) {
        Ok(Some(binding)) => binding,
        Ok(None) => return None,
        Err(error) => return Some(Err(error.to_string())),
    };
    Some(dispatch_editor_binding(runtime, binding))
}

pub(crate) fn dispatch_componentized_workbench_binding(
    runtime: &EditorEventRuntime,
    bridge: &mut BuiltinWorkbenchWindowTemplateSurfaceBridge,
    control_id: &str,
    binding_id: &str,
) -> Option<Result<UiHostEventEffects, String>> {
    let binding = match bridge.dispatch_binding_state_for_control(control_id, binding_id) {
        Ok(Some(binding)) => binding,
        Ok(None) => return None,
        Err(error) => return Some(Err(error.to_string())),
    };
    Some(dispatch_editor_binding(runtime, binding))
}

pub(crate) fn dispatch_componentized_workbench_option_selected(
    runtime: &EditorEventRuntime,
    bridge: &mut BuiltinWorkbenchWindowTemplateSurfaceBridge,
    control_id: &str,
    option_id: &str,
) -> Result<UiHostEventEffects, String> {
    let selected = bridge
        .select_dropdown_option(control_id, option_id)
        .map_err(|error| error.to_string())?;
    if !selected {
        return Ok(UiHostEventEffects::default());
    }

    let mut effects = match bridge
        .binding_for_control(control_id, UiEventKind::Change)
        .cloned()
    {
        Some(binding) => dispatch_editor_binding(runtime, binding)?,
        None => UiHostEventEffects::default(),
    };
    effects.request_paint_only();
    Ok(effects)
}

pub(crate) fn dispatch_componentized_workbench_surface_control_edited(
    bridge: &mut BuiltinWorkbenchWindowTemplateSurfaceBridge,
    control_id: &str,
    binding_id: &str,
    value: &str,
) -> Option<Result<UiHostEventEffects, String>> {
    let edited = match bridge
        .edit_inspector_component_property(control_id, binding_id, value)
        .and_then(|edited| match edited {
            Some(edited) => Ok(Some(edited)),
            None => bridge.edit_inspector_transform_axis(control_id, binding_id, value),
        })
        .and_then(|edited| match edited {
            Some(edited) => Ok(Some(edited)),
            None => bridge.edit_workbench_module_field(control_id, binding_id, value),
        }) {
        Ok(Some(edited)) => edited,
        Ok(None) => return None,
        Err(error) => return Some(Err(error.to_string())),
    };
    let mut effects = UiHostEventEffects::default();
    if edited {
        effects.request_paint_only();
    }
    Some(Ok(effects))
}

pub(crate) fn dispatch_componentized_workbench_menu_item_selected(
    runtime: &EditorEventRuntime,
    bridge: &mut BuiltinWorkbenchWindowTemplateSurfaceBridge,
    control_id: &str,
    action_id: &str,
) -> Option<Result<UiHostEventEffects, String>> {
    let selected = match bridge.select_popup_menu_item(control_id, action_id) {
        Ok(Some(selected)) => selected,
        Ok(None) => return None,
        Err(error) => return Some(Err(error.to_string())),
    };
    let mut effects = UiHostEventEffects::default();
    if selected {
        match bridge.dispatch_workbench_module_overflow_menu_item_state(control_id, action_id) {
            Ok(Some(binding)) => match dispatch_editor_binding(runtime, binding) {
                Ok(binding_effects) => merge_effects(&mut effects, binding_effects),
                Err(error) => return Some(Err(error)),
            },
            Ok(None) => {}
            Err(error) => return Some(Err(error.to_string())),
        }
        effects.request_paint_only();
    }
    Some(Ok(effects))
}

pub(crate) fn dispatch_componentized_workbench_popup_cancelled(
    bridge: &mut BuiltinWorkbenchWindowTemplateSurfaceBridge,
    control_id: &str,
    action_id: &str,
) -> Option<Result<UiHostEventEffects, String>> {
    if action_id != WORKBENCH_POPUP_CANCEL_ACTION_ID {
        return None;
    }
    let closed = match bridge.close_popup(control_id) {
        Ok(closed) => closed,
        Err(error) => return Some(Err(error.to_string())),
    };
    let mut effects = UiHostEventEffects::default();
    if closed {
        effects.request_paint_only();
    }
    Some(Ok(effects))
}
