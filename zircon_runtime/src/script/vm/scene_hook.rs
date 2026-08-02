use std::{
    cell::{Cell, RefCell},
    collections::{BTreeMap, HashMap},
    rc::Rc,
    sync::OnceLock,
};

use serde::Deserialize;

mod error;

use crate::core::framework::script::ScriptHostValue;
use crate::core::math::Real;
use crate::core::{CoreError, CoreHandle};
use crate::diagnostic_log::write_log_lazy;
use crate::scene::{EntityId, LevelSystem, SystemStage, World};
use crate::scene::{
    SceneRuntimeHook, SceneRuntimeHookContext, SceneRuntimeHookDescriptor,
    SceneRuntimeHookRegistration,
};
use crate::script::{
    with_script_runtime_call_context, ScriptRuntimeCallContext, VmCallbackHandle, VmPluginManager,
    VM_PLUGIN_MANAGER_NAME,
};

use self::error::{ScriptSceneHookError, ScriptSceneHookResult};

const SCRIPT_BINDINGS_COMPONENT: &str = "script.bindings";
const SCRIPT_HOOK_PLUGIN_ID: &str = "zr_vm_language";
const TRACE_SCRIPT_BINDINGS_ENV: &str = "ZIRCON_TRACE_SCRIPT_BINDINGS";

