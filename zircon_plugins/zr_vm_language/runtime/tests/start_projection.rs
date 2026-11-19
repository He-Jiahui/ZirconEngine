//! Dispatches the linked ZrVM scene system with a diagnostic export backend, not a native guest.

use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde_json::{json, Value};
use zircon_plugin_zr_vm_language_runtime::{plugin_registration, ZR_VM_LANGUAGE_MODULE_NAME};
use zircon_runtime::builtin::RuntimeModuleCompositionCompiler;
use zircon_runtime::core::framework::platform::RuntimeTargetMode;
use zircon_runtime::core::framework::project::ProjectPluginManifest;
use zircon_runtime::core::framework::script::ScriptHostValue;
use zircon_runtime::core::{CoreRuntime, EngineTaskGraphOptions};
use zircon_runtime::plugin::{RuntimePluginBridgeLifecycleState, RuntimePluginCatalog};
use zircon_runtime::scene::{
    create_level, install_world_runtime_extension_plan, EntityId, LevelMetadata, LevelSystem,
    NodeKind, SystemStage, World, WorldDriver, WorldReplacementReason, WorldReplacementRequest,
    WORLD_DRIVER_NAME,
};
use zircon_runtime::script::{
    CapabilitySet, HostExportRegistry, PluginSlotId, VmBackend, VmBackendFamily, VmError,
    VmPluginHostContext, VmPluginInstance, VmPluginManager, VmPluginManifest, VmPluginPackage,
    VmPluginSlotRecord, VmPluginSlotState, SCRIPT_SCENE_UPDATE_SYSTEM, VM_PLUGIN_MANAGER_NAME,
};

const BINDINGS: &str = "script.bindings";
const PACKAGE: &str = "start_projection_fixture";
const MODULE: &str = "main";
const FAMILY: &str = "start-projection";
const SELECTOR: &str = "start-projection:recording";
const COLOR_A: f64 = 0.25;
const COLOR_B: f64 = 0.5;
const COLOR_EDIT: f64 = 0.75;

#[test]
fn start_projection_preserves_unedited_binding_across_property_write() {
    let fixture = Fixture::new(1);
    let [a, b] = fixture.entities;
    let first = fixture.tick();
    assert_eq!(exports_for(&first, a), [start(COLOR_A), update(COLOR_A)]);
    assert_eq!(exports_for(&first, b), [start(COLOR_B), update(COLOR_B)]);

    let handle = fixture.level.world_handle();
    let epoch = fixture.level.capture_world_replacement_epoch();
    let b_binding = fixture.level.with_world(|world| binding(world, b).clone());
    fixture.level.with_world_mut(|world| {
        let before = bindings_generation(world);
        let mut authored = binding(world, a).clone();
        let color = authored[0]["properties"]
            .get_mut("color")
            .expect("edit an existing authored color, not a new binding");
        assert_eq!(color.as_f64(), Some(COLOR_A));
        *color = json!(COLOR_EDIT);
        assert!(world.set_dynamic_component(a, BINDINGS, authored).unwrap());
        assert!(bindings_generation(world) > before);
        assert!(world.contains_entity(a) && world.contains_entity(b));
        assert_eq!(binding(world, b), &b_binding);
    });
    assert_eq!(fixture.level.world_handle(), handle);
    assert_eq!(fixture.level.capture_world_replacement_epoch(), epoch);

    let second = fixture.tick();
    // A's new properties must be visible through the real host getter. This test
    // does not prescribe whether the *edited* binding executes onStart again.
    let a_updates: Vec<_> = exports_for(&second, a)
        .into_iter()
        .filter(|(export, _, _)| *export == "onUpdate")
        .collect();
    assert_eq!(a_updates, [update(COLOR_EDIT)]);
    assert_eq!(
        exports_for(&second, b),
        [update(COLOR_B)],
        "only A changed; B must update without executing onStart again"
    );
}

