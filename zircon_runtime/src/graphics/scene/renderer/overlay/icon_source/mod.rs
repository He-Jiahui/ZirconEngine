#[path = "../../../icon_source/empty_viewport_icon_source.rs"]
mod empty_viewport_icon_source;
#[path = "../../../overlay/icon_source/viewport_icon_source.rs"]
mod viewport_icon_source;

pub(crate) use empty_viewport_icon_source::EmptyViewportIconSource;
pub(crate) use viewport_icon_source::ViewportIconSource;
