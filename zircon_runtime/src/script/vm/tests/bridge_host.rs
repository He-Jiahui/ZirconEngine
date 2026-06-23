use super::*;

trait VmWeatherBridge: Send + Sync {}

impl PluginInterface for dyn VmWeatherBridge {
    const INTERFACE_ID: &'static str = "vm.weather.bridge.v1";
}

struct VmWeatherProvider;

impl VmWeatherBridge for VmWeatherProvider {}

#[test]
fn bridge_host_module_dispatches_vm_calls_through_resolved_bridge_slots() {
    let mut registry = RuntimeExtensionRegistry::default();
    let owner = registry.intern_plugin_module("weather.runtime").unwrap();
    registry
        .export_interface::<dyn VmWeatherBridge>(owner, Arc::new(VmWeatherProvider))
        .unwrap();
    let table = registry.frozen_bridge_table();
    let slot = table
        .resolve_slot(<dyn VmWeatherBridge as PluginInterface>::INTERFACE_ID)
        .unwrap();
    let exports = HostExportRegistry::default();
    let calls = Arc::new(Mutex::new(Vec::new()));
    let calls_for_method = Arc::clone(&calls);

    super::super::super::register_bridge_host_module(
        &exports,
        table.clone(),
        [ScriptBridgeMethodDescriptor::new(
            "sample_temperature",
            <dyn VmWeatherBridge as PluginInterface>::INTERFACE_ID,
            9,
            ScriptHostValueKind::Int,
            move |call| {
                calls_for_method.lock().unwrap().push((
                    call.interface_slot.raw(),
                    call.method_slot,
                    call.arguments.clone(),
                ));
                Ok(ScriptHostValue::Int(32))
            },
        )
        .with_parameter(ScriptHostParameterDescriptor::new(
            "zone",
            ScriptHostValueKind::String,
        ))],
    )
    .unwrap();

    let module = exports.module(BRIDGE_HOST_MODULE).unwrap();
    assert!(module
        .descriptor
        .capabilities
        .contains(&BRIDGE_HOST_CAPABILITY.to_string()));
    assert!(module
        .descriptor
        .functions
        .iter()
        .any(|function| function.name == "sample_temperature"));

    let value = exports
        .call_with_capabilities(
            BRIDGE_HOST_MODULE,
            "sample_temperature",
            vec![ScriptHostValue::String("outside".to_string())],
            &CapabilitySet::default().with(BRIDGE_HOST_CAPABILITY),
        )
        .unwrap();

    assert_eq!(value, ScriptHostValue::Int(32));
    assert_eq!(
        calls.lock().unwrap().as_slice(),
        &[(
            slot.raw(),
            9,
            vec![ScriptHostValue::String("outside".to_string())]
        )]
    );
}

