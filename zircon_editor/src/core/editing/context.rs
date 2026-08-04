use std::any::Any;
#[cfg(test)]
use std::sync::Arc;

use super::engine::{EditCommandError, EditContext, SelectionSnapshot};
use super::selection::SceneSelection;
use crate::core::gateway::{EditorRuntimeGatewayHandle, GatewayError};
#[cfg(test)]
use crate::core::gateway::{InProcessGateway, SharedEditorRuntimeGateway};
#[cfg(test)]
use zircon_runtime::scene::LevelSystem;
use zircon_runtime::scene::Scene;

/// Headless-safe owner for the core edit state used by the transaction engine.
pub(crate) struct CoreEditContext {
    selection: SelectionSnapshot,
    selection_generation: u64,
    gateway: EditorRuntimeGatewayHandle,
}

impl CoreEditContext {
    pub(crate) fn new(gateway: EditorRuntimeGatewayHandle) -> Self {
        Self {
            selection: SelectionSnapshot::default(),
            selection_generation: 0,
            gateway,
        }
    }

    #[cfg(test)]
    pub(crate) fn bind_scene(
        &mut self,
        scene: LevelSystem,
        selection: SceneSelection,
    ) -> Result<(), EditCommandError> {
        let gateway: SharedEditorRuntimeGateway =
            Arc::new(InProcessGateway::for_authoring_level(scene));
        self.gateway.replace(gateway).map_err(gateway_failure)?;
        self.bind_authoring_selection(selection)
    }

    pub(crate) fn bind_authoring_selection(
        &mut self,
        selection: SceneSelection,
    ) -> Result<(), EditCommandError> {
        self.set_scene_selection(selection)
    }

    pub(crate) fn clear_scene(&mut self) -> Result<(), EditCommandError> {
        let generation = self.next_selection_generation()?;
        self.selection = SelectionSnapshot::empty(generation);
        Ok(())
    }

    pub(crate) fn with_scene<R>(
        &self,
        read: impl FnOnce(&Scene) -> R,
    ) -> Result<R, EditCommandError> {
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
            .map_err(gateway_failure)?;
        if duplicate_callback {
            return Err(gateway_callback_protocol_failure());
        }
        result.ok_or_else(missing_scene)
    }

    pub(crate) fn with_scene_mut<R>(
        &self,
        write: impl FnOnce(&mut Scene) -> R,
    ) -> Result<R, EditCommandError> {
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
            .map_err(gateway_failure)?;
        if duplicate_callback {
            return Err(gateway_callback_protocol_failure());
        }
        result.ok_or_else(missing_scene)
    }

    pub(crate) fn scene_selection(&self) -> Result<SceneSelection, EditCommandError> {
        self.selection.scene_selection()
    }

    pub(crate) fn set_scene_selection(
        &mut self,
        selection: SceneSelection,
    ) -> Result<(), EditCommandError> {
        self.selection = SelectionSnapshot::scene(self.next_selection_generation()?, selection);
        Ok(())
    }

    fn next_selection_generation(&mut self) -> Result<u64, EditCommandError> {
        self.selection_generation = self
            .selection_generation
            .checked_add(1)
            .ok_or(EditCommandError::SelectionGenerationExhausted)?;
        Ok(self.selection_generation)
    }
}

fn missing_scene() -> EditCommandError {
    EditCommandError::TargetMissing {
        target: "active editor scene".to_string(),
    }
}

fn gateway_failure(error: GatewayError) -> EditCommandError {
    EditCommandError::ExternalEffect {
        source: Box::new(error),
    }
}

fn gateway_callback_protocol_failure() -> EditCommandError {
    gateway_failure(GatewayError::Protocol {
        message: "borrowed world callback was invoked more than once".to_owned(),
    })
}

impl Default for CoreEditContext {
    fn default() -> Self {
        Self::new(EditorRuntimeGatewayHandle::detached())
    }
}

impl EditContext for CoreEditContext {
    fn runtime_gateway(&self) -> &EditorRuntimeGatewayHandle {
        &self.gateway
    }

    fn selection_snapshot(&self) -> SelectionSnapshot {
        self.selection.clone()
    }

    fn restore_selection(&mut self, snapshot: &SelectionSnapshot) -> Result<(), EditCommandError> {
        self.selection = snapshot.clone();
        self.selection_generation = self.selection_generation.max(snapshot.generation());
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
