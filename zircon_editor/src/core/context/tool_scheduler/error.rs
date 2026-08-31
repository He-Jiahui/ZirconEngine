use crate::core::tools::{ToolAuthorityState, ToolOwnerGeneration, ToolResourceCatalogError};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ToolSchedulerServiceError {
    TransitionRevisionExhausted,
    ToolInstanceIdentityExhausted,
    OwnerGenerationIdentityExhausted,
    OwnerGenerationCapacityReached { max_active_owner_generations: usize },
    OwnerGenerationUnavailable { generation: ToolOwnerGeneration },
    ResourceCatalog(ToolResourceCatalogError),
    AuthorityUnavailable { state: ToolAuthorityState },
}

impl std::fmt::Display for ToolSchedulerServiceError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TransitionRevisionExhausted => {
                formatter.write_str("tool transition revision space is exhausted")
            }
            Self::ToolInstanceIdentityExhausted => {
                formatter.write_str("tool instance identity space is exhausted")
            }
            Self::OwnerGenerationIdentityExhausted => {
                formatter.write_str("tool owner generation identity space is exhausted")
            }
            Self::OwnerGenerationCapacityReached {
                max_active_owner_generations,
            } => write!(
                formatter,
                "tool owner generation registry reached its capacity of {max_active_owner_generations}"
            ),
            Self::OwnerGenerationUnavailable { generation } => write!(
                formatter,
                "tool owner generation {generation} is not active"
            ),
            Self::ResourceCatalog(error) => error.fmt(formatter),
            Self::AuthorityUnavailable { state } => {
                write!(
                    formatter,
                    "tool authority is {state:?} and rejects this operation"
                )
            }
        }
    }
}

impl From<ToolResourceCatalogError> for ToolSchedulerServiceError {
    fn from(error: ToolResourceCatalogError) -> Self {
        Self::ResourceCatalog(error)
    }
}

impl std::error::Error for ToolSchedulerServiceError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::ResourceCatalog(error) => Some(error),
            Self::TransitionRevisionExhausted
            | Self::ToolInstanceIdentityExhausted
            | Self::OwnerGenerationIdentityExhausted
            | Self::OwnerGenerationCapacityReached { .. }
            | Self::OwnerGenerationUnavailable { .. }
            | Self::AuthorityUnavailable { .. } => None,
        }
    }
}
