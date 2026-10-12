#[path = "../../../fanout/support/registration/event.rs"]
mod event;
#[path = "../../../fanout/support/registration/handlers.rs"]
mod handlers;

pub(super) use event::register_weapon_fire_event;
pub(super) use handlers::register_weapon_fire_handlers;
