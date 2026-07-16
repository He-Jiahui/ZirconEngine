use std::sync::Arc;

use crate::builtin::RuntimeModuleLoadReport;
use crate::core::manager::{input_manager_handle, resolve_manager_service};
use crate::core::math::{UVec2, Vec2};
use crate::core::CoreRuntime;
use crate::diagnostic_log::write_log;
use crate::operation::RuntimeOperationService;
use crate::plugin::{RuntimeExtensionRegistry, RuntimePluginRegistrationReport};
use crate::scene::components::NodeKind;

use super::super::camera_controller::RuntimeCameraController;
use super::super::runtime_loop::RuntimeRenderBridge;
use super::project::{RuntimePreparedProject, RuntimeProjectConfig};
use super::{
    event_mirror, install_builtin_scene_runtime_hooks, linked_plugins::LinkedRuntimePluginPlan,
    RuntimeDynamicSession, RuntimeDynamicSessionError, RuntimeDynamicSessionProfile,
    RuntimeDynamicSessionResult,
};

pub(super) fn build(
    profile: RuntimeDynamicSessionProfile,
    project_config: Option<RuntimeProjectConfig>,
    linked_plugin_registrations: Vec<RuntimePluginRegistrationReport>,
) -> RuntimeDynamicSessionResult<RuntimeDynamicSession> {
    crate::profile_scope!("runtime", "dynamic_api", "runtime_dynamic_session_new");
    crate::diagnostic_log::initialize_unity_process_log("runtime-dynamic");
    write_log(
        "runtime_session",
        format!(
            "runtime_dynamic_session_create_start profile={profile:?} project={}",
            project_config
                .as_ref()
                .map(RuntimeProjectConfig::root_display)
                .unwrap_or_else(|| "none".to_string())
        ),
    );
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
    let (mut modules, mut linked_extensions): (RuntimeModuleLoadReport, RuntimeExtensionRegistry) =
        linked_plugin_plan.into_parts();
    let linked_modules = linked_extensions.modules().to_vec();
    let runtime = {
        crate::profile_scope!("runtime", "dynamic_api", "runtime_session_core_new");
        CoreRuntime::new()
    };
    write_log("runtime_session", "runtime_dynamic_session_core_created");
    let core = runtime.handle();
    if !has_linked_navigation {
        modules
            .modules
            .push(Arc::new(crate::navigation::BuiltinNavigationModule));
    }
    modules
        .modules
        .push(Arc::new(crate::animation::AnimationModule));
    let fatal_diagnostics = modules.fatal_messages();
    if !fatal_diagnostics.is_empty() {
        return Err(RuntimeDynamicSessionError::ModuleDiscovery {
            message: fatal_diagnostics.join("; "),
        });
    }
    write_log(
        "runtime_session",
        format!(
            "runtime_dynamic_session_modules_discovered count={}",
            modules.modules.len()
        ),
    );
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
    if !linked_extensions.scene_hooks().is_empty() {
        crate::scene::install_scene_runtime_hooks(
            &runtime.handle(),
            linked_extensions.scene_hooks().iter().cloned(),
        )
        .map_err(|source| RuntimeDynamicSessionError::CoreStep {
            step: "install linked plugin scene runtime hooks",
            source,
        })?;
    }
    {
        crate::profile_scope!(
            "runtime",
            "dynamic_api",
            "runtime_session_install_scene_hooks"
        );
        install_builtin_scene_runtime_hooks(&runtime)?;
    }
    write_log(
        "runtime_session",
        "runtime_dynamic_session_scene_hooks_installed",
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
    let level = {
        crate::profile_scope!("runtime", "dynamic_api", "runtime_session_level");
        match &mut prepared_project {
            Some(project) => {
                write_log("runtime_session", "runtime_project_open_assets_start");
                project.open_project_assets(&core).map_err(|source| {
                    RuntimeDynamicSessionError::ProjectStep {
                        step: "open project assets",
                        source,
                    }
                })?;
                write_log("runtime_session", "runtime_project_open_assets_done");
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
                project.load_default_level(&core).map_err(|source| {
                    RuntimeDynamicSessionError::ProjectStep {
                        step: "load default level",
                        source,
                    }
                })?
            }
            None => crate::scene::create_default_level(&core).map_err(|source| {
                RuntimeDynamicSessionError::CoreStep {
                    step: "create default level",
                    source,
                }
            })?,
        }
    };
    level
        .with_world_mut(|world| linked_extensions.apply_to_world(world))
        .map_err(
            |source| RuntimeDynamicSessionError::RuntimeExtensionRegistryStep {
                step: "apply linked plugin extensions to runtime world",
                source,
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
    let (selected_node, orbit_target) = {
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
            let orbit_target = world
                .find_node(cube)
                .map(|node| node.transform.translation)
                .unwrap_or_default();
            (Some(cube), orbit_target)
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
        selected_node,
        camera_controller,
        extract_cache: Default::default(),
        cursor: Vec2::ZERO,
        input_manager,
        next_plugin_event_subscription: 1,
        plugin_event_subscriptions: event_mirror::empty_plugin_event_subscriptions(),
        operations,
    })
}
