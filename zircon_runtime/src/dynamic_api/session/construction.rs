use crate::core::framework::input::INPUT_MODULE_NAME;
use crate::core::framework::render::{
    RenderProfileBundle, RenderSubmissionConfig, RENDER_PROFILE_CONFIG_KEY,
};
use crate::core::framework::time::{ProductTimePolicy, ProductTimePolicyError};
use crate::core::manager::{input_manager_handle, resolve_manager_service};
use crate::core::math::{UVec2, Vec2};
use crate::core::{
    CoreError, CoreHandle, CoreRuntime, FrameClockRebaseReceipt, TaskGraphScopeDescriptor,
};
use crate::diagnostic_log::{write_log, write_log_lazy};
use crate::operation::RuntimeOperationService;
use crate::plugin::{
    RuntimeExtensionRegistryError, RuntimePluginFeatureRegistrationReport,
    RuntimePluginRegistrationReport,
};
use crate::scene::components::NodeKind;
use crate::text::{text_runtime_context_for_core, TEXT_MODULE_NAME};

use super::super::camera_controller::RuntimeCameraController;
use super::super::runtime_loop::RuntimeRenderBridge;
use super::ime_composition_route::RuntimeImeCompositionRoute;
use super::project::{project_opened_log, RuntimePreparedProject, RuntimeProjectConfig};
use super::ui_extract_cache::RuntimeUiExtractCache;

#[path = "construction/cleanup.rs"]
mod cleanup;

use super::{
    event_mirror, linked_plugins::LinkedRuntimePluginPlan, merge_builtin_script_scene_systems,
    RuntimeDynamicSession, RuntimeDynamicSessionError, RuntimeDynamicSessionProfile,
    RuntimeDynamicSessionResult,
};
pub(in crate::dynamic_api::session) use cleanup::RuntimeConstructionCleanup;
pub(in crate::dynamic_api::session) use cleanup::RuntimeConstructionFailure;

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

fn apply_profile_time_policy(
    runtime: &CoreRuntime,
    policy: ProductTimePolicy,
) -> RuntimeDynamicSessionResult<()> {
    let transaction = policy.time_policy_transaction().map_err(|source| {
        RuntimeDynamicSessionError::ProductTimePolicy {
            step: "prepare runtime product time policy",
            source,
        }
    })?;
    runtime.apply_time_policy(transaction).map_err(|source| {
        RuntimeDynamicSessionError::ProductTimePolicy {
            step: "apply runtime product time policy",
            source: ProductTimePolicyError::TimePolicy(source),
        }
    })?;
    Ok(())
}

fn activate_registered_modules(runtime: &CoreRuntime) -> RuntimeDynamicSessionResult<()> {
    runtime
        .activate_registered_modules()
        .map_err(|source| RuntimeDynamicSessionError::CoreStep {
            step: "activate runtime modules",
            source,
        })
}

fn rebase_frame_clock_after_session_activation(runtime: &CoreRuntime) -> FrameClockRebaseReceipt {
    runtime.rebase_frame_clock()
}

pub(super) enum RuntimePluginPlanInput {
    CoreOnly,
    Linked {
        plugins: Vec<RuntimePluginRegistrationReport>,
        features: Vec<RuntimePluginFeatureRegistrationReport>,
    },
}

