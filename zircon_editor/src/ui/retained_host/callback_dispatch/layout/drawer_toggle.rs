use crate::ui::binding::{DockCommand, EditorUiBindingPayload};

use crate::ui::host::EditorHostEventController;
use crate::ui::retained_host::event_bridge::{HostShellContentScope, UiHostEventEffects};
use crate::ui::workbench::layout::ActivityDrawerMode;
use crate::ui::workbench::layout::LayoutCommand;
use crate::ui::workbench::view::ViewInstanceId;

use super::super::{common::parse_activity_drawer_slot, BuiltinHostWindowTemplateBridge};
use super::dispatch_layout_command;

pub(crate) fn dispatch_builtin_host_drawer_toggle(
    runtime: &EditorHostEventController,
    bridge: &BuiltinHostWindowTemplateBridge,
    slot: &str,
    instance_id: &str,
) -> Option<Result<UiHostEventEffects, String>> {
    // Drawer-header tabs are projected from workbench state and may not have
    // static workbench-shell bindings; activity-rail controls still do.
    let (slot, target_instance) = match bridge.activity_binding_for_target(slot, instance_id) {
        Some(binding) => {
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
            (slot, ViewInstanceId::new(binding_instance_id))
        }
        None => {
            let slot = match parse_activity_drawer_slot(slot) {
                Ok(slot) => slot,
                Err(error) => return Some(Err(error)),
            };
            (slot, ViewInstanceId::new(instance_id))
        }
    };
    let layout = runtime.current_layout();
    let active_drawers = layout.active_activity_window_drawers();
    let Some(drawer) = active_drawers.get(&slot).cloned() else {
        return Some(Err(format!("missing drawer {:?}", slot)));
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
            if drawer_tab_switch_reuses_layout(drawer.mode, is_active, region_was_expanded) {
                effects.reuse_layout_for_shell_content(HostShellContentScope::new(
                    slot,
                    target_instance,
                ));
            }
            Ok(effects)
        },
    )
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
