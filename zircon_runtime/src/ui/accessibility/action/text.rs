#[path = "text/replace.rs"]
mod replace;
#[path = "text/selection.rs"]
mod selection;

pub(super) use replace::dispatch_replace_selected_text;
pub(super) use selection::dispatch_set_text_selection;
