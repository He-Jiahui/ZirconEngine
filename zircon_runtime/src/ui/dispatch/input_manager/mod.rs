mod bound_text_model_updates;
mod clipboard_host_requests;
mod ime_host_requests;
mod manager;
mod number_model_updates;
mod outcome;
mod pointer_table;
mod routing;
mod text_document_session;
mod text_focus_lifecycle;
mod timers;

pub use manager::UiInputManager;
pub use outcome::UiInputDispatchOutcome;
pub use pointer_table::{UiActivePointerEntry, UiActivePointerTable};
pub use routing::{
    route_policy_uses_stage, route_stage_name, route_stage_names_for_policy, UiInputRouteStage,
    UI_INPUT_ROUTE_ORDER,
};
pub(in crate::ui) use text_document_session::{
    UiTextDocumentSession, UiTextDocumentSessionError, UiTextHistoryCommit, UiTextHistoryDirection,
};
pub use timers::{UiInputTimerState, DEFAULT_TOOLTIP_DELAY_MS, DEFAULT_TOOLTIP_INTRO_DURATION_MS};
