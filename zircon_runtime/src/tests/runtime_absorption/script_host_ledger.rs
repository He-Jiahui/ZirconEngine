use std::{
    fs,
    path::{Path, PathBuf},
    sync::Arc,
};

use crate::core::framework::bridge::PluginInterface;
use crate::core::framework::script::{
    ScriptHostParameterDescriptor, ScriptHostValue, ScriptHostValueKind,
};
use crate::plugin::RuntimeExtensionRegistry;
use crate::script::{
    register_bridge_host_module, register_builtin_host_modules, CapabilitySet, HostExportRegistry,
    HostRegistry, ScriptBridgeMethodDescriptor, BRIDGE_HOST_CAPABILITY, BRIDGE_HOST_MODULE,
};

const FIXED_HOST_FUNCTIONS: &[(&str, &str)] = &[
    ("zr.zircon.foundation", "time_unix_millis"),
    ("zr.zircon.foundation", "log_info"),
    ("zr.zircon.foundation", "event_publish"),
    ("zr.zircon.asset", "locator_identity"),
    ("zr.zircon.asset", "status"),
    ("zr.zircon.asset", "revision"),
    ("zr.zircon.scene", "default_world_handle"),
    ("zr.zircon.scene", "handle_is_valid"),
    ("zr.zircon.scene", "summary"),
    ("zr.zircon.render", "backend_name"),
    ("zr.zircon.render", "frame_index"),
    ("zr.zircon.math", "vec3_length"),
    ("zr.zircon.math", "vec3_dot"),
    ("zr.zircon.gameplay", "delta_seconds"),
    ("zr.zircon.gameplay", "entity"),
    ("zr.zircon.gameplay", "key_pressed"),
    ("zr.zircon.gameplay", "position_json"),
    ("zr.zircon.gameplay", "position_x"),
    ("zr.zircon.gameplay", "position_y"),
    ("zr.zircon.gameplay", "position_z"),
    ("zr.zircon.gameplay", "set_position_json"),
    ("zr.zircon.gameplay", "set_position"),
    ("zr.zircon.gameplay", "translate_json"),
    ("zr.zircon.gameplay", "translate"),
    ("zr.zircon.gameplay", "face_direction"),
    ("zr.zircon.gameplay", "set_scale"),
    ("zr.zircon.gameplay", "follow_position"),
    ("zr.zircon.gameplay", "move_towards_entity"),
    ("zr.zircon.gameplay", "camera_follow"),
    ("zr.zircon.gameplay", "component_json"),
    ("zr.zircon.gameplay", "component_string"),
    ("zr.zircon.gameplay", "set_component_json"),
    ("zr.zircon.gameplay", "find_by_component"),
    ("zr.zircon.gameplay", "entity_exists"),
    ("zr.zircon.gameplay", "nearest_by_script_property"),
    ("zr.zircon.gameplay", "count_by_script_property"),
    ("zr.zircon.gameplay", "script_property_matches"),
    ("zr.zircon.gameplay", "script_number"),
    ("zr.zircon.gameplay", "script_number_at_most"),
    ("zr.zircon.gameplay", "set_animation_bool"),
    ("zr.zircon.gameplay", "damage_entity"),
    ("zr.zircon.gameplay", "heal_entity"),
    ("zr.zircon.gameplay", "current_hp"),
    ("zr.zircon.gameplay", "damage_entity_report"),
    ("zr.zircon.gameplay", "spawn_empty"),
    ("zr.zircon.gameplay", "spawn_model"),
    ("zr.zircon.gameplay", "set_hud_text"),
    ("zr.zircon.gameplay", "set_particle_sprites"),
    ("zr.zircon.gameplay", "set_world_hud_bar"),
    ("zr.zircon.gameplay", "despawn"),
    ("zr.zircon.gameplay", "nav_next_point_json"),
    ("zr.zircon.gameplay", "nav_move_towards_entity"),
];

const FIXED_HOST_MODULES: &[&str] = &[
    "zr.zircon.foundation",
    "zr.zircon.asset",
    "zr.zircon.scene",
    "zr.zircon.render",
    "zr.zircon.math",
    "zr.zircon.gameplay",
];

const HOST_CAPABILITIES: &[&str] = &[
    "foundation.log",
    "foundation.time",
    "foundation.event",
    "asset.query",
    "scene.query",
    "scene.handle",
    "render.query",
    "gameplay.input",
    "gameplay.entity",
    "gameplay.navigation",
    "bridge.call",
];

#[derive(Clone, Debug)]
struct CapabilityCase {
    module: &'static str,
    function: &'static str,
    capability: &'static str,
    arguments: Vec<ScriptHostValue>,
}

