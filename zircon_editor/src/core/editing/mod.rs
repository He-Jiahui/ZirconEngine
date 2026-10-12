//! Editor core state, intents, commands, history, and local path helpers.

#[path = "animation_document/mod.rs"]
pub(crate) mod animation_document;
#[path = "authoring_world.rs"]
pub(crate) mod authoring_world;
#[path = "command.rs"]
pub(crate) mod command;
#[path = "context.rs"]
pub(crate) mod context;
#[path = "engine/mod.rs"]
pub mod engine;
#[path = "intent.rs"]
pub(crate) mod intent;
#[path = "interactive_transform/mod.rs"]
pub(crate) mod interactive_transform;
#[path = "journal_codecs/mod.rs"]
pub(crate) mod journal_codecs;
#[path = "operation/mod.rs"]
pub mod operation;
#[path = "paths.rs"]
pub(crate) mod paths;
#[path = "selection.rs"]
pub(crate) mod selection;
