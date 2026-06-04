use zircon_runtime::asset::NavMeshAsset;
use zircon_runtime::core::framework::navigation::{NavigationError, NavigationErrorKind};

pub(crate) fn validate_query_agent(
    asset: &NavMeshAsset,
    agent_type: &str,
) -> Result<(), NavigationError> {
    if asset.agent_type == agent_type {
        return Ok(());
    }
    Err(NavigationError::new(
        NavigationErrorKind::InvalidConfiguration,
        format!(
            "query agent type `{agent_type}` does not match navmesh agent type `{}`",
            asset.agent_type
        ),
    ))
}
