//! Entry runners that bootstrap the core runtime and host editor/runtime shells.

mod entry;
pub mod plugins;
pub mod prelude;
mod runtime_presenter;

pub use entry::{
    first_party_runtime_plugin_registrations_for_config,
    first_party_runtime_plugin_registrations_for_manifest,
    first_party_runtime_plugin_registrations_for_runtime_profile,
};
pub use entry::{BuiltinEngineEntry, EngineEntry, EntryRunMode};
pub use entry::{EntryConfig, EntryProfile, EntryRunner, NativePluginRuntimeBootstrap};
pub use plugins::{
    DefaultPlugins, DevPlugins, HeadlessPlugins, MinimalPlugins, PluginGroup, PluginGroupBuilder,
    PluginGroupError, ResolvedPluginGroup,
};

#[cfg(test)]
mod tests;
