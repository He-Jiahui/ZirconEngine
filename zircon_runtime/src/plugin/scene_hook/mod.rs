use std::fmt;
use std::sync::Arc;

use crate::core::math::Real;
use crate::core::{CoreError, CoreHandle};
use crate::scene::components::SystemStage;
use crate::scene::LevelSystem;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SceneRuntimeHookDescriptor {
    pub id: String,
    pub plugin_id: String,
    pub stage: SystemStage,
    pub order: i32,
}

impl SceneRuntimeHookDescriptor {
    pub fn new(id: impl Into<String>, plugin_id: impl Into<String>, stage: SystemStage) -> Self {
        Self {
            id: id.into(),
            plugin_id: plugin_id.into(),
            stage,
            order: 0,
        }
    }

    pub fn with_order(mut self, order: i32) -> Self {
        self.order = order;
        self
    }
}

pub struct SceneRuntimeHookContext<'a> {
    pub core: &'a CoreHandle,
    pub level: &'a LevelSystem,
    pub delta_seconds: Real,
}

impl<'a> SceneRuntimeHookContext<'a> {
    pub fn new(core: &'a CoreHandle, level: &'a LevelSystem, delta_seconds: Real) -> Self {
        Self {
            core,
            level,
            delta_seconds,
        }
    }
}

pub trait SceneRuntimeHook: Send + Sync {
    fn run(&self, context: SceneRuntimeHookContext<'_>) -> Result<(), CoreError>;
}

#[derive(Clone)]
pub struct SceneRuntimeHookRegistration {
    descriptor: SceneRuntimeHookDescriptor,
    hook: Arc<dyn SceneRuntimeHook>,
}

impl SceneRuntimeHookRegistration {
    pub fn new(
        descriptor: SceneRuntimeHookDescriptor,
        hook: impl SceneRuntimeHook + 'static,
    ) -> Self {
        Self {
            descriptor,
            hook: Arc::new(hook),
        }
    }

    pub fn from_arc(
        descriptor: SceneRuntimeHookDescriptor,
        hook: Arc<dyn SceneRuntimeHook>,
    ) -> Self {
        Self { descriptor, hook }
    }

    pub fn descriptor(&self) -> &SceneRuntimeHookDescriptor {
        &self.descriptor
    }

    pub fn hook(&self) -> Arc<dyn SceneRuntimeHook> {
        self.hook.clone()
    }

    pub fn run(&self, context: SceneRuntimeHookContext<'_>) -> Result<(), CoreError> {
        self.hook.run(context)
    }
}

impl fmt::Debug for SceneRuntimeHookRegistration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SceneRuntimeHookRegistration")
            .field("descriptor", &self.descriptor)
            .finish_non_exhaustive()
    }
}
