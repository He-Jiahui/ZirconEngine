//! Concrete runtime owners for the versioned retained-allocation contract.
pub use zircon_runtime_interface::retained_memory::*;

use super::animation::AnimationParameterValue;
use super::render::RenderMaterialPropertyValue;

impl RetainedOwnerV1 for AnimationParameterValue {
    fn retained_bound_v1(
        &self,
        _: &mut RetainedBoundBuilderV1,
    ) -> Result<(), RetentionContractError> {
        match self {
            Self::Bool(_)
            | Self::Integer(_)
            | Self::Scalar(_)
            | Self::Vec2(_)
            | Self::Vec3(_)
            | Self::Vec4(_)
            | Self::Trigger => Ok(()),
        }
    }
}
// SAFETY: Every variant contains only inline values, with no shared mutable allocation.
unsafe impl ImmutableRetainedOwnerV1 for AnimationParameterValue {}
// SAFETY: All variants and their mutations are allocation-free scalar or vector values.
unsafe impl FixedRetainedOwnerV1 for AnimationParameterValue {}

impl RetainedOwnerV1 for RenderMaterialPropertyValue {
    fn retained_bound_v1(
        &self,
        m: &mut RetainedBoundBuilderV1,
    ) -> Result<(), RetentionContractError> {
        match self {
            Self::String { value } => value.retained_bound_v1(m),
            Self::Bool { .. }
            | Self::Float { .. }
            | Self::Int { .. }
            | Self::UInt { .. }
            | Self::Vec2 { .. }
            | Self::Vec3 { .. }
            | Self::Vec4 { .. } => Ok(()),
        }
    }
}
// SAFETY: The only owned child is String; growing it requires exclusive access and it is metered above.
unsafe impl ImmutableRetainedOwnerV1 for RenderMaterialPropertyValue {}

#[cfg(any(
    feature = "native-host",
    feature = "graphics",
    feature = "animation",
    feature = "ui"
))]
mod scene_owners {
    use super::*;
    use crate::scene::components::*;

    macro_rules! fixed_components {
        ($($t:ty),* $(,)?) => {$(
            impl RetainedOwnerV1 for $t {
                fn retained_bound_v1(&self, _: &mut RetainedBoundBuilderV1) -> Result<(), RetentionContractError> { Ok(()) }
            }
            // SAFETY: The listed concrete components contain inline values and resource handles only.
            unsafe impl ImmutableRetainedOwnerV1 for $t {}
            // SAFETY: None of the listed component representations owns a transitive allocation.
            unsafe impl FixedRetainedOwnerV1 for $t {}
        )*};
    }
    // Every field of these concrete component representations is allocation-free.
    fixed_components!(
        Hierarchy,
        LocalTransform,
        WorldMatrix,
        ActiveSelf,
        ActiveInHierarchy,
        RenderLayerMask,
        CameraComponent,
        Sprite2dComponent,
        Mesh2dComponent,
        RigidBodyComponent,
        AnimationSkeletonComponent,
        AnimationPlayerComponent,
        AnimationSequencePlayerComponent,
        AmbientLight,
        DirectionalLight,
        PointLight,
        RectLight,
        SpotLight,
        PostProcessSettingsComponent,
        PostProcessVolumeComponent,
        Mobility,
        MeshRendererPrimitiveBinding
    );

    zircon_runtime_interface::impl_retained_owner_v1!(Name { 0 });
    zircon_runtime_interface::impl_retained_owner_v1!(MeshRendererLodLevel { primitives });
    zircon_runtime_interface::impl_retained_owner_v1!(MeshRenderer {
        morph_weights,
        primitives,
        lods,
        material_property_overrides
    });
    zircon_runtime_interface::impl_retained_owner_v1!(ColliderComponent { shape });
    zircon_runtime_interface::impl_retained_owner_v1!(JointComponent { skeleton_binding });
    zircon_runtime_interface::impl_retained_owner_v1!(AnimationGraphPlayerComponent { parameters });
    zircon_runtime_interface::impl_retained_owner_v1!(AnimationStateMachinePlayerComponent {
        parameters,
        active_state
    });

