#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FileDragDropBackend {
    WinitWindowEvents,
    BrowserDragEvents,
}

impl FileDragDropBackend {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::WinitWindowEvents => "winit_window_events",
            Self::BrowserDragEvents => "browser_drag_events",
        }
    }
}
