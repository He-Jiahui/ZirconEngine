#[path = "event_kind.rs"]
mod event_kind;
#[path = "route.rs"]
mod route;

pub use event_kind::UiNavigationEventKind;
pub use route::UiNavigationRoute;
