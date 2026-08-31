use zircon_runtime_interface::ui::component::{UiComponentDescriptor, UiDragPayloadKind};

pub(super) fn accepted_drag_payloads(
    component_descriptor: Option<&UiComponentDescriptor>,
) -> String {
    component_descriptor
        .map(|descriptor| join_drag_payloads(&descriptor.drop_policy.accepts))
        .unwrap_or_default()
}

fn join_drag_payloads(accepts: &[UiDragPayloadKind]) -> String {
    let capacity = accepts
        .iter()
        .map(|kind| kind.as_str().len())
        .sum::<usize>()
        + accepts.len().saturating_sub(1);
    let mut joined = String::with_capacity(capacity);
    for (index, kind) in accepts.iter().enumerate() {
        if index != 0 {
            joined.push(',');
        }
        joined.push_str(kind.as_str());
    }
    joined
}

#[cfg(test)]
#[path = "drag_payloads/direct_join_tests.rs"]
mod direct_join_tests;