    impl RetainedOwnerV1 for ColliderShape {
        fn retained_bound_v1(
            &self,
            m: &mut RetainedBoundBuilderV1,
        ) -> Result<(), RetentionContractError> {
            m.nest(|m| match self {
                Self::ConvexHull { points } => points.retained_bound_v1(m),
                Self::TriangleMesh { mesh } => mesh.retained_bound_v1(m),
                Self::HeightField { heights, .. } => heights.retained_bound_v1(m),
                Self::Compound { children } => children.retained_bound_v1(m),
                Self::Box { .. }
                | Self::Sphere { .. }
                | Self::Capsule { .. }
                | Self::Cylinder { .. } => Ok(()),
            })
        }
    }
    // SAFETY: Vec, Box and asset-reference children are metered recursively; mutation requires exclusive access.
    unsafe impl ImmutableRetainedOwnerV1 for ColliderShape {}

    impl RetainedOwnerV1 for crate::core::framework::scene::physics::PhysicsSkeletonJointBinding {
        fn retained_bound_v1(
            &self,
            m: &mut RetainedBoundBuilderV1,
        ) -> Result<(), RetentionContractError> {
            self.bone_path.retained_bound_v1(m)?;
            self.parent_bone_path.retained_bound_v1(m)
        }
    }
    // SAFETY: Both owned paths are immutable through shared references and are included in the meter.
    unsafe impl ImmutableRetainedOwnerV1
        for crate::core::framework::scene::physics::PhysicsSkeletonJointBinding
    {
    }
}

#[cfg(any(
    feature = "native-host",
    feature = "graphics",
    feature = "animation",
    feature = "ui"
))]
pub(crate) trait EntityObserverOwnerV1: RetainedOwnerV1 + Send + Sync {
    fn dispatch(
        &self,
        world: &mut crate::scene::World,
        entity: crate::scene::EntityId,
        event: &dyn std::any::Any,
    );
}

/// Explicit bounded state plus a noncapturing callback, before Any erasure.
#[cfg(any(
    feature = "native-host",
    feature = "graphics",
    feature = "animation",
    feature = "ui"
))]
pub struct RetainedEntityObserverV1<E> {
    callback: std::sync::Arc<dyn EntityObserverOwnerV1>,
    event: std::marker::PhantomData<fn() -> E>,
}
#[cfg(any(
    feature = "native-host",
    feature = "graphics",
    feature = "animation",
    feature = "ui"
))]
impl<E: 'static> RetainedEntityObserverV1<E> {
    pub fn new<S>(
        state: S,
        callback: fn(&S, &mut crate::scene::World, crate::scene::EntityId, &E),
    ) -> Self
    where
        S: ImmutableRetainedOwnerV1 + Send + Sync + 'static,
    {
        struct Owner<S, E> {
            state: S,
            callback: fn(&S, &mut crate::scene::World, crate::scene::EntityId, &E),
        }
        impl<S: RetainedOwnerV1, E> RetainedOwnerV1 for Owner<S, E> {
            fn retained_bound_v1(
                &self,
                m: &mut RetainedBoundBuilderV1,
            ) -> Result<(), RetentionContractError> {
                self.state.retained_bound_v1(m)
            }
        }
        impl<S: ImmutableRetainedOwnerV1 + Send + Sync, E: 'static> EntityObserverOwnerV1 for Owner<S, E> {
            fn dispatch(
                &self,
                world: &mut crate::scene::World,
                entity: crate::scene::EntityId,
                event: &dyn std::any::Any,
            ) {
                if let Some(event) = event.downcast_ref::<E>() {
                    (self.callback)(&self.state, world, entity, event);
                }
            }
        }
        Self {
            callback: std::sync::Arc::new(Owner { state, callback }),
            event: std::marker::PhantomData,
        }
    }
    pub(crate) fn into_owner(self) -> std::sync::Arc<dyn EntityObserverOwnerV1> {
        self.callback
    }
}

