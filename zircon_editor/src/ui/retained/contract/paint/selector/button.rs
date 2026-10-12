#[path = "button/brightness.rs"]
mod brightness;
#[path = "button/command.rs"]
mod command;
#[path = "../../../../contract/paint/selector/button/component_variant.rs"]
mod component_variant;
#[path = "button/metrics.rs"]
mod metrics;
#[path = "button/model.rs"]
mod model;
#[path = "button/palette.rs"]
mod palette;
#[path = "button/selection.rs"]
mod selection;
#[path = "button/states.rs"]
mod states;
#[path = "button/tab_like.rs"]
mod tab_like;
#[cfg(test)]
#[path = "button/tests/cases.rs"]
mod tests;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use component_variant::is_compact_icon_text_workbench_button;
#[cfg(test)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use metrics::workbench_button_border_width_from_host;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use model::{
    WorkbenchButtonKind, WorkbenchButtonStyle,
};
#[cfg(test)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use palette::{
    add_component_glyph_color_from_host, add_component_text_color_from_host,
    workbench_button_transparent_surface,
};
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use selection::select_workbench_button_style;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use tab_like::{
    is_asset_browser_tab_like_button, is_asset_browser_toolbar_chip_button,
    is_asset_browser_utility_tab_button, is_tab_like_workbench_button,
};
