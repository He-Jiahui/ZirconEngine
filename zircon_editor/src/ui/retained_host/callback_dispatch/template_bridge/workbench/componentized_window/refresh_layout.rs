use zircon_runtime_interface::ui::layout::UiSize;

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
    bridge.refresh_command_palette_popup_anchor()?;
    bridge
        .template_surface
        .refresh_after_state_change(bridge.runtime.as_ref())?;
    Ok(())
}
