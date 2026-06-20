mod fill;
mod model;
mod palette;
mod selection;
mod separators;

#[cfg(test)]
mod tests;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use model::{
    WorkbenchChromeKind, WorkbenchChromeStyle,
};
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use selection::select_workbench_chrome_style;

#[cfg(test)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use palette::{
    WORKBENCH_CHROME_DRAWER_BG, WORKBENCH_CHROME_PANEL_BG, WORKBENCH_CHROME_SOFT_SEPARATOR,
    WORKBENCH_CHROME_STATUS_BG, WORKBENCH_CHROME_STRONG_SEPARATOR, WORKBENCH_CHROME_TOPBAR_BG,
};
