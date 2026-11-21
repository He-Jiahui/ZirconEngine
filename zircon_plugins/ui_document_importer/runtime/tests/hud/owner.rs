use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
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
        InputButton, InputDriver, InputEvent, InputEventRecordingConfig, InputIngressHealth,
        InputIngressPhase, InputRecordingFrame, INPUT_DRIVER_NAME,
    },
    plugin::{
        RuntimeExtensionRegistry, RuntimeExtensionRegistryError, RuntimePlugin,
        RuntimePluginDescriptor, RuntimePluginRegistrationReport,
    },
    scene::ecs::{RuntimeSceneSystemContext, SceneSystemTickPolicy, SystemStage},
};

use super::project::{ProjectSample, CARRIER};

const PLUGIN: &str = "gameplay"; // component descriptors must use their owning plugin-id prefix
const MODULE: &str = "gameplay.runtime";
const MENU: &str = "gameplay.menu_state";
const CONTROL: &str = "gameplay.control_state";

#[derive(Clone, Copy, Debug)]
pub(super) enum Menu {
    Start,
    Retry,
}

impl Menu {
    fn seed_json(self) -> &'static str {
        match self {
            Self::Start => r#"{"state":"start"}"#,
            Self::Retry => r#"{"state":"game_over"}"#,
        }
    }
    fn state(self) -> &'static str {
        match self {
            Self::Start => "start",
            Self::Retry => "game_over",
        }
    }
    fn command(self) -> &'static str {
        match self {
            Self::Start => "start_game",
            Self::Retry => "retry_game",
        }
    }
}

#[derive(Clone, Default)]
pub(super) struct Tap {
    seed: Arc<AtomicBool>,
    output: Arc<Mutex<Option<Sample>>>,
}

impl Tap {
    pub fn registration(&self, menu: Menu) -> RuntimePluginRegistrationReport {
        RuntimePluginRegistrationReport::from_plugin(&GameplayFixture {
            desc: RuntimePluginDescriptor::builder(
                PLUGIN,
                "HUD fixture slot owner",
                RuntimePluginId::new(PLUGIN),
                "zircon_plugin_gameplay_hud_fixture",
            )
            .with_target_modes([RuntimeTargetMode::ClientRuntime])
            .with_capability("runtime.gameplay.hud_fixture")
            .build(),
            tap: self.clone(),
            menu,
        })
    }

    pub fn arm_seed(&self) {
        assert!(!self.seed.swap(true, Ordering::Release));
    }

    pub fn take(&self) -> Sample {
        self.output
            .lock()
            .expect("fixture output lock")
            .take()
            .expect("PostUpdate observation after this V8 tick")
    }
}

struct GameplayFixture {
    desc: RuntimePluginDescriptor,
    tap: Tap,
    menu: Menu,
}

impl RuntimePlugin for GameplayFixture {
    fn descriptor(&self) -> &RuntimePluginDescriptor {
        &self.desc
    }

    fn register(
        &self,
        registry: &mut RuntimeExtensionRegistry,
    ) -> Result<(), RuntimeExtensionRegistryError> {
        let owner = registry.intern_plugin_module(MODULE)?;
        for type_id in [MENU, CONTROL] {
            registry.register_component_for_owner(
                owner,
                ComponentTypeDescriptor::new(type_id, PLUGIN, type_id),
            )?;
        }
        let menu = self.menu;
        let seed = Arc::clone(&self.tap.seed);
        registry
            .register_runtime_scene_system(
                owner,
                "gameplay.hud_fixture_seed",
                SystemStage::Update,
                move || {
                    let seed = Arc::clone(&seed);
                    let mut initialized = false;
                    let mut seeded = false;
                    move |cx: RuntimeSceneSystemContext<'_>| {
                        if !initialized {
                            let input =
                                resolve_manager_service(cx.core, input_manager_handle(cx.core)?)?;
                            input
                                .set_event_recording_config(InputEventRecordingConfig::enabled(32));
                            initialized = true;
                        }
                        if !seeded && seed.load(Ordering::Acquire) {
                            cx.level.with_world_mut(|world| -> Result<(), CoreError> {
                                if world.dynamic_component(CARRIER, MENU).is_some()
                                    || world.dynamic_component(CARRIER, CONTROL).is_some()
                                {
                                    return Err(probe_error(
                                        "fixture must not overwrite any gameplay values",
                                    ));
                                }
                                // Initial authored gameplay values only. This system never writes an action.
                                let changed_menu = world
                                    .set_dynamic_component(
                                        CARRIER,
                                        MENU,
                                        menu.seed_json().parse().expect("literal menu JSON"),
                                    )
                                    .map_err(probe_error)?;
                                let changed_control = world
                                    .set_dynamic_component(
                                        CARRIER,
                                        CONTROL,
                                        r#""none""#.parse().expect("literal initial control JSON"),
                                    )
                                    .map_err(probe_error)?;
                                if !changed_menu || !changed_control {
                                    return Err(probe_error(
                                        "fixture failed to seed the original two slots",
                                    ));
                                }
                                Ok(())
                            })?;
                            seeded = true;
                        }
                        Ok(())
                    }
                },
            )
            .with_tick_policy(SceneSystemTickPolicy::monotonic_real())
            .register()?;

        let output = Arc::clone(&self.tap.output);
        registry
            .register_runtime_scene_system(
                owner,
                "gameplay.hud_fixture_observe",
                SystemStage::PostUpdate,
                move || {
                    let output = Arc::clone(&output);
                    let mut tick = 0;
                    move |cx: RuntimeSceneSystemContext<'_>| {
                        tick += 1;
                        let sample = Sample::read(cx, tick)?;
                        *output.lock().expect("fixture output lock") = Some(sample);
                        Ok(())
                    }
                },
            )
            .with_tick_policy(SceneSystemTickPolicy::monotonic_real())
            .register()?;
        Ok(())
    }
}

