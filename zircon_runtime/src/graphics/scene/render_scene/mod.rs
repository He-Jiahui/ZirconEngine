//! Persistent CPU render-scene ownership and incremental change publication.

mod change_journal;
mod component_projector;
mod deformation;
mod journal_cursor;
mod mesh_source;
mod primitive;
mod registry;
mod resource_dependencies;
mod scene;

pub(crate) use change_journal::{
    RenderSceneAddedPrimitive, RenderSceneApplyStats, RenderSceneChangeJournal,
    RenderSceneDirtyDomainCounts, RenderScenePrimitiveDirtyFlags, RenderScenePrimitiveRelocation,
    RenderSceneRemovedPrimitive, RenderSceneUpdatedPrimitive,
};
pub(crate) use component_projector::{
    RenderSceneComponentProjectionCommit, RenderSceneComponentProjectionError,
    RenderSceneComponentProjectionTransactionError, RenderSceneComponentProjector,
    RenderSceneGeometryResolveIssue, RenderSceneGeometryResolver, RenderSceneRequiredComponent,
    RenderSceneResolvedGeometry,
};
pub(crate) use deformation::RenderSceneSkeletalPose;
pub(crate) use journal_cursor::{
    RenderSceneJournalCommit, RenderSceneJournalCursor, RenderSceneJournalCursorError,
    RenderSceneJournalPreflight,
};
pub(crate) use mesh_source::{
    RenderSceneMeshBinding, RenderSceneMeshLod, RenderSceneMeshSelection, RenderSceneMeshSource,
    RenderSceneMeshSourceIssue, RenderSceneMeshSourceLevel,
};
pub(crate) use primitive::{
    RenderScenePrimitive, RenderScenePrimitiveDescriptor, RenderScenePrimitiveField,
    RenderScenePrimitiveInputError, RenderScenePrimitiveLocalBounds, RenderScenePrimitiveRevisions,
};
pub(crate) use registry::RenderSceneRegistry;
pub(crate) use resource_dependencies::{
    RenderSceneResourceReferenceDelta, RenderSceneResourceReferenceDeltaStats,
};
pub(crate) use scene::{
    RenderScene, RenderSceneApplyError, RenderSceneDelta, RenderSceneDeltaTransactionError,
    RenderSceneGeneration, RenderScenePrimitiveHandle, RenderSceneReadView,
    RenderSceneStorageStats,
};

#[cfg(test)]
#[path = "tests/cases.rs"]
mod tests;
