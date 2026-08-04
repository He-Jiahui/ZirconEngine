use std::sync::Arc;

use zircon_runtime::scene::{LevelSystem, Scene};

use crate::core::gateway::{
    DetachedEditorRuntimeGateway, EditorRuntimeGatewayHandle, GatewayError, InProcessGateway,
    SharedEditorRuntimeGateway,
};

/// Core-owned construction input for an editor authoring world.
///
/// The runtime type never crosses into the workbench state. Once installed, every editor-side
/// borrowed-world access goes through the stable gateway handle carried by this facade.
pub(crate) struct AuthoringWorldSeed(LevelSystem);

impl From<LevelSystem> for AuthoringWorldSeed {
    fn from(level: LevelSystem) -> Self {
        Self(level)
    }
}

/// UI-safe view of the edit domain's stable authoring gateway.
pub(crate) struct EditorAuthoringWorld {
    gateway: EditorRuntimeGatewayHandle,
    loaded: bool,
}

impl EditorAuthoringWorld {
    pub(crate) fn loaded(
        gateway: &EditorRuntimeGatewayHandle,
        seed: impl Into<AuthoringWorldSeed>,
    ) -> Result<Self, GatewayError> {
        let mut facade = Self {
            gateway: gateway.clone(),
            loaded: false,
        };
        facade.replace(seed)?;
        Ok(facade)
    }

    pub(crate) fn unloaded(gateway: &EditorRuntimeGatewayHandle) -> Result<Self, GatewayError> {
        let mut facade = Self {
            gateway: gateway.clone(),
            loaded: false,
        };
        facade.clear()?;
        Ok(facade)
    }

    pub(crate) fn is_loaded(&self) -> bool {
        self.loaded
    }

    pub(crate) fn try_snapshot(&self) -> Option<Scene> {
        self.try_with_world(Clone::clone)
    }

    pub(crate) fn try_with_world<R>(&self, read: impl FnOnce(&Scene) -> R) -> Option<R> {
        if !self.loaded {
            return None;
        }
        let mut read = Some(read);
        let mut result = None;
        let mut duplicate_callback = false;
        self.gateway
            .with_world(&mut |scene| {
                let Some(read) = read.take() else {
                    duplicate_callback = true;
                    return;
                };
                result = Some(read(scene));
            })
            .ok()?;
        (!duplicate_callback).then_some(result).flatten()
    }

    pub(crate) fn try_with_world_mut<R>(&self, write: impl FnOnce(&mut Scene) -> R) -> Option<R> {
        if !self.loaded {
            return None;
        }
        let mut write = Some(write);
        let mut result = None;
        let mut duplicate_callback = false;
        self.gateway
            .with_world_mut(&mut |scene| {
                let Some(write) = write.take() else {
                    duplicate_callback = true;
                    return;
                };
                result = Some(write(scene));
            })
            .ok()?;
        (!duplicate_callback).then_some(result).flatten()
    }

    #[cfg(test)]
    pub(crate) fn snapshot(&self) -> Scene {
        self.try_snapshot().expect("editor world is not loaded")
    }

    #[cfg(test)]
    pub(crate) fn with_world<R>(&self, read: impl FnOnce(&Scene) -> R) -> R {
        self.try_with_world(read)
            .expect("editor world is not loaded")
    }

    #[cfg(test)]
    pub(crate) fn with_world_mut<R>(&self, write: impl FnOnce(&mut Scene) -> R) -> R {
        self.try_with_world_mut(write)
            .expect("editor world is not loaded")
    }

    pub(crate) fn replace(
        &mut self,
        seed: impl Into<AuthoringWorldSeed>,
    ) -> Result<(), GatewayError> {
        let AuthoringWorldSeed(level) = seed.into();
        let gateway: SharedEditorRuntimeGateway =
            Arc::new(InProcessGateway::for_authoring_level(level));
        self.gateway.replace(gateway)?;
        self.loaded = true;
        Ok(())
    }

    pub(crate) fn clear(&mut self) -> Result<(), GatewayError> {
        self.gateway
            .replace(Arc::new(DetachedEditorRuntimeGateway))?;
        self.loaded = false;
        Ok(())
    }
}