fn probe_error(error: impl ToString) -> CoreError {
    CoreError::Initialization("HUD fixture owner probe".into(), error.to_string())
}

#[derive(Debug)]
pub(super) struct Sample {
    tick: u64,
    pub world: u64,
    pub epoch: u64,
    pub camera: u64,
    pub carrier_present: bool,
    menu: Option<String>,
    command: Option<String>,
    revision: u64,
    pub project: ProjectSample,
    pub ingress: InputIngressHealth,
    pub journal: InputRecordingFrame,
    pointer: [f32; 2],
    left: [bool; 3],
}

impl Sample {
    fn read(cx: RuntimeSceneSystemContext<'_>, tick: u64) -> Result<Self, CoreError> {
        let world = cx.level.world_handle().0;
        let epoch = cx.level.capture_world_replacement_epoch();
        let (camera, carrier_present, menu, command, revision) = cx.level.with_world(|world| {
            let menu = world
                .dynamic_component(CARRIER, MENU)
                .and_then(|v| v.get("state"))
                .and_then(|v| v.as_str())
                .map(str::to_owned);
            let command = world
                .dynamic_component(CARRIER, CONTROL)
                .and_then(|v| v.as_str())
                .map(str::to_owned);
            (
                world.active_camera(),
                world.contains_entity(CARRIER),
                menu,
                command,
                world.dynamic_component_generation(CONTROL),
            )
        });
        let manager = resolve_manager_service(cx.core, project_asset_manager_handle(cx.core)?)?;
        let project = ProjectSample::read(&manager).map_err(probe_error)?;
        let input = resolve_manager_service(cx.core, input_manager_handle(cx.core)?)?;
        let ingress = cx
            .core
            .resolve_driver::<InputDriver>(INPUT_DRIVER_NAME)?
            .health();
        let state = input.frame_snapshot();
        let left = [
            state.buttons.pressed(&InputButton::MouseLeft),
            state.buttons.just_pressed(&InputButton::MouseLeft),
            state.buttons.just_released(&InputButton::MouseLeft),
        ];
        // Only this probe's opt-in recorder is consumed. Do not drain frame events or mutate input.
        let journal = InputRecordingFrame::capture_from_manager(tick, &*input);
        Ok(Self {
            tick,
            world,
            epoch,
            camera,
            carrier_present,
            menu,
            command,
            revision,
            project,
            ingress,
            journal,
            pointer: state.cursor_position,
            left,
        })
    }

    pub fn check_successor(&self, previous: &Self) {
        assert_eq!(self.tick, previous.tick + 1, "not the next owner sample");
        assert_eq!(
            (self.world, self.epoch, self.camera),
            (previous.world, previous.epoch, previous.camera),
            "different runtime World/epoch/camera"
        );
        assert_eq!(
            self.project, previous.project,
            "project registration changed mid-case"
        );
        assert_eq!(self.ingress.generation, previous.ingress.generation);
    }

    pub fn check_unseeded(&self) {
        assert!(self.carrier_present && self.menu.is_none() && self.command.is_none());
        assert!(self.journal.is_empty());
    }

    pub fn check_seed(&self, menu: Menu) {
        assert!(
            self.carrier_present && self.revision > 0,
            "invalid seed: {self:?}"
        );
        assert_eq!(self.menu.as_deref(), Some(menu.state()));
        assert_eq!(self.command.as_deref(), Some("none"));
        assert!(self.journal.is_empty());
    }

    pub fn check_click(&self, previous: &Self, [x, y]: [f32; 2]) {
        self.check_successor(previous);
        assert_eq!(self.menu, previous.menu, "fixture menu state changed");
        assert_eq!(self.ingress.phase, InputIngressPhase::Receiving);
        assert_eq!(
            self.ingress.accepted_batches,
            previous.ingress.accepted_batches + 3
        );
        assert_eq!(self.ingress.frame_events, 3);
        assert_eq!(
            self.ingress.rejected_batches,
            previous.ingress.rejected_batches
        );
        assert_eq!(self.ingress.gap_resets, previous.ingress.gap_resets);
        let records = self.journal.records();
        assert_eq!(records.len(), 3, "native InputManager journal {records:?}");
        assert_eq!(records[0].event, InputEvent::CursorMoved { x, y });
        assert_eq!(
            records[1].event,
            InputEvent::ButtonPressed(InputButton::MouseLeft)
        );
        assert_eq!(
            records[2].event,
            InputEvent::ButtonReleased(InputButton::MouseLeft)
        );
        assert!(records
            .windows(2)
            .all(|pair| pair[0].sequence + 1 == pair[1].sequence));
        assert_eq!(self.pointer, [x, y]);
        assert_eq!(
            self.left,
            [false, true, true],
            "physical held/down/up did not survive UI routing"
        );
    }

    pub fn expect_command(&self, previous: &Self, menu: Menu, why: &str) {
        assert_eq!(
            self.command.as_deref(),
            Some(menu.command()),
            "{why}: original control slot"
        );
        assert_eq!(
            self.revision,
            previous.revision + 1,
            "{why}: one committed value transition"
        );
    }

    pub fn expect_unchanged(&self, previous: &Self, why: &str) {
        assert_eq!(
            self.command, previous.command,
            "{why}: original control slot (revision {} -> {})",
            previous.revision, self.revision
        );
        assert_eq!(
            self.revision, previous.revision,
            "{why}: original slot revision changed"
        );
    }
}
