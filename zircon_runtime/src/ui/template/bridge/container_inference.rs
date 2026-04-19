use crate::ui::layout::UiContainerKind;

pub(super) fn infer_container(component: &str) -> UiContainerKind {
    match component {
        "Container" => UiContainerKind::Container,
        "Overlay" => UiContainerKind::Overlay,
        "Space" => UiContainerKind::Space,
        "HorizontalBox" => UiContainerKind::HorizontalBox(Default::default()),
        "VerticalBox" => UiContainerKind::VerticalBox(Default::default()),
        "ScrollableBox" => UiContainerKind::ScrollableBox(Default::default()),
        _ => UiContainerKind::Free,
    }
}
