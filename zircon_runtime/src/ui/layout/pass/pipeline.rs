/// 供诊断和结构校验描述一次布局工作的先后关系；枚举不充当后端调度器。
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

/// 完整和增量布局共用的依赖顺序；安排阶段会在一次遍历中选择后端并传播裁剪。
pub const UI_LAYOUT_PASS_ORDER: [UiLayoutPassStage; 7] = [
    UiLayoutPassStage::ResponsiveStyleResolution,
    UiLayoutPassStage::Measurement,
    UiLayoutPassStage::BackendSelection,
    UiLayoutPassStage::TaffyBridgeArrangement,
    UiLayoutPassStage::ZirconFallbackArrangement,
    UiLayoutPassStage::ClipAndVirtualWindowPropagation,
    UiLayoutPassStage::SelectionReport,
];

/// 诊断消费者读取稳定的阶段名；新增阶段必须同步入口的顺序检查。
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
