use super::declarations::{
    HybridGiMeshSdfObject, HybridGiMeshSdfSceneState, HybridGiMeshSdfSyncReport,
};
use super::dirty_regions::changed_object_regions;

impl HybridGiMeshSdfSceneState {
    pub(in crate::hybrid_gi) fn synchronize(
        &mut self,
        objects: impl IntoIterator<Item = HybridGiMeshSdfObject>,
    ) -> HybridGiMeshSdfSyncReport {
        let mut next_objects = objects.into_iter().collect::<Vec<_>>();
        next_objects.sort_by_key(|object| object.stable_instance_key);
        next_objects.dedup_by_key(|object| object.stable_instance_key);
        let dirty_regions = changed_object_regions(&self.objects, &next_objects);
        self.objects = next_objects;
        HybridGiMeshSdfSyncReport { dirty_regions }
    }

    pub(in crate::hybrid_gi) fn objects(&self) -> &[HybridGiMeshSdfObject] {
        &self.objects
    }
}
