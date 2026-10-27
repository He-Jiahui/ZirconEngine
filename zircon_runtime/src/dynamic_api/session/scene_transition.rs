//! Consume gameplay travel after a successful tick or at the host-request drain boundary.

use zircon_runtime_interface::runtime_api::{
    ZrRuntimeProjectSceneTransitionPolicyV1, ZrRuntimeProjectSceneTransitionRequestV1,
    ZrRuntimeProjectSceneTransitionResultV1, ZrRuntimeProjectSceneTransitionStatusV1,
};

use crate::asset::{asset_manager_handle, AssetUri};
use crate::core::manager::resolve_manager_service;
use crate::scene::serializer::SceneAssetSerializer;
use crate::scene::{
    LevelMetadata, WorldDriver, WorldReplacementReason, WorldReplacementRequest, WORLD_DRIVER_NAME,
};

use super::{merge_builtin_script_scene_systems, RuntimeDynamicSession};

impl RuntimeDynamicSession {
    pub(super) fn consume_scene_transition(&mut self) {
        let Some(request) = self.level.with_world_mut(|world| {
            world.remove_resource::<ZrRuntimeProjectSceneTransitionRequestV1>()
        }) else {
            return;
        };

        // Direct resource insertion and Deserialize can bypass the producer's URI checks.
        let outcome = match ZrRuntimeProjectSceneTransitionRequestV1::try_new(
            request.request_id,
            request.scene_uri.clone(),
            request.policy,
        ) {
            Err(error) => (
                ZrRuntimeProjectSceneTransitionStatusV1::Rejected,
                Some(error.to_string()),
            ),
            Ok(_) => match request.policy {
                ZrRuntimeProjectSceneTransitionPolicyV1::ReplaceActive => {
                    match self.replace_active_scene(&request.scene_uri) {
                        Ok(()) => (ZrRuntimeProjectSceneTransitionStatusV1::Succeeded, None),
                        Err(error) => {
                            (ZrRuntimeProjectSceneTransitionStatusV1::Failed, Some(error))
                        }
                    }
                }
            },
        };
        // On success this is the destination World; on failure it is the unchanged source.
        self.level.with_world_mut(|world| {
            world.insert_resource(ZrRuntimeProjectSceneTransitionResultV1 {
                request_id: request.request_id,
                scene_uri: request.scene_uri,
                status: outcome.0,
                diagnostic: outcome.1,
            });
        });
    }

    fn replace_active_scene(&mut self, scene_uri: &str) -> Result<(), String> {
        let expected = self.level.capture_world_replacement_expectation();
        let core = self.runtime.handle();
        let uri = AssetUri::parse(scene_uri).map_err(|error| error.to_string())?;
        let project = {
            let manager = asset_manager_handle(&core)
                .and_then(|handle| resolve_manager_service(&core, handle))
                .map_err(|error| error.to_string())?;
            manager
                .current_project_snapshot()
                .ok_or_else(|| "scene transition requires an active project".to_string())?
        };

        // Reuse the same serializer and extension admission as initial level loading.
        // Stage before replacing so load/registration errors keep the live World intact.
        let mut world =
            SceneAssetSerializer::load_world(&project, &uri).map_err(|error| error.to_string())?;
        core.resolve_driver::<WorldDriver>(WORLD_DRIVER_NAME)
            .and_then(|driver| driver.apply_world_runtime_extensions(&mut world))
            .map_err(|error| error.to_string())?;
        merge_builtin_script_scene_systems(&self._runtime_extension_registry)
            .map_err(|error| error.to_string())?
            .apply_to_world(&mut world)
            .map_err(|error| error.to_string())?;

        // Keep the Level owner and its watches. The transaction retires the source World,
        // invalidates binding projections, and resets World-coupled runtime state.
        let retired = self
            .level
            .replace_world_with_time_policy(
                &expected,
                WorldReplacementRequest::preserve(world),
                WorldReplacementReason::SceneTravel,
            )
            .map_err(|error| error.to_string())?;
        drop(retired);
        self.level.set_metadata(LevelMetadata {
            project_root: Some(project.paths().root().to_string_lossy().into_owned()),
            asset_uri: Some(scene_uri.to_string()),
            display_name: uri.path().rsplit('/').next().map(ToString::to_string),
        });
        // This is exclusively the Play/runtime session. The editor's entry document
        // remains owned by its authoring session and is restored when Play detaches.
        self.scene_uri = Some(scene_uri.to_string());
        self.scene_transition_start_pending = true;
        self.selected_model_resource_id = None;
        self.selected_material_resource_id = None;
        Ok(())
    }
}

#[cfg(test)]
#[path = "tests/scene_transition.rs"]
mod tests;
