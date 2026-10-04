use std::any::TypeId;

use crate::scene::ecs::{
    Component, ComponentStorageLocation, Mut, QueryAccess, QueryAccessError, Ref,
    StableEntityLocation,
};
use crate::scene::{EntityId, World};

/// 查询编译期的访问声明与实体匹配契约；QueryState 先登记访问，再据此规划原型与组件位置。
///
/// # Safety
///
/// Implementations must add complete, truthful access for this type's `QueryData`,
/// `CachedQueryData`, and `QueryMutData` methods, including future implementations.
/// Reads may use declared read or write access. Preserve existing declarations and
/// propagate conflicts when composing data; structural restrictions must agree
/// with matching and fetches from coherent locations in the same current World.
/// Candidate matching and fetching must not access incompatible retained items or
/// invalidate their storage. Preserve protected derived/authored mutation
/// authority; access declarations do not authorize a forbidden write.
/// Synchronized change bookkeeping is permitted.
/// Safe matching methods accept arbitrary public arguments; invalid locations
/// do not become a caller safety precondition. Raw readonly and mutable fetches
/// have explicit caller obligations on their unsafe methods.
/// Items may retain only declared compatible row/tick leaves, copied values,
/// structurally protected immutable leaves, or the exact synchronized recorder.
/// They must not retain World, ordinary eager-mutated metadata, owning storage,
/// DerivedStateDirty or RenderDirtyJournalState parent references. Later legal
/// fetches, matching, failure and panic must preserve all already returned items.
///
///
/// A custom implementation must explicitly accept this contract.
///
/// ```compile_fail
/// use zircon_runtime::scene::{EntityId, World};
/// use zircon_runtime::scene::ecs::{
///     Component, ComponentStorageLocation, QueryAccess, QueryAccessError, QueryDataAccess,
/// };
///
/// #[derive(Debug)]
/// struct Health(u32);
/// impl Component for Health {}
/// type Base = &'static mut Health;
/// struct Faithful;
///
/// impl QueryDataAccess for Faithful {
///     fn update_access(world: &mut World, access: &mut QueryAccess) -> Result<(), QueryAccessError> {
///         <Base as QueryDataAccess>::update_access(world, access)
///     }
///     fn matches_data(world: &World, entity: EntityId) -> bool {
///         <Base as QueryDataAccess>::matches_data(world, entity)
///     }
///     fn matches_component_locations(
///         world: &World, entity: EntityId, locations: &[ComponentStorageLocation],
///     ) -> bool {
///         <Base as QueryDataAccess>::matches_component_locations(world, entity, locations)
///     }
/// }
/// fn main() {}
/// ```
///
/// The same faithful implementation is supported with `unsafe impl`:
///
/// ```no_run
/// use zircon_runtime::scene::{EntityId, World};
/// use zircon_runtime::scene::ecs::{
///     Component, ComponentStorageLocation, QueryAccess, QueryAccessError, QueryDataAccess,
/// };
///
/// #[derive(Debug)]
/// struct Health(u32);
/// impl Component for Health {}
/// type Base = &'static mut Health;
/// struct Faithful;
///
/// unsafe impl QueryDataAccess for Faithful {
///     fn update_access(world: &mut World, access: &mut QueryAccess) -> Result<(), QueryAccessError> {
///         <Base as QueryDataAccess>::update_access(world, access)
///     }
///     fn matches_data(world: &World, entity: EntityId) -> bool {
///         <Base as QueryDataAccess>::matches_data(world, entity)
///     }
///     fn matches_component_locations(
///         world: &World, entity: EntityId, locations: &[ComponentStorageLocation],
///     ) -> bool {
///         <Base as QueryDataAccess>::matches_component_locations(world, entity, locations)
///     }
/// }
/// fn main() {}
/// ```
pub unsafe trait QueryDataAccess {
    fn update_access(world: &mut World, access: &mut QueryAccess) -> Result<(), QueryAccessError>;
    fn matches_data(world: &World, entity: EntityId) -> bool;
    fn matches_component_locations(
        world: &World,
        entity: EntityId,
        component_locations: &[ComponentStorageLocation],
    ) -> bool;
}

/// 只读查询项的提取契约；tuple 实现可组合多个共享借用，缓存路径使用预先绑定的位置。
pub trait QueryData: QueryDataAccess {
    type Item<'world>;

