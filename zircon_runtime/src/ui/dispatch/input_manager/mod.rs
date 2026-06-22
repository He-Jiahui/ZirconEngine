mod manager;
mod outcome;
mod pointer_table;
mod routing;
mod timers;

pub use manager::UiInputManager;
pub use outcome::UiInputDispatchOutcome;
pub use pointer_table::{UiActivePointerEntry, UiActivePointerTable};
pub use routing::{
    route_policy_uses_stage, route_stage_name, route_stage_names_for_policy, UiInputRouteStage,
    UI_INPUT_ROUTE_ORDER,
};
pub use timers::UiInputTimerState;
