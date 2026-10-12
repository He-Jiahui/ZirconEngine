#[path = "../../tab_drag/payload/document.rs"]
mod document;
#[path = "../../tab_drag/payload/drawer.rs"]
mod drawer;
#[path = "../../tab_drag/payload/route.rs"]
mod route;

pub(super) use self::route::tab_drag_payload_for_route;