#[test]
fn start_projection_equal_write_preserves_duplicate_slots() {
    let fixture = Fixture::new(2);
    let [a, b] = fixture.entities;
    let first = fixture.tick();
    assert_eq!(exports_for(&first, a), [start(COLOR_A), update(COLOR_A)]);
    assert_eq!(
        exports_for(&first, b),
        [
            start(COLOR_B),
            update(COLOR_B),
            start(COLOR_B),
            update(COLOR_B)
        ],
        "two identical authored slots must each start"
    );

    let epoch = fixture.level.capture_world_replacement_epoch();
    fixture.level.with_world_mut(|world| {
        let before = (bindings_generation(world), world.world_generation());
        let equal = binding(world, a).clone();
        assert!(!world.set_dynamic_component(a, BINDINGS, equal).unwrap());
        assert_eq!(
            (bindings_generation(world), world.world_generation()),
            before
        );
    });
    assert_eq!(fixture.level.capture_world_replacement_epoch(), epoch);

    let second = fixture.tick();
    assert_eq!(exports_for(&second, a), [update(COLOR_A)]);
    assert_eq!(exports_for(&second, b), [update(COLOR_B), update(COLOR_B)]);
}

#[test]
fn start_projection_world_replacement_starts_new_instances() {
    let fixture = Fixture::new(1);
    let [a, b] = fixture.entities;
    let first = fixture.tick();
    assert_eq!(exports_for(&first, a), [start(COLOR_A), update(COLOR_A)]);
    assert_eq!(exports_for(&first, b), [start(COLOR_B), update(COLOR_B)]);

    let handle = fixture.level.world_handle();
    let expected = fixture.level.capture_world_replacement_expectation();
    let (mut replacement, ids) = authored_world(1);
    assert_eq!(
        ids, fixture.entities,
        "a new World can reuse numeric entity IDs"
    );
    let generation = fixture.level.with_world(|world| {
        for entity in ids {
            assert_eq!(binding(world, entity), binding(&replacement, entity));
        }
        bindings_generation(world)
    });
    assert_eq!(bindings_generation(&replacement), generation);
    fixture
        .runtime
        .resolve_driver::<WorldDriver>(WORLD_DRIVER_NAME)
        .unwrap()
        .apply_world_runtime_extensions(&mut replacement)
        .unwrap();
    let commit = fixture
        .level
        .replace_world_with_time_policy(
            &expected,
            WorldReplacementRequest::preserve(replacement),
            WorldReplacementReason::ExplicitHostRequest,
        )
        .expect("publish a genuinely new World through the Level owner");
    assert_eq!(commit.receipt().previous(), expected.identity());
    assert!(
        commit.receipt().current().replacement_epoch() > expected.identity().replacement_epoch()
    );
    drop(commit);
    assert_eq!(fixture.level.world_handle(), handle);
    assert!(fixture.level.with_world(bindings_generation) > generation);

    let second = fixture.tick();
    assert_eq!(exports_for(&second, a), [start(COLOR_A), update(COLOR_A)]);
    assert_eq!(exports_for(&second, b), [start(COLOR_B), update(COLOR_B)]);
}

struct Fixture {
    level: LevelSystem,
    runtime: CoreRuntime,
    entities: [EntityId; 2],
    stage: SystemStage,
    slot: VmPluginSlotRecord,
    recorder: Recorder,
}