#[test]
fn bridge_host_module_registers_methods_from_package_manifest() {
    let mut registry = RuntimeExtensionRegistry::default();
    let owner = registry.intern_plugin_module("weather.runtime").unwrap();
    registry
        .export_interface::<dyn VmWeatherBridge>(owner, Arc::new(VmWeatherProvider))
        .unwrap();
    let table = registry.frozen_bridge_table();
    let slot = table
        .resolve_slot(<dyn VmWeatherBridge as PluginInterface>::INTERFACE_ID)
        .unwrap();
    let exports = HostExportRegistry::default();
    let calls = Arc::new(Mutex::new(Vec::new()));
    let calls_for_method = Arc::clone(&calls);
    let manifest = PluginPackageManifest::new("weather", "Weather").with_provided_interface(
        PluginInterfaceManifest::new(<dyn VmWeatherBridge as PluginInterface>::INTERFACE_ID)
            .with_method(
                PluginInterfaceMethodManifest::new("sample_temperature", 9)
                    .with_return_value_kind(ScriptHostValueKind::Int)
                    .with_parameter(ScriptHostParameterDescriptor::new(
                        "zone",
                        ScriptHostValueKind::String,
                    ))
                    .with_required_capability("runtime.plugin.weather.query")
                    .with_documentation("Samples a weather bridge query."),
            ),
    );

    super::super::super::register_bridge_host_module_from_manifest(
        &exports,
        table,
        &manifest,
        [ScriptBridgeMethodBinding::new(
            <dyn VmWeatherBridge as PluginInterface>::INTERFACE_ID,
            "sample_temperature",
            move |call| {
                calls_for_method
                    .lock()
                    .unwrap()
                    .push((call.interface_slot.raw(), call.method_slot));
                Ok(ScriptHostValue::Int(36))
            },
        )],
    )
    .unwrap();

    let module = exports.module(BRIDGE_HOST_MODULE).unwrap();
    let function = module
        .descriptor
        .functions
        .iter()
        .find(|function| function.name == "sample_temperature")
        .unwrap();
    assert_eq!(function.return_value_kind, ScriptHostValueKind::Int);
    assert_eq!(function.parameters[0].name, "zone");
    assert!(function
        .required_capabilities
        .contains(&BRIDGE_HOST_CAPABILITY.to_string()));
    assert!(function
        .required_capabilities
        .contains(&"runtime.plugin.weather.query".to_string()));
    assert_eq!(
        function.documentation.as_deref(),
        Some("Samples a weather bridge query.")
    );

    let value = exports
        .call_with_capabilities(
            BRIDGE_HOST_MODULE,
            "sample_temperature",
            vec![ScriptHostValue::String("outside".to_string())],
            &CapabilitySet::default()
                .with(BRIDGE_HOST_CAPABILITY)
                .with("runtime.plugin.weather.query"),
        )
        .unwrap();

    assert_eq!(value, ScriptHostValue::Int(36));
    assert_eq!(calls.lock().unwrap().as_slice(), &[(slot.raw(), 9)]);
}

#[test]
fn bridge_host_module_rejects_manifest_method_without_binding() {
    let manifest = PluginPackageManifest::new("weather", "Weather").with_provided_interface(
        PluginInterfaceManifest::new(<dyn VmWeatherBridge as PluginInterface>::INTERFACE_ID)
            .with_method(PluginInterfaceMethodManifest::new("sample_temperature", 9)),
    );

    let error =
        match super::super::super::script_bridge_method_descriptors_from_manifest(&manifest, []) {
            Ok(_) => panic!("manifest method without binding should be rejected"),
            Err(error) => error,
        };

    assert!(format!("{error}").contains("declared but has no binding"));
}

#[test]
fn bridge_host_module_reports_disabled_bridge_to_vm_callers() {
    let mut registry = RuntimeExtensionRegistry::default();
    let owner = registry.intern_plugin_module("weather.runtime").unwrap();
    registry
        .export_interface::<dyn VmWeatherBridge>(owner, Arc::new(VmWeatherProvider))
        .unwrap();
    let table = registry.frozen_bridge_table();
    let slot = table
        .resolve_slot(<dyn VmWeatherBridge as PluginInterface>::INTERFACE_ID)
        .unwrap();
    table.set_enabled(slot, false).unwrap();
    let exports = HostExportRegistry::default();
    super::super::super::register_bridge_host_module(
        &exports,
        table,
        [ScriptBridgeMethodDescriptor::new(
            "sample_temperature",
            <dyn VmWeatherBridge as PluginInterface>::INTERFACE_ID,
            9,
            ScriptHostValueKind::Int,
            |_| Ok(ScriptHostValue::Int(32)),
        )],
    )
    .unwrap();

    let error = exports
        .call_with_capabilities(
            BRIDGE_HOST_MODULE,
            "sample_temperature",
            Vec::new(),
            &CapabilitySet::default().with(BRIDGE_HOST_CAPABILITY),
        )
        .unwrap_err();

    assert!(format!("{error}").contains("is not enabled"));
}
