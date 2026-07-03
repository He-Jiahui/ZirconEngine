use zircon_runtime::ui::{surface::UiSurface, tree::UiRuntimeTreeLayoutExt};
use zircon_runtime_interface::ui::{
    event_ui::UiNodeId,
    layout::{AxisConstraint, StretchMode, UiSize},
};

use super::componentized_window::BuiltinWorkbenchWindowTemplateSurfaceBridge;
use super::error::BuiltinHostWindowTemplateBridgeError;
use super::module_overflow_menu::{
    WORKBENCH_MODULE_OVERFLOW_MENU_CONTROL_ID, WORKBENCH_MODULE_OVERFLOW_TRIGGER_CONTROL_ID,
};

const COMPACT_TOOLBAR_MAX_WIDTH: f32 = 1100.0;
const FULL_TOOLBAR_MIN_WIDTH: f32 = 1440.0;
const MODULE_COMMAND_GROUP_CONTROL_ID: &str = "WorkbenchModuleCommands";
const MODULE_COMMAND_GROUP_COMPACT_WIDTH: f32 = 276.0;
const MODULE_COMMAND_GROUP_FULL_WIDTH: f32 = 350.0;

const COMPACT_HIDDEN_MODULE_TABS: &[&str] = &[
    "WorkbenchModuleBehavior",
    "WorkbenchModuleRender",
    "WorkbenchModuleAssets",
    "WorkbenchModuleVfx",
    "WorkbenchModuleHud",
];

const SECONDARY_MODULE_COMMANDS: &[&str] = &["WorkbenchModuleDiff", "WorkbenchModuleSimulate"];
const SECONDARY_TOOLBAR_GROUPS: &[&str] =
    &["WorkbenchToolbarToolGroup", "WorkbenchToolbarRunGroup"];

impl BuiltinWorkbenchWindowTemplateSurfaceBridge {
    pub(super) fn apply_responsive_toolbar_layout(
        &mut self,
        shell_size: UiSize,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        let compact = shell_size.width <= COMPACT_TOOLBAR_MAX_WIDTH;
        for control_id in COMPACT_HIDDEN_MODULE_TABS {
            self.set_visible(control_id, !compact)?;
        }
        self.set_visible(WORKBENCH_MODULE_OVERFLOW_TRIGGER_CONTROL_ID, compact)?;
        if !compact {
            self.close_workbench_window_menu_control(WORKBENCH_MODULE_OVERFLOW_MENU_CONTROL_ID)?;
        }

        let full_toolbar = shell_size.width >= FULL_TOOLBAR_MIN_WIDTH;
        for control_id in SECONDARY_MODULE_COMMANDS {
            self.set_visible(control_id, full_toolbar)?;
        }
        for control_id in SECONDARY_TOOLBAR_GROUPS {
            self.set_visible(control_id, full_toolbar)?;
        }
        apply_fixed_control_width(
            &mut self.template_surface.surface,
            MODULE_COMMAND_GROUP_CONTROL_ID,
            if full_toolbar {
                MODULE_COMMAND_GROUP_FULL_WIDTH
            } else {
                MODULE_COMMAND_GROUP_COMPACT_WIDTH
            },
        )?;
        Ok(())
    }
}

fn apply_fixed_control_width(
    surface: &mut UiSurface,
    control_id: &str,
    width: f32,
) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
    let Some(node_id) = surface_control_node_id(surface, control_id) else {
        return Ok(());
    };
    let changed = {
        let Some(node) = surface.tree.node_mut(node_id) else {
            return Ok(());
        };
        let next_width = fixed_axis(width);
        let changed = node.constraints.width != next_width;
        node.constraints.width = next_width;
        changed
    };

    if changed {
        surface.tree.mark_layout_dirty(node_id)?;
    }
    Ok(())
}

fn surface_control_node_id(surface: &UiSurface, control_id: &str) -> Option<UiNodeId> {
    surface.tree.nodes.values().find_map(|node| {
        node.template_metadata
            .as_ref()
            .and_then(|metadata| metadata.control_id.as_deref())
            .filter(|candidate| *candidate == control_id)
            .map(|_| node.node_id)
    })
}

fn fixed_axis(size: f32) -> AxisConstraint {
    AxisConstraint {
        min: size.max(0.0),
        max: size.max(0.0),
        preferred: size.max(0.0),
        priority: 100,
        weight: 1.0,
        stretch_mode: StretchMode::Fixed,
    }
}
