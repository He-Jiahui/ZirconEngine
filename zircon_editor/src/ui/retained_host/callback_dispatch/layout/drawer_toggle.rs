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
    let layout = runtime.current_layout();
    let active_window_id = layout
        .active_activity_window_id()
        .ok_or_else(|| "missing active activity window".to_string())?;
    let activity_windows = layout.activity_windows();
    let active_drawers = &activity_windows
        .get(&active_window_id)
        .ok_or_else(|| format!("missing active activity window {active_window_id:?}"))?
        .activity_drawers;
    let Some(drawer) = active_drawers.get(&slot) else {
        return Err(format!("missing drawer {:?}", slot));
    };
    let region_was_expanded = active_drawers.iter().any(|(candidate_slot, candidate)| {
        candidate_slot.shares_region(slot)
            && candidate.visible
            && !candidate.tab_stack.tabs.is_empty()
            && candidate.mode != ActivityDrawerMode::Collapsed
    });

    let is_active = drawer
        .tab_stack
        .active_tab
        .as_ref()
        .is_some_and(|active| active == instance_id);

    if is_active && drawer.mode != ActivityDrawerMode::Collapsed {
        dispatch_layout_command(
            runtime,
            LayoutCommand::SetDrawerMode {
                slot,
                mode: ActivityDrawerMode::Collapsed,
            },
        )
    } else {
        let reuse_layout =
            drawer_tab_switch_reuses_layout(drawer.mode, is_active, region_was_expanded);
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
mod tests {
    use super::drawer_tab_switch_reuses_layout;
    use crate::ui::workbench::layout::ActivityDrawerMode;

    #[test]
    fn switching_tabs_in_an_open_drawer_reuses_layout() {
        assert!(drawer_tab_switch_reuses_layout(
            ActivityDrawerMode::Pinned,
            false,
            true,
        ));
        assert!(drawer_tab_switch_reuses_layout(
            ActivityDrawerMode::AutoHide,
            false,
            true,
        ));
    }

    #[test]
    fn switching_between_drawers_in_an_expanded_region_reuses_layout() {
        assert!(drawer_tab_switch_reuses_layout(
            ActivityDrawerMode::Collapsed,
            false,
            true,
        ));
    }

    #[test]
    fn collapse_and_reopen_keep_the_full_layout_path() {
        assert!(!drawer_tab_switch_reuses_layout(
            ActivityDrawerMode::Pinned,
            true,
            true,
        ));
        assert!(!drawer_tab_switch_reuses_layout(
            ActivityDrawerMode::Collapsed,
            false,
            false,
        ));
    }
}
