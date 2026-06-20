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
    ComponentDrawer,
    DrawerBody,
    DrawerColumn,
    StatusBar,
    TabsBand,
    InspectorSection,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) struct WorkbenchChromeStyle {
    pub fill: Option<[u8; 4]>,
    pub separator: [u8; 4],
    pub strong_separator: [u8; 4],
    pub soft_separator: [u8; 4],
    pub state: UiPainterResolvedState,
}
