use super::super::*;
use crate::ui::retained_host::PaneSurfaceHostContext;

impl RetainedEditorHost {
    // 指针状态写回固定取得主窗的 pane context；浮窗有独立的 UiHostWindow 与交互代际。
    // TODO: [CR-EDITOR-APP-SHELL-0005] 核实浮窗 hover/scroll 是否因主窗写回而丢失子窗状态与重绘。
    pub(in crate::ui::retained_host::app) fn pane_surface_host(
        &self,
    ) -> PaneSurfaceHostContext<'_> {
        self.ui.global::<PaneSurfaceHostContext>()
    }
}