    /// Fetches leaves under a genuine shared World loan or the existing System grant.
    ///
    /// # Safety
    /// `world` must remain valid/aligned for `'world`, with stable structure/storage.
    /// The selected payload/ticks and every access used by matching must be declared
    /// and compatible with all live items/Params. The invocation serializes ordinary
    /// metadata writes; no returned item may retain those metadata/owning parents.
    /// A scoped World read requires compatible actual live borrows; a short reference
    /// lifetime alone does not prove that compatibility. The provider must project
    /// returned references from their real leaves, not from a retained World reference.
    unsafe fn fetch<'world>(world: *const World, entity: EntityId) -> Option<Self::Item<'world>>;

    /// # Safety
    /// The grant/access/lifetime obligations of `fetch` apply; `ticks` is this run's window.
    unsafe fn fetch_with_ticks<'world>(
        world: *const World,
        entity: EntityId,
        _ticks: crate::scene::ecs::ChangeTickWindow,
    ) -> Option<Self::Item<'world>> {
        unsafe { Self::fetch(world, entity) }
    }

    /// # Safety
    /// `fetch`'s obligations apply, and the locations must coherently name this current
    /// candidate's actual declared types/rows. Copied public type metadata is not authority.
    unsafe fn fetch_with_component_locations<'world>(
        world: *const World,
        entity: EntityId,
        _stable_location: StableEntityLocation,
        _component_locations: &[ComponentStorageLocation],
        ticks: crate::scene::ecs::ChangeTickWindow,
    ) -> Option<Self::Item<'world>>;
}

/// 可变查询项的提取契约；目前只对单个可变组件实现，避免 tuple 中产生别名的可变引用。
///
/// # Safety
///
/// For coherent projections in the same current World, every fetch variant must
/// yield compatible, entity-local items within its truthful declared access.
/// Preserve protected derived/authored write authority. Later fetches, including
/// a panic, must leave earlier items valid for their full lifetime. Do not
/// substitute another entity or invalidate storage used by live items or plans.
/// Do not return World/container access or any safe capability that can access
/// a conflicting retained item. Synchronized change bookkeeping is permitted.
/// Safe inherited matching methods retain their arbitrary-argument contract; raw
/// readonly and mutable fetches require their method-specific caller proof.
///
///
/// A custom implementation must explicitly accept this contract.
///
/// ```compile_fail
/// use zircon_runtime::scene::{EntityId, World};
/// use zircon_runtime::scene::ecs::{
///     ChangeTickWindow, Component, ComponentStorageLocation, QueryAccess, QueryAccessError,
///     QueryDataAccess, QueryMutData,
/// };
///
/// #[derive(Debug)]
/// struct Health(u32);
/// impl Component for Health {}
/// type Base = &'static mut Health;
/// struct Faithful;
///
/// // SAFETY: access and matching are exactly Base's declarations for this component.
/// unsafe impl QueryDataAccess for Faithful {
///     fn update_access(world: &mut World, access: &mut QueryAccess) -> Result<(), QueryAccessError> {
///         <Base as QueryDataAccess>::update_access(world, access)
///     }
///     fn matches_data(world: &World, entity: EntityId) -> bool {
///         <Base as QueryDataAccess>::matches_data(world, entity)
///     }
///     fn matches_component_locations(
///         world: &World, entity: EntityId, locations: &[ComponentStorageLocation],
///     ) -> bool {
///         <Base as QueryDataAccess>::matches_component_locations(world, entity, locations)
///     }
/// }
/// impl QueryMutData for Faithful {
///     type Item<'world> = &'world mut Health;
///     unsafe fn fetch_mut<'world>(world: *mut World, entity: EntityId) -> Option<Self::Item<'world>> {
///         unsafe { <Base as QueryMutData>::fetch_mut(world, entity) }
///     }
///     unsafe fn fetch_mut_with_ticks<'world>(
///         world: *mut World, entity: EntityId, ticks: ChangeTickWindow,
///     ) -> Option<Self::Item<'world>> {
///         unsafe { <Base as QueryMutData>::fetch_mut_with_ticks(world, entity, ticks) }
///     }
///     unsafe fn fetch_mut_with_component_locations<'world>(
///         world: *mut World, entity: EntityId,
///         locations: &[ComponentStorageLocation], ticks: ChangeTickWindow,
///     ) -> Option<Self::Item<'world>> {
///         unsafe { <Base as QueryMutData>::fetch_mut_with_component_locations(world, entity, locations, ticks) }
///     }
/// }
/// fn main() {}
/// ```
///
/// The same faithful implementation is supported with `unsafe impl`:
///
/// ```no_run
/// use zircon_runtime::scene::{EntityId, World};
/// use zircon_runtime::scene::ecs::{
///     ChangeTickWindow, Component, ComponentStorageLocation, QueryAccess, QueryAccessError,
///     QueryDataAccess, QueryMutData,
/// };
///
/// #[derive(Debug)]
/// struct Health(u32);
/// impl Component for Health {}
/// type Base = &'static mut Health;
/// struct Faithful;
///
/// // SAFETY: access and matching are exactly Base's declarations for this component.
/// unsafe impl QueryDataAccess for Faithful {
///     fn update_access(world: &mut World, access: &mut QueryAccess) -> Result<(), QueryAccessError> {
///         <Base as QueryDataAccess>::update_access(world, access)
///     }
///     fn matches_data(world: &World, entity: EntityId) -> bool {
///         <Base as QueryDataAccess>::matches_data(world, entity)
///     }
///     fn matches_component_locations(
///         world: &World, entity: EntityId, locations: &[ComponentStorageLocation],
///     ) -> bool {
///         <Base as QueryDataAccess>::matches_component_locations(world, entity, locations)
///     }
/// }
/// unsafe impl QueryMutData for Faithful {
///     type Item<'world> = &'world mut Health;
///     unsafe fn fetch_mut<'world>(world: *mut World, entity: EntityId) -> Option<Self::Item<'world>> {
///         unsafe { <Base as QueryMutData>::fetch_mut(world, entity) }
///     }
///     unsafe fn fetch_mut_with_ticks<'world>(
///         world: *mut World, entity: EntityId, ticks: ChangeTickWindow,
///     ) -> Option<Self::Item<'world>> {
///         unsafe { <Base as QueryMutData>::fetch_mut_with_ticks(world, entity, ticks) }
///     }
///     unsafe fn fetch_mut_with_component_locations<'world>(
///         world: *mut World, entity: EntityId,
///         locations: &[ComponentStorageLocation], ticks: ChangeTickWindow,
///     ) -> Option<Self::Item<'world>> {
///         unsafe { <Base as QueryMutData>::fetch_mut_with_component_locations(world, entity, locations, ticks) }
///     }
/// }
/// fn main() {}
/// ```
pub unsafe trait QueryMutData: QueryDataAccess {
    type Item<'world>;