impl CapabilityCase {
    fn new(
        module: &'static str,
        function: &'static str,
        capability: &'static str,
        arguments: Vec<ScriptHostValue>,
    ) -> Self {
        Self {
            module,
            function,
            capability,
            arguments,
        }
    }
}

trait Runtime13LedgerBridge: Send + Sync {}

impl PluginInterface for dyn Runtime13LedgerBridge {
    const INTERFACE_ID: &'static str = "runtime13.ledger.bridge.v1";
}

struct Runtime13LedgerBridgeProvider;

impl Runtime13LedgerBridge for Runtime13LedgerBridgeProvider {}

#[test]
fn host_function_registry_matches_documented_ledger() {
    let builtin_source = include_str!("../../script/vm/host/builtin_host_modules.rs");
    let gameplay_source = include_str!("../../script/vm/gameplay_host.rs");
    let bridge_source = include_str!("../../script/vm/host/bridge_host_module.rs");
    let ledger = include_str!("../../../../docs/zircon_runtime/script/vm/host/function_ledger.md");
    let plan = include_str!(
        "../../../../docs/plans/zircon_runtime/runtime/13-script-binding-and-reflection.md"
    );

    assert_eq!(
        count_occurrences(builtin_source, "HostExportFunction::new("),
        11,
        "builtin host callback count changed; update function_ledger.md and Runtime 13 status"
    );
    assert_eq!(
        count_occurrences(gameplay_source, "HostExportFunction::new("),
        39,
        "gameplay host callback count changed; update function_ledger.md and Runtime 13 status"
    );
    assert_eq!(
        count_occurrences(builtin_source, "#[crate::zircon_host_function("),
        2,
        "macro host function count changed; update function_ledger.md and Runtime 13 status"
    );

    for module in FIXED_HOST_MODULES {
        assert!(
            combined_fixed_sources_contain_module(builtin_source, gameplay_source, module),
            "fixed host module `{module}` should exist in host registration sources"
        );
        assert!(
            ledger.contains(module),
            "function ledger should document fixed host module `{module}`"
        );
    }

    for (module, function) in FIXED_HOST_FUNCTIONS {
        assert!(
            fixed_sources_contain_function(builtin_source, gameplay_source, function),
            "fixed host function `{module}.{function}` should exist in registration sources"
        );
        assert!(
            ledger.contains(&format!("| `{function}` |"))
                || ledger.contains(&format!("| Type `{function}` |")),
            "function ledger should document fixed host function `{module}.{function}`"
        );
    }

    for capability in HOST_CAPABILITIES {
        assert!(
            ledger.contains(capability),
            "function ledger should document host capability `{capability}`"
        );
    }

    for required_bridge_anchor in [
        "pub const BRIDGE_HOST_MODULE: &str = \"zr.zircon.bridge\";",
        "pub const BRIDGE_HOST_CAPABILITY: &str = \"bridge.call\";",
        "ScriptBridgeMethodDescriptor",
        "register_bridge_host_module",
    ] {
        assert!(
            bridge_source.contains(required_bridge_anchor),
            "bridge host source should keep dynamic module anchor `{required_bridge_anchor}`"
        );
    }

    for required_ledger_anchor in [
        "6 host modules, 52 fixed host functions, and 2 fixed script type descriptors",
        "`zr.zircon.bridge`",
        "dynamic module shape contract",
        "Value descriptors",
        "Host handles",
        "Serialized payloads",
        "ZrHostEcsApiV1",
        "host_function_registry_matches_documented_ledger",
        "host_capability_representatives_are_declared_on_registered_modules",
        "host_function_without_required_capability_is_rejected_with_explicit_error",
    ] {
        assert!(
            ledger.contains(required_ledger_anchor),
            "function ledger should record `{required_ledger_anchor}`"
        );
    }

    for required_plan_anchor in [
        "host_function_registry_matches_documented_ledger",
        "host_capability_representatives_are_declared_on_registered_modules",
        "host_function_without_required_capability_is_rejected_with_explicit_error",
        "builtin_callbacks=11",
        "gameplay_callbacks=39",
        "macro_host_functions=2",
    ] {
        assert!(
            plan.contains(required_plan_anchor),
            "Runtime 13 plan should record `{required_plan_anchor}`"
        );
    }
}

#[test]
fn host_capability_representatives_are_declared_on_registered_modules() {
    let exports = registered_builtin_exports();

    for case in fixed_capability_cases() {
        assert_registered_capability_descriptor(&exports, &case);
    }

    let bridge_exports = registered_bridge_exports();
    assert_registered_capability_descriptor(&bridge_exports, &bridge_capability_case());
}