mod authoring_asset_owners {
    use super::*;
    use crate::core::framework::animation::*;
    use crate::core::framework::navigation::*;
    zircon_runtime_interface::impl_retained_owner_v1!(AnimationSequenceAsset { name, bindings });
    zircon_runtime_interface::impl_retained_owner_v1!(AnimationSequenceBindingAsset {
        entity_path,
        target_id,
        tracks
    });
    zircon_runtime_interface::impl_retained_owner_v1!(AnimationSequenceTrackAsset {
        property_path,
        channel
    });
    zircon_runtime_interface::impl_retained_owner_v1!(AnimationChannelAsset { keys });
    zircon_runtime_interface::impl_retained_owner_v1!(AnimationChannelKeyAsset {});
    zircon_runtime_interface::impl_retained_owner_v1!(AnimationGraphAsset {
        name,
        parameters,
        nodes
    });
    zircon_runtime_interface::impl_retained_owner_v1!(AnimationGraphParameterAsset {
        name,
        default_value
    });
    zircon_runtime_interface::impl_retained_owner_v1!(AnimationStateMachineAsset {
        name,
        entry_state,
        states,
        transitions,
        layers
    });
    zircon_runtime_interface::impl_retained_owner_v1!(AnimationStateAsset { name, kind });
    zircon_runtime_interface::impl_retained_owner_v1!(AnimationStateTransitionAsset {
        from_state,
        to_state,
        conditions
    });
    zircon_runtime_interface::impl_retained_owner_v1!(AnimationTransitionConditionAsset {
        parameter,
        value
    });
    zircon_runtime_interface::impl_retained_owner_v1!(AnimationStateMachineLayerAsset {
        name,
        state_machine,
        mask_weights
    });
    zircon_runtime_interface::impl_retained_owner_v1!(AnimationBlendSpace1DAsset {
        parameter,
        samples
    });
    zircon_runtime_interface::impl_retained_owner_v1!(AnimationBlendSpace1DSampleAsset { graph });
    zircon_runtime_interface::impl_retained_owner_v1!(AnimationBlendSpace2DAsset {
        parameter,
        samples
    });
    zircon_runtime_interface::impl_retained_owner_v1!(AnimationBlendSpace2DSampleAsset { graph });
    impl RetainedOwnerV1 for AnimationGraphNodeAsset {
        fn retained_bound_v1(
            &self,
            m: &mut RetainedBoundBuilderV1,
        ) -> Result<(), RetentionContractError> {
            match self {
                Self::Clip { id, clip, .. } => {
                    id.retained_bound_v1(m)?;
                    clip.retained_bound_v1(m)
                }
                Self::Blend {
                    id,
                    inputs,
                    weight_parameter,
                } => {
                    id.retained_bound_v1(m)?;
                    inputs.retained_bound_v1(m)?;
                    weight_parameter.retained_bound_v1(m)
                }
                Self::Additive {
                    id,
                    base,
                    additive,
                    weight_parameter,
                } => {
                    id.retained_bound_v1(m)?;
                    base.retained_bound_v1(m)?;
                    additive.retained_bound_v1(m)?;
                    weight_parameter.retained_bound_v1(m)
                }
                Self::Mask {
                    id,
                    input,
                    target_ids,
                } => {
                    id.retained_bound_v1(m)?;
                    input.retained_bound_v1(m)?;
                    target_ids.retained_bound_v1(m)
                }
                Self::Output { source } => source.retained_bound_v1(m),
            }
        }
    }
    impl RetainedOwnerV1 for AnimationStateKindAsset {
        fn retained_bound_v1(
            &self,
            m: &mut RetainedBoundBuilderV1,
        ) -> Result<(), RetentionContractError> {
            match self {
                Self::Clip { clip } => clip.retained_bound_v1(m),
                Self::BlendSpace1D(v) => v.retained_bound_v1(m),
                Self::BlendSpace2D(v) => v.retained_bound_v1(m),
                Self::SubMachine { state_machine } => state_machine.retained_bound_v1(m),
                Self::GraphRef { graph } => graph.retained_bound_v1(m),
            }
        }
    }
    zircon_runtime_interface::impl_retained_owner_v1!(NavMeshAsset {
        agent_type,
        area_costs,
        vertices,
        indices,
        polygons,
        tiles,
        off_mesh_links
    });
    zircon_runtime_interface::impl_retained_owner_v1!(NavMeshAreaCostAsset {});
    zircon_runtime_interface::impl_retained_owner_v1!(NavMeshPolygonAsset {});
    zircon_runtime_interface::impl_retained_owner_v1!(NavMeshTileAsset {});
    zircon_runtime_interface::impl_retained_owner_v1!(NavMeshLinkAsset {});
    zircon_runtime_interface::impl_retained_owner_v1!(NavMeshBakeDiagnostic { message });
    zircon_runtime_interface::impl_retained_owner_v1!(NavMeshBakeReport {
        asset,
        output_asset,
        diagnostics
    });
    zircon_runtime_interface::impl_retained_owner_v1!(NavigationGeneratedBakeSnapshot {
        asset,
        output_asset
    });
    zircon_runtime_interface::impl_retained_owner_v1!(NavigationGeneratedBakeChange {
        before,
        after,
        report
    });
}