    /// Fetch one mutable item without reborrowing its containing World.
    ///
    /// # Safety
    ///
    /// `world` must be valid and aligned for the complete chosen `'world`.
    /// An original exclusive World loan or the existing scheduler's proven
    /// disjoint access must protect all declared component/tick access. The
    /// candidate must be compatible with every live item; no structural mutation
    /// or conflicting value read may occur until those items' loans end.
    /// The grant must also exclude conflicting access to ordinary World clock
    /// and mutation-bookkeeping leaves. Distinct components alone do not grant
    /// that exclusion. Items must not retain references to those leaves or their
    /// owning World/dirty-journal containers.
    /// Only contracted structural/clock reads may use a scoped shared World;
    /// arbitrary safe World value reads are not authorized by the raw pointer.
    /// Calling this hook from safe code is rejected.
    ///
    /// ```compile_fail,E0133
    /// use zircon_runtime::scene::{EntityId, World};
    /// use zircon_runtime::scene::ecs::{Component, QueryMutData};
    /// struct Health(u32);
    /// impl Component for Health {}
    /// fn rejected(world: &mut World, entity: EntityId) {
    ///     let _ = <&mut Health as QueryMutData>::fetch_mut(world as *mut World, entity);
    /// }
    /// fn main() {}
    /// ```
    unsafe fn fetch_mut<'world>(world: *mut World, entity: EntityId) -> Option<Self::Item<'world>>;

