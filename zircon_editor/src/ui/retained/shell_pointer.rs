#[path = "shell_pointer/bridge.rs"]
mod bridge;
#[path = "shell_pointer/common.rs"]
mod common;
#[path = "shell_pointer/drag_frames.rs"]
mod drag_frames;
#[path = "shell_pointer/drag_surface.rs"]
mod drag_surface;
#[path = "shell_pointer/effects.rs"]
mod effects;
#[path = "shell_pointer/node_ids.rs"]
mod node_ids;
#[path = "shell_pointer/resize_surface.rs"]
mod resize_surface;
#[path = "shell_pointer/route.rs"]
mod route;

pub(crate) use bridge::HostShellPointerBridge;
pub(crate) use route::HostShellPointerRoute;
