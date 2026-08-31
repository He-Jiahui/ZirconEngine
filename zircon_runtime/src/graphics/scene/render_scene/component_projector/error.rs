use std::error::Error;
use std::fmt;

use crate::core::framework::render::{RenderComponentSourceWorldId, RenderWorldSnapshotHandle};
use crate::core::framework::scene::EntityId;

use super::super::{
    RenderSceneApplyError, RenderSceneMeshSourceIssue, RenderScenePrimitiveInputError,
};
use super::RenderSceneGeometryResolveIssue;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RenderSceneRequiredComponent {
    MeshRenderer,
    WorldMatrix,
    ActiveInHierarchy,
    RenderLayerMask,
    Mobility,
}

impl fmt::Display for RenderSceneRequiredComponent {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::MeshRenderer => "MeshRenderer",
            Self::WorldMatrix => "WorldMatrix",
            Self::ActiveInHierarchy => "ActiveInHierarchy",
            Self::RenderLayerMask => "RenderLayerMask",
            Self::Mobility => "Mobility",
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RenderSceneComponentProjectionError {
    IncrementalWithoutBaseline,
    FrameWorldMismatch {
        expected: RenderWorldSnapshotHandle,
        incoming: RenderWorldSnapshotHandle,
    },
    WorldMismatch {
        expected: RenderComponentSourceWorldId,
        incoming: RenderComponentSourceWorldId,
    },
    StaleArtifact {
        applied_generation: u64,
        incoming_generation: u64,
    },
    JournalDiscontinuity {
        applied_generation: u64,
        incoming_generation: u64,
    },
    EntityExceedsStableKeyCapacity {
        entity: EntityId,
    },
    MissingPrimitive {
        entity: EntityId,
    },
    MissingRequiredComponent {
        entity: EntityId,
        component: RenderSceneRequiredComponent,
    },
    RemovedMeshRendererInUpsert {
        entity: EntityId,
    },
    InvalidLodSource {
        entity: EntityId,
        issue: RenderSceneMeshSourceIssue,
    },
    GeometryResolution {
        entity: EntityId,
        issue: RenderSceneGeometryResolveIssue,
    },
    PrimitiveInput(RenderScenePrimitiveInputError),
    Apply(RenderSceneApplyError),
}

impl fmt::Display for RenderSceneComponentProjectionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IncrementalWithoutBaseline => {
                formatter.write_str("incremental render-component artifact has no scene baseline")
            }
            Self::FrameWorldMismatch { expected, incoming } => write!(
                formatter,
                "render frame world mismatch: expected {expected:?}, incoming {incoming:?}"
            ),
            Self::WorldMismatch { expected, incoming } => write!(
                formatter,
                "render-component artifact world mismatch: expected {expected:?}, incoming {incoming:?}"
            ),
            Self::StaleArtifact {
                applied_generation,
                incoming_generation,
            } => write!(
                formatter,
                "render-component artifact generation {incoming_generation} is older than applied generation {applied_generation}"
            ),
            Self::JournalDiscontinuity {
                applied_generation,
                incoming_generation,
            } => write!(
                formatter,
                "render-component artifact jumps from generation {applied_generation} to {incoming_generation}"
            ),
            Self::EntityExceedsStableKeyCapacity { entity } => write!(
                formatter,
                "entity {entity} exceeds render-scene stable-key capacity"
            ),
            Self::MissingPrimitive { entity } => write!(
                formatter,
                "incremental render-component update has no primitive for entity {entity}"
            ),
            Self::MissingRequiredComponent { entity, component } => write!(
                formatter,
                "render-component upsert for entity {entity} is missing required {component}"
            ),
            Self::RemovedMeshRendererInUpsert { entity } => write!(
                formatter,
                "render-component upsert for entity {entity} removes MeshRenderer"
            ),
            Self::InvalidLodSource { entity, issue } => write!(
                formatter,
                "render-component upsert for entity {entity} has invalid LOD source: {issue:?}"
            ),
            Self::GeometryResolution { entity, issue } => write!(
                formatter,
                "render-component geometry resolution for entity {entity} is {issue}"
            ),
            Self::PrimitiveInput(error) => error.fmt(formatter),
            Self::Apply(error) => error.fmt(formatter),
        }
    }
}

impl Error for RenderSceneComponentProjectionError {}

impl From<RenderScenePrimitiveInputError> for RenderSceneComponentProjectionError {
    fn from(value: RenderScenePrimitiveInputError) -> Self {
        Self::PrimitiveInput(value)
    }
}

impl From<RenderSceneApplyError> for RenderSceneComponentProjectionError {
    fn from(value: RenderSceneApplyError) -> Self {
        Self::Apply(value)
    }
}