impl Fixture {
    fn new(b_slots: usize) -> Self {
        let mut registration = plugin_registration();
        assert!(registration.is_success(), "{:?}", registration.diagnostics);
        // The host explicitly selects an opt-in plugin, as linked Editor Play does.
        // No descriptor, dependency, or callback is replaced by the fixture.
        registration.project_selection.enabled = true;
        let mut selection = registration.project_selection.clone();
        selection.required = true;
        let manifest = ProjectPluginManifest {
            selections: vec![selection],
        };
        let catalog = RuntimePluginCatalog::from_registration_reports([registration], []);
        let plan = catalog.compiled_project_plan(&manifest, RuntimeTargetMode::ClientRuntime);
        let composition = RuntimeModuleCompositionCompiler::new(&plan)
            .compile()
            .expect("compose the full linked runtime graph");
        let registry = &plan.runtime_extensions().registry;
        let (owner, system) = registry
            .plugin_runtime_systems()
            .find(|(_, system)| system.id == SCRIPT_SCENE_UPDATE_SYSTEM)
            .expect("the linked plugin must contribute the scene Update system");
        assert_eq!(
            registry.plugin_module_name(owner),
            Some(ZR_VM_LANGUAGE_MODULE_NAME)
        );
        let stage = system.stage;
        assert_eq!(stage, SystemStage::Update);

        let runtime = CoreRuntime::try_with_task_graph_options(
            EngineTaskGraphOptions::with_worker_threads(3),
        )
        .expect("create the runtime task graph");
        let core = runtime.handle();
        for descriptor in composition.module_descriptors() {
            runtime.register_module(descriptor.clone()).unwrap();
        }
        core.install_runtime_module_lifecycle_observer(Arc::new(
            RuntimePluginBridgeLifecycleState::from_extension_report(
                catalog,
                plan.runtime_extensions_handle(),
            ),
        ));
        // Activate the actual dependency closure, not unrelated desktop services/configuration.
        runtime
            .activate_module(ZR_VM_LANGUAGE_MODULE_NAME)
            .expect("activate the registered module and its real dependencies");
        install_world_runtime_extension_plan(
            &core,
            registry.world_runtime_extension_plan().unwrap(),
        )
        .expect("apply the selected catalog's executable contributions through WorldDriver");

        let recorder = Recorder::default();
        let slot = {
            let manager = runtime
                .resolve_manager::<VmPluginManager>(VM_PLUGIN_MANAGER_NAME)
                .unwrap();
            manager.register_family(Arc::new(recorder.clone()));
            let id = manager
                .load_package_with_backend(
                    SELECTOR,
                    VmPluginPackage {
                        manifest: VmPluginManifest {
                            name: PACKAGE.to_owned(),
                            version: "1.0.0".to_owned(),
                            entry: MODULE.to_owned(),
                            capabilities: CapabilitySet::default().with("gameplay.entity"),
                            management: Default::default(),
                        },
                        zr_vm_project: None,
                        bytecode: Vec::new(),
                    },
                )
                .expect("publish a real active slot with diagnostic exports");
            let slot = manager.slot(id).unwrap();
            assert_eq!(slot.state, VmPluginSlotState::Active);
            slot
        };
        let (world, entities) = authored_world(b_slots);
        let level = create_level(&core, world, LevelMetadata::default()).unwrap();
        Self {
            level,
            runtime,
            entities,
            stage,
            slot,
            recorder,
        }
    }

    fn tick(&self) -> Vec<ExportCall> {
        // Positive virtual delta for Update, no fixed steps in this witness.
        let frame = self.runtime.advance_time_by(Duration::from_millis(16), 0);
        self.level
            .tick(&self.runtime.handle(), frame)
            .expect("execute the registered scene frame");
        let calls = std::mem::take(&mut *self.recorder.0.lock().unwrap());
        eprintln!(
            "START_PROJECTION frame={} registered_stage={:?} successful={calls:?}",
            frame.outer_frame_index(),
            self.stage
        );
        let manager = self
            .runtime
            .resolve_manager::<VmPluginManager>(VM_PLUGIN_MANAGER_NAME)
            .unwrap();
        let id = manager.slot_for_package_name(PACKAGE).unwrap();
        assert_eq!(manager.slot(id).unwrap(), self.slot);
        for call in &calls {
            assert_eq!(call.owner, (id, self.slot.generation));
            assert_eq!(call.module, MODULE);
            assert!(self.entities.contains(&call.entity));
        }
        calls
    }
}

fn authored_world(b_slots: usize) -> (World, [EntityId; 2]) {
    let mut world = World::empty();
    let [a, b] = [(); 2].map(|_| world.spawn_node(NodeKind::Empty).unwrap());
    assert_ne!(a, b);
    for (entity, count, color) in [(a, 1, COLOR_A), (b, b_slots, COLOR_B)] {
        let slot = json!({ "package": PACKAGE, "module": MODULE, "enabled": true,
            "update": true, "fixed_update": false, "properties": {"color": color} });
        assert!(world
            .set_dynamic_component(entity, BINDINGS, Value::Array(vec![slot; count]))
            .unwrap());
    }
    (world, [a, b])
}

fn bindings_generation(world: &World) -> u64 {
    world.dynamic_component_generation(BINDINGS)
}

fn binding(world: &World, entity: EntityId) -> &Value {
    world
        .dynamic_component(entity, BINDINGS)
        .expect("authored binding")
}

#[derive(Debug)]
struct ExportCall {
    owner: (PluginSlotId, u32),
    entity: EntityId,
    module: String,
    export: String,
    reply: i64,
    color: f64,
}

fn exports_for(calls: &[ExportCall], entity: EntityId) -> Vec<(&str, i64, f64)> {
    calls
        .iter()
        .filter(|call| call.entity == entity)
        .map(|call| (call.export.as_str(), call.reply, call.color))
        .collect()
}

fn start(color: f64) -> (&'static str, i64, f64) {
    ("onStart", 101, color)
}

