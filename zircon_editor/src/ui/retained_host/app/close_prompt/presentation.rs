use super::model::PendingClosePrompt;
use crate::ui::retained_host::UiHostWindow;

mod data;
mod layout;
mod text;

pub(in crate::ui::retained_host::app) fn show_prompt(
    ui: &UiHostWindow,
    prompt: &PendingClosePrompt,
) {
    ui.set_close_prompt(data::host_prompt_data(ui, prompt));
}

pub(in crate::ui::retained_host::app) fn clear_prompt(ui: &UiHostWindow) {
    ui.clear_close_prompt();
}
