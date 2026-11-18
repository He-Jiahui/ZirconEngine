use std::{
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    thread::{self, ThreadId},
};

use zircon_runtime::{
    asset::project_asset_manager_handle,
    builtin::RuntimePluginId,
    core::{
        framework::{platform::RuntimeTargetMode, scene::ComponentTypeDescriptor},
        manager::{input_manager_handle, resolve_manager_service},
        CoreError,
    },
    input::{
        InputDriver, InputEventRecord, InputEventRecordingConfig, InputEventRecordingStatus,
        InputIngressHealth, TouchPoint, INPUT_DRIVER_NAME,
    },
    plugin::{
        RuntimeExtensionRegistry, RuntimeExtensionRegistryError, RuntimePlugin,
        RuntimePluginDescriptor, RuntimePluginRegistrationReport,
    },
    scene::{ecs::RuntimeSceneSystemContext, NodeKind, SceneError, SystemStage},
};

const MENU: &str = "gameplay.menu_state";
const CONTROL: &str = "gameplay.control_state";

#[derive(Debug)]
pub(super) struct ProjectReceipt {
    pub(super) root: PathBuf,
    pub(super) ui_roots: usize,
    pub(super) scene: Option<String>,
    pub(super) importer: String,
    pub(super) importer_available: bool,
}

#[derive(Debug, PartialEq, Eq)]
pub(super) struct Slot {
    pub(super) world: u64,
    pub(super) epoch: u64,
    pub(super) entity: u64,
    pub(super) menu: Option<String>,
    pub(super) menu_generation: u64,
    pub(super) command: Option<String>,
    pub(super) command_generation: u64,
}

#[derive(Debug)]
pub(super) struct Observation {
    pub(super) owner: ThreadId,
    pub(super) project: Option<ProjectReceipt>,
    pub(super) slot: Slot,
    pub(super) ingress: InputIngressHealth,
    pub(super) recording: InputEventRecordingStatus,
    pub(super) journal: Vec<InputEventRecord>,
    pub(super) active: Vec<TouchPoint>,
    pub(super) cursor_inside: bool,
}

pub(super) type Inbox = Arc<Mutex<Option<Observation>>>;

pub(super) fn registration(state: &'static str, inbox: &Inbox) -> RuntimePluginRegistrationReport {
    RuntimePluginRegistrationReport::from_plugin(&FixturePlugin {
        descriptor: RuntimePluginDescriptor::builder(
            "gameplay",
            "Public touch fixture",
            RuntimePluginId::new("gameplay"),
            "zircon_fixture_touch_cancel",
        )
        .with_target_modes([RuntimeTargetMode::ClientRuntime])
        .build(),
        state,
        inbox: Arc::clone(inbox),
    })
}

struct FixturePlugin {
    descriptor: RuntimePluginDescriptor,
    state: &'static str,
    inbox: Inbox,
}

impl RuntimePlugin for FixturePlugin {
    fn descriptor(&self) -> &RuntimePluginDescriptor {
        &self.descriptor
    }

    fn register(
        &self,
        registry: &mut RuntimeExtensionRegistry,
    ) -> Result<(), RuntimeExtensionRegistryError> {
        let owner = registry.intern_plugin_module("gameplay.runtime")?;
        for id in [MENU, CONTROL] {
            registry.register_component_for_owner(
                owner,
                ComponentTypeDescriptor::new(id, "gameplay", id),
            )?;
        }
        let state = self.state;
        let inbox = Arc::clone(&self.inbox);
        registry
            .register_runtime_scene_system(
                owner,
                "gameplay.touch_fixture",
                SystemStage::PostUpdate,
                move || {
                    let inbox = Arc::clone(&inbox);
                    let mut entity = None;
                    move |context: RuntimeSceneSystemContext<'_>| {
                        observe_owner(context, state, &mut entity, &inbox)
                    }
                },
            )
            .register()
    }
}

fn fixture_error(message: impl Into<String>) -> CoreError {
    CoreError::Initialization("public touch fixture".to_owned(), message.into())
}

fn observe_owner(
    context: RuntimeSceneSystemContext<'_>,
    state: &str,
    entity: &mut Option<u64>,
    inbox: &Inbox,
) -> Result<(), CoreError> {
    let core = context.core;
    let level = context.level;
    let input = resolve_manager_service(core, input_manager_handle(core)?)?;
    let driver = core.resolve_driver::<InputDriver>(INPUT_DRIVER_NAME)?;
    // Seed the legal menu and enable its session recorder only on bootstrap. Later calls only
    // read live state and consume this recorder's diagnostic FIFO, not physical input state.
    let project = if entity.is_none() {
        let assets = resolve_manager_service(core, project_asset_manager_handle(core)?)?;
        let project = assets
            .current_project_manager()
            .ok_or_else(|| fixture_error("missing active project"))?;
        let importer = project
            .importer()
            .capability_report_for_source(Path::new("probe.zui"))
            .map_err(|error| fixture_error(error.to_string()))?;
        let (id, seeded) = level
            .with_world_mut(|world| -> Result<_, SceneError> {
                let camera_loaded = world.find_node(1).is_some_and(|camera| {
                    camera.name == "Touch fixture camera" && camera.camera.is_some()
                });
                let id = world.spawn_node(NodeKind::Empty)?;
                let menu = world.set_dynamic_component(
                    id,
                    MENU,
                    format!(r#"{{"state":"{state}"}}"#).parse().unwrap(),
                )?;
                let control =
                    world.set_dynamic_component(id, CONTROL, r#""none""#.parse().unwrap())?;
                Ok((id, camera_loaded && menu && control))
            })
            .map_err(|error| fixture_error(error.to_string()))?;
        if !seeded {
            return Err(fixture_error("authored camera or legal menu seed missing"));
        }
        *entity = Some(id);
        input.set_event_recording_config(InputEventRecordingConfig::enabled(32));
        Some(ProjectReceipt {
            root: project.paths().root().to_path_buf(),
            ui_roots: project.manifest().ui_roots.len(),
            scene: level.metadata().asset_uri,
            importer: importer.descriptor.id,
            importer_available: importer.status.is_available(),
        })
    } else {
        None
    };
    let entity = entity.expect("fixture entity assigned on owner");
    let epoch = level.capture_world_replacement_epoch();
    let slot = level.with_world(|world| Slot {
        world: level.world_handle().get(),
        epoch,
        entity,
        menu: world
            .dynamic_component(entity, MENU)
            .and_then(|value| value.get("state"))
            .and_then(|value| value.as_str())
            .map(str::to_owned),
        menu_generation: world.dynamic_component_generation(MENU),
        command: world
            .dynamic_component(entity, CONTROL)
            .and_then(|value| value.as_str())
            .map(str::to_owned),
        command_generation: world.dynamic_component_generation(CONTROL),
    });
    let physical = input.frame_snapshot();
    let (journal, recording) = input.drain_event_records_with_status();
    let observation = Observation {
        owner: thread::current().id(),
        project,
        slot,
        ingress: driver.health(),
        recording,
        journal,
        active: physical.active_touches,
        cursor_inside: physical.cursor_inside_window,
    };
    let mut inbox = inbox.lock().unwrap();
    if inbox.is_some() {
        return Err(fixture_error(
            "more than one PostUpdate observation per tick",
        ));
    }
    *inbox = Some(observation);
    Ok(())
}
