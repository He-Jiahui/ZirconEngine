use std::collections::BTreeSet;

use zircon_scene::Mobility;

use super::super::declarations::{
    VisibilityBvhInstance, VisibilityBvhUpdatePlan, VisibilityBvhUpdateStrategy,
    VisibilityInstanceUploadPlan,
};

pub(crate) fn build_instance_upload_plan(
    bvh_instances: &[VisibilityBvhInstance],
    bvh_update_plan: &VisibilityBvhUpdatePlan,
) -> VisibilityInstanceUploadPlan {
    let static_instance_entities = bvh_instances
        .iter()
        .filter(|instance| instance.key.mobility == Mobility::Static)
        .map(|instance| instance.entity)
        .collect::<Vec<_>>();
    let dynamic_instance_entities = bvh_instances
        .iter()
        .filter(|instance| instance.key.mobility == Mobility::Dynamic)
        .map(|instance| instance.entity)
        .collect::<Vec<_>>();
    let dynamic_entity_set = dynamic_instance_entities
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    let dirty_dynamic_set = match bvh_update_plan.strategy {
        VisibilityBvhUpdateStrategy::FullRebuild => dynamic_entity_set,
        VisibilityBvhUpdateStrategy::Incremental => bvh_update_plan
            .inserted_entities
            .iter()
            .chain(bvh_update_plan.updated_entities.iter())
            .copied()
            .filter(|entity| dynamic_entity_set.contains(entity))
            .collect(),
    };
    let dirty_dynamic_entities = dynamic_instance_entities
        .iter()
        .copied()
        .filter(|entity| dirty_dynamic_set.contains(entity))
        .collect::<Vec<_>>();

    VisibilityInstanceUploadPlan {
        static_instance_entities,
        dynamic_instance_entities,
        dirty_dynamic_entities,
    }
}
