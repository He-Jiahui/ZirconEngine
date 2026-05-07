use crate::core::editor_event::EditorEventRuntime;
use crate::ui::slint_host::{
    event_bridge::UiHostEventEffects,
    tab_drag::{ResolvedHostTabDropRoute, ResolvedHostTabDropTarget},
};
use crate::ui::workbench::layout::{ActivityDrawerMode, LayoutCommand};
use crate::ui::workbench::view::{ViewHost, ViewInstanceId};

use super::super::common::merge_effects;
use super::dispatch_layout_command;

pub(crate) fn dispatch_tab_drop(
    runtime: &EditorEventRuntime,
    instance_id: &str,
    route: &ResolvedHostTabDropRoute,
) -> Result<UiHostEventEffects, String> {
    match &route.target {
        ResolvedHostTabDropTarget::Attach(drop) => {
            let reopen_drawer_slot = match &drop.host {
                ViewHost::Drawer(slot) => runtime
                    .current_layout()
                    .active_activity_window_drawers()
                    .get(slot)
                    .and_then(|drawer| {
                        (drawer.mode == ActivityDrawerMode::Collapsed).then_some(*slot)
                    }),
                _ => None,
            };

            let mut effects = dispatch_layout_command(
                runtime,
                LayoutCommand::AttachView {
                    instance_id: ViewInstanceId::new(instance_id),
                    target: drop.host.clone(),
                    anchor: drop.anchor.clone(),
                },
            )?;

            if let Some(slot) = reopen_drawer_slot {
                let reopen = dispatch_layout_command(
                    runtime,
                    LayoutCommand::SetDrawerMode {
                        slot,
                        mode: ActivityDrawerMode::Pinned,
                    },
                )?;
                merge_effects(&mut effects, reopen);
            }

            Ok(effects)
        }
        ResolvedHostTabDropTarget::DetachToWindow { new_window } => dispatch_layout_command(
            runtime,
            LayoutCommand::DetachViewToWindow {
                instance_id: ViewInstanceId::new(instance_id),
                new_window: new_window.clone(),
            },
        ),
        ResolvedHostTabDropTarget::Split {
            workspace,
            path,
            axis,
            placement,
        } => dispatch_layout_command(
            runtime,
            LayoutCommand::CreateSplit {
                workspace: workspace.clone(),
                path: path.clone(),
                axis: *axis,
                placement: *placement,
                new_instance: ViewInstanceId::new(instance_id),
            },
        ),
    }
}
