use std::sync::Arc;

use crate::builtin::RuntimeModuleLoadReport;
use crate::core::framework::render::{
    RenderProfileBundle, RenderSubmissionConfig, RENDER_PROFILE_CONFIG_KEY,
};
use crate::core::manager::{input_manager_handle, resolve_manager_service};
use crate::core::math::{UVec2, Vec2};
use crate::core::{CoreHandle, CoreRuntime};
use crate::diagnostic_log::{write_log, write_log_lazy};
use crate::operation::RuntimeOperationService;
use crate::plugin::{
    RuntimeExtensionCatalogReport, RuntimeExtensionRegistryError, RuntimePluginRegistrationReport,
};
use crate::scene::components::NodeKind;

use super::super::camera_controller::RuntimeCameraController;
use super::super::runtime_loop::RuntimeRenderBridge;
use super::project::{project_opened_log, RuntimePreparedProject, RuntimeProjectConfig};
use super::{
    event_mirror, linked_plugins::LinkedRuntimePluginPlan, merge_builtin_script_scene_systems,
    RuntimeDynamicSession, RuntimeDynamicSessionError, RuntimeDynamicSessionProfile,
    RuntimeDynamicSessionResult,
};

fn store_profile_submission_config(
    core: &CoreHandle,
    profile: RuntimeDynamicSessionProfile,
) -> RuntimeDynamicSessionResult<()> {
    if !profile.pipelined_render() {
        return Ok(());
    }
    let profile_bundle = RenderProfileBundle::default_render()
        .with_submission_config(RenderSubmissionConfig::pipelined());
    core.store_config(RENDER_PROFILE_CONFIG_KEY, &profile_bundle)
        .map_err(|source| RuntimeDynamicSessionError::CoreStep {
            step: "store pipelined render profile",
            source,
        })
}

