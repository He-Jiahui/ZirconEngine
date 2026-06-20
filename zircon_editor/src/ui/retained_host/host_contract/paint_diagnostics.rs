mod marker;
mod top_bar;
mod union;
mod visibility;

pub(in crate::ui::retained_host::host_contract) use marker::debug_refresh_overlay_frame;
pub(in crate::ui::retained_host::host_contract) use top_bar::presentation_top_bar_frame;
pub(in crate::ui::retained_host::host_contract) use union::union_diagnostic_frames;

#[cfg(test)]
#[path = "paint_diagnostics_tests.rs"]
mod tests;
