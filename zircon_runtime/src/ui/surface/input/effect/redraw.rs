use zircon_runtime_interface::ui::{
    dispatch::UiDispatchEffect, event_ui::UiNodeId, tree::UiDirtyFlags,
};

use super::super::super::surface::UiSurface;
use super::super::{UiSurfaceInputEffectError, UiSurfaceInputEffectResult};

pub(super) fn apply_redraw_effect(
    surface: &mut UiSurface,
    effect: &UiDispatchEffect,
) -> UiSurfaceInputEffectResult<Option<UiNodeId>> {
    match effect {
        UiDispatchEffect::DirtyRedraw { target, dirty, .. } => {
            let node = surface
                .tree
                .nodes
                .get_mut(target)
                .ok_or(UiSurfaceInputEffectError::MissingDirtyTarget { node_id: *target })?;
            merge_dirty(&mut node.dirty, *dirty);
            node.state_flags.dirty |= dirty.hit_test || dirty.input;
            Ok(Some(*target))
        }
        _ => Err(UiSurfaceInputEffectError::UnexpectedEffect {
            expected: "dirty redraw",
        }),
    }
}

fn merge_dirty(target: &mut UiDirtyFlags, dirty: UiDirtyFlags) {
    target.layout |= dirty.layout;
    target.hit_test |= dirty.hit_test;
    target.render |= dirty.render;
    target.style |= dirty.style;
    target.text |= dirty.text;
    target.input |= dirty.input;
    target.visible_range |= dirty.visible_range;
}
