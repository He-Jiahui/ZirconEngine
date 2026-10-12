#[path = "bootstrap.rs"]
mod bootstrap;
#[path = "editor.rs"]
mod editor;
#[cfg(feature = "diagnostic-log")]
#[path = "headless.rs"]
mod headless;
#[cfg(feature = "platform-winit")]
#[path = "runtime.rs"]
mod runtime;
#[cfg(any(feature = "platform-winit", feature = "diagnostic-log"))]
#[path = "runtime_session_args.rs"]
mod runtime_session_args;

#[cfg(feature = "diagnostic-log")]
pub use headless::{HeadlessController, HeadlessHostError, HeadlessRunReport, HeadlessStopReason};

#[cfg(feature = "target-editor-host")]
pub use editor::EditorApplicationComposition;

#[derive(Debug, Default)]
pub struct EntryRunner;
