use crate::scene::ecs::{
    Component, ComponentId, ComponentLifecycleEvent, LifecycleEventKind, ObserverId,
};
use crate::scene::EntityId;

use super::{SceneResult, World};

impl World {
    pub fn observe_component_lifecycle<T>(
        &mut self,
        kind: LifecycleEventKind,
        observer: crate::core::framework::retained_memory::RetainedLifecycleObserverV1,
    ) -> ObserverId
    where
        T: Component,
    {
        let component_id = self.component_id::<T>();
        self.observers
            .observe_lifecycle(kind, component_id, observer)
    }

    pub fn observe_event<E>(
        &mut self,
        observer: impl Fn(&mut World, &E) + Send + Sync + 'static,
    ) -> ObserverId
    where
        E: 'static + Send + Sync,
    {
        self.observers.observe_event(observer)
    }

    pub fn observe_entity_event<E>(
        &mut self,
        entity: EntityId,
        observer: crate::core::framework::retained_memory::RetainedEntityObserverV1<E>,
    ) -> ObserverId
    where
        E: 'static + Send + Sync,
    {
        self.observers.observe_entity_event(entity, observer)
    }

    pub fn remove_observer(&mut self, observer: ObserverId) -> SceneResult<()> {
        self.observers.remove(observer)
    }

    pub fn trigger_event<E>(&mut self, event: E)
    where
        E: 'static + Send + Sync,
    {
        let Some(callbacks) = self.observers.event_callbacks::<E>() else {
            return;
        };
        callbacks.dispatch(self, &event);
    }

    pub fn trigger_entity_event<E>(&mut self, entity: EntityId, event: E)
    where
        E: 'static + Send + Sync,
    {
        let owner = self.owner_identity().clone();
        let execution = self.begin_schedule_execution();
        let plan_owner = execution.plan();
        if let Some(callbacks) = self.observers.event_callbacks::<E>() {
            callbacks.dispatch(self, &event);
        }
        if !owner.is_same_world(self.owner_identity())
            || !self.schedule().owns_stage_plan(plan_owner)
        {
            return;
        }
        // 目标桶在全局回调执行后才取得快照；全局回调对目标注册表的修改会影响本次目标阶段。
        if let Some(callbacks) = self.observers.entity_event_callbacks::<E>(entity) {
            callbacks.dispatch(self, entity, &event);
        }
    }

    pub(crate) fn trigger_component_lifecycle(
        &mut self,
        kind: LifecycleEventKind,
        entity: EntityId,
        component_id: ComponentId,
    ) {
        let Some(descriptor) = self.component_registry.descriptor(component_id) else {
            return;
        };
        let event =
            ComponentLifecycleEvent::new(kind, entity, component_id, descriptor.type_name.as_str());
        if self.record_staged_lifecycle_events {
            self.staged_lifecycle_events.push(event);
            return;
        }
        self.dispatch_component_lifecycle(event);
    }

    pub(in crate::scene) fn dispatch_component_lifecycle(
        &mut self,
        event: ComponentLifecycleEvent,
    ) {
        let Some(callbacks) = self
            .observers
            .lifecycle_callbacks(event.kind(), event.component_id())
        else {
            return;
        };
        callbacks.dispatch(self, event);
    }
}