    /// Fetch one mutable item in the supplied observation window.
    ///
    /// # Safety
    ///
    /// The pointer, lifetime, access and retained-item obligations of
    /// [`Self::fetch_mut`] apply. The window belongs to this query invocation.
    /// Calling this hook from safe code is rejected.
    ///
    /// ```compile_fail,E0133
    /// use zircon_runtime::scene::{EntityId, World};
    /// use zircon_runtime::scene::ecs::{Component, QueryMutData, ChangeTickWindow};
    /// struct Health(u32);
    /// impl Component for Health {}
    /// fn rejected(world: &mut World, entity: EntityId, ticks: ChangeTickWindow) {
    ///     let _ = <&mut Health as QueryMutData>::fetch_mut_with_ticks(world as *mut World, entity, ticks);
    /// }
    /// fn main() {}
    /// ```
    unsafe fn fetch_mut_with_ticks<'world>(
        world: *mut World,
        entity: EntityId,
        _ticks: crate::scene::ecs::ChangeTickWindow,
    ) -> Option<Self::Item<'world>> {
        // SAFETY: the caller provides the same pointer/access/lifetime proof.
        unsafe { Self::fetch_mut(world, entity) }
    }

    /// Fetch through already checked locations in this same current World.
    ///
    /// # Safety
    ///
    /// The obligations of [`Self::fetch_mut_with_ticks`] apply. Locations must
    /// coherently project this entity's declared types, storage and current rows;
    /// all mutable payload/tick references must be disjoint from live items.
    /// Do not replace this proof with caller-supplied type metadata alone.
    /// Calling this hook from safe code is rejected.
    ///
    /// ```compile_fail,E0133
    /// use zircon_runtime::scene::{EntityId, World};
    /// use zircon_runtime::scene::ecs::{Component, QueryMutData, ChangeTickWindow, ComponentStorageLocation};
    /// struct Health(u32);
    /// impl Component for Health {}
    /// fn rejected(world: &mut World, entity: EntityId, locations: &[ComponentStorageLocation], ticks: ChangeTickWindow) {
    ///     let _ = <&mut Health as QueryMutData>::fetch_mut_with_component_locations(world as *mut World, entity, locations, ticks);
    /// }
    /// fn main() {}
    /// ```
    unsafe fn fetch_mut_with_component_locations<'world>(
        world: *mut World,
        entity: EntityId,
        _component_locations: &[ComponentStorageLocation],
        ticks: crate::scene::ecs::ChangeTickWindow,
    ) -> Option<Self::Item<'world>>;
}

// SAFETY: Read access covers the candidate component and both ordinary/location fetches.
unsafe impl<'query, T> QueryDataAccess for &'query T
where
    T: Component,
{
    fn update_access(world: &mut World, access: &mut QueryAccess) -> Result<(), QueryAccessError> {
        let component_id = world.component_id::<T>();
        access.add_read(component_id)?;
        access.add_with(component_id);
        Ok(())
    }

    fn matches_data(world: &World, entity: EntityId) -> bool {
        world.get::<T>(entity).is_some()
    }

    fn matches_component_locations(
        _world: &World,
        _entity: EntityId,
        component_locations: &[ComponentStorageLocation],
    ) -> bool {
        component_location::<T>(component_locations).is_some()
    }
}

impl<'query, T> QueryData for &'query T
where
    T: Component,
{
    type Item<'world> = &'world T;

    unsafe fn fetch<'world>(world: *const World, entity: EntityId) -> Option<Self::Item<'world>> {
        unsafe { World::query_component_ref::<T>(world, entity) }
    }

    unsafe fn fetch_with_component_locations<'world>(
        world: *const World,
        _entity: EntityId,
        _stable_location: StableEntityLocation,
        component_locations: &[ComponentStorageLocation],
        _ticks: crate::scene::ecs::ChangeTickWindow,
    ) -> Option<Self::Item<'world>> {
        unsafe {
            let location = component_location::<T>(component_locations)?;
            let (value, _) =
                World::query_component_ref_with_ticks_at_location::<T>(world, *location)?;
            Some(value)
        }
    }
}

