use crate::ui::retained_host::host_contract::data::HostPaneInteractionStateData;
use crate::ui::retained_host::host_contract::frame_geometry::union_frame;
use crate::ui::retained_host::host_contract::redraw::NativePointerDispatchResult;

use super::super::super::routing::{PanePointerRoute, PanePointerTarget};
use super::super::super::template_hover_damage::{
    activity_reference_hover_damage, browser_reference_hover_damage, template_hover_damage,
};
use super::hierarchy::hierarchy_pointer_move_redraw;
use super::template::template_pointer_move_redraw;

pub(in crate::ui::retained_host::host_contract) fn pointer_move_redraw(
    pointer: &PanePointerRoute,
    before: &HostPaneInteractionStateData,
    after: &HostPaneInteractionStateData,
) -> NativePointerDispatchResult {
    if before == after {
        return NativePointerDispatchResult::idle();
    }

    let template_damage = template_hover_damage(before, after);
    let reference_damage = merge_hover_damage(
        browser_reference_hover_damage(before, after),
        activity_reference_hover_damage(before, after),
    );
    let damage = merge_hover_damage(template_damage, reference_damage);
    if matches!(
        &pointer.target,
        PanePointerTarget::SceneViewport(_) | PanePointerTarget::GameViewport(_)
    ) {
        return damage
            .map(NativePointerDispatchResult::region)
            .unwrap_or_else(NativePointerDispatchResult::idle);
    }
    if matches!(&pointer.target, PanePointerTarget::Hierarchy) {
        return hierarchy_pointer_move_redraw(pointer, before, after, damage);
    }

    if let Some(damage) = damage {
        return template_pointer_move_redraw(pointer, &damage);
    }

    NativePointerDispatchResult::region(pointer.frame.clone())
}

fn merge_hover_damage(
    template_damage: Option<crate::ui::retained_host::host_contract::data::FrameRect>,
    reference_damage: Option<crate::ui::retained_host::host_contract::data::FrameRect>,
) -> Option<crate::ui::retained_host::host_contract::data::FrameRect> {
    match (template_damage, reference_damage) {
        (Some(template_damage), Some(reference_damage)) => {
            Some(union_frame(&template_damage, &reference_damage))
        }
        (Some(damage), None) | (None, Some(damage)) => Some(damage),
        (None, None) => None,
    }
}

#[cfg(test)]
mod tests {
    use crate::ui::retained_host::host_contract::data::{FrameRect, HostPaneInteractionStateData};
    use crate::ui::retained_host::host_contract::native_pointer::template_hover_damage::{
        activity_reference_hover_damage, browser_reference_hover_damage,
    };

    #[test]
    fn browser_reference_hover_damage_covers_list_switch_and_leave() {
        let before = HostPaneInteractionStateData {
            browser_asset_references_hovered_index: 1,
            browser_asset_reference_hover_frame: frame(10.0, 20.0, 100.0, 60.0),
            ..HostPaneInteractionStateData::default()
        };
        let used_by = HostPaneInteractionStateData {
            browser_asset_used_by_hovered_index: 2,
            browser_asset_reference_hover_frame: frame(130.0, 20.0, 100.0, 60.0),
            ..HostPaneInteractionStateData::default()
        };
        assert_eq!(
            browser_reference_hover_damage(&before, &used_by),
            Some(frame(10.0, 20.0, 220.0, 60.0))
        );

        let none = HostPaneInteractionStateData::default();
        assert_eq!(
            browser_reference_hover_damage(&used_by, &none),
            Some(frame(130.0, 20.0, 100.0, 60.0))
        );
    }

    #[test]
    fn activity_reference_hover_damage_covers_list_switch_and_leave() {
        let before = HostPaneInteractionStateData {
            activity_asset_references_hovered_index: 1,
            activity_asset_reference_hover_frame: frame(10.0, 20.0, 100.0, 60.0),
            ..HostPaneInteractionStateData::default()
        };
        let used_by = HostPaneInteractionStateData {
            activity_asset_used_by_hovered_index: 2,
            activity_asset_reference_hover_frame: frame(130.0, 20.0, 100.0, 60.0),
            ..HostPaneInteractionStateData::default()
        };
        assert_eq!(
            activity_reference_hover_damage(&before, &used_by),
            Some(frame(10.0, 20.0, 220.0, 60.0))
        );

        let none = HostPaneInteractionStateData::default();
        assert_eq!(
            activity_reference_hover_damage(&used_by, &none),
            Some(frame(130.0, 20.0, 100.0, 60.0))
        );
    }

    fn frame(x: f32, y: f32, width: f32, height: f32) -> FrameRect {
        FrameRect {
            x,
            y,
            width,
            height,
        }
    }
}