#[test]
fn host_function_without_required_capability_is_rejected_with_explicit_error() {
    let exports = registered_builtin_exports();

    for case in fixed_capability_cases() {
        let error = exports
            .call_with_capabilities(
                case.module,
                case.function,
                case.arguments.clone(),
                &CapabilitySet::default(),
            )
            .unwrap_err();

        assert!(
            format!("{error}").contains(&format!("missing capability {}", case.capability)),
            "call to {}.{} should reject missing capability `{}`",
            case.module,
            case.function,
            case.capability
        );
    }

    let bridge_exports = registered_bridge_exports();
    let bridge_case = bridge_capability_case();
    let error = bridge_exports
        .call_with_capabilities(
            bridge_case.module,
            bridge_case.function,
            bridge_case.arguments,
            &CapabilitySet::default(),
        )
        .unwrap_err();

    assert!(
        format!("{error}").contains(&format!("missing capability {}", bridge_case.capability)),
        "bridge host call should reject missing capability `{}`",
        bridge_case.capability
    );
}

#[test]
fn script_ecs_access_path_stays_on_gameplay_facade_not_native_ecs_abi() {
    let gameplay_source = include_str!("../../script/vm/gameplay_host.rs");
    let runtime_context_source = include_str!("../../script/vm/runtime_context.rs");
    let ledger = include_str!("../../../../docs/zircon_runtime/script/vm/host/function_ledger.md");

    for required_ledger_anchor in [
        "The current script gameplay ECS path is `zr.zircon.gameplay` through `ScriptRuntimeCallContext`",
        "`ZrHostEcsApiV1` belongs to the native/plugin ABI layer",
        "A VM plugin that needs plugin-owned bridge behavior should route through `zr.zircon.bridge`",
    ] {
        assert!(
            ledger.contains(required_ledger_anchor),
            "function ledger should record ECS access path judgement `{required_ledger_anchor}`"
        );
    }

    for required_source_anchor in [
        "const GAMEPLAY_MODULE: &str = \"zr.zircon.gameplay\";",
        "pub fn register_gameplay_host_module(",
        "current_script_runtime_call_context()?",
    ] {
        assert!(
            gameplay_source.contains(required_source_anchor),
            "gameplay host source should keep ECS facade anchor `{required_source_anchor}`"
        );
    }
    assert!(
        runtime_context_source.contains("pub struct ScriptRuntimeCallContext")
            && runtime_context_source.contains("pub level: LevelSystem")
            && runtime_context_source.contains("pub entity: EntityId"),
        "script runtime call context should continue to carry the gameplay facade ECS scope"
    );

    for file in script_source_files() {
        let source = fs::read_to_string(&file)
            .unwrap_or_else(|error| panic!("read script source `{}`: {error}", file.display()));
        for forbidden in ["ZrHostEcsApiV1", "ZrHostEcsApi", "HostEcsApi"] {
            assert!(
                !source.contains(forbidden),
                "script source `{}` must stay off native ECS ABI symbol `{forbidden}`",
                file.display()
            );
        }
    }
}

#[test]
fn host_function_registry_ledger_guard_rejects_missing_entry() {
    let ledger = include_str!("../../../../docs/zircon_runtime/script/vm/host/function_ledger.md")
        .replace("| `time_unix_millis` |", "| `time_unix_millis_removed` |");

    let missing_entries = missing_documented_functions(&ledger);

    assert!(
        missing_entries
            .iter()
            .any(|entry| entry == "zr.zircon.foundation.time_unix_millis"),
        "ledger guard negative self-check should reject a missing fixed host function"
    );
}

fn fixed_sources_contain_function(
    builtin_source: &str,
    gameplay_source: &str,
    function: &str,
) -> bool {
    builtin_source.contains(&format!("HostExportFunction::new(\"{function}\""))
        || gameplay_source.contains(&format!("HostExportFunction::new(\"{function}\""))
        || builtin_source.contains(&format!("name = \"{function}\""))
}

fn combined_fixed_sources_contain_module(
    builtin_source: &str,
    gameplay_source: &str,
    module: &str,
) -> bool {
    builtin_source.contains(module) || gameplay_source.contains(module)
}

fn missing_documented_functions(ledger: &str) -> Vec<String> {
    FIXED_HOST_FUNCTIONS
        .iter()
        .filter_map(|(module, function)| {
            let documented = ledger.contains(&format!("| `{function}` |"))
                || ledger.contains(&format!("| Type `{function}` |"));
            (!documented).then(|| format!("{module}.{function}"))
        })
        .collect()
}

fn registered_builtin_exports() -> HostExportRegistry {
    let registry = HostRegistry::default();
    let exports = HostExportRegistry::new(registry.clone());
    register_builtin_host_modules(&exports, &registry).unwrap();
    exports
}