pub(super) fn build(
    profile: RuntimeDynamicSessionProfile,
    project_config: Option<RuntimeProjectConfig>,
    plugin_plan_input: RuntimePluginPlanInput,
) -> Result<RuntimeDynamicSession, RuntimeConstructionFailure> {
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
    let linked_plugin_plan = match plugin_plan_input {
        RuntimePluginPlanInput::CoreOnly => {
            LinkedRuntimePluginPlan::prepare_core_only(profile.target_mode())?
        }
        RuntimePluginPlanInput::Linked { plugins, features } => LinkedRuntimePluginPlan::prepare(
            &plugins,
            &features,
            project_plugin_manifest,
            profile.target_mode(),
        )?,
    };
    let (modules, runtime_plugin_catalog_snapshot, compiled_project_plugin_plan) =
        linked_plugin_plan.into_parts();
    let module_composition_identity = modules.identity().clone();
    let linked_extensions = compiled_project_plugin_plan.runtime_extensions_handle();
    let mut runtime_extension_registry = linked_extensions.registry.clone();
    let linked_extension_world_plan =
        merge_builtin_script_scene_systems(&runtime_extension_registry)?;
    let time_policy = profile.product_time_policy();
    let runtime = {
        crate::profile_scope!("runtime", "dynamic_api", "runtime_session_core_new");
        CoreRuntime::try_new().map_err(|source| {
            RuntimeDynamicSessionError::EngineTaskGraphInitialization { source }
        })?
    };
    let mut construction_cleanup = RuntimeConstructionCleanup::new(
        &runtime,
        &runtime_plugin_catalog_snapshot,
        &compiled_project_plugin_plan,
    );
    let task_graph_scope = construction_cleanup.check(
        runtime
            .create_task_graph_scope(TaskGraphScopeDescriptor::new("dynamic-session"))
            .map_err(|source| RuntimeDynamicSessionError::TaskGraphScopeAdmission { source }),
    )?;
    construction_cleanup.attach_scope(&task_graph_scope);
    construction_cleanup.check(apply_profile_time_policy(&runtime, time_policy))?;
    write_log("runtime_session", "runtime_dynamic_session_core_created");
    let core = runtime.handle();
    construction_cleanup.check(store_profile_submission_config(&core, profile))?;
    write_log_lazy("runtime_session", || {
        format!(
            "runtime_dynamic_session_modules_discovered count={} composition_hash={}",
            modules.modules().len(),
            modules.identity().composition_hash_hex(),
        )
    });
    {
        crate::profile_scope!("runtime", "dynamic_api", "runtime_session_register_modules");
        for descriptor in modules.module_descriptors() {
            let mut descriptor = descriptor.clone();
            #[cfg(feature = "native-host")]
            if descriptor.name == crate::core::framework::foundation::FOUNDATION_MODULE_NAME {
                if let Some(path) = prepared_project
                    .as_ref()
                    .and_then(RuntimePreparedProject::play_config_file_path)
                {
                    construction_cleanup.check(
                        crate::foundation::bind_config_file_path(&mut descriptor, path).map_err(
                            |source| RuntimeDynamicSessionError::CoreStep {
                                step: "bind Play configuration",
                                source,
                            },
                        ),
                    )?;
                }
            }
            construction_cleanup.check(runtime.register_module(descriptor).map_err(|source| {
                RuntimeDynamicSessionError::CoreStep {
                    step: "register runtime module",
                    source,
                }
            }))?;
        }
    }
    write_log(
        "runtime_session",
        "runtime_dynamic_session_modules_registered",
    );
    {
        crate::profile_scope!("runtime", "dynamic_api", "runtime_session_activate_modules");
        construction_cleanup.check(activate_registered_modules(&runtime))?;
    }
    write_log(
        "runtime_session",
        "runtime_dynamic_session_modules_activated",
    );
    let input_manager = {
        crate::profile_scope!("runtime", "dynamic_api", "runtime_session_resolve_input");
        let handle = construction_cleanup.check(input_manager_handle(&core).map_err(|source| {
            RuntimeDynamicSessionError::CoreStep {
                step: "capture input manager handle",
                source,
            }
        }))?;
        construction_cleanup.check(resolve_manager_service(&core, handle.clone()).map_err(
            |source| RuntimeDynamicSessionError::CoreStep {
                step: "resolve input",
                source,
            },
        ))?;
        handle
    };
    write_log("runtime_session", "runtime_dynamic_session_input_ready");
    let render_bridge = if profile.uses_render_bridge() {
        crate::profile_scope!("runtime", "dynamic_api", "runtime_session_render_bridge");
        let render_bridge =
            construction_cleanup.check(RuntimeRenderBridge::new(&core).map_err(|source| {
                RuntimeDynamicSessionError::CoreStep {
                    step: "create render bridge",
                    source,
                }
            }))?;
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
                let project_info = construction_cleanup.check(
                    project
                        .open_project_assets(&core, &mut runtime_extension_registry)
                        .map_err(|source| RuntimeDynamicSessionError::ProjectStep {
                            step: "open project assets",
                            source,
                        }),
                )?;
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
                construction_cleanup.check(project.load_default_navigation(&core).map_err(
                    |source| RuntimeDynamicSessionError::ProjectStep {
                        step: "load default project navigation",
                        source,
                    },
                ))?;
                write_log("runtime_session", "runtime_project_navigation_load_done");
                write_log("runtime_session", "runtime_project_scripts_load_start");
                construction_cleanup.check(project.load_startup_scripts(&core).map_err(
                    |source| RuntimeDynamicSessionError::ProjectStep {
                        step: "load startup script packages",
                        source,
                    },
                ))?;
                write_log("runtime_session", "runtime_project_scripts_load_done");
                write_log("runtime_session", "runtime_project_level_load_start");
                let level = if project.has_play_scene_override() {
                    construction_cleanup.check(project.load_play_scene_level(&core).map_err(
                        |source| RuntimeDynamicSessionError::ProjectStep {
                            step: "load Play scene override",
                            source,
                        },
                    ))?
                } else {
                    construction_cleanup.check(project.load_default_level(&core).map_err(
                        |source| RuntimeDynamicSessionError::ProjectStep {
                            step: "load default level",
                            source,
                        },
                    ))?
                };
                (level, project_identity, scene_uri)
            }
            None => (
                construction_cleanup.check(crate::scene::create_default_level(&core).map_err(
                    |source| RuntimeDynamicSessionError::CoreStep {
                        step: "create default level",
                        source,
                    },
                ))?,
                None,
                None,
            ),
        }
    };
    construction_cleanup.check(
        level
            .with_world_mut(|world| linked_extension_world_plan.apply_to_world(world))
            .map_err(
                |source| RuntimeDynamicSessionError::RuntimeExtensionRegistryStep {
                    step: "apply linked plugin extensions to runtime world",
                    source: RuntimeExtensionRegistryError::WorldRegistration(source.to_string()),
                },
            ),
    )?;
    write_log("runtime_session", "runtime_dynamic_session_level_ready");
    let scene_asset_reload_queue = match &prepared_project {
        Some(project) => Some(
            construction_cleanup
                .check(project.scene_asset_reload_queue(&core).map_err(|source| {
                    RuntimeDynamicSessionError::ProjectStep {
                        step: "create scene asset reload queue",
                        source,
                    }
                }))?
                .with_task_graph_scope(task_graph_scope.clone()),
        ),
        None => None,
    };
    if scene_asset_reload_queue.is_some() {
        write_log("runtime_session", "runtime_scene_asset_reload_queue_ready");
    }
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
            let orbit_node = world.find_node(cube);
            let orbit_target = orbit_node
                .as_ref()
                .map(|node| node.transform.translation)
                .unwrap_or_default();
            let selected_mesh = orbit_node.and_then(|node| node.mesh);
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
    construction_cleanup.check(
        crate::navigation::register_navigation_operation_handlers(&mut operations)
            .map_err(|source| RuntimeDynamicSessionError::RuntimeOperationRegistry { source }),
    )?;
    // Preserve the scoped service lookups. On an error, partially constructed
    // layout families also leave this closure before the original Core closes.
    let ui_resources = (|| {
        let runtime_ui_result = (|| {
            let text_context = text_runtime_context_for_core(&core).map_err(|source| {
                RuntimeDynamicSessionError::CoreStep {
                    step: "resolve runtime text context",
                    source,
                }
            })?;
            match &prepared_project {
                Some(project) => project
                    .load_runtime_ui_surfaces(&core, &text_context)
                    .map_err(|source| RuntimeDynamicSessionError::ProjectStep {
                        step: "load declared project UI roots",
                        source,
                    }),
                None => Ok(Default::default()),
            }
        })();
        let runtime_ui = runtime_ui_result?;
        let ui_extract_cache_result = (|| {
            let text_context = text_runtime_context_for_core(&core).map_err(|source| {
                RuntimeDynamicSessionError::CoreStep {
                    step: "resolve runtime text context",
                    source,
                }
            })?;
            RuntimeUiExtractCache::new_with_text_context(&text_context).map_err(|source| {
                RuntimeDynamicSessionError::CoreStep {
                    step: "create runtime UI text extract cache",
                    source: CoreError::Initialization(
                        TEXT_MODULE_NAME.to_owned(),
                        source.to_string(),
                    ),
                }
            })
        })();
        let ui_extract_cache = ui_extract_cache_result?;
        Ok((runtime_ui, ui_extract_cache))
    })();
    let (runtime_ui, ui_extract_cache) = construction_cleanup.check(ui_resources)?;
    // Claim only after the other fallible resources are ready. The Session owns
    // this exact capability until physical teardown, not a lookup of "current".
    let input_attachment = construction_cleanup.check((|| {
        let driver = core
            .resolve_driver::<crate::input::InputDriver>(crate::input::INPUT_DRIVER_NAME)
            .map_err(|source| RuntimeDynamicSessionError::CoreStep {
                step: "resolve input ingress",
                source,
            })?;
        driver
            .claim_attachment()
            .map_err(|reason| RuntimeDynamicSessionError::CoreStep {
                step: "claim input ingress attachment",
                source: CoreError::Initialization(INPUT_MODULE_NAME.to_owned(), reason.to_string()),
            })
    })())?;
    let frame_clock_activation_rebase = rebase_frame_clock_after_session_activation(&runtime);
    write_log(
        "runtime_session",
        "runtime_dynamic_session_post_create_ready",
    );

    Ok(RuntimeDynamicSession {
        runtime,
        task_graph_scope,
        profile,
        module_composition_identity,
        time_policy,
        frame_clock_activation_rebase,
        last_render_frame_timing: Default::default(),
        diagnostic_log_schedule: profile.diagnostic_log_schedule(),
        render_bridge,
        level,
        scene_asset_reload_queue,
        last_scene_asset_reload_report: None,
        project_identity,
        scene_uri,
        scene_transition_start_pending: false,
        selected_model_resource_id,
        selected_material_resource_id,
        camera_controller,
        extract_cache: Default::default(),
        ui_extract_cache: Some(ui_extract_cache),
        cursor: Vec2::ZERO,
        input_manager,
        input_attachment,
        input_ingress_detached: false,
        input_ingress_gap: false,
        input_consumer_reset_pending: false,
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
        _runtime_plugin_catalog_snapshot: runtime_plugin_catalog_snapshot,
        _compiled_project_plugin_plan: compiled_project_plugin_plan,
        _runtime_extension_registry: runtime_extension_registry,
        project_watchers_shutdown: false,
        dynamic_process_log: None,
        runtime_ui,
        ime_composition_route: RuntimeImeCompositionRoute::legacy(1),
        viewport_picks: Default::default(),
        editor_transform: Default::default(),
    })
}

#[cfg(test)]
#[path = "tests/construction.rs"]
mod tests;

#[cfg(test)]
#[path = "construction/tests/error_cleanup_tests.rs"]
mod error_cleanup_tests;