// SAFETY: Protected write checks and declared writes cover both shared and mutable projections.
unsafe impl<'query, T> QueryDataAccess for &'query mut T
where
    T: Component,
{
    fn update_access(world: &mut World, access: &mut QueryAccess) -> Result<(), QueryAccessError> {
        if let Some(component) = World::protected_derived_component_name::<T>() {
            return Err(QueryAccessError::ProtectedDerivedComponentWrite { component });
        }
        if let Some(component) = World::protected_authored_component_name::<T>() {
            return Err(QueryAccessError::ProtectedAuthoredComponentWrite { component });
        }
        let component_id = world.component_id::<T>();
        access.add_write(component_id)?;
        access.add_with(component_id);
        if TypeId::of::<T>() == TypeId::of::<crate::scene::components::Name>()
            || TypeId::of::<T>() == TypeId::of::<crate::scene::components::ActiveSelf>()
        {
            let hierarchy_id = world.component_id::<crate::scene::components::Hierarchy>();
            access.add_read(hierarchy_id)?;
        }
        Ok(())
    }

    fn matches_data(world: &World, entity: EntityId) -> bool {
        world.get::<T>(entity).is_some()
    }

    fn matches_component_locations(
        _world: &World,
        _entity: EntityId,
        component_locations: &[ComponentStorageLocation],
    ) -> bool {
        component_location::<T>(component_locations).is_some()
    }
}

impl<'query, T> QueryData for &'query mut T
where
    T: Component,
{
    type Item<'world> = &'world T;

    unsafe fn fetch<'world>(world: *const World, entity: EntityId) -> Option<Self::Item<'world>> {
        unsafe { World::query_component_ref::<T>(world, entity) }
    }

    unsafe fn fetch_with_component_locations<'world>(
        world: *const World,
        _entity: EntityId,
        _stable_location: StableEntityLocation,
        component_locations: &[ComponentStorageLocation],
        _ticks: crate::scene::ecs::ChangeTickWindow,
    ) -> Option<Self::Item<'world>> {
        unsafe {
            let location = component_location::<T>(component_locations)?;
            let (value, _) =
                World::query_component_ref_with_ticks_at_location::<T>(world, *location)?;
            Some(value)
        }
    }
}

// SAFETY: Declared candidate reads cover the value and its copied change ticks.
unsafe impl<'query, T> QueryDataAccess for Ref<'query, T>
where
    T: Component,
{
    fn update_access(world: &mut World, access: &mut QueryAccess) -> Result<(), QueryAccessError> {
        let component_id = world.component_id::<T>();
        access.add_read(component_id)?;
        access.add_with(component_id);
        Ok(())
    }

    fn matches_data(world: &World, entity: EntityId) -> bool {
        world.get::<T>(entity).is_some()
    }

    fn matches_component_locations(
        _world: &World,
        _entity: EntityId,
        component_locations: &[ComponentStorageLocation],
    ) -> bool {
        component_location::<T>(component_locations).is_some()
    }
}

impl<'query, T> QueryData for Ref<'query, T>
where
    T: Component,
{
    type Item<'world> = Ref<'world, T>;

    unsafe fn fetch<'world>(world: *const World, entity: EntityId) -> Option<Self::Item<'world>> {
        unsafe {
            Self::fetch_with_ticks(
                world,
                entity,
                crate::scene::ecs::ChangeTickWindow::all((&*world).read_change_tick()),
            )
        }
    }

    unsafe fn fetch_with_ticks<'world>(
        world: *const World,
        entity: EntityId,
        ticks: crate::scene::ecs::ChangeTickWindow,
    ) -> Option<Self::Item<'world>> {
        unsafe {
            let (value, component_ticks) =
                World::query_component_ref_with_ticks::<T>(world, entity)?;
            Some(Ref::new(value, component_ticks, ticks))
        }
    }

    unsafe fn fetch_with_component_locations<'world>(
        world: *const World,
        _entity: EntityId,
        _stable_location: StableEntityLocation,
        component_locations: &[ComponentStorageLocation],
        ticks: crate::scene::ecs::ChangeTickWindow,
    ) -> Option<Self::Item<'world>> {
        unsafe {
            let location = component_location::<T>(component_locations)?;
            let (value, component_ticks) =
                World::query_component_ref_with_ticks_at_location::<T>(world, *location)?;
            Some(Ref::new(value, component_ticks, ticks))
        }
    }
}