#[cfg(any(
    feature = "native-host",
    feature = "graphics",
    feature = "animation",
    feature = "ui"
))]
impl RetainedOwnerV1 for crate::scene::components::NodeRecord {
    fn retained_bound_v1(
        &self,
        m: &mut RetainedBoundBuilderV1,
    ) -> Result<(), RetentionContractError> {
        self.name.retained_bound_v1(m)?;
        self.mesh.retained_bound_v1(m)?;
        self.collider.retained_bound_v1(m)?;
        self.joint.retained_bound_v1(m)?;
        self.animation_graph_player.retained_bound_v1(m)?;
        self.animation_state_machine_player.retained_bound_v1(m)
    }
}

/// Quote for one complete Create, including mesh tail inserts and callbacks.
/// The Arc belongs to the World-private observer registry from World creation.
/// Preflight clones only its handle. World moves keep it; clones/replacements
/// create a fresh Arc, and an outstanding quote keeps old identity from reuse.
#[cfg(any(
    feature = "native-host",
    feature = "graphics",
    feature = "animation",
    feature = "ui"
))]
#[derive(Clone, Debug)]
pub struct RetainedCreateQuoteV1 {
    pub(crate) world_owner: std::sync::Arc<()>,
    pub(crate) registry_epoch: u64,
    pub(crate) world_generation: u64,
    pub(crate) model: RetainedAccountingModelV1,
    pub record_bytes: u64,
}
#[cfg(any(
    feature = "native-host",
    feature = "graphics",
    feature = "animation",
    feature = "ui"
))]
impl RetainedOwnerV1 for RetainedCreateQuoteV1 {
    fn retained_bound_v1(
        &self,
        m: &mut RetainedBoundBuilderV1,
    ) -> Result<(), RetentionContractError> {
        m.arc_payload(self.world_owner.as_ref())
    }
}

#[cfg(any(
    feature = "native-host",
    feature = "graphics",
    feature = "animation",
    feature = "ui"
))]
pub(crate) trait LifecycleEffectOwnerV1: Send + Sync {
    fn effects(
        &self,
        world: &crate::scene::World,
        meter: &mut RetainedBoundBuilderV1,
    ) -> Result<(), RetentionContractError>;
    fn dispatch(
        &self,
        world: &mut crate::scene::World,
        event: &crate::scene::ecs::ComponentLifecycleEvent,
    );
}

