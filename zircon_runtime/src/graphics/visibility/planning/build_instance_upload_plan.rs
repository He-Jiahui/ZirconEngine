use std::collections::BTreeSet;

use crate::core::framework::scene::Mobility;

use super::super::declarations::{
    VisibilityBvhInstance, VisibilityBvhUpdatePlan, VisibilityBvhUpdateStrategy,
    VisibilityInstanceUploadPlan,
};

pub(crate) fn build_instance_upload_plan(
    bvh_instances: &[VisibilityBvhInstance],
    bvh_update_plan: &VisibilityBvhUpdatePlan,
) -> VisibilityInstanceUploadPlan {
    let static_instance_keys = bvh_instances
        .iter()
        .filter(|instance| instance.key.mobility == Mobility::Static)
        .map(|instance| instance.stable_instance_key)
        .collect::<Vec<_>>();
    let dynamic_instance_keys = bvh_instances
        .iter()
        .filter(|instance| instance.key.mobility == Mobility::Dynamic)
        .map(|instance| instance.stable_instance_key)
        .collect::<Vec<_>>();
    let dynamic_instance_key_set = dynamic_instance_keys
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    let dirty_dynamic_set = match bvh_update_plan.strategy {
        VisibilityBvhUpdateStrategy::FullRebuild => dynamic_instance_key_set,
        VisibilityBvhUpdateStrategy::Incremental => bvh_update_plan
            .inserted_stable_instance_keys
            .iter()
            .chain(bvh_update_plan.updated_stable_instance_keys.iter())
            .copied()
            .filter(|key| dynamic_instance_key_set.contains(key))
            .collect(),
    };
    let dirty_dynamic_instance_keys = dynamic_instance_keys
        .iter()
        .copied()
        .filter(|key| dirty_dynamic_set.contains(key))
        .collect::<Vec<_>>();

    VisibilityInstanceUploadPlan {
        static_instance_keys,
        dynamic_instance_keys,
        dirty_dynamic_instance_keys,
    }
}
