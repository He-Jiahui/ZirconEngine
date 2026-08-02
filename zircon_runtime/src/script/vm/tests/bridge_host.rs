use super::*;

trait VmWeatherBridge: Send + Sync {}

impl PluginInterface for dyn VmWeatherBridge {
    const INTERFACE_ID: &'static str = "vm.weather.bridge.v1";
}

struct VmWeatherProvider;

impl VmWeatherBridge for VmWeatherProvider {}

#[test]
fn runtime13_bridge_host_module_borrows_vm_call_arguments_through_resolved_slots() {
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

    super::super::register_bridge_host_module(
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
                    call.arguments.as_ptr() as usize,
                    call.arguments.len(),
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

    let arguments = vec![ScriptHostValue::String("outside".to_string())];
    let argument_pointer = arguments.as_ptr() as usize;
    let value = exports
        .call_with_capabilities(
            BRIDGE_HOST_MODULE,
            "sample_temperature",
            arguments,
            &CapabilitySet::default().with(BRIDGE_HOST_CAPABILITY),
        )
        .unwrap();

    assert_eq!(value, ScriptHostValue::Int(32));
    assert_eq!(
        calls.lock().unwrap().as_slice(),
        &[(slot.raw(), 9, argument_pointer, 1)]
    );
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
    super::super::register_bridge_host_module(
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
