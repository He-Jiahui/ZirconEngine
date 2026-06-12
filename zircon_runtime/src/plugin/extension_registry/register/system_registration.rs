use std::fmt;
use std::marker::PhantomData;
use std::sync::{Arc, Mutex};

use crate::plugin::RuntimeExtensionRegistryError;
use crate::scene::ecs::{
    BoxedSceneSystem, SceneSystem, SceneSystemMetadata, ScheduleError, SystemOrderingConstraint,
    SystemParam, SystemParamAccess, SystemParamError, SystemRef, SystemSetId, SystemStage,
    SystemState,
};
use crate::scene::World;

use super::super::owner::PluginModuleId;
use super::super::typed_extension_point::ExtensionSlot;
use super::super::RuntimeExtensionRegistry;

type SystemBuildFn =
    Arc<dyn Fn(&mut World) -> Result<BoxedSceneSystem, ScheduleError> + Send + Sync>;

#[derive(Clone)]
struct SharedSystemBuild {
    system_id: String,
    inner: SystemBuildFn,
}

impl SharedSystemBuild {
    fn new(system_id: impl Into<String>, build: SystemBuildFn) -> Self {
        Self {
            system_id: system_id.into(),
            inner: build,
        }
    }

    fn build(&self, world: &mut World) -> Result<BoxedSceneSystem, ScheduleError> {
        (self.inner)(world)
    }
}

impl fmt::Debug for SharedSystemBuild {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SharedSystemBuild")
            .field("system_id", &self.system_id)
            .finish_non_exhaustive()
    }
}

#[derive(Clone, Debug)]
pub struct SystemRegistration {
    pub id: String,
    pub stage: SystemStage,
    pub sets: Vec<SystemSetId>,
    pub constraints: Vec<SystemOrderingConstraint>,
    pub order: i32,
    build: SharedSystemBuild,
}

impl SystemRegistration {
    pub fn build(&self, world: &mut World) -> Result<BoxedSceneSystem, ScheduleError> {
        self.build.build(world)
    }
}

pub struct SystemRegistrationBuilder<'registry, P, S>
where
    P: SystemParam,
{
    registry: &'registry mut RuntimeExtensionRegistry,
    owner: PluginModuleId,
    id: String,
    stage: SystemStage,
    system: S,
    sets: Vec<SystemSetId>,
    constraints: Vec<SystemOrderingConstraint>,
    order: i32,
    _marker: PhantomData<fn() -> P>,
}

impl<'registry, P, S> SystemRegistrationBuilder<'registry, P, S>
where
    P: SystemParam + 'static,
    P::State: Send,
    S: for<'world> FnMut(P::Item<'world>) + Send + 'static,
{
    pub(super) fn new(
        registry: &'registry mut RuntimeExtensionRegistry,
        owner: PluginModuleId,
        id: impl Into<String>,
        stage: SystemStage,
        system: S,
    ) -> Self {
        Self {
            registry,
            owner,
            id: id.into(),
            stage,
            system,
            sets: Vec::new(),
            constraints: Vec::new(),
            order: 0,
            _marker: PhantomData,
        }
    }

    pub fn in_set(mut self, set: SystemSetId) -> Self {
        self.sets.push(set);
        self
    }

    pub fn with_order(mut self, order: i32) -> Self {
        self.order = order;
        self
    }

    pub fn before(mut self, reference: SystemRef) -> Self {
        self.constraints
            .push(SystemOrderingConstraint::Before(reference));
        self
    }

    pub fn after(mut self, reference: SystemRef) -> Self {
        self.constraints
            .push(SystemOrderingConstraint::After(reference));
        self
    }

    pub fn register(self) -> Result<(), RuntimeExtensionRegistryError> {
        let id = self.id;
        let stage = self.stage;
        let order = self.order;
        let sets = self.sets;
        let constraints = self.constraints;
        let shared_system = Arc::new(Mutex::new(self.system));
        let metadata = SceneSystemMetadata::new(id.clone(), stage, order)
            .with_sets(sets.clone())
            .with_constraints(constraints.clone());
        let build_id = id.clone();
        let build = SharedSystemBuild::new(
            build_id.clone(),
            Arc::new(move |world| {
                let system = SharedCallbackSceneSystem::<P, S>::new(
                    metadata.clone(),
                    world,
                    Arc::clone(&shared_system),
                )
                .map_err(|source| ScheduleError::SystemParam {
                    system_id: build_id.clone(),
                    source,
                })?;
                Ok(Box::new(system))
            }),
        );
        self.registry.register_system(
            self.owner,
            SystemRegistration {
                id,
                stage,
                sets,
                constraints,
                order,
                build,
            },
        )
    }
}

