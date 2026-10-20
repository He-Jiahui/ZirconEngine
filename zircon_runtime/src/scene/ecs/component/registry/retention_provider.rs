//! Descriptor-owned retained capabilities preserve concrete identity across erased transfer.
use super::{Component, ComponentDescriptorSource, ComponentId, ComponentRegistry, StorageType};
use std::any::{Any, TypeId};
use zircon_runtime_interface::retained_memory::{
    RetainedBoundBuilderV1, RetainedOwnerV1, RetentionContractError, RETAINED_PROVIDER_VERSION_V1,
};

/// The concrete provider survives Any erasure and descriptor transfer.
#[derive(Clone, Copy, Debug)]
pub(crate) struct ComponentRetentionProviderV1 {
    pub(super) type_id: TypeId,
    pub(super) version: u32,
    measure: fn(&dyn Any, &mut RetainedBoundBuilderV1) -> Result<(), RetentionContractError>,
    measure_world: fn(
        &crate::scene::World,
        crate::scene::EntityId,
        ComponentId,
        &mut RetainedBoundBuilderV1,
    ) -> Result<(), RetentionContractError>,
}
impl PartialEq for ComponentRetentionProviderV1 {
    fn eq(&self, other: &Self) -> bool {
        self.type_id == other.type_id && self.version == other.version
    }
}
impl Eq for ComponentRetentionProviderV1 {}
impl ComponentRetentionProviderV1 {
    pub(super) fn of<T: Component>() -> Self {
        fn measure<T: Component>(
            value: &dyn Any,
            meter: &mut RetainedBoundBuilderV1,
        ) -> Result<(), RetentionContractError> {
            let value = value
                .downcast_ref::<T>()
                .ok_or(RetentionContractError::Provider {
                    reason: "component retention type mismatch",
                })?;
            meter.owned(value)
        }
        fn measure_world<T: Component>(
            world: &crate::scene::World,
            entity: crate::scene::EntityId,
            _component_id: ComponentId,
            m: &mut RetainedBoundBuilderV1,
        ) -> Result<(), RetentionContractError> {
            m.owned(
                world
                    .get::<T>(entity)
                    .ok_or(RetentionContractError::Stale)?,
            )
        }
        Self {
            type_id: TypeId::of::<T>(),
            version: RETAINED_PROVIDER_VERSION_V1,
            measure: measure::<T>,
            measure_world: measure_world::<T>,
        }
    }
    pub(crate) fn measure_world(
        &self,
        world: &crate::scene::World,
        entity: crate::scene::EntityId,
        component_id: ComponentId,
        m: &mut RetainedBoundBuilderV1,
    ) -> Result<(), RetentionContractError> {
        (self.measure_world)(world, entity, component_id, m)
    }
    pub(crate) fn measurement_v1(
        &self,
    ) -> fn(&dyn Any, &mut RetainedBoundBuilderV1) -> Result<(), RetentionContractError> {
        self.measure
    }
    pub(crate) fn measure(
        &self,
        value: &dyn Any,
        meter: &mut RetainedBoundBuilderV1,
    ) -> Result<(), RetentionContractError> {
        if self.version != RETAINED_PROVIDER_VERSION_V1 || value.type_id() != self.type_id {
            return Err(RetentionContractError::Provider {
                reason: "component retention provider identity changed",
            });
        }
        (self.measure)(value, meter)
    }
}

impl ComponentRegistry {
    /// Builds the same concrete retained capability for descriptor admission and instance binding.
    pub(in crate::scene) fn dynamic_retention_provider_v1<T: Component>(
        measure_world: fn(
            &crate::scene::World,
            crate::scene::EntityId,
            ComponentId,
            &mut RetainedBoundBuilderV1,
        ) -> Result<(), RetentionContractError>,
    ) -> Option<ComponentRetentionProviderV1> {
        if T::STORAGE_TYPE != StorageType::SparseSet {
            return None;
        }
        let mut provider = ComponentRetentionProviderV1::of::<T>();
        provider.measure_world = measure_world;
        Some(provider)
    }

    /// The World owns dynamic-marker construction and supplies an id-aware
    /// borrow. Erased detached rows still use the same concrete T provider.
    pub(in crate::scene) fn bind_dynamic_retention_provider_v1<T: Component>(
        &mut self,
        id: ComponentId,
        measure_world: fn(
            &crate::scene::World,
            crate::scene::EntityId,
            ComponentId,
            &mut RetainedBoundBuilderV1,
        ) -> Result<(), RetentionContractError>,
    ) -> Result<(), RetentionContractError> {
        let descriptor = self.descriptor(id).ok_or(RetentionContractError::Stale)?;
        if !matches!(
            &descriptor.source,
            ComponentDescriptorSource::DynamicPlugin { .. }
        ) || descriptor.storage_type != T::STORAGE_TYPE
        {
            return Err(RetentionContractError::Provider {
                reason: "dynamic retained owner descriptor mismatch",
            });
        }
        let provider = Self::dynamic_retention_provider_v1::<T>(measure_world).ok_or(
            RetentionContractError::Provider {
                reason: "dynamic retained owner must use sparse-set storage",
            },
        )?;
        if self
            .retention_providers
            .get(&id)
            .is_some_and(|existing| existing != &provider)
        {
            return Err(RetentionContractError::Provider {
                reason: "dynamic retained owner type changed",
            });
        }
        self.retention_providers.insert(id, provider);
        Ok(())
    }
}
