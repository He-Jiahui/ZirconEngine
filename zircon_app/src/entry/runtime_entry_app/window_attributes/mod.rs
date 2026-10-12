#[path = "builder.rs"]
mod builder;
#[path = "fullscreen.rs"]
mod fullscreen;
#[path = "monitor.rs"]
mod monitor;
#[path = "position.rs"]
mod position;
#[path = "video_mode.rs"]
mod video_mode;

pub(super) use builder::runtime_window_attributes;
