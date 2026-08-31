use std::any::TypeId;

use crate::scene::EntityId;
use crate::scene::ecs::{Bundle, BundleStaging, Component, InternalEntity};

use super::super::{SceneError, SceneResult};
use super::{BundleInsertionTransaction, MAX_BUNDLE_COMPONENTS};

/// Adapts one deferred bundle to the long-lived final-row transaction. The
/// local type set preserves normal Bundle duplicate validation, while the
/// transaction itself permits a later command in the same queue segment to
/// overwrite an earlier command's value for the same component type.
pub(super) fn stage_bundle<B>(
    transaction: &mut BundleInsertionTransaction<'_>,
    bundle: B,
) -> SceneResult<()>
where
    B: Bundle,
{
    let mut staging = DeferredBundleStaging {
        transaction,
        component_types: [None; MAX_BUNDLE_COMPONENTS],
        component_count: 0,
    };
    bundle.stage_into(&mut staging)
}

impl<'world> BundleInsertionTransaction<'world> {
    pub(crate) fn new_deferred_existing(
        world: &'world mut crate::scene::World,
        entity: EntityId,
        internal_entity: InternalEntity,
    ) -> Self {
        let mut transaction = Self::new(world, entity, internal_entity);
        transaction.defer_final_state_validation = true;
        transaction
    }

    pub(crate) fn stage_deferred_bundle<B>(&mut self, bundle: B) -> SceneResult<()>
    where
        B: Bundle,
    {
        stage_bundle(self, bundle)
    }
}

struct DeferredBundleStaging<'transaction, 'world> {
    transaction: &'transaction mut BundleInsertionTransaction<'world>,
    component_types: [Option<TypeId>; MAX_BUNDLE_COMPONENTS],
    component_count: usize,
}

impl BundleStaging for DeferredBundleStaging<'_, '_> {
    fn stage<T>(&mut self, component: T) -> SceneResult<()>
    where
        T: Component,
    {
        let type_id = TypeId::of::<T>();
        if self.component_types[..self.component_count]
            .iter()
            .flatten()
            .any(|existing| *existing == type_id)
        {
            return Err(SceneError::DuplicateBundleComponentType);
        }
        if self.component_count >= MAX_BUNDLE_COMPONENTS {
            return Err(SceneError::BundleComponentLimitExceeded {
                limit: MAX_BUNDLE_COMPONENTS,
            });
        }
        self.component_types[self.component_count] = Some(type_id);
        self.component_count += 1;
        self.transaction.stage_deferred(component)
    }

    fn validate_final_state(&self) -> SceneResult<()> {
        Ok(())
    }
}
