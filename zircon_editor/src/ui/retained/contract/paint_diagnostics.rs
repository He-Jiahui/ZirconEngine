#[path = "paint_diagnostics/marker.rs"]
mod marker;
#[path = "paint_diagnostics/top_bar.rs"]
mod top_bar;
#[path = "paint_diagnostics/union.rs"]
mod union;
#[path = "paint_diagnostics/visibility.rs"]
mod visibility;

pub(in crate::ui::retained_host::host_contract) use marker::debug_refresh_overlay_frame;
pub(in crate::ui::retained_host::host_contract) use top_bar::presentation_top_bar_frame;
pub(in crate::ui::retained_host::host_contract) use union::union_diagnostic_frames;

#[cfg(test)]
#[path = "tests/paint_diagnostics.rs"]
mod tests;
