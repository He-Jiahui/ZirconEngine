use std::sync::Arc;

use crate::core::framework::bridge::PluginInterface;
use crate::core::framework::script::{
    ScriptHostParameterDescriptor, ScriptHostValue, ScriptHostValueKind,
};
use crate::plugin::RuntimeExtensionRegistry;
use crate::script::{
    register_bridge_host_module, register_builtin_host_modules, HostExportRegistry, HostRegistry,
    ScriptBridgeMethodDescriptor, BRIDGE_HOST_CAPABILITY, BRIDGE_HOST_MODULE,
};

#[derive(Clone, Debug)]
pub(super) struct CapabilityCase {
    pub(super) module: &'static str,
    pub(super) function: &'static str,
    pub(super) capability: &'static str,
    pub(super) arguments: Vec<ScriptHostValue>,
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

pub(super) fn registered_builtin_exports() -> HostExportRegistry {
    let registry = HostRegistry::default();
    let exports = HostExportRegistry::new(registry.clone());
    register_builtin_host_modules(&exports, &registry).unwrap();
    exports
}

pub(super) fn registered_bridge_exports() -> HostExportRegistry {
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

pub(super) fn fixed_capability_cases() -> Vec<CapabilityCase> {
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

pub(super) fn bridge_capability_case() -> CapabilityCase {
    CapabilityCase::new(
        BRIDGE_HOST_MODULE,
        "runtime13_bridge_probe",
        BRIDGE_HOST_CAPABILITY,
        vec![ScriptHostValue::String("probe".to_string())],
    )
}

pub(super) fn assert_registered_capability_descriptor(
    exports: &HostExportRegistry,
    case: &CapabilityCase,
) {
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
