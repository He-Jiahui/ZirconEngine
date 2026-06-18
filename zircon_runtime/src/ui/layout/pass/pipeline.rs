#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiLayoutPassStage {
    ResponsiveStyleResolution,
    Measurement,
    BackendSelection,
    TaffyBridgeArrangement,
    ZirconFallbackArrangement,
    ClipAndVirtualWindowPropagation,
    SelectionReport,
}

impl UiLayoutPassStage {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ResponsiveStyleResolution => "responsive_style_resolution",
            Self::Measurement => "measurement",
            Self::BackendSelection => "backend_selection",
            Self::TaffyBridgeArrangement => "taffy_bridge_arrangement",
            Self::ZirconFallbackArrangement => "zircon_fallback_arrangement",
            Self::ClipAndVirtualWindowPropagation => "clip_and_virtual_window_propagation",
            Self::SelectionReport => "selection_report",
        }
    }
}

pub const UI_LAYOUT_PASS_ORDER: [UiLayoutPassStage; 7] = [
    UiLayoutPassStage::ResponsiveStyleResolution,
    UiLayoutPassStage::Measurement,
    UiLayoutPassStage::BackendSelection,
    UiLayoutPassStage::TaffyBridgeArrangement,
    UiLayoutPassStage::ZirconFallbackArrangement,
    UiLayoutPassStage::ClipAndVirtualWindowPropagation,
    UiLayoutPassStage::SelectionReport,
];

pub fn ui_layout_pass_stage_names() -> [&'static str; 7] {
    UI_LAYOUT_PASS_ORDER.map(UiLayoutPassStage::as_str)
}

pub(super) fn assert_layout_pass_stage(stage: UiLayoutPassStage, index: usize) {
    debug_assert_eq!(
        UI_LAYOUT_PASS_ORDER.get(index).copied(),
        Some(stage),
        "runtime UI layout pass stage order is out of sync"
    );
}
