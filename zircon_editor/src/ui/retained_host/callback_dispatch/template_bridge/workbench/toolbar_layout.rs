mod priority;

use zircon_runtime::ui::{surface::UiSurface, tree::UiRuntimeTreeLayoutExt};
use zircon_runtime_interface::ui::{
    event_ui::UiNodeId,
    layout::{AxisConstraint, StretchMode, UiContainerKind, UiSize},
};

use self::priority::resolve_toolbar_priority;
use super::componentized_window::BuiltinWorkbenchWindowTemplateSurfaceBridge;
use super::error::BuiltinHostWindowTemplateBridgeError;
use super::module_overflow_menu::{
    WORKBENCH_MODULE_OVERFLOW_MENU_CONTROL_ID, WORKBENCH_MODULE_OVERFLOW_TRIGGER_CONTROL_ID,
};

const MODULE_COMMAND_GROUP_CONTROL_ID: &str = "WorkbenchModuleCommands";
const RUN_GROUP_CONTROL_ID: &str = "WorkbenchToolbarRunGroup";
const TOOL_GROUP_DIVIDER_CONTROL_ID: &str = "WorkbenchToolbarToolGroupDivider";

const COMPACT_HIDDEN_MODULE_TABS: &[&str] = &[
    "WorkbenchModuleBehavior",
    "WorkbenchModuleRender",
    "WorkbenchModuleAssets",
    "WorkbenchModuleVfx",
    "WorkbenchModuleHud",
];

const SECONDARY_MODULE_COMMANDS: &[&str] = &["WorkbenchModuleDiff", "WorkbenchModuleSimulate"];
const SECONDARY_RUN_CONTROLS: &[&str] = &["WorkbenchLayoutGrid", "WorkbenchThemeToggle"];

impl BuiltinWorkbenchWindowTemplateSurfaceBridge {
    pub(super) fn apply_responsive_toolbar_layout(
        &mut self,
        shell_size: UiSize,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        let priority = resolve_toolbar_priority(&self.template_surface.surface, shell_size.width);
        let compact = priority.compact_module_tabs;
        for control_id in COMPACT_HIDDEN_MODULE_TABS {
            self.set_visible(control_id, !compact)?;
        }
        self.set_visible(WORKBENCH_MODULE_OVERFLOW_TRIGGER_CONTROL_ID, compact)?;
        if !compact {
            self.close_workbench_window_menu_control(WORKBENCH_MODULE_OVERFLOW_MENU_CONTROL_ID)?;
        }

        let full_toolbar = priority.full_command_set;
        for control_id in SECONDARY_MODULE_COMMANDS {
            self.set_visible(control_id, full_toolbar)?;
        }
        self.set_visible("WorkbenchToolbarToolGroup", priority.transform_tools)?;
        self.set_visible(TOOL_GROUP_DIVIDER_CONTROL_ID, priority.transform_tools)?;
        // Play and Run Mode are MVP commands. Keep their group reachable at every
        // breakpoint and collapse only the secondary Layout/Theme children.
        self.set_visible(RUN_GROUP_CONTROL_ID, true)?;
        for control_id in SECONDARY_RUN_CONTROLS {
            self.set_visible(control_id, full_toolbar)?;
        }
        apply_horizontal_content_width(
            &mut self.template_surface.surface,
            MODULE_COMMAND_GROUP_CONTROL_ID,
        )?;
        apply_horizontal_content_width(&mut self.template_surface.surface, RUN_GROUP_CONTROL_ID)?;
        Ok(())
    }
}

fn apply_horizontal_content_width(
    surface: &mut UiSurface,
    control_id: &str,
) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
    let Some(node_id) = surface_control_node_id(surface, control_id) else {
        return Ok(());
    };
    let width = {
        let Some(node) = surface.tree.node(node_id) else {
            return Ok(());
        };
        let UiContainerKind::HorizontalBox(config) = &node.container else {
            return Ok(());
        };
        let (content_width, visible_count) = node
            .children
            .iter()
            .filter_map(|child_id| {
                let child = surface.tree.node(*child_id)?;
                child
                    .effective_visibility()
                    .occupies_layout()
                    .then_some(child.constraints.width.preferred.max(0.0))
            })
            .fold((0.0_f32, 0_usize), |(width, count), child_width| {
                (width + child_width, count + 1)
            });
        content_width + config.gap.max(0.0) * visible_count.saturating_sub(1) as f32
    };
    apply_fixed_control_width(surface, control_id, width)
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