fn update(color: f64) -> (&'static str, i64, f64) {
    ("onUpdate", 202, color)
}

// Only the guest boundary is replaced. The coordinator creates the real owner
// and the scene dispatcher is the only code here that can reach call_export.
#[derive(Clone, Default)]
struct Recorder(Arc<Mutex<Vec<ExportCall>>>);

impl VmBackendFamily for Recorder {
    fn family_name(&self) -> &str {
        FAMILY
    }

    fn resolve(&self, selector: &str) -> Result<Arc<dyn VmBackend>, VmError> {
        if selector != SELECTOR {
            return Err(VmError::UnknownBackend(selector.to_owned()));
        }
        Ok(Arc::new(self.clone()))
    }

    fn visit_selectors(&self, visitor: &mut dyn FnMut(&str)) {
        visitor(SELECTOR);
    }
}

impl VmBackend for Recorder {
    fn backend_name(&self) -> &str {
        SELECTOR
    }

    fn load_package(
        &self,
        package: &VmPluginPackage,
        host: &VmPluginHostContext,
    ) -> Result<Box<dyn VmPluginInstance>, VmError> {
        let owner = host
            .vm_owner()
            .ok_or_else(|| VmError::Operation("no active slot owner".into()))?;
        Ok(Box::new(RecordingInstance {
            manifest: package.manifest.clone(),
            exports: host.host_exports.clone(),
            capabilities: host.capabilities.clone(),
            owner,
            recorder: self.clone(),
        }))
    }
}

struct RecordingInstance {
    manifest: VmPluginManifest,
    exports: HostExportRegistry,
    capabilities: CapabilitySet,
    owner: (PluginSlotId, u32),
    recorder: Recorder,
}

impl VmPluginInstance for RecordingInstance {
    fn manifest(&self) -> &VmPluginManifest {
        &self.manifest
    }

    fn call_export(
        &mut self,
        module: &str,
        export: &str,
        args: &[ScriptHostValue],
    ) -> Result<Option<ScriptHostValue>, VmError> {
        let reply = match (module == self.manifest.entry.as_str(), export) {
            (true, "onStart") => 101,
            (true, "onUpdate") => 202,
            _ => {
                return Err(VmError::Operation(format!(
                    "unexpected export: {module}::{export}"
                )))
            }
        };
        let [ScriptHostValue::Int(entity), ScriptHostValue::Float(delta)] = args else {
            return Err(VmError::Operation(format!(
                "unexpected scene export arguments: {args:?}"
            )));
        };
        if *entity <= 0 || !delta.is_finite() || *delta <= 0.0 {
            return Err(VmError::Operation(format!(
                "invalid scene export arguments: {args:?}"
            )));
        }
        let color = self.exports.call_with_capabilities(
            "zr.zircon.gameplay",
            "script_number",
            vec![
                ScriptHostValue::Int(*entity),
                ScriptHostValue::String("color".into()),
                ScriptHostValue::Float(-1.0),
            ],
            &self.capabilities,
        )?;
        let ScriptHostValue::Float(color) = color else {
            return Err(VmError::Operation(format!(
                "expected script_number float, got {color:?}"
            )));
        };
        self.recorder.0.lock().unwrap().push(ExportCall {
            owner: self.owner,
            entity: *entity as EntityId,
            module: module.to_owned(),
            export: export.to_owned(),
            reply,
            color,
        });
        Ok(Some(ScriptHostValue::Int(reply)))
    }
}

#[test]
fn start_projection_reinstalled_row_restarts_only_its_own_binding() {
    assert_only_b_restarts(|world, b, original| {
        let before = bindings_generation(world);
        assert!(world.remove_dynamic_component(b, BINDINGS).unwrap());
        assert!(world.contains_scene_node(b), "the actor was not removed");
        assert!(world.dynamic_component(b, BINDINGS).is_none());
        let removed = bindings_generation(world);
        assert!(removed > before);

        assert!(world
            .set_dynamic_component(b, BINDINGS, original.clone())
            .unwrap());
        assert!(bindings_generation(world) > removed);
    });
}

