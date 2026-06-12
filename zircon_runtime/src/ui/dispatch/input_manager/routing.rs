#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiInputRouteStage {
    PointerCapture,
    PopupStack,
    PreviewTunnel,
    DirectTarget,
    BubblePath,
    FocusPath,
    DefaultAction,
}

/// Slate-style authority order used by the manager. Concrete event dispatchers
/// still own their leaf behavior; this list fixes the cross-cutting route order.
pub const UI_INPUT_ROUTE_ORDER: [UiInputRouteStage; 7] = [
    UiInputRouteStage::PointerCapture,
    UiInputRouteStage::PopupStack,
    UiInputRouteStage::PreviewTunnel,
    UiInputRouteStage::DirectTarget,
    UiInputRouteStage::BubblePath,
    UiInputRouteStage::FocusPath,
    UiInputRouteStage::DefaultAction,
];
