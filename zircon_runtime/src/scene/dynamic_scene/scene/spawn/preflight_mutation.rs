use crate::scene::World;
use crate::scene::ecs::TransferredResourceRow;
use crate::scene::reflect::ReflectResource;
use crate::scene::world::{PreflightComponentRow, PreflightDynamicComponent};

use super::resource::transfer_preflight_resource_writes;
use super::transaction::PreparedSceneSpawnCommit;
use crate::scene::dynamic_scene::DynamicSceneError;

/// Owned output of the isolated preflight World. It contains concrete rows,
/// never target-bound storage slots or source-world entity handles.
pub(crate) struct PreflightedSceneMutation {
    pub(super) commit: PreparedSceneSpawnCommit,
    pub(super) component_rows: Vec<PreflightComponentRow>,
    pub(super) dynamic_components: Vec<PreflightDynamicComponent>,
    pub(super) resource_rows: Vec<TransferredResourceRow>,
}

pub(crate) fn extract_preflighted_scene_mutation(
    preflight: &mut World,
    commit: PreparedSceneSpawnCommit,
    resource_adapters: &[ReflectResource],
) -> Result<PreflightedSceneMutation, DynamicSceneError> {
    crate::profile_scope!(
        "runtime",
        "dynamic_scene.transaction",
        "extract_preflight_mutation"
    );
    let entities = commit
        .records
        .iter()
        .map(|record| record.id)
        .collect::<Vec<_>>();
    let dynamic_components = preflight.preflight_dynamic_components(&entities);
    let component_rows = preflight.take_preflight_component_rows(&entities)?;

    let mut resource_artifact = World::empty();
    transfer_preflight_resource_writes(preflight, &mut resource_artifact, resource_adapters)?;
    let resource_rows = resource_artifact.take_preflight_resource_rows();
    crate::profile_counter!(
        "runtime",
        "dynamic_scene.transaction.commit_artifact.materialized_component_rows",
        component_rows.len()
    );
    crate::profile_counter!(
        "runtime",
        "dynamic_scene.transaction.commit_artifact.materialized_dynamic_components",
        dynamic_components.len()
    );
    crate::profile_counter!(
        "runtime",
        "dynamic_scene.transaction.commit_artifact.materialized_resource_rows",
        resource_rows.len()
    );
    Ok(PreflightedSceneMutation {
        commit,
        component_rows,
        dynamic_components,
        resource_rows,
    })
}