/// Mandatory producer for callback effects on a history snapshot. The callbacks
/// themselves remain World-owned; they are not captured by a NodeRecord.
#[cfg(any(
    feature = "native-host",
    feature = "graphics",
    feature = "animation",
    feature = "ui"
))]
pub struct RetainedLifecycleObserverV1 {
    owner: std::sync::Arc<dyn LifecycleEffectOwnerV1>,
}
#[cfg(any(
    feature = "native-host",
    feature = "graphics",
    feature = "animation",
    feature = "ui"
))]
impl RetainedLifecycleObserverV1 {
    /// State is explicit, recursively metered and immutable through aliases.
    /// effects adds the cumulative extra ceiling for the WHOLE Create: every
    /// invocation, recursive dispatch, child creation and newly registered
    /// producer reachable from this state, including later mesh-entry events.
    /// It must be pure. Existing state is metered first. A producer violation
    /// after writes is an applied/interrupted failure, not admission refusal.
    pub fn new<S>(
        state: S,
        effects: fn(
            &S,
            &crate::scene::World,
            &mut RetainedBoundBuilderV1,
        ) -> Result<(), RetentionContractError>,
        callback: fn(&S, &mut crate::scene::World, &crate::scene::ecs::ComponentLifecycleEvent),
    ) -> Self
    where
        S: ImmutableRetainedOwnerV1 + Send + Sync + 'static,
    {
        struct Owner<S> {
            state: S,
            effects: fn(
                &S,
                &crate::scene::World,
                &mut RetainedBoundBuilderV1,
            ) -> Result<(), RetentionContractError>,
            callback: fn(&S, &mut crate::scene::World, &crate::scene::ecs::ComponentLifecycleEvent),
        }
        impl<S: ImmutableRetainedOwnerV1 + Send + Sync> LifecycleEffectOwnerV1 for Owner<S> {
            fn effects(
                &self,
                world: &crate::scene::World,
                m: &mut RetainedBoundBuilderV1,
            ) -> Result<(), RetentionContractError> {
                m.owned(&self.state)?;
                (self.effects)(&self.state, world, m)
            }
            fn dispatch(
                &self,
                world: &mut crate::scene::World,
                event: &crate::scene::ecs::ComponentLifecycleEvent,
            ) {
                (self.callback)(&self.state, world, event);
            }
        }
        Self {
            owner: std::sync::Arc::new(Owner {
                state,
                effects,
                callback,
            }),
        }
    }

    /// Noncapturing callback: no captured World/Level alias can mutate components.
    pub fn read_only_v1(
        callback: fn(&crate::scene::World, &crate::scene::ecs::ComponentLifecycleEvent),
    ) -> Self {
        struct ReadOnly(fn(&crate::scene::World, &crate::scene::ecs::ComponentLifecycleEvent));
        impl LifecycleEffectOwnerV1 for ReadOnly {
            fn effects(
                &self,
                _: &crate::scene::World,
                _: &mut RetainedBoundBuilderV1,
            ) -> Result<(), RetentionContractError> {
                Ok(())
            }
            fn dispatch(
                &self,
                world: &mut crate::scene::World,
                event: &crate::scene::ecs::ComponentLifecycleEvent,
            ) {
                (self.0)(world, event);
            }
        }
        Self {
            owner: std::sync::Arc::new(ReadOnly(callback)),
        }
    }

    /// Adapt an audited callback that adds no allocation reachable from the final
    /// NodeRecord/dynamic-presence snapshot.
    ///
    /// # Safety
    /// This is an accounting invariant, not a memory-safety license. All calls,
    /// aliases, recursive dispatch and new producers must preserve that ceiling
    /// for the complete operation. World/schedule retirement may interrupt but
    /// must not adopt a replacement owner's payload. Diagnostic counters/logs
    /// and World-owned scheduling/resources are outside history payload.
    /// This does NOT report an opaque closure's owned allocation as zero.
    pub unsafe fn preserves_history_snapshot_v1<F>(callback: F) -> Self
    where
        F: Fn(&mut crate::scene::World, &crate::scene::ecs::ComponentLifecycleEvent)
            + Send
            + Sync
            + 'static,
    {
        struct Preserving<F>(F);
        impl<F> LifecycleEffectOwnerV1 for Preserving<F>
        where
            F: Fn(&mut crate::scene::World, &crate::scene::ecs::ComponentLifecycleEvent)
                + Send
                + Sync,
        {
            fn effects(
                &self,
                _: &crate::scene::World,
                _: &mut RetainedBoundBuilderV1,
            ) -> Result<(), RetentionContractError> {
                Ok(())
            }
            fn dispatch(
                &self,
                world: &mut crate::scene::World,
                event: &crate::scene::ecs::ComponentLifecycleEvent,
            ) {
                (self.0)(world, event);
            }
        }
        Self {
            owner: std::sync::Arc::new(Preserving(callback)),
        }
    }
    pub(crate) fn into_owner(self) -> std::sync::Arc<dyn LifecycleEffectOwnerV1> {
        self.owner
    }
}
