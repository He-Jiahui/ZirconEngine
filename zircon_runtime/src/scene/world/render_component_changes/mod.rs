mod projector;

use crate::core::framework::render::RenderComponentChangeArtifact;
pub(super) use projector::RenderComponentChangeProjector;

use std::sync::Arc;

use crate::scene::World;

pub(super) fn is_render_component_change_source<T>() -> bool
where
    T: 'static,
{
    let component = std::any::TypeId::of::<T>();
    component == std::any::TypeId::of::<crate::scene::components::MeshRenderer>()
        || component == std::any::TypeId::of::<crate::scene::components::WorldMatrix>()
        || component == std::any::TypeId::of::<crate::scene::components::ActiveInHierarchy>()
        || component == std::any::TypeId::of::<crate::scene::components::RenderLayerMask>()
        || component == std::any::TypeId::of::<crate::scene::components::Mobility>()
}

impl World {
    pub(super) fn publish_render_component_changes(&mut self) {
        let journal = self.derived_state_dirty.render_dirty_entity_journal();
        let mut projector = self
            .derived_state_dirty
            .take_render_component_change_projector();
        projector.publish(self, &journal);
        self.derived_state_dirty
            .restore_render_component_change_projector(projector);
    }

    pub(crate) fn render_component_change_artifact(
        &self,
    ) -> Option<Arc<RenderComponentChangeArtifact>> {
        self.derived_state_dirty.render_component_change_artifact()
    }

    pub(crate) fn request_full_render_component_projection(&mut self) {
        self.derived_state_dirty.mark_render_dirty();
    }
}
