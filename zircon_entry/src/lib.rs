//! Entry runners that bootstrap the core runtime and host editor/runtime shells.

mod entry;
mod runtime_presenter;

pub use entry::{BuiltinEngineEntry, EngineEntry, EntryRunMode};
pub use entry::{BuiltinEntryModuleSet, EntryConfig, EntryProfile, EntryRunner};
