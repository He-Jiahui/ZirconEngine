use zircon_runtime_interface::ui::surface::UiPointerEventKind;

pub(super) fn world_space_ui_pointer_status(
    kind: UiPointerEventKind,
    control_id: &str,
) -> Option<String> {
    match kind {
        UiPointerEventKind::Down => Some(format!("World-space UI target selected: {control_id}")),
        UiPointerEventKind::Scroll => Some(format!("World-space UI scroll routed: {control_id}")),
        UiPointerEventKind::Up => Some(format!("World-space UI target released: {control_id}")),
        UiPointerEventKind::Move => None,
        UiPointerEventKind::Cancel => Some(format!("World-space UI target canceled: {control_id}")),
    }
}
