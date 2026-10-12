use zircon_runtime_interface::ui::surface::UiPointerEventKind;

pub(super) fn world_space_ui_pointer_status(
    kind: UiPointerEventKind,
    control_id: &str,
) -> Option<String> {
    match kind {
        UiPointerEventKind::Down => Some(world_space_status(
            "World-space UI target selected: ",
            control_id,
        )),
        UiPointerEventKind::Scroll => Some(world_space_status(
            "World-space UI scroll routed: ",
            control_id,
        )),
        UiPointerEventKind::Up => Some(world_space_status(
            "World-space UI target released: ",
            control_id,
        )),
        UiPointerEventKind::Move => None,
        UiPointerEventKind::Cancel => Some(world_space_status(
            "World-space UI target canceled: ",
            control_id,
        )),
    }
}

fn world_space_status(prefix: &str, control_id: &str) -> String {
    let mut status = String::with_capacity(prefix.len() + control_id.len());
    status.push_str(prefix);
    status.push_str(control_id);
    status
}

#[cfg(test)]
#[path = "tests/world_space.rs"]
mod tests;