impl World {
    /// Pure quote: no NodeRecord construction, component clone or temporary table.
    pub fn create_history_quote_v1(
        &self,
        kind: crate::scene::components::NodeKind,
        model: crate::core::framework::retained_memory::RetainedAccountingModelV1,
        limit: u64,
    ) -> Result<
        crate::core::framework::retained_memory::RetainedCreateQuoteV1,
        crate::core::framework::retained_memory::RetentionContractError,
    > {
        use crate::core::framework::retained_memory::*;
        use crate::scene::components::{MeshRendererPrimitiveBinding, NodeKind, NodeRecord};
        let mut m = RetainedBoundBuilderV1::with_limit(model, RetainedBytesV1::new(limit));
        m.layout(
            std::mem::size_of::<NodeRecord>(),
            std::mem::align_of::<NodeRecord>(),
        )?;
        m.array::<u8>(super::bootstrap::history_name_capacity_v1(kind))?;
        if matches!(kind, NodeKind::Cube | NodeKind::Mesh) {
            m.array::<MeshRendererPrimitiveBinding>(1)?;
            m.btree::<String, crate::core::framework::render::RenderMaterialPropertyValue>(0)?;
        }
        // The identity allocation remains alive while this quote is retained.
        m.arc_payload(self.observers.create_owner_v1().as_ref())?;
        self.observers.create_effects_v1(self, &mut m)?;
        Ok(RetainedCreateQuoteV1 {
            world_owner: self.observers.create_owner_v1().clone(),
            registry_epoch: self.observers.create_epoch_v1(),
            world_generation: self.world_generation(),
            model,
            record_bytes: m.bytes().get(),
        })
    }

    /// Before the first mutation, reject registry/World/generation drift.
    pub fn validate_create_history_quote_v1(
        &self,
        quote: &crate::core::framework::retained_memory::RetainedCreateQuoteV1,
    ) -> Result<(), crate::core::framework::retained_memory::RetentionContractError> {
        if !std::sync::Arc::ptr_eq(&quote.world_owner, self.observers.create_owner_v1())
            || quote.registry_epoch != self.observers.create_epoch_v1()
            || quote.world_generation != self.world_generation()
        {
            return Err(crate::core::framework::retained_memory::RetentionContractError::Stale);
        }
        Ok(())
    }

    /// Called after the WHOLE spawn entry, before World::node_record clones.
    /// Epoch changes inside legal callbacks are expected: their future producers
    /// were included in the original owner's cumulative complete-operation proof.
    pub fn check_created_node_retention_v1(
        &self,
        entity: EntityId,
        quote: &crate::core::framework::retained_memory::RetainedCreateQuoteV1,
    ) -> Result<(), crate::core::framework::retained_memory::RetentionContractError> {
        use crate::core::framework::retained_memory::*;
        use crate::scene::components::*;
        if !std::sync::Arc::ptr_eq(&quote.world_owner, self.observers.create_owner_v1()) {
            return Err(RetentionContractError::Stale);
        }
        let mut m = RetainedBoundBuilderV1::with_limit(
            quote.model,
            RetainedBytesV1::new(quote.record_bytes),
        );
        m.layout(
            std::mem::size_of::<NodeRecord>(),
            std::mem::align_of::<NodeRecord>(),
        )?;
        // Borrow all fixed components directly. Do not call node_record here:
        // that API clones Name, mesh parameters and recursive collider children.
        macro_rules! component {
            ($($t:ty),* $(,)?) => {$(if let Some(value) = self.get::<$t>(entity) {
                m.items(1)?; value.retained_bound_v1(&mut m)?;
            })*};
        }
        component!(
            Name,
            Hierarchy,
            LocalTransform,
            crate::scene::components::WorldMatrix,
            ActiveSelf,
            ActiveInHierarchy,
            RenderLayerMask,
            Mobility,
            CameraComponent,
            MeshRenderer,
            Sprite2dComponent,
            Mesh2dComponent,
            AmbientLight,
            DirectionalLight,
            PointLight,
            RectLight,
            SpotLight,
            RigidBodyComponent,
            ColliderComponent,
            JointComponent,
            AnimationSkeletonComponent,
            AnimationPlayerComponent,
            AnimationSequencePlayerComponent,
            AnimationGraphPlayerComponent,
            AnimationStateMachinePlayerComponent,
            PostProcessSettingsComponent,
            PostProcessVolumeComponent
        );
        // Dynamic presence payload is borrowed recursively too. Its World-owned
        // hash table is neither cloned nor adopted by NodeRecord.
        if let Some(values) = self.dynamic_components.get(&entity) {
            m.items(values.len())?;
            for (name, value) in values {
                name.retained_bound_v1(&mut m)?;
                value.retained_bound_v1(&mut m)?;
            }
        }
        Ok(())
    }
}
