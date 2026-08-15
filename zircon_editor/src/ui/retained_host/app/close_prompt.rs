mod actions;
mod model;
mod presentation;

pub(super) use actions::close_action_id;
pub(super) use model::{
    all_dirty_close_views, dirty_close_views, ClosePromptTarget, DirtyCloseView, PendingClosePrompt,
};
pub(super) use presentation::{clear_prompt, show_prompt};