fn registered_bridge_exports() -> HostExportRegistry {
    let mut registry = RuntimeExtensionRegistry::default();
    let owner = registry.intern_plugin_module("runtime13.ledger").unwrap();
    registry
        .export_interface::<dyn Runtime13LedgerBridge>(
            owner,
            Arc::new(Runtime13LedgerBridgeProvider),
        )
        .unwrap();
    let exports = HostExportRegistry::default();
    register_bridge_host_module(
        &exports,
        registry.frozen_bridge_table(),
        [ScriptBridgeMethodDescriptor::new(
            bridge_capability_case().function,
            <dyn Runtime13LedgerBridge as PluginInterface>::INTERFACE_ID,
            1,
            ScriptHostValueKind::Bool,
            |_| Ok(ScriptHostValue::Bool(true)),
        )
        .with_parameter(ScriptHostParameterDescriptor::new(
            "payload",
            ScriptHostValueKind::String,
        ))],
    )
    .unwrap();
    exports
}

fn fixed_capability_cases() -> Vec<CapabilityCase> {
    vec![
        CapabilityCase::new(
            "zr.zircon.foundation",
            "time_unix_millis",
            "foundation.time",
            Vec::new(),
        ),
        CapabilityCase::new(
            "zr.zircon.foundation",
            "log_info",
            "foundation.log",
            vec![ScriptHostValue::String("runtime13 ledger".to_string())],
        ),
        CapabilityCase::new(
            "zr.zircon.foundation",
            "event_publish",
            "foundation.event",
            vec![
                ScriptHostValue::String("runtime13.ledger".to_string()),
                ScriptHostValue::String("{}".to_string()),
            ],
        ),
        CapabilityCase::new(
            "zr.zircon.asset",
            "locator_identity",
            "asset.query",
            vec![ScriptHostValue::String(
                "asset://runtime13/probe".to_string(),
            )],
        ),
        CapabilityCase::new(
            "zr.zircon.scene",
            "default_world_handle",
            "scene.handle",
            Vec::new(),
        ),
        CapabilityCase::new(
            "zr.zircon.scene",
            "handle_is_valid",
            "scene.query",
            vec![ScriptHostValue::HostHandle(1)],
        ),
        CapabilityCase::new(
            "zr.zircon.render",
            "backend_name",
            "render.query",
            Vec::new(),
        ),
        CapabilityCase::new(
            "zr.zircon.gameplay",
            "key_pressed",
            "gameplay.input",
            vec![ScriptHostValue::String("Space".to_string())],
        ),
        CapabilityCase::new(
            "zr.zircon.gameplay",
            "entity",
            "gameplay.entity",
            Vec::new(),
        ),
        CapabilityCase::new(
            "zr.zircon.gameplay",
            "nav_next_point_json",
            "gameplay.navigation",
            vec![
                ScriptHostValue::String("{\"x\":0.0,\"y\":0.0,\"z\":0.0}".to_string()),
                ScriptHostValue::String("{\"x\":1.0,\"y\":0.0,\"z\":0.0}".to_string()),
            ],
        ),
    ]
}

fn bridge_capability_case() -> CapabilityCase {
    CapabilityCase::new(
        BRIDGE_HOST_MODULE,
        "runtime13_bridge_probe",
        BRIDGE_HOST_CAPABILITY,
        vec![ScriptHostValue::String("probe".to_string())],
    )
}

fn assert_registered_capability_descriptor(exports: &HostExportRegistry, case: &CapabilityCase) {
    let module = exports
        .module(case.module)
        .unwrap_or_else(|| panic!("host module `{}` should be registered", case.module));
    assert!(
        module
            .descriptor
            .capabilities
            .contains(&case.capability.to_string()),
        "host module `{}` should declare capability `{}`",
        case.module,
        case.capability
    );
    let function = module
        .descriptor
        .functions
        .iter()
        .find(|function| function.name == case.function)
        .unwrap_or_else(|| {
            panic!(
                "host function `{}.{}` should be registered",
                case.module, case.function
            )
        });
    assert!(
        function
            .required_capabilities
            .contains(&case.capability.to_string()),
        "host function `{}.{}` should require capability `{}`",
        case.module,
        case.function,
        case.capability
    );
}

fn script_source_files() -> Vec<PathBuf> {
    let mut files = Vec::new();
    collect_rs_files(
        &Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("script"),
        &mut files,
    );
    files.sort();
    files
}

fn collect_rs_files(root: &Path, files: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(root)
        .unwrap_or_else(|error| panic!("read source directory `{}`: {error}", root.display()))
    {
        let path = entry
            .unwrap_or_else(|error| panic!("read source directory entry: {error}"))
            .path();
        if path.is_dir() {
            collect_rs_files(&path, files);
        } else if path.extension().and_then(|extension| extension.to_str()) == Some("rs") {
            files.push(path);
        }
    }
}

fn count_occurrences(haystack: &str, needle: &str) -> usize {
    haystack.match_indices(needle).count()
}
