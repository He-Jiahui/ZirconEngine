use std::fmt;
use std::marker::PhantomData;
use std::sync::Arc;

use crate::plugin::RuntimeExtensionRegistryError;
use crate::scene::World;
use crate::scene::ecs::{
    BoxedSceneSystem, SceneSystem, SceneSystemMetadata, SceneSystemThreadAffinity, ScheduleError,
    SystemOrderingConstraint, SystemParam, SystemParamAccess, SystemParamError, SystemRef,
    SystemSetId, SystemStage, SystemState, WorkerCommandBuffer,
};

use super::super::RuntimeExtensionRegistry;
use super::super::owner::PluginModuleId;

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
    S: for<'world> FnMut(P::Item<'world>) + Send + Sync + Clone + 'static,
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
        let system_template = self.system;
        let metadata = SceneSystemMetadata::new(id.clone(), stage, order)
            .with_sets(sets.clone())
            .with_constraints(constraints.clone());
        let build_id = id.clone();
        let build = SharedSystemBuild::new(
            build_id.clone(),
            Arc::new(move |world| {
                let system = CallbackSceneSystem::<P, S>::new(
                    metadata.clone(),
                    world,
                    system_template.clone(),
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

type ExternalAccessBuildFn =
    Arc<dyn Fn(&mut World) -> Result<SystemParamAccess, String> + Send + Sync>;

const DEFAULT_WORKER_COMMAND_BUFFER_CAPACITY: usize = 32;

pub(crate) struct ExternalSystemRegistrationBuilder<'registry, S> {
    registry: &'registry mut RuntimeExtensionRegistry,
    owner: PluginModuleId,
    id: String,
    stage: SystemStage,
    affinity: SceneSystemThreadAffinity,
    access_build: ExternalAccessBuildFn,
    system: S,
    sets: Vec<SystemSetId>,
    constraints: Vec<SystemOrderingConstraint>,
    order: i32,
}

impl<'registry, S> ExternalSystemRegistrationBuilder<'registry, S>
where
    S: FnMut() + Send + Sync + Clone + 'static,
{
    fn new(
        registry: &'registry mut RuntimeExtensionRegistry,
        owner: PluginModuleId,
        id: impl Into<String>,
        stage: SystemStage,
        affinity: SceneSystemThreadAffinity,
        access_build: ExternalAccessBuildFn,
        system: S,
    ) -> Self {
        Self {
            registry,
            owner,
            id: id.into(),
            stage,
            affinity,
            access_build,
            system,
            sets: Vec::new(),
            constraints: Vec::new(),
            order: 0,
        }
    }

    pub(crate) fn in_set(mut self, set: SystemSetId) -> Self {
        self.sets.push(set);
        self
    }

    pub(crate) fn with_order(mut self, order: i32) -> Self {
        self.order = order;
        self
    }

    pub(crate) fn before(mut self, reference: SystemRef) -> Self {
        self.constraints
            .push(SystemOrderingConstraint::Before(reference));
        self
    }

    pub(crate) fn after(mut self, reference: SystemRef) -> Self {
        self.constraints
            .push(SystemOrderingConstraint::After(reference));
        self
    }

    pub(crate) fn register(self) -> Result<(), RuntimeExtensionRegistryError> {
        let id = self.id;
        let stage = self.stage;
        let order = self.order;
        let sets = self.sets;
        let constraints = self.constraints;
        let system_template = self.system;
        let metadata = SceneSystemMetadata::new(id.clone(), stage, order)
            .with_sets(sets.clone())
            .with_constraints(constraints.clone())
            .with_thread_affinity(self.affinity);
        let build_id = id.clone();
        let access_build = self.access_build;
        let build = SharedSystemBuild::new(
            build_id.clone(),
            Arc::new(move |world| {
                let access =
                    access_build(world).map_err(|message| ScheduleError::ExternalAccess {
                        system_id: build_id.clone(),
                        message,
                    })?;
                Ok(Box::new(ExternalCallbackSceneSystem::new(
                    metadata.clone(),
                    access,
                    system_template.clone(),
                )))
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

pub(crate) struct ExternalCommandSystemRegistrationBuilder<'registry, S> {
    registry: &'registry mut RuntimeExtensionRegistry,
    owner: PluginModuleId,
    id: String,
    stage: SystemStage,
    affinity: SceneSystemThreadAffinity,
    access_build: ExternalAccessBuildFn,
    system: S,
    sets: Vec<SystemSetId>,
    constraints: Vec<SystemOrderingConstraint>,
    order: i32,
    command_capacity: usize,
}

impl<'registry, S> ExternalCommandSystemRegistrationBuilder<'registry, S>
where
    S: FnMut(&mut WorkerCommandBuffer) + Send + Sync + Clone + 'static,
{
    fn new(
        registry: &'registry mut RuntimeExtensionRegistry,
        owner: PluginModuleId,
        id: impl Into<String>,
        stage: SystemStage,
        affinity: SceneSystemThreadAffinity,
        access_build: ExternalAccessBuildFn,
        system: S,
    ) -> Self {
        Self {
            registry,
            owner,
            id: id.into(),
            stage,
            affinity,
            access_build,
            system,
            sets: Vec::new(),
            constraints: Vec::new(),
            order: 0,
            command_capacity: DEFAULT_WORKER_COMMAND_BUFFER_CAPACITY,
        }
    }

    pub(crate) fn in_set(mut self, set: SystemSetId) -> Self {
        self.sets.push(set);
        self
    }

    pub(crate) fn with_order(mut self, order: i32) -> Self {
        self.order = order;
        self
    }

    pub(crate) fn before(mut self, reference: SystemRef) -> Self {
        self.constraints
            .push(SystemOrderingConstraint::Before(reference));
        self
    }

    pub(crate) fn after(mut self, reference: SystemRef) -> Self {
        self.constraints
            .push(SystemOrderingConstraint::After(reference));
        self
    }

    pub(crate) fn with_command_capacity(mut self, command_capacity: usize) -> Self {
        self.command_capacity = command_capacity;
        self
    }

    pub(crate) fn register(self) -> Result<(), RuntimeExtensionRegistryError> {
        let id = self.id;
        let stage = self.stage;
        let order = self.order;
        let sets = self.sets;
        let constraints = self.constraints;
        let command_capacity = self.command_capacity;
        let system_template = self.system;
        let metadata = SceneSystemMetadata::new(id.clone(), stage, order)
            .with_sets(sets.clone())
            .with_constraints(constraints.clone())
            .with_thread_affinity(self.affinity);
        let build_id = id.clone();
        let access_build = self.access_build;
        let build = SharedSystemBuild::new(
            build_id.clone(),
            Arc::new(move |world| {
                let mut access =
                    access_build(world).map_err(|message| ScheduleError::ExternalAccess {
                        system_id: build_id.clone(),
                        message,
                    })?;
                access.add_deferred_commands();
                Ok(Box::new(ExternalCommandCallbackSceneSystem::new(
                    metadata.clone(),
                    access,
                    command_capacity,
                    system_template.clone(),
                )))
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
        S: for<'world> FnMut(P::Item<'world>) + Send + Sync + Clone + 'static,
    {
        SystemRegistrationBuilder::new(self, owner, id, stage, system)
    }

    pub(crate) fn register_external_native_system<S>(
        &mut self,
        owner: PluginModuleId,
        id: impl Into<String>,
        stage: SystemStage,
        affinity: SceneSystemThreadAffinity,
        access_build: impl Fn(&mut World) -> Result<SystemParamAccess, String> + Send + Sync + 'static,
        system: S,
    ) -> ExternalSystemRegistrationBuilder<'_, S>
    where
        S: FnMut() + Send + Sync + Clone + 'static,
    {
        ExternalSystemRegistrationBuilder::new(
            self,
            owner,
            id,
            stage,
            affinity,
            Arc::new(access_build),
            system,
        )
    }

    pub(crate) fn register_external_native_command_system<S>(
        &mut self,
        owner: PluginModuleId,
        id: impl Into<String>,
        stage: SystemStage,
        affinity: SceneSystemThreadAffinity,
        access_build: impl Fn(&mut World) -> Result<SystemParamAccess, String> + Send + Sync + 'static,
        system: S,
    ) -> ExternalCommandSystemRegistrationBuilder<'_, S>
    where
        S: FnMut(&mut WorkerCommandBuffer) + Send + Sync + Clone + 'static,
    {
        ExternalCommandSystemRegistrationBuilder::new(
            self,
            owner,
            id,
            stage,
            affinity,
            Arc::new(access_build),
            system,
        )
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
            .iter()
            .map(|(owner, _key, registration)| (owner, registration))
    }
}

struct CallbackSceneSystem<P, S>
where
    P: SystemParam,
{
    metadata: SceneSystemMetadata,
    state: SystemState<P>,
    system: S,
    _marker: PhantomData<fn() -> P>,
}

impl<P, S> CallbackSceneSystem<P, S>
where
    P: SystemParam,
    S: for<'world> FnMut(P::Item<'world>) + Send + 'static,
{
    fn new(
        metadata: SceneSystemMetadata,
        world: &mut World,
        system: S,
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

impl<P, S> SceneSystem for CallbackSceneSystem<P, S>
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
            (self.system)(params);
        });
    }
}

struct ExternalCallbackSceneSystem<S> {
    metadata: SceneSystemMetadata,
    access: SystemParamAccess,
    system: S,
}

impl<S> ExternalCallbackSceneSystem<S>
where
    S: FnMut() + Send + 'static,
{
    fn new(metadata: SceneSystemMetadata, access: SystemParamAccess, system: S) -> Self {
        Self {
            metadata,
            access,
            system,
        }
    }

    fn run_callback(&mut self) {
        (self.system)();
    }
}

impl<S> SceneSystem for ExternalCallbackSceneSystem<S>
where
    S: FnMut() + Send + 'static,
{
    fn metadata(&self) -> &SceneSystemMetadata {
        &self.metadata
    }

    fn access(&self) -> &SystemParamAccess {
        &self.access
    }

    fn run(&mut self, _world: &mut World) {
        self.run_callback();
    }

    fn run_without_world(&mut self) {
        self.run_callback();
    }

    fn supports_worldless_execution(&self) -> bool {
        true
    }
}

struct ExternalCommandCallbackSceneSystem<S> {
    metadata: SceneSystemMetadata,
    access: SystemParamAccess,
    command_buffer: WorkerCommandBuffer,
    system: S,
}

impl<S> ExternalCommandCallbackSceneSystem<S>
where
    S: FnMut(&mut WorkerCommandBuffer) + Send + 'static,
{
    fn new(
        metadata: SceneSystemMetadata,
        access: SystemParamAccess,
        command_capacity: usize,
        system: S,
    ) -> Self {
        let command_buffer =
            WorkerCommandBuffer::with_capacity(metadata.order(), metadata.id(), command_capacity);
        Self {
            metadata,
            access,
            command_buffer,
            system,
        }
    }

    fn run_callback(&mut self) {
        (self.system)(&mut self.command_buffer);
    }
}

impl<S> SceneSystem for ExternalCommandCallbackSceneSystem<S>
where
    S: FnMut(&mut WorkerCommandBuffer) + Send + 'static,
{
    fn metadata(&self) -> &SceneSystemMetadata {
        &self.metadata
    }

    fn access(&self) -> &SystemParamAccess {
        &self.access
    }

    fn run(&mut self, world: &mut World) {
        self.run_callback();
        world.merge_worker_command_buffer(&mut self.command_buffer);
    }

    fn run_without_world(&mut self) {
        self.run_callback();
    }

    fn supports_worldless_execution(&self) -> bool {
        true
    }

    fn worker_command_buffer_mut(&mut self) -> Option<&mut WorkerCommandBuffer> {
        Some(&mut self.command_buffer)
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

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Condvar, Mutex};
    use std::thread;
    use std::time::Duration;

    use super::*;

    #[test]
    fn typed_scene_system_callback_state_is_private_per_world() {
        let observed = Arc::new(Mutex::new(Vec::new()));
        let mut registry = RuntimeExtensionRegistry::default();
        let owner = registry.intern_plugin_module("tests.typed").unwrap();
        let observed_for_system = Arc::clone(&observed);
        let mut calls = 0usize;

        registry
            .register_native_system::<(), _>(
                owner,
                "tests.typed.private-state",
                SystemStage::Update,
                move |_| {
                    calls += 1;
                    observed_for_system.lock().unwrap().push(calls);
                },
            )
            .register()
            .unwrap();

        let registration = registry.plugin_systems().next().unwrap().1;
        let mut first_world = World::empty();
        let mut second_world = World::empty();
        let mut first = registration.build(&mut first_world).unwrap();
        let mut second = registration.build(&mut second_world).unwrap();

        first.run(&mut first_world);
        second.run(&mut second_world);

        assert_eq!(*observed.lock().unwrap(), vec![1, 1]);
    }

    #[test]
    fn external_scene_system_callback_state_is_private_per_world() {
        let observed = Arc::new(Mutex::new(Vec::new()));
        let mut registry = RuntimeExtensionRegistry::default();
        let owner = registry.intern_plugin_module("tests.external").unwrap();
        let observed_for_system = Arc::clone(&observed);
        let mut calls = 0usize;

        registry
            .register_external_native_system(
                owner,
                "tests.external.private-state",
                SystemStage::Update,
                SceneSystemThreadAffinity::WorkerSafe,
                |_world| Ok(SystemParamAccess::default()),
                move || {
                    calls += 1;
                    observed_for_system.lock().unwrap().push(calls);
                },
            )
            .register()
            .unwrap();

        let registration = registry.plugin_systems().next().unwrap().1;
        let mut first_world = World::empty();
        let mut second_world = World::empty();
        let mut first = registration.build(&mut first_world).unwrap();
        let mut second = registration.build(&mut second_world).unwrap();

        first.run_without_world();
        second.run_without_world();

        assert_eq!(*observed.lock().unwrap(), vec![1, 1]);
    }

    #[test]
    fn external_scene_system_callbacks_overlap_across_worlds() {
        #[derive(Default)]
        struct CallbackProgress {
            active: usize,
            max_active: usize,
            both_started: bool,
        }

        let progress = Arc::new((Mutex::new(CallbackProgress::default()), Condvar::new()));
        let mut registry = RuntimeExtensionRegistry::default();
        let owner = registry
            .intern_plugin_module("tests.external.concurrent")
            .unwrap();
        let progress_for_system = Arc::clone(&progress);

        registry
            .register_external_native_system(
                owner,
                "tests.external.concurrent-worlds",
                SystemStage::Update,
                SceneSystemThreadAffinity::WorkerSafe,
                |_world| Ok(SystemParamAccess::default()),
                move || {
                    let (progress_lock, progress_changed) = &*progress_for_system;
                    let mut progress = progress_lock.lock().unwrap();
                    progress.active += 1;
                    progress.max_active = progress.max_active.max(progress.active);
                    if progress.active == 2 {
                        progress.both_started = true;
                        progress_changed.notify_all();
                    }
                    let (mut progress, _) = progress_changed
                        .wait_timeout_while(progress, Duration::from_secs(1), |progress| {
                            !progress.both_started
                        })
                        .unwrap();
                    progress.active -= 1;
                    progress_changed.notify_all();
                },
            )
            .register()
            .unwrap();

        let registration = registry.plugin_systems().next().unwrap().1;
        let mut first_world = World::empty();
        let mut second_world = World::empty();
        let mut first = registration.build(&mut first_world).unwrap();
        let mut second = registration.build(&mut second_world).unwrap();

        let first = thread::spawn(move || first.run_without_world());
        let second = thread::spawn(move || second.run_without_world());
        first.join().unwrap();
        second.join().unwrap();

        assert_eq!(progress.0.lock().unwrap().max_active, 2);
    }
}
