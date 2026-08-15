use std::sync::Arc;

use zircon_runtime::asset::{MeshSdfAsset, MeshSdfValidationError};
use zircon_runtime::graphics::{RuntimePrepareMeshSdfDeformationReason, RuntimePrepareMeshSdfSeed};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::hybrid_gi) enum HybridGiMeshSdfFallbackReason {
    Missing {
        primitive_count: usize,
        payload_count: usize,
    },
    Invalid {
        primitive_index: usize,
        error: MeshSdfValidationError,
    },
    Deforming(RuntimePrepareMeshSdfDeformationReason),
}

#[derive(Clone, Debug)]
pub(in crate::hybrid_gi) enum HybridGiMeshSdfAssetState {
    Ready(Arc<[MeshSdfAsset]>),
    VoxelFallback(HybridGiMeshSdfFallbackReason),
}

impl Default for HybridGiMeshSdfAssetState {
    fn default() -> Self {
        Self::VoxelFallback(HybridGiMeshSdfFallbackReason::Missing {
            primitive_count: 0,
            payload_count: 0,
        })
    }
}

impl HybridGiMeshSdfAssetState {
    pub(in crate::hybrid_gi) fn from_runtime(seed: RuntimePrepareMeshSdfSeed) -> Self {
        match seed {
            RuntimePrepareMeshSdfSeed::Ready(payloads) => Self::Ready(payloads),
            RuntimePrepareMeshSdfSeed::Missing {
                primitive_count,
                payload_count,
            } => Self::VoxelFallback(HybridGiMeshSdfFallbackReason::Missing {
                primitive_count,
                payload_count,
            }),
            RuntimePrepareMeshSdfSeed::Invalid {
                primitive_index,
                error,
            } => Self::VoxelFallback(HybridGiMeshSdfFallbackReason::Invalid {
                primitive_index,
                error,
            }),
            RuntimePrepareMeshSdfSeed::Deforming(reason) => {
                Self::VoxelFallback(HybridGiMeshSdfFallbackReason::Deforming(reason))
            }
        }
    }

    pub(in crate::hybrid_gi) fn uses_unbounded_skinning_fallback(&self) -> bool {
        matches!(
            self,
            Self::VoxelFallback(HybridGiMeshSdfFallbackReason::Deforming(
                RuntimePrepareMeshSdfDeformationReason::Skinning
            ))
        )
    }

    pub(in crate::hybrid_gi) fn ready_payloads(&self) -> Option<&[MeshSdfAsset]> {
        match self {
            Self::Ready(payloads) => Some(payloads),
            Self::VoxelFallback(_) => None,
        }
    }
}

impl PartialEq for HybridGiMeshSdfAssetState {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Ready(left), Self::Ready(right)) => Arc::ptr_eq(left, right),
            (Self::VoxelFallback(left), Self::VoxelFallback(right)) => left == right,
            _ => false,
        }
    }
}
