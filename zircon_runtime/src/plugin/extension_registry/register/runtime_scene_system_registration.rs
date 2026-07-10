use std::fmt;
use std::sync::{Arc, Mutex};

use crate::core::CoreError;
use crate::plugin::RuntimeExtensionRegistryError;
use crate::scene::ecs::{
    BoxedRuntimeSceneSystem, FunctionRuntimeSceneSystem, RuntimeSceneSystemContext,
    SceneSystemMetadata, SystemOrderingConstraint, SystemRef, SystemSetId, SystemStage,
};

use super::super::owner::PluginModuleId;
use super::super::RuntimeExtensionRegistry;
use super::system_registration::validate_plugin_system_id;

type RuntimeSceneSystemBuildFn = Arc<dyn Fn() -> BoxedRuntimeSceneSystem + Send + Sync>;

#[derive(Clone)]
struct SharedRuntimeSceneSystemBuild {
    system_id: String,
    inner: RuntimeSceneSystemBuildFn,
}

impl SharedRuntimeSceneSystemBuild {
    fn new(system_id: impl Into<String>, build: RuntimeSceneSystemBuildFn) -> Self {
        Self {
            system_id: system_id.into(),
            inner: build,
        }
    }

    fn build(&self) -> BoxedRuntimeSceneSystem {
        (self.inner)()
    }
}

impl fmt::Debug for SharedRuntimeSceneSystemBuild {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SharedRuntimeSceneSystemBuild")
            .field("system_id", &self.system_id)
            .finish_non_exhaustive()
    }
}

#[derive(Clone, Debug)]
pub struct RuntimeSceneSystemRegistration {
    pub id: String,
    pub stage: SystemStage,
    pub sets: Vec<SystemSetId>,
    pub constraints: Vec<SystemOrderingConstraint>,
    pub order: i32,
    build: SharedRuntimeSceneSystemBuild,
}

impl RuntimeSceneSystemRegistration {
    pub fn build(&self) -> BoxedRuntimeSceneSystem {
        self.build.build()
    }
}

pub struct RuntimeSceneSystemRegistrationBuilder<'registry, S> {
    registry: &'registry mut RuntimeExtensionRegistry,
    owner: PluginModuleId,
    id: String,
    stage: SystemStage,
    system: S,
    sets: Vec<SystemSetId>,
    constraints: Vec<SystemOrderingConstraint>,
    order: i32,
}

impl<'registry, S> RuntimeSceneSystemRegistrationBuilder<'registry, S>
where
    S: FnMut(RuntimeSceneSystemContext<'_>) -> Result<(), CoreError> + Send + 'static,
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
        let build = SharedRuntimeSceneSystemBuild::new(
            id.clone(),
            Arc::new(move || {
                let shared_system = Arc::clone(&shared_system);
                let system: BoxedRuntimeSceneSystem = Box::new(FunctionRuntimeSceneSystem::new(
                    metadata.clone(),
                    move |context| {
                        let mut system = shared_system
                            .lock()
                            .expect("runtime scene system callback lock was poisoned");
                        (*system)(context)
                    },
                ));
                system
            }),
        );
        self.registry.register_runtime_scene_system_registration(
            self.owner,
            RuntimeSceneSystemRegistration {
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
    pub fn register_runtime_scene_system<S>(
        &mut self,
        owner: PluginModuleId,
        id: impl Into<String>,
        stage: SystemStage,
        system: S,
    ) -> RuntimeSceneSystemRegistrationBuilder<'_, S>
    where
        S: FnMut(RuntimeSceneSystemContext<'_>) -> Result<(), CoreError> + Send + 'static,
    {
        RuntimeSceneSystemRegistrationBuilder::new(self, owner, id, stage, system)
    }

    pub(crate) fn register_runtime_scene_system_registration(
        &mut self,
        owner: PluginModuleId,
        registration: RuntimeSceneSystemRegistration,
    ) -> Result<(), RuntimeExtensionRegistryError> {
        validate_plugin_system_id(&registration.id)?;
        if self.plugin_runtime_systems.contains_key(&registration.id)
            || self.plugin_systems.contains_key(&registration.id)
        {
            return Err(RuntimeExtensionRegistryError::DuplicatePluginSystem(
                registration.id,
            ));
        }
        self.plugin_runtime_systems
            .register(owner, registration.id.clone(), registration)
            .expect("runtime plugin system duplicate was prechecked");
        Ok(())
    }

    pub fn plugin_runtime_systems(
        &self,
    ) -> impl Iterator<Item = (PluginModuleId, &RuntimeSceneSystemRegistration)> {
        self.plugin_runtime_systems
            .iter()
            .map(|(owner, _key, registration)| (owner, registration))
    }
}
