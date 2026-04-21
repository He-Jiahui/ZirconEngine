use crate::ui::binding::{DockCommand, EditorUiBindingPayload};

use crate::core::editor_event::EditorEventRuntime;
use crate::ui::slint_host::event_bridge::SlintDispatchEffects;
use crate::ui::workbench::layout::ActivityDrawerMode;
use crate::ui::workbench::layout::LayoutCommand;
use crate::ui::workbench::view::ViewInstanceId;

use super::super::{
    common::{merge_effects, parse_activity_drawer_slot},
    BuiltinWorkbenchTemplateBridge,
};
use super::dispatch_layout_command;

pub(crate) fn dispatch_builtin_workbench_drawer_toggle(
    runtime: &EditorEventRuntime,
    bridge: &BuiltinWorkbenchTemplateBridge,
    slot: &str,
    instance_id: &str,
) -> Option<Result<SlintDispatchEffects, String>> {
    let binding = bridge.activity_binding_for_target(slot, instance_id)?;
    let EditorUiBindingPayload::DockCommand(DockCommand::ActivateDrawerTab {
        slot: binding_slot,
        instance_id: binding_instance_id,
    }) = binding.payload()
    else {
        return None;
    };

    let slot = match parse_activity_drawer_slot(binding_slot.as_str()) {
        Ok(slot) => slot,
        Err(error) => return Some(Err(error)),
    };
    let target_instance = ViewInstanceId::new(binding_instance_id);
    let layout = runtime.current_layout();
    let Some(drawer) = layout.drawers.get(&slot).cloned() else {
        return Some(Err(format!("missing drawer {:?}", slot)));
    };

    let is_active = drawer
        .tab_stack
        .active_tab
        .as_ref()
        .is_some_and(|active| active == &target_instance);

    Some(
        if is_active && drawer.mode != ActivityDrawerMode::Collapsed {
            dispatch_layout_command(
                runtime,
                LayoutCommand::SetDrawerMode {
                    slot,
                    mode: ActivityDrawerMode::Collapsed,
                },
            )
        } else {
            let mut effects = match dispatch_layout_command(
                runtime,
                LayoutCommand::ActivateDrawerTab {
                    slot,
                    instance_id: target_instance.clone(),
                },
            ) {
                Ok(effects) => effects,
                Err(error) => return Some(Err(error)),
            };
            if drawer.mode == ActivityDrawerMode::Collapsed {
                let reopen = match dispatch_layout_command(
                    runtime,
                    LayoutCommand::SetDrawerMode {
                        slot,
                        mode: ActivityDrawerMode::Pinned,
                    },
                ) {
                    Ok(effects) => effects,
                    Err(error) => return Some(Err(error)),
                };
                merge_effects(&mut effects, reopen);
            }
            Ok(effects)
        },
    )
}