pub(super) fn build(
    profile: RuntimeDynamicSessionProfile,
    project_config: Option<RuntimeProjectConfig>,
    linked_plugin_registrations: Vec<RuntimePluginRegistrationReport>,
) -> RuntimeDynamicSessionResult<RuntimeDynamicSession> {
    crate::profile_scope!("runtime", "dynamic_api", "runtime_dynamic_session_new");
    crate::diagnostic_log::initialize_unity_process_log("runtime-dynamic");
    write_log_lazy("runtime_session", || {
        format!(
            "runtime_dynamic_session_create_start profile={profile:?} project={}",
            project_config
                .as_ref()
                .map(RuntimeProjectConfig::root_display)
                .unwrap_or_else(|| "none".to_string())
        )
    });
    let mut prepared_project = project_config
        .map(RuntimeProjectConfig::prepare)
        .transpose()
        .map_err(|source| RuntimeDynamicSessionError::ProjectStep {
            step: "prepare runtime project",
            source,
        })?;
    let project_plugin_manifest = prepared_project
        .as_ref()
        .map(RuntimePreparedProject::plugin_manifest);
    let linked_plugin_plan = LinkedRuntimePluginPlan::prepare(
        &linked_plugin_registrations,
        project_plugin_manifest,
        profile.target_mode(),
    )?;
    let has_linked_navigation = linked_plugin_plan.contains_package("navigation");
    let has_linked_animation = linked_plugin_plan.contains_package("animation");
    let (mut modules, linked_extensions): (
        RuntimeModuleLoadReport,
        Arc<RuntimeExtensionCatalogReport>,
    ) = linked_plugin_plan.into_parts();
    let linked_modules = linked_extensions.registry.modules().to_vec();
    let linked_extension_world_plan = linked_extensions
        .registry
        .world_runtime_extension_plan()
        .map_err(
            |source| RuntimeDynamicSessionError::RuntimeExtensionRegistryStep {
                step: "prepare linked plugin extensions for runtime world",
                source,
            },
        )?;
    let linked_extension_world_plan = merge_builtin_script_scene_systems(
        &linked_extensions.registry,
        linked_extension_world_plan,
    )?;
    let runtime = {
        crate::profile_scope!("runtime", "dynamic_api", "runtime_session_core_new");
        CoreRuntime::new()
    };
    write_log("runtime_session", "runtime_dynamic_session_core_created");
    let core = runtime.handle();
    store_profile_submission_config(&core, profile)?;
    if !has_linked_navigation {
        modules
            .modules
            .push(Arc::new(crate::navigation::BuiltinNavigationModule));
    }
    if !has_linked_animation {
        modules
            .modules
            .push(Arc::new(crate::animation::AnimationModule));
    }
    let fatal_diagnostics = modules.fatal_messages();
    if !fatal_diagnostics.is_empty() {
        return Err(RuntimeDynamicSessionError::ModuleDiscovery {
            message: fatal_diagnostics.join("; "),
        });
    }
    write_log_lazy("runtime_session", || {
        format!(
            "runtime_dynamic_session_modules_discovered count={}",
            modules.modules.len()
        )
    });
    {
        crate::profile_scope!("runtime", "dynamic_api", "runtime_session_register_modules");
        for module in &modules.modules {
            runtime
                .register_module(module.descriptor())
                .map_err(|source| RuntimeDynamicSessionError::CoreStep {
                    step: "register runtime module",
                    source,
                })?;
        }
        for module in &linked_modules {
            runtime.register_module(module.clone()).map_err(|source| {
                RuntimeDynamicSessionError::CoreStep {
                    step: "register linked runtime module",
                    source,
                }
            })?;
        }
    }
    write_log(
        "runtime_session",
        "runtime_dynamic_session_modules_registered",
    );
    {
        crate::profile_scope!("runtime", "dynamic_api", "runtime_session_activate_modules");
        runtime.activate_registered_modules().map_err(|source| {
            RuntimeDynamicSessionError::CoreStep {
                step: "activate runtime modules",
                source,
            }
        })?;
    }
    write_log(
        "runtime_session",
        "runtime_dynamic_session_modules_activated",
    );
    let input_manager = {
        crate::profile_scope!("runtime", "dynamic_api", "runtime_session_resolve_input");
        let handle =
            input_manager_handle(&core).map_err(|source| RuntimeDynamicSessionError::CoreStep {
                step: "capture input manager handle",
                source,
            })?;
        resolve_manager_service(&core, handle.clone()).map_err(|source| {
            RuntimeDynamicSessionError::CoreStep {
                step: "resolve input",
                source,
            }
        })?;
        handle
    };
    write_log("runtime_session", "runtime_dynamic_session_input_ready");
    let render_bridge = if profile.uses_render_bridge() {
        crate::profile_scope!("runtime", "dynamic_api", "runtime_session_render_bridge");
        let render_bridge = RuntimeRenderBridge::new(&core).map_err(|source| {
            RuntimeDynamicSessionError::CoreStep {
                step: "create render bridge",
                source,
            }
        })?;
        write_log(
            "runtime_session",
            "runtime_dynamic_session_render_bridge_ready",
        );
        Some(render_bridge)
    } else {
        write_log(
            "runtime_session",
            "runtime_dynamic_session_render_bridge_skipped",
        );
        None
    };
    let (level, project_identity, scene_uri) = {
        crate::profile_scope!("runtime", "dynamic_api", "runtime_session_level");
        match &mut prepared_project {
            Some(project) => {
                write_log("runtime_session", "runtime_project_open_assets_start");
                let project_info = project.open_project_assets(&core).map_err(|source| {
                    RuntimeDynamicSessionError::ProjectStep {
                        step: "open project assets",
                        source,
                    }
                })?;
                write_log("runtime_session", "runtime_project_open_assets_done");
                write_log_lazy("runtime_session", || project_opened_log(&project_info));
                let project_identity =
                    (!project_info.name.trim().is_empty()).then(|| project_info.name.clone());
                let play_scene_override = project.play_scene_identifier();
                let scene_uri = play_scene_override.clone().or_else(|| {
                    (!project_info.default_scene_uri.trim().is_empty())
                        .then(|| project_info.default_scene_uri.clone())
                });
                write_log("runtime_session", "runtime_project_navigation_load_start");
                project.load_default_navigation(&core).map_err(|source| {
                    RuntimeDynamicSessionError::ProjectStep {
                        step: "load default project navigation",
                        source,
                    }
                })?;
                write_log("runtime_session", "runtime_project_navigation_load_done");
                write_log("runtime_session", "runtime_project_scripts_load_start");
                project.load_startup_scripts(&core).map_err(|source| {
                    RuntimeDynamicSessionError::ProjectStep {
                        step: "load startup script packages",
                        source,
                    }
                })?;
                write_log("runtime_session", "runtime_project_scripts_load_done");
                write_log("runtime_session", "runtime_project_level_load_start");
                let level = if project.has_play_scene_override() {
                    project.load_play_scene_level(&core).map_err(|source| {
                        RuntimeDynamicSessionError::ProjectStep {
                            step: "load Play scene override",
                            source,
                        }
                    })?
                } else {
                    project.load_default_level(&core).map_err(|source| {
                        RuntimeDynamicSessionError::ProjectStep {
                            step: "load default level",
                            source,
                        }
                    })?
                };
                (level, project_identity, scene_uri)
            }
            None => (
                crate::scene::create_default_level(&core).map_err(|source| {
                    RuntimeDynamicSessionError::CoreStep {
                        step: "create default level",
                        source,
                    }
                })?,
                None,
                None,
            ),
        }
    };
    level
        .with_world_mut(|world| linked_extension_world_plan.apply_to_world(world))
        .map_err(
            |source| RuntimeDynamicSessionError::RuntimeExtensionRegistryStep {
                step: "apply linked plugin extensions to runtime world",
                source: RuntimeExtensionRegistryError::WorldRegistration(source.to_string()),
            },
        )?;
    write_log("runtime_session", "runtime_dynamic_session_level_ready");
    let scene_asset_reload_queue = match &prepared_project {
        Some(project) => Some(project.scene_asset_reload_queue(&core).map_err(|source| {
            RuntimeDynamicSessionError::ProjectStep {
                step: "create scene asset reload queue",
                source,
            }
        })?),
        None => None,
    };
    if scene_asset_reload_queue.is_some() {
        write_log("runtime_session", "runtime_scene_asset_reload_queue_ready");
    }
    let runtime_ui = match &prepared_project {
        Some(project) => project.load_runtime_ui_surfaces(&core).map_err(|source| {
            RuntimeDynamicSessionError::ProjectStep {
                step: "load declared project UI roots",
                source,
            }
        })?,
        None => Default::default(),
    };
    let (orbit_target, selected_model_resource_id, selected_material_resource_id) = {
        crate::profile_scope!(
            "runtime",
            "dynamic_api",
            "runtime_session_select_orbit_target"
        );
        level.with_world(|world| {
            let cube = world
                .nodes()
                .iter()
                .find(|node| matches!(&node.kind, NodeKind::Cube))
                .map(|node| node.id)
                .unwrap_or(world.active_camera());
            let selected_node = world.find_node(cube);
            let orbit_target = selected_node
                .as_ref()
                .map(|node| node.transform.translation)
                .unwrap_or_default();
            let selected_mesh = selected_node.and_then(|node| node.mesh);
            (
                orbit_target,
                selected_mesh
                    .as_ref()
                    .map(|mesh| mesh.model.id().to_string()),
                selected_mesh
                    .as_ref()
                    .map(|mesh| mesh.material.id().to_string()),
            )
        })
    };
    let mut camera_controller = {
        crate::profile_scope!(
            "runtime",
            "dynamic_api",
            "runtime_session_camera_controller"
        );
        RuntimeCameraController::new(UVec2::new(1280, 720))
    };
    camera_controller.set_orbit_target(orbit_target);
    write_log("runtime_session", "runtime_dynamic_session_create_done");

    let mut operations = RuntimeOperationService::new();
    crate::navigation::register_navigation_operation_handlers(&mut operations)
        .map_err(|source| RuntimeDynamicSessionError::RuntimeOperationRegistry { source })?;

    Ok(RuntimeDynamicSession {
        runtime,
        profile,
        diagnostic_log_schedule: profile.diagnostic_log_schedule(),
        render_bridge,
        level,
        scene_asset_reload_queue,
        last_scene_asset_reload_report: None,
        project_identity,
        scene_uri,
        selected_model_resource_id,
        selected_material_resource_id,
        camera_controller,
        extract_cache: Default::default(),
        cursor: Vec2::ZERO,
        input_manager,
        input_diagnostics: Default::default(),
        pending_host_request_output: None,
        host_request_output_commit_count: 0,
        host_request_output_in_flight: false,
        pending_world_invalidation_output: None,
        world_invalidation_output_page: None,
        world_invalidation_output_in_flight: false,
        next_plugin_event_subscription: 1,
        plugin_event_subscriptions: event_mirror::empty_plugin_event_subscriptions(),
        operations,
        project_watchers_shutdown: false,
        dynamic_process_log: None,
        runtime_ui,
    })
}

