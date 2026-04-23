#[path = "ui/apply_presentation.rs"]
mod apply_presentation_impl;
#[path = "ui/pane_data_conversion.rs"]
mod pane_data_conversion;
#[cfg(test)]
mod tests;

pub(crate) use apply_presentation_impl::apply_presentation;
