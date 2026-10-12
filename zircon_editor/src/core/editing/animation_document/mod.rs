//! Core-owned animation authoring source and reversible source edits.
//!
//! UI sessions receive only a read handle. All source replacement is serialized by an
//! `EditorTransactionEngine` command against this store.

#[path = "asset.rs"]
mod asset;
#[path = "command.rs"]
mod command;
#[path = "compilation.rs"]
mod compilation;
#[path = "document.rs"]
mod document;
#[path = "error.rs"]
mod error;
#[path = "kind.rs"]
mod kind;
#[path = "mutation/mod.rs"]
mod mutation;
#[path = "revision.rs"]
mod revision;
#[path = "store.rs"]
mod store;

pub(crate) use asset::AnimationAuthoringAsset;
pub(crate) use command::AnimationEditCommand;
pub(crate) use compilation::AnimationDocumentCompilation;
pub(crate) use document::{AnimationAuthoringDocument, AnimationAuthoringDocumentReadHandle};
pub(crate) use error::{AnimationAuthoringDocumentError, AnimationDocumentMutationError};
pub(crate) use kind::{AnimationAuthoringDocumentKind, AnimationGraphNodeKind};
pub(crate) use mutation::AnimationDocumentMutation;
pub(crate) use revision::AnimationDocumentRevision;
pub(crate) use store::AnimationAuthoringDocumentStore;

#[cfg(test)]
#[path = "tests/cases.rs"]
mod tests;