#[cfg(test)]
mod tests {
    use super::{
        store_profile_submission_config, CoreRuntime, RenderProfileBundle, RenderSubmissionConfig,
        RuntimeDynamicSessionProfile, RENDER_PROFILE_CONFIG_KEY,
    };

    #[test]
    fn pipelined_runtime_profile_stores_the_render_submission_config_before_activation() {
        let runtime = CoreRuntime::new();
        let core = runtime.handle();

        store_profile_submission_config(&core, RuntimeDynamicSessionProfile::RuntimePipelined)
            .expect("pipelined runtime profile should store the render submission config");

        let profile = core
            .load_config::<RenderProfileBundle>(RENDER_PROFILE_CONFIG_KEY)
            .expect("pipelined runtime profile should be readable before module activation");
        assert_eq!(
            profile.submission_config(),
            RenderSubmissionConfig::pipelined()
        );
    }

    #[test]
    fn standard_runtime_profile_does_not_override_the_default_submission_config() {
        let runtime = CoreRuntime::new();
        let core = runtime.handle();

        store_profile_submission_config(&core, RuntimeDynamicSessionProfile::Runtime)
            .expect("standard runtime profile should not require render submission configuration");

        assert_eq!(core.load_config_value(RENDER_PROFILE_CONFIG_KEY), None);
    }
}
