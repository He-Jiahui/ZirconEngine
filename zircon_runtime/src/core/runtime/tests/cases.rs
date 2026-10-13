#[path = "activation/mod.rs"]
mod activation;
#[path = "channel.rs"]
mod channel;
#[path = "events/mod.rs"]
mod events;
#[path = "fixtures.rs"]
mod fixtures;
// Runtime plugin integration tests require the optional native-host plugin surface;
// keep them enabled in native-host test runs while core-min omits that API.
#[cfg(feature = "native-host")]
#[path = "plugin.rs"]
mod plugin;
#[path = "registration/mod.rs"]
mod registration;
#[path = "registry_name.rs"]
mod registry_name;
#[path = "resolution/mod.rs"]
mod resolution;
#[path = "tasks.rs"]
mod tasks;
#[path = "weak.rs"]
mod weak;
