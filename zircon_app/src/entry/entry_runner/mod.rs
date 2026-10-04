mod bootstrap;
mod editor;
#[cfg(feature = "diagnostic-log")]
mod headless;
#[cfg(feature = "platform-winit")]
mod runtime;
#[cfg(any(feature = "platform-winit", feature = "diagnostic-log"))]
mod runtime_session_args;

#[cfg(feature = "diagnostic-log")]
pub use headless::{HeadlessController, HeadlessHostError, HeadlessRunReport, HeadlessStopReason};

#[cfg(feature = "target-editor-host")]
pub use editor::EditorApplicationComposition;

#[derive(Debug, Default)]
pub struct EntryRunner;
