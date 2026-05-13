use zircon_runtime_interface::ui::layout::UiContainerKind;

pub(super) fn infer_container(component: &str) -> UiContainerKind {
    match component {
        "Container" => UiContainerKind::Container,
        "Overlay" => UiContainerKind::Overlay,
        "Space" => UiContainerKind::Space,
        "HorizontalBox" | "HorizontalGroup" => UiContainerKind::HorizontalBox(Default::default()),
        "VerticalBox" | "VerticalGroup" | "ListView" => {
            UiContainerKind::VerticalBox(Default::default())
        }
        "ScrollableBox" => UiContainerKind::ScrollableBox(Default::default()),
        "WrapBox" => UiContainerKind::WrapBox(Default::default()),
        "FlowBox" | "FlexBox" => UiContainerKind::WrapBox(Default::default()),
        "GridBox" | "GridGroup" => UiContainerKind::GridBox(Default::default()),
        "CanvasBox" => UiContainerKind::Free,
        "SizeBox" => UiContainerKind::SizeBox(Default::default()),
        _ => UiContainerKind::Free,
    }
}
