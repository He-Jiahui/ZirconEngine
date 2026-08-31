mod bootstrap;
mod editor;
mod headless;
#[cfg(feature = "platform-winit")]
mod runtime;
#[cfg(feature = "platform-winit")]
mod runtime_session_args;

#[cfg(feature = "target-editor-host")]
pub use editor::EditorApplicationComposition;

#[derive(Debug, Default)]
pub struct EntryRunner;