thread_local! {
    // Projection state stays local to the script execution thread, avoiding a global cache lock.
    static ACTIVE_SCRIPT_BINDING_PROJECTION: RefCell<Option<ScriptBindingProjectionCache>> =
        const { RefCell::new(None) };
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScriptSceneLifecyclePhase {
    FixedUpdate,
    Update,
}

#[derive(Clone, Debug)]
pub struct ScriptSceneRuntimeHook {
    phase: ScriptSceneLifecyclePhase,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
struct RuntimeSceneScriptBinding {
    package: String,
    module: String,
    #[serde(default = "default_true")]
    enabled: bool,
    #[serde(default = "default_true")]
    update: bool,
    #[serde(default = "default_true")]
    fixed_update: bool,
    #[serde(default)]
    properties: BTreeMap<String, serde_json::Value>,
}

#[derive(Debug)]
struct ActiveScriptBinding {
    entity: EntityId,
    package: String,
    module: String,
    binding_key: String,
    started: Cell<bool>,
    callbacks: RefCell<ScriptBindingCallbackHandles>,
}

#[derive(Debug, Default)]
struct ScriptBindingCallbackHandles {
    on_start: Option<VmCallbackHandle>,
    on_update: Option<VmCallbackHandle>,
    on_fixed_update: Option<VmCallbackHandle>,
}

impl ScriptBindingCallbackHandles {
    fn take(&mut self, export_name: &str) -> Option<VmCallbackHandle> {
        match export_name {
            "onStart" => self.on_start.take(),
            "onUpdate" => self.on_update.take(),
            "onFixedUpdate" => self.on_fixed_update.take(),
            _ => None,
        }
    }

    fn replace(&mut self, export_name: &str, handle: VmCallbackHandle) {
        match export_name {
            "onStart" => self.on_start = Some(handle),
            "onUpdate" => self.on_update = Some(handle),
            "onFixedUpdate" => self.on_fixed_update = Some(handle),
            _ => {}
        }
    }
}

#[derive(Debug)]
struct ActiveScriptBindingProjection {
    binding_generation: u64,
    fixed_update_bindings: Vec<Rc<ActiveScriptBinding>>,
    update_bindings: Vec<Rc<ActiveScriptBinding>>,
    property_matches: HashMap<String, HashMap<String, Vec<EntityId>>>,
    numeric_properties: HashMap<EntityId, HashMap<String, f64>>,
}

impl ActiveScriptBindingProjection {
    fn bindings_for_phase(&self, phase: ScriptSceneLifecyclePhase) -> &[Rc<ActiveScriptBinding>] {
        match phase {
            ScriptSceneLifecyclePhase::FixedUpdate => &self.fixed_update_bindings,
            ScriptSceneLifecyclePhase::Update => &self.update_bindings,
        }
    }

    fn property_matches(&self, property: &str, expected_value: &str) -> &[EntityId] {
        self.property_matches
            .get(property)
            .and_then(|expected_values| expected_values.get(expected_value))
            .map(Vec::as_slice)
            .unwrap_or_default()
    }

    fn number_for_entity(&self, entity: EntityId, property: &str) -> Option<f64> {
        self.numeric_properties
            .get(&entity)
            .and_then(|properties| properties.get(property))
            .copied()
    }
}

#[derive(Debug)]
struct ScriptBindingProjectionCache {
    world_handle: u64,
    projection: Rc<ActiveScriptBindingProjection>,
}

impl ScriptSceneRuntimeHook {
    pub fn fixed_update() -> Self {
        Self {
            phase: ScriptSceneLifecyclePhase::FixedUpdate,
        }
    }

    pub fn update() -> Self {
        Self {
            phase: ScriptSceneLifecyclePhase::Update,
        }
    }
}

pub fn script_scene_fixed_update_hook_registration() -> SceneRuntimeHookRegistration {
    SceneRuntimeHookRegistration::new(
        SceneRuntimeHookDescriptor::new(
            "zr_vm_language.script.scene.fixed_update",
            SCRIPT_HOOK_PLUGIN_ID,
            SystemStage::FixedUpdate,
        )
        .with_order(10),
        ScriptSceneRuntimeHook::fixed_update(),
    )
}

pub fn script_scene_update_hook_registration() -> SceneRuntimeHookRegistration {
    SceneRuntimeHookRegistration::new(
        SceneRuntimeHookDescriptor::new(
            "zr_vm_language.script.scene.update",
            SCRIPT_HOOK_PLUGIN_ID,
            SystemStage::Update,
        )
        .with_order(10),
        ScriptSceneRuntimeHook::update(),
    )
}

impl SceneRuntimeHook for ScriptSceneRuntimeHook {
    fn run(&self, context: SceneRuntimeHookContext<'_>) -> Result<(), CoreError> {
        tick_script_bindings(
            context.core,
            context.level,
            context.delta_seconds,
            self.phase,
        )
        .map_err(|error| {
            CoreError::Initialization("ScriptSceneRuntimeHook".to_string(), error.to_string())
        })
    }
}

fn tick_script_bindings(
    core: &CoreHandle,
    level: &LevelSystem,
    delta_seconds: Real,
    phase: ScriptSceneLifecyclePhase,
) -> ScriptSceneHookResult<()> {
    let projection = active_script_binding_projection(level)?;
    let bindings = projection.bindings_for_phase(phase);
    if bindings.is_empty() {
        return Ok(());
    }
    let manager = core
        .resolve_manager::<VmPluginManager>(VM_PLUGIN_MANAGER_NAME)
        .map_err(ScriptSceneHookError::from)?;

    for binding in bindings {
        call_script_binding(core, level, manager.as_ref(), delta_seconds, phase, binding)?;
    }

    Ok(())
}

impl RuntimeSceneScriptBinding {
    fn into_active(self, entity: EntityId, binding_index: usize) -> ActiveScriptBinding {
        let binding_key = format!("{}::{}#{binding_index}", self.package, self.module);
        ActiveScriptBinding {
            entity,
            package: self.package,
            module: self.module,
            binding_key,
            started: Cell::new(false),
            callbacks: RefCell::new(ScriptBindingCallbackHandles::default()),
        }
    }
}

fn call_script_binding(
    core: &CoreHandle,
    level: &LevelSystem,
    manager: &VmPluginManager,
    delta_seconds: Real,
    phase: ScriptSceneLifecyclePhase,
    binding: &ActiveScriptBinding,
) -> ScriptSceneHookResult<()> {
    if phase == ScriptSceneLifecyclePhase::Update && !binding.started.get() {
        call_export_for_binding(core, level, manager, delta_seconds, binding, "onStart")?;
        binding.started.set(true);
    }

    let export_name = match phase {
        ScriptSceneLifecyclePhase::FixedUpdate => "onFixedUpdate",
        ScriptSceneLifecyclePhase::Update => "onUpdate",
    };
    call_export_for_binding(core, level, manager, delta_seconds, binding, export_name)
}

fn call_export_for_binding(
    core: &CoreHandle,
    level: &LevelSystem,
    manager: &VmPluginManager,
    delta_seconds: Real,
    binding: &ActiveScriptBinding,
    export_name: &'static str,
) -> ScriptSceneHookResult<()> {
    let arguments = [
        ScriptHostValue::Int(binding.entity as i64),
        ScriptHostValue::Float(f64::from(delta_seconds)),
    ];
    let call_context = ScriptRuntimeCallContext {
        core: core.downgrade(),
        level: level.clone(),
        entity: binding.entity,
        delta_seconds,
    };
    let mut callback = match binding.callbacks.borrow_mut().take(export_name) {
        Some(callback) => callback,
        None => manager
            .resolve_package_callback(&binding.package, &binding.module, export_name)
            .map_err(|source| {
                ScriptSceneHookError::export_call(
                    binding.binding_key.clone(),
                    export_name,
                    crate::script::VmError::Operation(source.to_string()),
                )
            })?,
    };
    trace_script_binding_export(binding, export_name, "start", None);
    let result = with_script_runtime_call_context(call_context, || {
        manager.invoke_callback(&mut callback, &arguments)
    });
    binding
        .callbacks
        .borrow_mut()
        .replace(export_name, callback);
    trace_script_binding_export(binding, export_name, "done", Some(result.is_ok()));
    result.map(|_| ()).map_err(|source| {
        ScriptSceneHookError::export_call(
            binding.binding_key.clone(),
            export_name,
            crate::script::VmError::Operation(source.to_string()),
        )
    })
}

fn trace_script_binding_export(
    binding: &ActiveScriptBinding,
    export_name: &str,
    phase: &str,
    success: Option<bool>,
) {
    if !trace_script_bindings_enabled() {
        return;
    }
    write_log_lazy("zr_vm_project_backend", || {
        let success = success
            .map(|success| format!(" success={success}"))
            .unwrap_or_default();
        format!(
            "script_binding_export_{phase} package={} module={} entity={} export={export_name}{success}",
            binding.package, binding.module, binding.entity
        )
    });
}

fn trace_script_bindings_enabled() -> bool {
    static ENABLED: OnceLock<bool> = OnceLock::new();
    *ENABLED.get_or_init(|| {
        std::env::var_os(TRACE_SCRIPT_BINDINGS_ENV)
            .filter(|value| !value.is_empty())
            .map(|value| {
                let value = value.to_string_lossy();
                !matches!(
                    value.trim().to_ascii_lowercase().as_str(),
                    "0" | "false" | "off" | "none"
                )
            })
            .unwrap_or(false)
    })
}

fn active_script_binding_projection(
    level: &LevelSystem,
) -> ScriptSceneHookResult<Rc<ActiveScriptBindingProjection>> {
    let world_handle = level.world_handle().get();
    level.with_world(|world| active_script_binding_projection_for_world(world_handle, world))
}

fn active_script_binding_projection_for_world(
    world_handle: u64,
    world: &World,
) -> ScriptSceneHookResult<Rc<ActiveScriptBindingProjection>> {
    let binding_generation = world.dynamic_component_generation(SCRIPT_BINDINGS_COMPONENT);
    if let Some(projection) = ACTIVE_SCRIPT_BINDING_PROJECTION.with(|cache| {
        let cache = cache.borrow();
        cache
            .as_ref()
            .filter(|cached| {
                cached.world_handle == world_handle
                    && cached.projection.binding_generation == binding_generation
            })
            .map(|cached| Rc::clone(&cached.projection))
    }) {
        return Ok(projection);
    }

    let projection = Rc::new(collect_active_script_binding_projection(world)?);
    ACTIVE_SCRIPT_BINDING_PROJECTION.with(|cache| {
        *cache.borrow_mut() = Some(ScriptBindingProjectionCache {
            world_handle,
            projection: Rc::clone(&projection),
        });
    });
    Ok(projection)
}

pub(super) fn with_script_binding_property_matches<R>(
    level: &LevelSystem,
    property: &str,
    expected_value: &str,
    operation: impl FnOnce(&[EntityId], &World) -> R,
) -> ScriptSceneHookResult<R> {
    let world_handle = level.world_handle().get();
    level.with_world(|world| {
        let projection = active_script_binding_projection_for_world(world_handle, world)?;
        Ok(operation(
            projection.property_matches(property, expected_value),
            world,
        ))
    })
}

pub(super) fn script_binding_number_for_entity(
    level: &LevelSystem,
    entity: EntityId,
    property: &str,
) -> ScriptSceneHookResult<Option<f64>> {
    let projection = active_script_binding_projection(level)?;
    Ok(projection.number_for_entity(entity, property))
}

pub(super) fn with_script_binding_number_and_world_mut<R>(
    level: &LevelSystem,
    entity: EntityId,
    property: &str,
    operation: impl FnOnce(Option<f64>, &mut World) -> R,
) -> ScriptSceneHookResult<R> {
    let world_handle = level.world_handle().get();
    level.with_world_mut(|world| {
        let projection = active_script_binding_projection_for_world(world_handle, world)?;
        Ok(operation(
            projection.number_for_entity(entity, property),
            world,
        ))
    })
}

fn collect_active_script_binding_projection(
    world: &World,
) -> ScriptSceneHookResult<ActiveScriptBindingProjection> {
    let mut rows = Vec::new();
    world.dynamic_component_rows(SCRIPT_BINDINGS_COMPONENT, &mut rows);
    let mut fixed_update_bindings = Vec::with_capacity(rows.len());
    let mut update_bindings = Vec::with_capacity(rows.len());
    let mut property_matches = HashMap::<String, HashMap<String, Vec<EntityId>>>::new();
    let mut numeric_properties = HashMap::<EntityId, HashMap<String, f64>>::new();
    for (entity, value) in rows {
        let bindings = serde_json::from_value::<Vec<RuntimeSceneScriptBinding>>(value.clone())
            .map_err(|source| ScriptSceneHookError::invalid_binding_component(entity, source))?;
        for (binding_index, binding) in bindings.into_iter().enumerate() {
            if !binding.enabled {
                continue;
            }
            index_script_binding_properties(
                entity,
                &binding.properties,
                &mut property_matches,
                &mut numeric_properties,
            );
            let fixed_update = binding.fixed_update;
            let update = binding.update;
            let binding = Rc::new(binding.into_active(entity, binding_index));
            if fixed_update {
                fixed_update_bindings.push(Rc::clone(&binding));
            }
            if update {
                update_bindings.push(binding);
            }
        }
    }
    for expected_values in property_matches.values_mut() {
        for entities in expected_values.values_mut() {
            entities.sort_unstable();
            entities.dedup();
        }
    }
    Ok(ActiveScriptBindingProjection {
        binding_generation: world.dynamic_component_generation(SCRIPT_BINDINGS_COMPONENT),
        fixed_update_bindings,
        update_bindings,
        property_matches,
        numeric_properties,
    })
}

fn index_script_binding_properties(
    entity: EntityId,
    properties: &BTreeMap<String, serde_json::Value>,
    property_matches: &mut HashMap<String, HashMap<String, Vec<EntityId>>>,
    numeric_properties: &mut HashMap<EntityId, HashMap<String, f64>>,
) {
    for (property, value) in properties {
        let Some(expected_value) = script_binding_scalar_match_key(value) else {
            continue;
        };
        property_matches
            .entry(property.clone())
            .or_default()
            .entry(expected_value)
            .or_default()
            .push(entity);
        if let Some(number) = value.as_f64() {
            numeric_properties
                .entry(entity)
                .or_default()
                .entry(property.clone())
                .or_insert(number);
        }
    }
}

fn script_binding_scalar_match_key(value: &serde_json::Value) -> Option<String> {
    match value {
        serde_json::Value::String(value) => Some(value.clone()),
        serde_json::Value::Bool(value) => Some(value.to_string()),
        serde_json::Value::Number(value) => Some(value.to_string()),
        _ => None,
    }
}

fn default_true() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;
    use std::sync::{Arc, Mutex};

    use crate::core::framework::scene::WorldHandle;
    use crate::core::math::{Transform, Vec3};
    use crate::scene::components::NodeKind;
    use crate::scene::{LevelMetadata, LevelSystem, World};

    use super::{
        active_script_binding_projection, ScriptSceneLifecyclePhase, SCRIPT_BINDINGS_COMPONENT,
    };

    #[test]
    fn runtime13_script_binding_projection_ignores_unrelated_world_mutations() {
        let mut world = World::empty();
        let entity = world.spawn_node(NodeKind::Empty);
        world
            .set_dynamic_component(
                entity,
                SCRIPT_BINDINGS_COMPONENT,
                serde_json::json!([{
                    "package": "runtime13_projection",
                    "module": "main",
                    "enabled": true,
                    "update": true,
                    "fixed_update": false,
                    "properties": { "role": "player", "hp": 73.0 }
                }]),
            )
            .expect("script bindings component is accepted");
        let level = LevelSystem::new(
            WorldHandle::new(130_013),
            Arc::new(Mutex::new(world)),
            LevelMetadata::default(),
        );

        let first = active_script_binding_projection(&level).expect("first projection");
        let second = active_script_binding_projection(&level).expect("stable projection");
        assert!(Rc::ptr_eq(&first, &second));
        assert_eq!(first.update_bindings.len(), 1);
        assert!(first.fixed_update_bindings.is_empty());
        assert_eq!(first.property_matches("role", "player"), &[entity]);
        assert_eq!(first.number_for_entity(entity, "hp"), Some(73.0));
        assert!(!first.update_bindings[0].started.replace(true));

        level
            .with_world_mut(|world| {
                world.update_transform(
                    entity,
                    Transform::from_translation(Vec3::new(3.0, 0.0, 0.0)),
                )
            })
            .expect("transform update succeeds");

        let after_transform_update =
            active_script_binding_projection(&level).expect("transform projection");
        assert!(Rc::ptr_eq(&first, &after_transform_update));
        assert!(after_transform_update.update_bindings[0].started.get());

        level
            .with_world_mut(|world| {
                world.set_dynamic_component(entity, "runtime13.unrelated", serde_json::json!(true))
            })
            .expect("unrelated component update succeeds");

        let after_unrelated_update =
            active_script_binding_projection(&level).expect("unrelated projection");
        assert!(Rc::ptr_eq(&first, &after_unrelated_update));
        assert!(after_unrelated_update.update_bindings[0].started.get());

        level
            .with_world_mut(|world| {
                world.set_dynamic_component(
                    entity,
                    SCRIPT_BINDINGS_COMPONENT,
                    serde_json::json!([{
                        "package": "runtime13_projection",
                        "module": "replacement",
                        "enabled": true,
                        "update": true,
                        "fixed_update": false,
                        "properties": { "role": "enemy", "hp": 19.0 }
                    }]),
                )
            })
            .expect("generation-changing update succeeds");

        let updated = active_script_binding_projection(&level).expect("updated projection");
        assert!(!Rc::ptr_eq(&first, &updated));
        assert_eq!(updated.update_bindings[0].module, "replacement");
        assert_eq!(updated.property_matches("role", "enemy"), &[entity]);
        assert_eq!(updated.number_for_entity(entity, "hp"), Some(19.0));

        level.with_world_mut(|world| assert!(world.remove_entity(entity)));
        let after_removal = active_script_binding_projection(&level).expect("removed projection");
        assert!(!Rc::ptr_eq(&updated, &after_removal));
        assert!(after_removal.update_bindings.is_empty());
    }

    #[test]
    fn runtime13_projection_rebuilds_after_deserialized_world_replacement() {
        let level = LevelSystem::new(
            WorldHandle::new(130_015),
            Arc::new(Mutex::new(World::empty())),
            LevelMetadata::default(),
        );
        let empty_projection =
            active_script_binding_projection(&level).expect("empty world projection");
        assert!(empty_projection.update_bindings.is_empty());

        let mut persisted = World::empty();
        let entity = persisted.spawn_node(NodeKind::Empty);
        persisted
            .set_dynamic_component(
                entity,
                SCRIPT_BINDINGS_COMPONENT,
                serde_json::json!([{
                    "package": "runtime13_projection",
                    "module": "restored",
                    "enabled": true,
                    "update": true
                }]),
            )
            .expect("script bindings component is accepted");
        let replacement = serde_json::from_value(
            serde_json::to_value(&persisted).expect("world serialization succeeds"),
        )
        .expect("world deserialization succeeds");
        assert_eq!(
            replacement.dynamic_component_generation(SCRIPT_BINDINGS_COMPONENT),
            0,
            "deserialization resets runtime-only component revisions"
        );

        level.replace(replacement);

        let restored_projection =
            active_script_binding_projection(&level).expect("restored world projection");
        assert!(!Rc::ptr_eq(&empty_projection, &restored_projection));
        assert_eq!(restored_projection.update_bindings.len(), 1);
        assert_eq!(restored_projection.update_bindings[0].entity, entity);
        assert_eq!(restored_projection.update_bindings[0].module, "restored");
    }

    #[test]
    fn runtime13_projection_invalidates_when_world_replacement_removes_bindings() {
        let mut persisted = World::empty();
        let entity = persisted.spawn_node(NodeKind::Empty);
        persisted
            .set_dynamic_component(
                entity,
                SCRIPT_BINDINGS_COMPONENT,
                serde_json::json!([{
                    "package": "runtime13_projection",
                    "module": "retired",
                    "enabled": true,
                    "update": true
                }]),
            )
            .expect("script bindings component is accepted");
        let source = serde_json::from_value(
            serde_json::to_value(&persisted).expect("world serialization succeeds"),
        )
        .expect("world deserialization succeeds");
        assert_eq!(
            source.dynamic_component_generation(SCRIPT_BINDINGS_COMPONENT),
            0,
            "deserialization starts without a runtime component revision"
        );

        let level = LevelSystem::new(
            WorldHandle::new(130_016),
            Arc::new(Mutex::new(source)),
            LevelMetadata::default(),
        );
        let before = active_script_binding_projection(&level).expect("binding projection");
        assert_eq!(before.update_bindings.len(), 1);

        level.replace(World::empty());

        let after = active_script_binding_projection(&level).expect("empty replacement projection");
        assert!(!Rc::ptr_eq(&before, &after));
        assert!(after.update_bindings.is_empty());
    }

    #[test]
    fn runtime13_projection_invalidates_when_staged_world_removes_bindings() {
        let mut persisted = World::empty();
        let entity = persisted.spawn_node(NodeKind::Empty);
        persisted
            .set_dynamic_component(
                entity,
                SCRIPT_BINDINGS_COMPONENT,
                serde_json::json!([{
                    "package": "runtime13_projection",
                    "module": "retired_transactionally",
                    "enabled": true,
                    "update": true
                }]),
            )
            .expect("script bindings component is accepted");
        let source = serde_json::from_value(
            serde_json::to_value(&persisted).expect("world serialization succeeds"),
        )
        .expect("world deserialization succeeds");
        let level = LevelSystem::new(
            WorldHandle::new(130_017),
            Arc::new(Mutex::new(source)),
            LevelMetadata::default(),
        );
        let before = active_script_binding_projection(&level).expect("binding projection");
        let expected_generation = level.with_world(World::world_generation);

        level
            .replace_world_if_generation(expected_generation, World::empty())
            .expect("current generation commits staged world");

        let after = active_script_binding_projection(&level).expect("empty staged projection");
        assert!(!Rc::ptr_eq(&before, &after));
        assert!(after.update_bindings.is_empty());
    }

    #[test]
    fn runtime13_projection_keeps_duplicate_activation_and_reload_start_state_independent() {
        let mut world = World::empty();
        let entity = world.spawn_node(NodeKind::Empty);
        world
            .set_dynamic_component(
                entity,
                SCRIPT_BINDINGS_COMPONENT,
                serde_json::json!([
                    { "package": "runtime13_projection", "module": "main", "update": true },
                    { "package": "runtime13_projection", "module": "main", "update": true }
                ]),
            )
            .expect("duplicate bindings are accepted");
        let level = LevelSystem::new(
            WorldHandle::new(130_014),
            Arc::new(Mutex::new(world)),
            LevelMetadata::default(),
        );

        let initial = active_script_binding_projection(&level).expect("initial projection");
        let bindings = initial.bindings_for_phase(ScriptSceneLifecyclePhase::Update);
        assert_eq!(bindings.len(), 2);
        assert_ne!(bindings[0].binding_key, bindings[1].binding_key);
        assert!(!bindings[0].started.replace(true));
        assert!(!bindings[1].started.replace(true));

        level
            .with_world_mut(|world| {
                world.set_dynamic_component(
                    entity,
                    SCRIPT_BINDINGS_COMPONENT,
                    serde_json::json!([
                        { "package": "runtime13_projection", "module": "main", "enabled": false, "update": true },
                        { "package": "runtime13_projection", "module": "main", "update": true }
                    ]),
                )
            })
            .expect("disabled binding update succeeds");

        let reloaded = active_script_binding_projection(&level).expect("reloaded projection");
        let bindings = reloaded.bindings_for_phase(ScriptSceneLifecyclePhase::Update);
        assert_eq!(bindings.len(), 1);
        assert!(!bindings[0].started.get());
    }
}
