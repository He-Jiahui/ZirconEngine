#[path = "bound_text_model_updates.rs"]
mod bound_text_model_updates;
#[path = "clipboard_host_requests.rs"]
mod clipboard_host_requests;
#[path = "ime_host_requests.rs"]
mod ime_host_requests;
#[path = "manager.rs"]
mod manager;
#[path = "number_model_updates.rs"]
mod number_model_updates;
#[path = "outcome.rs"]
mod outcome;
#[path = "pointer_table.rs"]
mod pointer_table;
#[path = "routing.rs"]
mod routing;
#[path = "text_document_session/mod.rs"]
mod text_document_session;
#[path = "text_focus_lifecycle.rs"]
mod text_focus_lifecycle;
#[path = "timers.rs"]
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
