use crate::scene::dynamic_scene::{DynamicSceneError, EntityRemap};
use crate::scene::World;

use super::preflight_mutation::PreflightedSceneMutation;
use super::transaction::{ensure_compiled_spawn_target_is_current, PreparedSceneSpawnCommit};

/// Performs the no-fail publication half of a completed scene preflight.
pub(super) fn commit_preflighted_scene_mutation(
    world: &mut World,
    mutation: PreflightedSceneMutation,
) -> Result<EntityRemap, DynamicSceneError> {
    crate::profile_scope!(
        "runtime",
        "dynamic_scene.transaction",
        "commit_preflight_mutation"
    );
    let PreflightedSceneMutation {
        commit,
        component_rows,
        dynamic_components,
        resource_rows,
    } = mutation;
    ensure_compiled_spawn_target_is_current(world, &commit.target)?;
    let PreparedSceneSpawnCommit {
        target: _,
        remap,
        records,
        component_type_descriptors,
    } = commit;
    let publication = world.preflight_dynamic_scene_publication(
        component_type_descriptors,
        records,
        component_rows,
        dynamic_components,
        resource_rows,
    )?;
    world.publish_preflighted_dynamic_scene(publication);
    Ok(remap)
}
