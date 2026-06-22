mod row;
mod scroll;

use crate::ui::retained_host::host_contract::data::{FrameRect, HostPaneInteractionStateData};
use crate::ui::retained_host::host_contract::redraw::NativePointerDispatchResult;

use self::row::hierarchy_row_move_damage;
use self::scroll::hierarchy_scroll_damage;
use super::super::super::routing::PanePointerRoute;

pub(super) fn hierarchy_pointer_move_redraw(
    pointer: &PanePointerRoute,
    before: &HostPaneInteractionStateData,
    after: &HostPaneInteractionStateData,
    template_damage: Option<FrameRect>,
) -> NativePointerDispatchResult {
    if (before.hierarchy_scroll_px - after.hierarchy_scroll_px).abs() > f32::EPSILON {
        let damage = hierarchy_scroll_damage(pointer, template_damage);
        return NativePointerDispatchResult::region(damage);
    }

    let damage = hierarchy_row_move_damage(pointer, before, after, template_damage);
    NativePointerDispatchResult::region(damage)
}
