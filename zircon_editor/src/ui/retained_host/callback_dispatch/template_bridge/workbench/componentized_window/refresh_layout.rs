use zircon_runtime::ui::tree::UiRuntimeTreeLayoutExt;
use zircon_runtime_interface::ui::layout::{StretchMode, UiSize};

use super::super::error::BuiltinHostWindowTemplateBridgeError;
use super::BuiltinWorkbenchWindowTemplateSurfaceBridge;

pub(super) fn recompute(
    bridge: &mut BuiltinWorkbenchWindowTemplateSurfaceBridge,
    shell_size: UiSize,
) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
    bridge
        .template_surface
        .recompute_layout(bridge.runtime.as_ref(), shell_size)?;
    bridge.apply_responsive_toolbar_layout(shell_size)?;
    bridge
        .template_surface
        .refresh_after_state_change(bridge.runtime.as_ref())?;
    Ok(())
}

impl BuiltinWorkbenchWindowTemplateSurfaceBridge {
    pub(crate) fn set_fixed_control_extent(
        &mut self,
        control_id: &str,
        extent: UiSize,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        let Some(node_id) = self.control_node_id(control_id) else {
            return Ok(());
        };
        let changed = {
            let Some(node) = self.template_surface.surface.tree.node_mut(node_id) else {
                return Ok(());
            };
            let mut width = node.constraints.width;
            width.min = extent.width;
            width.preferred = extent.width;
            width.max = extent.width;
            width.stretch_mode = StretchMode::Fixed;
            let mut height = node.constraints.height;
            height.min = extent.height;
            height.preferred = extent.height;
            height.max = extent.height;
            height.stretch_mode = StretchMode::Fixed;
            let changed = node.constraints.width != width || node.constraints.height != height;
            node.constraints.width = width;
            node.constraints.height = height;
            changed
        };
        if changed {
            self.template_surface
                .surface
                .tree
                .mark_layout_dirty(node_id)?;
        }
        Ok(())
    }
}
