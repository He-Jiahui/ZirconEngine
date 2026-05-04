mod bootstrap;
mod diagnostic_log_args;
mod editor;
mod headless;
mod runtime;

pub use bootstrap::NativePluginRuntimeBootstrap;

#[derive(Debug, Default)]
pub struct EntryRunner;
