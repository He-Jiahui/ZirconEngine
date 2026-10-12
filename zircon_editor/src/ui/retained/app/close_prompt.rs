#[path = "close_prompt/actions.rs"]
mod actions;
#[path = "close_prompt/model.rs"]
mod model;
#[path = "close_prompt/presentation.rs"]
mod presentation;

pub(super) use actions::close_action_id;
pub(super) use model::{
    all_dirty_close_views, dirty_close_views, ClosePromptTarget, DirtyCloseView,
    FloatingClosePermit, PendingClosePrompt,
};
pub(super) use presentation::{clear_prompt, show_prompt};
