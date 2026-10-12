use crate::ui::retained_host::host_contract::data::FrameRect;
use crate::ui::retained_host::host_contract::frame_geometry::union_frame;
use crate::ui::retained_host::host_contract::redraw::NativePointerDispatchResult;

use super::super::super::routing::{PanePointerRoute, PanePointerTarget};

pub(super) fn template_pointer_move_redraw(
    pointer: &PanePointerRoute,
    template_damage: &FrameRect,
) -> NativePointerDispatchResult {
    match &pointer.target {
        PanePointerTarget::AssetTree(_)
        | PanePointerTarget::AssetContent(_)
        | PanePointerTarget::AssetReference(_, _)
        | PanePointerTarget::Welcome => {
            NativePointerDispatchResult::region(union_frame(template_damage, &pointer.frame))
        }
        _ => NativePointerDispatchResult::region(template_damage.clone()),
    }
}