// SAFETY: Protected write checks cover candidate values, change ticks and tracked projections.
unsafe impl<'query, T> QueryDataAccess for Mut<'query, T>
where
    T: Component,
{
    fn update_access(world: &mut World, access: &mut QueryAccess) -> Result<(), QueryAccessError> {
        if let Some(component) = World::protected_derived_component_name::<T>() {
            return Err(QueryAccessError::ProtectedDerivedComponentWrite { component });
        }
        if let Some(component) = World::protected_authored_component_name::<T>() {
            return Err(QueryAccessError::ProtectedAuthoredComponentWrite { component });
        }
        let component_id = world.component_id::<T>();
        access.add_write(component_id)?;
        access.add_with(component_id);
        Ok(())
    }

    fn matches_data(world: &World, entity: EntityId) -> bool {
        world.get::<T>(entity).is_some()
    }

    fn matches_component_locations(
        _world: &World,
        _entity: EntityId,
        component_locations: &[ComponentStorageLocation],
    ) -> bool {
        component_location::<T>(component_locations).is_some()
    }
}

impl<'query, T> QueryData for Mut<'query, T>
where
    T: Component,
{
    type Item<'world> = Ref<'world, T>;

    unsafe fn fetch<'world>(world: *const World, entity: EntityId) -> Option<Self::Item<'world>> {
        unsafe {
            Self::fetch_with_ticks(
                world,
                entity,
                crate::scene::ecs::ChangeTickWindow::all((&*world).read_change_tick()),
            )
        }
    }

    unsafe fn fetch_with_ticks<'world>(
        world: *const World,
        entity: EntityId,
        ticks: crate::scene::ecs::ChangeTickWindow,
    ) -> Option<Self::Item<'world>> {
        unsafe {
            let (value, component_ticks) =
                World::query_component_ref_with_ticks::<T>(world, entity)?;
            Some(Ref::new(value, component_ticks, ticks))
        }
    }

    unsafe fn fetch_with_component_locations<'world>(
        world: *const World,
        _entity: EntityId,
        _stable_location: StableEntityLocation,
        component_locations: &[ComponentStorageLocation],
        ticks: crate::scene::ecs::ChangeTickWindow,
    ) -> Option<Self::Item<'world>> {
        unsafe {
            let location = component_location::<T>(component_locations)?;
            let (value, component_ticks) =
                World::query_component_ref_with_ticks_at_location::<T>(world, *location)?;
            Some(Ref::new(value, component_ticks, ticks))
        }
    }
}

// SAFETY: Coherent fetches target the candidate value/ticks without structural changes; tracking is synchronized.
unsafe impl<'query, T> QueryMutData for Mut<'query, T>
where
    T: Component,
{
    type Item<'world> = Mut<'world, T>;

    unsafe fn fetch_mut<'world>(world: *mut World, entity: EntityId) -> Option<Self::Item<'world>> {
        // SAFETY: the caller permits this scoped clock-only shared read.
        let change_tick = unsafe { (&*world).read_change_tick() };
        // SAFETY: the caller's raw pointer/access/lifetime proof is unchanged.
        unsafe {
            Self::fetch_mut_with_ticks(
                world,
                entity,
                crate::scene::ecs::ChangeTickWindow::all(change_tick),
            )
        }
    }

    unsafe fn fetch_mut_with_ticks<'world>(
        world: *mut World,
        entity: EntityId,
        ticks: crate::scene::ecs::ChangeTickWindow,
    ) -> Option<Self::Item<'world>> {
        let (value, component_ticks, this_run, mutation_recorder) =
            // SAFETY: inherited raw fetch obligations cover this entity's checked leaves.
            unsafe { World::query_component_mut_with_ticks::<T>(world, entity)? };
        Some(Mut::new_tracked(
            value,
            component_ticks,
            this_run,
            ticks,
            mutation_recorder,
        ))
    }

    unsafe fn fetch_mut_with_component_locations<'world>(
        world: *mut World,
        entity: EntityId,
        component_locations: &[ComponentStorageLocation],
        ticks: crate::scene::ecs::ChangeTickWindow,
    ) -> Option<Self::Item<'world>> {
        let location = component_location::<T>(component_locations)?;
        let (value, component_ticks, this_run, mutation_recorder) =
            // SAFETY: the caller supplied coherent locations and compatible retained items.
            unsafe { World::query_component_mut_with_ticks_at_location::<T>(world, entity, *location)? };
        Some(Mut::new_tracked(
            value,
            component_ticks,
            this_run,
            ticks,
            mutation_recorder,
        ))
    }
}

