//! Persistent CPU render-scene ownership and incremental change publication.

#[path = "change_journal.rs"]
mod change_journal;
#[path = "component_projector/mod.rs"]
mod component_projector;
#[path = "deformation.rs"]
mod deformation;
#[path = "journal_cursor.rs"]
mod journal_cursor;
#[path = "mesh_source.rs"]
mod mesh_source;
#[path = "primitive.rs"]
mod primitive;
#[path = "registry.rs"]
mod registry;
#[path = "resource_dependencies.rs"]
mod resource_dependencies;
#[path = "scene.rs"]
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
