use zircon_runtime_interface::ui::{
    binding::{UiBindingSourceKind, UiBindingUpdateStatus},
    component::UiValue,
    dispatch::UiInputDispatchResult,
    event_ui::UiNodeId,
    tree::UiDirtyFlags,
};

use crate::ui::binding::{binding_update_report, runtime_state_update_with_source_kind};
use crate::ui::surface::UiSurface;

pub(super) fn append_scroll_binding_report(
    surface: &UiSurface,
    result: &mut UiInputDispatchResult,
    target: UiNodeId,
    previous_offset: f32,
    next_offset: f32,
    status: UiBindingUpdateStatus,
) {
    let dirty = if status == UiBindingUpdateStatus::Applied {
        surface
            .tree
            .nodes
            .get(&target)
            .map(|node| node.dirty)
            .unwrap_or(UiDirtyFlags {
                layout: true,
                hit_test: true,
                render: true,
                input: true,
                ..UiDirtyFlags::default()
            })
    } else {
        UiDirtyFlags::default()
    };
    let report = binding_update_report(vec![runtime_state_update_with_source_kind(
        target,
        "scroll_to",
        UiBindingSourceKind::AccessibilityAction,
        target,
        "scroll_offset",
        Some(UiValue::Float(f64::from(previous_offset))),
        UiValue::Float(f64::from(next_offset)),
        dirty,
        status,
        None,
    )]);
    result.record_binding_report(report);
    result.diagnostics.notes.push(format!(
        "accessibility_scroll_binding_update:{status:?}:{}->{}",
        previous_offset, next_offset
    ));
}

pub(super) fn scroll_state_offset(surface: &UiSurface, target: UiNodeId) -> Option<f32> {
    surface
        .tree
        .nodes
        .get(&target)
        .and_then(|node| node.scroll_state)
        .map(|state| state.offset)
}