impl RuntimeExtensionRegistry {
    pub fn register_native_system<P, S>(
        &mut self,
        owner: PluginModuleId,
        id: impl Into<String>,
        stage: SystemStage,
        system: S,
    ) -> SystemRegistrationBuilder<'_, P, S>
    where
        P: SystemParam + 'static,
        P::State: Send,
        S: for<'world> FnMut(P::Item<'world>) + Send + 'static,
    {
        SystemRegistrationBuilder::new(self, owner, id, stage, system)
    }

    pub fn register_system(
        &mut self,
        owner: PluginModuleId,
        registration: SystemRegistration,
    ) -> Result<(), RuntimeExtensionRegistryError> {
        validate_plugin_system_id(&registration.id)?;
        if self.plugin_systems.contains_key(&registration.id)
            || self.plugin_runtime_systems.contains_key(&registration.id)
        {
            return Err(RuntimeExtensionRegistryError::DuplicatePluginSystem(
                registration.id,
            ));
        }
        self.plugin_systems
            .register(owner, registration.id.clone(), registration)
            .expect("plugin system duplicate was prechecked");
        Ok(())
    }

    pub(crate) fn register_system_registration(
        &mut self,
        owner: PluginModuleId,
        registration: SystemRegistration,
    ) -> Result<(), RuntimeExtensionRegistryError> {
        self.register_system(owner, registration)
    }

    pub fn plugin_systems(&self) -> impl Iterator<Item = (PluginModuleId, &SystemRegistration)> {
        self.plugin_systems
            .values()
            .iter()
            .enumerate()
            .filter_map(|(index, registration)| {
                let slot = ExtensionSlot::from_raw(index as u32);
                self.plugin_systems
                    .owner_for_slot(slot)
                    .map(|owner| (owner, registration))
            })
    }
}

struct SharedCallbackSceneSystem<P, S>
where
    P: SystemParam,
{
    metadata: SceneSystemMetadata,
    state: SystemState<P>,
    system: Arc<Mutex<S>>,
    _marker: PhantomData<fn() -> P>,
}

impl<P, S> SharedCallbackSceneSystem<P, S>
where
    P: SystemParam,
    S: for<'world> FnMut(P::Item<'world>) + Send + 'static,
{
    fn new(
        metadata: SceneSystemMetadata,
        world: &mut World,
        system: Arc<Mutex<S>>,
    ) -> Result<Self, SystemParamError> {
        let state = SystemState::<P>::new(world)?;
        Ok(Self {
            metadata,
            state,
            system,
            _marker: PhantomData,
        })
    }
}

impl<P, S> SceneSystem for SharedCallbackSceneSystem<P, S>
where
    P: SystemParam + 'static,
    P::State: Send,
    S: for<'world> FnMut(P::Item<'world>) + Send + 'static,
{
    fn metadata(&self) -> &SceneSystemMetadata {
        &self.metadata
    }

    fn access(&self) -> &SystemParamAccess {
        self.state.access()
    }

    fn run(&mut self, world: &mut World) {
        self.state.run(world, |params| {
            let mut system = self
                .system
                .lock()
                .expect("native plugin scene system callback lock was poisoned");
            (*system)(params);
        });
    }
}

pub(super) fn validate_plugin_system_id(id: &str) -> Result<(), RuntimeExtensionRegistryError> {
    if id.trim().is_empty() || id.trim() != id || !id.contains('.') {
        return Err(RuntimeExtensionRegistryError::InvalidPluginSystem(
            id.to_string(),
        ));
    }
    Ok(())
}