#[test]
fn start_projection_value_round_trip_restarts_only_its_own_binding() {
    assert_only_b_restarts(|world, b, original| {
        let before = bindings_generation(world);
        let mut changed = original.clone();
        let color = changed[0]["properties"].get_mut("color").unwrap();
        assert_eq!(color.as_f64(), Some(COLOR_B));
        *color = json!(COLOR_EDIT);
        assert!(world.set_dynamic_component(b, BINDINGS, changed).unwrap());
        assert_eq!(
            binding(world, b)[0]["properties"]["color"].as_f64(),
            Some(COLOR_EDIT)
        );
        let edited = bindings_generation(world);
        assert!(edited > before);

        assert!(world
            .set_dynamic_component(b, BINDINGS, original.clone())
            .unwrap());
        assert!(bindings_generation(world) > edited);
    });
}

fn assert_only_b_restarts(change_b: impl FnOnce(&mut World, EntityId, &Value)) {
    let fixture = Fixture::new(1);
    let [a, b] = fixture.entities;
    let first = fixture.tick();
    assert_eq!(exports_for(&first, a), [start(COLOR_A), update(COLOR_A)]);
    assert_eq!(exports_for(&first, b), [start(COLOR_B), update(COLOR_B)]);

    let handle = fixture.level.world_handle();
    let epoch = fixture.level.capture_world_replacement_epoch();
    // No intervening frame observes the row absent or its temporary JSON value.
    fixture.level.with_world_mut(|world| {
        let saved_a = binding(world, a).clone();
        let saved_b = binding(world, b).clone();
        change_b(world, b, &saved_b);
        assert!(world.contains_scene_node(a) && world.contains_scene_node(b));
        assert_eq!(binding(world, a), &saved_a);
        assert_eq!(binding(world, b), &saved_b, "the final JSON is identical");
    });
    assert_eq!(fixture.level.world_handle(), handle);
    assert_eq!(fixture.level.capture_world_replacement_epoch(), epoch);

    let second = fixture.tick();
    assert_eq!(
        exports_for(&second, b),
        [start(COLOR_B), update(COLOR_B)],
        "B's real binding mutation must not carry the old activation"
    );
    assert_eq!(
        exports_for(&second, a),
        [update(COLOR_A)],
        "never-written A must update without executing onStart again"
    );
}

#[test]
fn start_projection_new_core_with_same_numeric_ids_starts_its_own_bindings() {
    let first = Fixture::new(1);
    let next = Fixture::new(1);
    let [a, b] = first.entities;
    let first_owner = first
        .runtime
        .resolve_manager_handle::<VmPluginManager>(VM_PLUGIN_MANAGER_NAME)
        .unwrap();
    let next_owner = next
        .runtime
        .resolve_manager_handle::<VmPluginManager>(VM_PLUGIN_MANAGER_NAME)
        .unwrap();
    assert_ne!(first_owner, next_owner, "independent Core service owners");
    assert_eq!(first.level.world_handle(), next.level.world_handle());
    assert_eq!(
        first.level.capture_world_replacement_epoch(),
        next.level.capture_world_replacement_epoch()
    );
    assert_eq!(first.entities, next.entities);
    assert_eq!(
        first.slot, next.slot,
        "the numeric VM slot and generation can coincide too"
    );
    let input = |fixture: &Fixture| {
        fixture.level.with_world(|world| {
            assert!(world.contains_scene_node(a) && world.contains_scene_node(b));
            (
                bindings_generation(world),
                binding(world, a).clone(),
                binding(world, b).clone(),
            )
        })
    };
    assert_eq!(input(&first), input(&next));

    // Let the second owner legally resolve both symbols, without running them.
    // If an old dense handle aliases it, the Update should still be observable;
    // a missing dense target must not hide the missing onStart in this test.
    {
        let manager = next_owner.enter().unwrap();
        for export in ["onStart", "onUpdate"] {
            manager
                .resolve_package_callback(PACKAGE, MODULE, export)
                .unwrap();
        }
    }
    assert!(first.recorder.0.lock().unwrap().is_empty());
    assert!(next.recorder.0.lock().unwrap().is_empty());

    let expected = (
        vec![start(COLOR_A), update(COLOR_A)],
        vec![start(COLOR_B), update(COLOR_B)],
    );
    // Keep both owners alive and dispatch their first frames on this same thread.
    let first_frame = first.tick();
    assert_eq!(
        (exports_for(&first_frame, a), exports_for(&first_frame, b)),
        expected
    );
    let next_frame = next.tick();
    assert!(first.recorder.0.lock().unwrap().is_empty());
    assert_eq!(
        (exports_for(&next_frame, a), exports_for(&next_frame, b)),
        expected,
        "a different Core/Level must not inherit either binding's started state"
    );
}
