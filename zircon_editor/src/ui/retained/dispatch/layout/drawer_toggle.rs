use crate::ui::host::EditorHostEventController;
use crate::ui::retained_host::event_bridge::{HostShellContentScope, UiHostEventEffects};
use crate::ui::workbench::layout::{ActivityDrawerMode, ActivityDrawerSlot, LayoutCommand};
use crate::ui::workbench::view::ViewInstanceId;

use super::dispatch_layout_command;

pub(crate) fn dispatch_builtin_host_drawer_toggle(
    runtime: &EditorHostEventController,
    slot: ActivityDrawerSlot,
    instance_id: &ViewInstanceId,
) -> Result<UiHostEventEffects, String> {
    let (drawer_mode, is_active, region_was_expanded) =
        runtime.active_drawer_toggle_state(slot, instance_id)?;

    if is_active && drawer_mode != ActivityDrawerMode::Collapsed {
        dispatch_layout_command(
            runtime,
            LayoutCommand::SetDrawerMode {
                slot,
                mode: ActivityDrawerMode::Collapsed,
            },
        )
    } else {
        let reuse_layout =
            drawer_tab_switch_reuses_layout(drawer_mode, is_active, region_was_expanded);
        let mut effects = dispatch_layout_command(
            runtime,
            LayoutCommand::ActivateDrawerTab {
                slot,
                instance_id: instance_id.clone(),
            },
        )?;
        if reuse_layout {
            effects.reuse_layout_for_shell_content(HostShellContentScope::new(
                slot,
                instance_id.clone(),
            ));
        }
        Ok(effects)
    }
}

fn drawer_tab_switch_reuses_layout(
    mode: ActivityDrawerMode,
    is_active: bool,
    region_was_expanded: bool,
) -> bool {
    !is_active && (mode != ActivityDrawerMode::Collapsed || region_was_expanded)
}

#[cfg(test)]
#[path = "tests/drawer_toggle.rs"]
mod tests;
