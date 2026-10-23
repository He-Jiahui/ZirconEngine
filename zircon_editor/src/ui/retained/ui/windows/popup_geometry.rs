use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
use zircon_runtime_interface::ui::surface::UiSurfaceFrame;

/// Runtime has already resolved the control anchor, window clipping and edge
/// flipping. Its published hit entry is the same geometry used by popup paint.
/// Copy it before the host's single logical-to-physical mount transformation.
pub(super) fn apply_published_popup_geometry(
    node: &mut TemplatePaneNodeData,
    frame: &UiSurfaceFrame,
) {
    if !node.popup_open || node.role.as_str() != "Menu" {
        return;
    }
    let Some(entry) = frame.hit_grid.entries.iter().find(|entry| {
        Some(entry.node_id) == node.surface_node_id
            && entry.control_id.as_deref() == Some(node.control_id.as_str())
    }) else {
        return;
    };
    node.frame = super::template_frame(entry.frame);
    node.has_clip_frame = true;
    node.clip_frame = super::template_frame(entry.clip_frame);
}