// SAFETY: Coherent fetches use the candidate's typed component and preserve storage structure.
unsafe impl<'query, T> QueryMutData for &'query mut T
where
    T: Component,
{
    type Item<'world> = &'world mut T;

    unsafe fn fetch_mut<'world>(world: *mut World, entity: EntityId) -> Option<Self::Item<'world>> {
        // SAFETY: raw fetch callers own the declared candidate's mutable access.
        unsafe { World::query_component_mut::<T>(world, entity) }
    }

    unsafe fn fetch_mut_with_component_locations<'world>(
        world: *mut World,
        entity: EntityId,
        component_locations: &[ComponentStorageLocation],
        _ticks: crate::scene::ecs::ChangeTickWindow,
    ) -> Option<Self::Item<'world>> {
        let location = component_location::<T>(component_locations)?;
        // SAFETY: the caller supplied coherent locations and compatible retained items.
        unsafe { World::query_component_mut_at_location::<T>(world, entity, *location) }
    }
}

// SAFETY: The optional value is a declared read; absence imposes no structural requirement.
unsafe impl<'query, T> QueryDataAccess for Option<&'query T>
where
    T: Component,
{
    fn update_access(world: &mut World, access: &mut QueryAccess) -> Result<(), QueryAccessError> {
        let component_id = world.component_id::<T>();
        access.add_read(component_id)
    }

    fn matches_data(_world: &World, _entity: EntityId) -> bool {
        true
    }

    fn matches_component_locations(
        _world: &World,
        _entity: EntityId,
        _component_locations: &[ComponentStorageLocation],
    ) -> bool {
        true
    }
}

impl<'query, T> QueryData for Option<&'query T>
where
    T: Component,
{
    type Item<'world> = Option<&'world T>;

    unsafe fn fetch<'world>(world: *const World, entity: EntityId) -> Option<Self::Item<'world>> {
        unsafe { Some(World::query_component_ref::<T>(world, entity)) }
    }

    unsafe fn fetch_with_component_locations<'world>(
        world: *const World,
        _entity: EntityId,
        _stable_location: StableEntityLocation,
        component_locations: &[ComponentStorageLocation],
        _ticks: crate::scene::ecs::ChangeTickWindow,
    ) -> Option<Self::Item<'world>> {
        unsafe {
            let Some(location) = component_location::<T>(component_locations) else {
                return Some(None);
            };
            let Some((value, _)) =
                World::query_component_ref_with_ticks_at_location::<T>(world, *location)
            else {
                return Some(None);
            };
            Some(Some(value))
        }
    }
}

// SAFETY: Fetches only copy the candidate identity and do not borrow component data.
unsafe impl QueryDataAccess for EntityId {
    fn update_access(
        _world: &mut World,
        _access: &mut QueryAccess,
    ) -> Result<(), QueryAccessError> {
        Ok(())
    }

    fn matches_data(_world: &World, _entity: EntityId) -> bool {
        true
    }

    fn matches_component_locations(
        _world: &World,
        _entity: EntityId,
        _component_locations: &[ComponentStorageLocation],
    ) -> bool {
        true
    }
}

impl QueryData for EntityId {
    type Item<'world> = EntityId;

    unsafe fn fetch<'world>(_world: *const World, entity: EntityId) -> Option<Self::Item<'world>> {
        unsafe { Some(entity) }
    }

    unsafe fn fetch_with_component_locations<'world>(
        _world: *const World,
        entity: EntityId,
        _stable_location: StableEntityLocation,
        _component_locations: &[ComponentStorageLocation],
        _ticks: crate::scene::ecs::ChangeTickWindow,
    ) -> Option<Self::Item<'world>> {
        unsafe { Some(entity) }
    }
}

// SAFETY: Fetches copy checked structural location metadata and do not borrow component data.
unsafe impl QueryDataAccess for StableEntityLocation {
    fn update_access(
        _world: &mut World,
        _access: &mut QueryAccess,
    ) -> Result<(), QueryAccessError> {
        Ok(())
    }

    fn matches_data(_world: &World, _entity: EntityId) -> bool {
        true
    }

    fn matches_component_locations(
        _world: &World,
        _entity: EntityId,
        _component_locations: &[ComponentStorageLocation],
    ) -> bool {
        true
    }
}

impl QueryData for StableEntityLocation {
    type Item<'world> = StableEntityLocation;

    unsafe fn fetch<'world>(world: *const World, entity: EntityId) -> Option<Self::Item<'world>> {
        unsafe { (&*world).internal_entity_location(entity) }
    }

