mod bootstrap;
mod diagnostic_log_args;
mod editor;
mod headless;
#[cfg(feature = "platform-winit")]
mod runtime;
#[cfg(feature = "platform-winit")]
mod runtime_session_args;

pub use bootstrap::{EntryRuntimeBootstrap, NativePluginRuntimeBootstrap};

#[derive(Debug, Default)]
pub struct EntryRunner;
