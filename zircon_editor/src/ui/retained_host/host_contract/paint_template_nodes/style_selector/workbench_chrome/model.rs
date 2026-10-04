//! 外壳区域身份由模板画家提供；绘制配方允许无填充，同时保留不同强度的分隔线。

use zircon_runtime_interface::ui::style::UiPainterResolvedState;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) enum WorkbenchChromeKind {
    WindowRoot,
    TopToolbar,
    MainBand,
    ActivityRail,
    ScenePanel,
    ViewportPanel,
    InspectorPanel,
    ContentPanel,
    ComponentDrawer,
    DrawerBody,
    DrawerColumn,
    StatusBar,
    TabsBand,
    InspectorSection,
}

/// 外壳绘制配方；无填充仍允许绘制分隔线。
#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) struct WorkbenchChromeStyle {
    pub fill: Option<[u8; 4]>,
    pub separator: [u8; 4],
    pub strong_separator: [u8; 4],
    pub soft_separator: [u8; 4],
    pub state: UiPainterResolvedState,
}
