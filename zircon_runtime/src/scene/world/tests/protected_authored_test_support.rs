use crate::scene::components::Hierarchy;
use crate::scene::ecs::{ChangeTickWindow, Mut};
use crate::scene::{EntityId, World};

impl World {
    pub(crate) fn corrupt_hierarchy_parent_for_tests(
        &mut self,
        entity: EntityId,
        parent: Option<EntityId>,
    ) -> bool {
        let Some(hierarchy) = self.get_mut_prevalidated_authored::<Hierarchy>(entity) else {
            return false;
        };
        hierarchy.parent = parent;
        true
    }

    pub(crate) fn touch_hierarchy_parent_for_tests(&mut self, entity: EntityId) -> bool {
        self.get_mut_prevalidated_authored::<Hierarchy>(entity)
            .is_some()
    }

    pub(crate) fn corrupt_hierarchy_parent_with_pending_mutation_for_tests(
        &mut self,
        entity: EntityId,
        parent: Option<EntityId>,
    ) -> bool {
        let Some((value, changed_tick, this_run, recorder)) =
            self.component_mut_with_ticks_prevalidated_authored::<Hierarchy>(entity)
        else {
            return false;
        };
        let mut tracked = Mut::new_tracked(
            value,
            changed_tick,
            this_run,
            ChangeTickWindow::all(this_run),
            recorder,
        );
        tracked.parent = parent;
        true
    }
}
