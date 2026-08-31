//! Core-owned animation authoring source and reversible source edits.
//!
//! UI sessions receive only a read handle. All source replacement is serialized by an
//! `EditorTransactionEngine` command against this store.

mod asset;
mod command;
mod compilation;
mod document;
mod error;
mod kind;
mod mutation;
mod revision;
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
mod tests;
