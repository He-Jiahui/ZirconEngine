#[path = "chrome/fill.rs"]
mod fill;
#[path = "chrome/model.rs"]
mod model;
#[path = "chrome/palette.rs"]
mod palette;
#[path = "chrome/selection.rs"]
mod selection;
#[path = "chrome/separators.rs"]
mod separators;

#[cfg(test)]
#[path = "chrome/tests/cases.rs"]
mod tests;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use model::{
    WorkbenchChromeKind, WorkbenchChromeStyle,
};
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use selection::select_workbench_chrome_style;

#[cfg(test)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use palette::{
    WORKBENCH_CHROME_CONTENT_BG, WORKBENCH_CHROME_DRAWER_BG, WORKBENCH_CHROME_PANEL_BG,
    WORKBENCH_CHROME_SOFT_SEPARATOR, WORKBENCH_CHROME_STATUS_BG, WORKBENCH_CHROME_STRONG_SEPARATOR,
    WORKBENCH_CHROME_TOPBAR_BG,
};
