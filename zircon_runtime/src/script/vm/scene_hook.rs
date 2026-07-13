use std::{collections::BTreeMap, sync::OnceLock};

use serde::{Deserialize, Serialize};

mod error;

use crate::core::framework::script::ScriptHostValue;
use crate::core::math::Real;
use crate::core::{CoreError, CoreHandle};
use crate::diagnostic_log::write_log;
use crate::scene::{EntityId, LevelSystem, SystemStage};
use crate::scene::{
    SceneRuntimeHook, SceneRuntimeHookContext, SceneRuntimeHookDescriptor,
    SceneRuntimeHookRegistration,
};
use crate::script::{
    with_script_runtime_call_context, ScriptRuntimeCallContext, VmPluginManager,
    VM_PLUGIN_MANAGER_NAME,
};

use self::error::{ScriptSceneHookError, ScriptSceneHookResult};

const SCRIPT_BINDINGS_COMPONENT: &str = "script.bindings";
const SCRIPT_HOOK_PLUGIN_ID: &str = "zr_vm_language";
const TRACE_SCRIPT_BINDINGS_ENV: &str = "ZIRCON_TRACE_SCRIPT_BINDINGS";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScriptSceneLifecyclePhase {
    FixedUpdate,
    Update,
}

#[derive(Clone, Debug)]
pub struct ScriptSceneRuntimeHook {
    phase: ScriptSceneLifecyclePhase,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct RuntimeSceneScriptBinding {
    package: String,
    module: String,
    #[serde(default = "default_true")]
    enabled: bool,
    #[serde(default = "default_true")]
    update: bool,
    #[serde(default = "default_true")]
    fixed_update: bool,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    properties: BTreeMap<String, serde_json::Value>,
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
    let bindings = active_script_bindings_for_phase(collect_script_bindings(level)?, phase);
    if bindings.is_empty() {
        return Ok(());
    }
    let manager = core
        .resolve_manager::<VmPluginManager>(VM_PLUGIN_MANAGER_NAME)
        .map_err(ScriptSceneHookError::from)?;

    for entity_bindings in bindings {
        for binding in entity_bindings.bindings {
            call_script_binding(
                core,
                level,
                manager.as_ref(),
                delta_seconds,
                phase,
                entity_bindings.entity,
                binding,
            )?;
        }
    }

    Ok(())
}

fn active_script_bindings_for_phase(
    bindings: Vec<EntityScriptBindings>,
    phase: ScriptSceneLifecyclePhase,
) -> Vec<EntityScriptBindings> {
    bindings
        .into_iter()
        .filter_map(|entity_bindings| {
            let bindings = entity_bindings
                .bindings
                .into_iter()
                .filter(|binding| binding.enabled && binding.runs_in_phase(phase))
                .collect::<Vec<_>>();
            if bindings.is_empty() {
                None
            } else {
                Some(EntityScriptBindings {
                    entity: entity_bindings.entity,
                    bindings,
                })
            }
        })
        .collect()
}

impl RuntimeSceneScriptBinding {
    fn runs_in_phase(&self, phase: ScriptSceneLifecyclePhase) -> bool {
        match phase {
            ScriptSceneLifecyclePhase::FixedUpdate => self.fixed_update,
            ScriptSceneLifecyclePhase::Update => self.update,
        }
    }
}

fn call_script_binding(
    core: &CoreHandle,
    level: &LevelSystem,
    manager: &VmPluginManager,
    delta_seconds: Real,
    phase: ScriptSceneLifecyclePhase,
    entity: EntityId,
    binding: RuntimeSceneScriptBinding,
) -> ScriptSceneHookResult<()> {
    let binding_key = binding_key(&binding);
    if phase == ScriptSceneLifecyclePhase::Update && !binding_started(level, entity, &binding_key) {
        call_export_for_binding(
            core,
            level,
            manager,
            delta_seconds,
            entity,
            &binding,
            "onStart",
        )?;
        mark_binding_started(level, entity, &binding_key);
    }

    let export_name = match phase {
        ScriptSceneLifecyclePhase::FixedUpdate => "onFixedUpdate",
        ScriptSceneLifecyclePhase::Update => "onUpdate",
    };
    call_export_for_binding(
        core,
        level,
        manager,
        delta_seconds,
        entity,
        &binding,
        export_name,
    )
}

fn call_export_for_binding(
    core: &CoreHandle,
    level: &LevelSystem,
    manager: &VmPluginManager,
    delta_seconds: Real,
    entity: EntityId,
    binding: &RuntimeSceneScriptBinding,
    export_name: &'static str,
) -> ScriptSceneHookResult<()> {
    let arguments = [
        ScriptHostValue::Int(entity as i64),
        ScriptHostValue::Float(f64::from(delta_seconds)),
    ];
    let call_context = ScriptRuntimeCallContext {
        core: core.downgrade(),
        level: level.clone(),
        entity,
        delta_seconds,
    };
    trace_script_binding_export(binding, entity, export_name, "start", None);
    let result = with_script_runtime_call_context(call_context, || {
        manager.call_package_export(&binding.package, &binding.module, export_name, &arguments)
    });
    trace_script_binding_export(binding, entity, export_name, "done", Some(result.is_ok()));
    result.map(|_| ()).map_err(|source| {
        ScriptSceneHookError::export_call(binding_key(binding), export_name, source)
    })
}

fn trace_script_binding_export(
    binding: &RuntimeSceneScriptBinding,
    entity: EntityId,
    export_name: &str,
    phase: &str,
    success: Option<bool>,
) {
    if !trace_script_bindings_enabled() {
        return;
    }
    let success = success
        .map(|success| format!(" success={success}"))
        .unwrap_or_default();
    write_log(
        "zr_vm_project_backend",
        format!(
            "script_binding_export_{phase} package={} module={} entity={} export={export_name}{success}",
            binding.package, binding.module, entity
        ),
    );
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

#[derive(Clone, Debug)]
struct EntityScriptBindings {
    entity: EntityId,
    bindings: Vec<RuntimeSceneScriptBinding>,
}

fn collect_script_bindings(
    level: &LevelSystem,
) -> ScriptSceneHookResult<Vec<EntityScriptBindings>> {
    level.with_world(|world| {
        world
            .node_records()
            .into_iter()
            .filter_map(|node| {
                let value = world.dynamic_component(node.id, SCRIPT_BINDINGS_COMPONENT)?;
                Some(
                    serde_json::from_value::<Vec<RuntimeSceneScriptBinding>>(value.clone())
                        .map(|bindings| EntityScriptBindings {
                            entity: node.id,
                            bindings,
                        })
                        .map_err(|source| {
                            ScriptSceneHookError::invalid_binding_component(node.id, source)
                        }),
                )
            })
            .collect::<Result<Vec<_>, _>>()
    })
}

fn binding_started(level: &LevelSystem, entity: EntityId, binding_key: &str) -> bool {
    level.script_binding_started(entity, binding_key)
}

fn mark_binding_started(level: &LevelSystem, entity: EntityId, binding_key: &str) {
    level.mark_script_binding_started(entity, binding_key.to_string());
}

fn binding_key(binding: &RuntimeSceneScriptBinding) -> String {
    format!("{}::{}", binding.package, binding.module)
}

fn default_true() -> bool {
    true
}
