#[path = "create_or_open.rs"]
mod create_or_open;
#[path = "recent_projects.rs"]
mod recent_projects;
#[path = "resolve_session.rs"]
mod resolve_session;
#[path = "welcome_page.rs"]
mod welcome_page;
#[path = "welcome_view.rs"]
mod welcome_view;

pub(super) use welcome_view::welcome_view_descriptor;