    unsafe fn fetch_with_component_locations<'world>(
        _world: *const World,
        _entity: EntityId,
        stable_location: StableEntityLocation,
        _component_locations: &[ComponentStorageLocation],
        _ticks: crate::scene::ecs::ChangeTickWindow,
    ) -> Option<Self::Item<'world>> {
        unsafe { Some(stable_location) }
    }
}

// SAFETY: Matching and fetches use no component data.
unsafe impl QueryDataAccess for () {
    fn update_access(
        _world: &mut World,
        _access: &mut QueryAccess,
    ) -> Result<(), QueryAccessError> {
        Ok(())
    }

    fn matches_data(_world: &World, _entity: EntityId) -> bool {
        true
    }

    fn matches_component_locations(
        _world: &World,
        _entity: EntityId,
        _component_locations: &[ComponentStorageLocation],
    ) -> bool {
        true
    }
}

impl QueryData for () {
    type Item<'world> = ();

    unsafe fn fetch<'world>(_world: *const World, _entity: EntityId) -> Option<Self::Item<'world>> {
        unsafe { Some(()) }
    }

    unsafe fn fetch_with_component_locations<'world>(
        _world: *const World,
        _entity: EntityId,
        _stable_location: StableEntityLocation,
        _component_locations: &[ComponentStorageLocation],
        _ticks: crate::scene::ecs::ChangeTickWindow,
    ) -> Option<Self::Item<'world>> {
        unsafe { Some(()) }
    }
}

macro_rules! tuple_query_data {
    ($($name:ident),*) => {
        // SAFETY: Members uphold access promises; declarations compose with conflict propagation.
        unsafe impl<$($name),*> QueryDataAccess for ($($name,)*)
        where
            $($name: QueryDataAccess,)*
        {
            fn update_access(
                world: &mut World,
                access: &mut QueryAccess,
            ) -> Result<(), QueryAccessError> {
                $($name::update_access(world, access)?;)*
                Ok(())
            }

            fn matches_data(world: &World, entity: EntityId) -> bool {
                true $(&& $name::matches_data(world, entity))*
            }

            #[allow(non_snake_case)]
            fn matches_component_locations(
                world: &World,
                entity: EntityId,
                component_locations: &[ComponentStorageLocation],
            ) -> bool {
                true $(&& $name::matches_component_locations(world, entity, component_locations))*
            }
        }

        impl<$($name),*> QueryData for ($($name,)*)
        where
            $($name: QueryData,)*
        {
            type Item<'world> = ($($name::Item<'world>,)*);

            #[allow(non_snake_case)]
            unsafe fn fetch<'world>(world: *const World, entity: EntityId) -> Option<Self::Item<'world>> {
                unsafe {
                    Some(($($name::fetch(world, entity)?,)*))

                }
            }

            #[allow(non_snake_case)]
            unsafe fn fetch_with_ticks<'world>(
                world: *const World,
                entity: EntityId,
                ticks: crate::scene::ecs::ChangeTickWindow,
            ) -> Option<Self::Item<'world>> {
                unsafe {
                    Some(($($name::fetch_with_ticks(world, entity, ticks)?,)*))

                }
            }

            #[allow(non_snake_case)]
            unsafe fn fetch_with_component_locations<'world>(
                world: *const World,
                entity: EntityId,
                stable_location: StableEntityLocation,
                component_locations: &[ComponentStorageLocation],
                ticks: crate::scene::ecs::ChangeTickWindow,
            ) -> Option<Self::Item<'world>> {
                unsafe {
                    Some(($($name::fetch_with_component_locations(
                        world,
                        entity,
                        stable_location,
                        component_locations,
                        ticks,
                    )?,)*))

                }
            }
        }
    };
}

tuple_query_data!(A);
tuple_query_data!(A, B);
tuple_query_data!(A, B, C);
tuple_query_data!(A, B, C, D);
tuple_query_data!(A, B, C, D, E);
tuple_query_data!(A, B, C, D, E, F);
tuple_query_data!(A, B, C, D, E, F, G);
tuple_query_data!(A, B, C, D, E, F, G, H);

fn component_location<'locations, T>(
    component_locations: &'locations [ComponentStorageLocation],
) -> Option<&'locations ComponentStorageLocation>
where
    T: Component,
{
    let rust_type_id = TypeId::of::<T>();
    component_locations
        .iter()
        .find(|location| location.rust_type_id == Some(rust_type_id))
}
