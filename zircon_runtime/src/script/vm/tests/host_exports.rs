use super::*;

#[test]
fn host_handles_are_stable_and_valid() {
    let registry = HostRegistry::default();
    let handle = registry.register_capability("RenderingManager").unwrap();
    assert!(registry.is_valid(handle));
}

#[test]
fn host_registry_exposes_stable_capability_records_without_concrete_objects() {
    let registry = HostRegistry::default();
    let ui_shell = registry
        .register_capability("editor.host.ui_shell")
        .unwrap();
    let asset_core = registry
        .register_capability("editor.host.asset_core")
        .unwrap();

    let ui_record = registry.resolve(ui_shell).unwrap();
    assert_eq!(ui_record.handle, ui_shell);
    assert_eq!(ui_record.label, "editor.host.ui_shell");

    let records = registry.capabilities();
    assert_eq!(records.len(), 2);
    assert_eq!(records[0].handle, ui_shell);
    assert_eq!(records[1].handle, asset_core);
    assert!(records.iter().all(|record| !record.label.is_empty()));
    assert!(registry
        .resolve(super::super::HostHandle::from_raw(999))
        .is_err());
}

#[test]
fn host_export_registry_validates_descriptors_and_dispatches_callbacks() {
    let registry = HostRegistry::default();
    let exports = HostExportRegistry::new(registry.clone());
    let descriptor = ScriptHostModuleDescriptor::new("test.host", "0.1.0")
        .with_capability("test.add")
        .with_function(
            ScriptHostFunctionDescriptor::new("add", 2, 2, ScriptHostValueKind::Int)
                .with_parameter(ScriptHostParameterDescriptor::new(
                    "left",
                    ScriptHostValueKind::Int,
                ))
                .with_parameter(ScriptHostParameterDescriptor::new(
                    "right",
                    ScriptHostValueKind::Int,
                ))
                .with_required_capability("test.add"),
        );
    let handle = exports
        .register_module(
            descriptor,
            [HostExportFunction::new("add", |context| {
                let left = match context.arguments[0] {
                    ScriptHostValue::Int(value) => value,
                    _ => 0,
                };
                let right = match context.arguments[1] {
                    ScriptHostValue::Int(value) => value,
                    _ => 0,
                };
                Ok(ScriptHostValue::Int(left + right))
            })],
        )
        .unwrap();

    assert!(registry.is_valid(handle));
    assert_eq!(exports.modules().len(), 1);
    let granted = CapabilitySet::default().with("test.add");
    let value = exports
        .call_with_capabilities(
            "test.host",
            "add",
            vec![ScriptHostValue::Int(2), ScriptHostValue::Int(5)],
            &granted,
        )
        .unwrap();
    assert_eq!(value, ScriptHostValue::Int(7));
}

#[test]
fn script_call_table_pre_resolves_host_export_callbacks() {
    let exports = HostExportRegistry::default();
    let seen_context = Arc::new(Mutex::new(Vec::new()));
    let seen_context_for_add = Arc::clone(&seen_context);
    let descriptor = ScriptHostModuleDescriptor::new("test.host", "0.1.0")
        .with_capability("test.add")
        .with_function(
            ScriptHostFunctionDescriptor::new("add", 2, 2, ScriptHostValueKind::Int)
                .with_parameter(ScriptHostParameterDescriptor::new(
                    "left",
                    ScriptHostValueKind::Int,
                ))
                .with_parameter(ScriptHostParameterDescriptor::new(
                    "right",
                    ScriptHostValueKind::Int,
                ))
                .with_required_capability("test.add"),
        );

    exports
        .register_module(
            descriptor,
            [HostExportFunction::new("add", move |context| {
                seen_context_for_add
                    .lock()
                    .unwrap()
                    .push((context.module_name.clone(), context.function_name.clone()));
                let left = match context.arguments[0] {
                    ScriptHostValue::Int(value) => value,
                    _ => 0,
                };
                let right = match context.arguments[1] {
                    ScriptHostValue::Int(value) => value,
                    _ => 0,
                };
                Ok(ScriptHostValue::Int(left + right))
            })],
        )
        .unwrap();

    let call_table = exports.script_call_table().unwrap();
    let site = call_table.resolve("test.host", "add").unwrap();

    assert_eq!(call_table.len(), 1);
    assert_eq!(site.id().raw(), 0);
    assert_eq!(site.module_name(), "test.host");
    assert_eq!(site.function_name(), "add");
    assert_eq!(
        call_table
            .call(
                site.id(),
                vec![ScriptHostValue::Int(2), ScriptHostValue::Int(5)],
                &CapabilitySet::default().with("test.add"),
            )
            .unwrap(),
        ScriptHostValue::Int(7)
    );
    assert_eq!(
        seen_context.lock().unwrap().as_slice(),
        &[("test.host".to_string(), "add".to_string())]
    );
}

#[test]
fn host_export_registry_preserves_precise_type_refs_for_zr_vm_registration() {
    let exports = HostExportRegistry::default();
    let descriptor = ScriptHostModuleDescriptor::new("test.types", "0.1.0")
        .with_type(
            ScriptHostTypeDescriptor::new("Vec3", ScriptHostValueKind::Float)
                .with_prototype_kind(ScriptHostPrototypeKind::Struct)
                .allow_value_construction(true),
        )
        .with_function(
            ScriptHostFunctionDescriptor::new("identity", 1, 1, ScriptHostValueKind::Float)
                .with_return_type(ScriptHostTypeRef::new(ScriptHostValueKind::Float, "Vec3"))
                .with_parameter(
                    ScriptHostParameterDescriptor::new("value", ScriptHostValueKind::Float)
                        .with_type_ref(ScriptHostTypeRef::new(ScriptHostValueKind::Float, "Vec3")),
                ),
        );

    exports
        .register_module(
            descriptor,
            [HostExportFunction::new("identity", |context| {
                Ok(context.arguments[0].clone())
            })],
        )
        .unwrap();

    let module = exports.module("test.types").unwrap();
    assert_eq!(module.descriptor.types[0].type_ref.type_name, "Vec3");
    assert_eq!(
        module.descriptor.types[0].prototype_kind,
        ScriptHostPrototypeKind::Struct
    );
    assert!(module.descriptor.types[0].allow_value_construction);
    assert_eq!(module.descriptor.functions[0].return_type.type_name, "Vec3");
    assert_eq!(
        module.descriptor.functions[0].parameters[0]
            .type_ref
            .type_name,
        "Vec3"
    );
}

#[test]
fn zr_vm_real_backend_uses_script_call_table_for_host_callbacks() {
    let source = include_str!("../backend/zr_vm_project_backend/real_backend/host_modules.rs");

    assert!(source.contains("script_call_table()"));
    assert!(source.contains("ScriptCallSite"));
    assert!(
        !source.contains(".call_with_capabilities("),
        "real zr_vm callbacks must use pre-resolved ScriptCallSite dispatch"
    );
}

#[test]
fn host_export_registry_rejects_duplicates_invalid_callbacks_and_missing_capabilities() {
    let exports = HostExportRegistry::default();
    let descriptor = ScriptHostModuleDescriptor::new("test.host", "0.1.0")
        .with_capability("test.read")
        .with_function(
            ScriptHostFunctionDescriptor::new("read", 0, 0, ScriptHostValueKind::Null)
                .with_required_capability("test.read"),
        );

    exports
        .register_module(
            descriptor.clone(),
            [HostExportFunction::new("read", |_| {
                Ok(ScriptHostValue::Null)
            })],
        )
        .unwrap();
    assert!(matches!(
        exports.register_module(
            descriptor.clone(),
            [HostExportFunction::new("read", |_| Ok(ScriptHostValue::Null))]
        ),
        Err(VmError::Operation(message)) if message.contains("already registered")
    ));
    assert!(matches!(
        HostExportRegistry::default().register_module(
            descriptor.clone(),
            [HostExportFunction::new("unknown", |_| Ok(ScriptHostValue::Null))]
        ),
        Err(VmError::Operation(message)) if message.contains("callback missing")
    ));
    assert!(matches!(
        HostExportRegistry::default().register_module(
            descriptor.clone(),
            [
                HostExportFunction::new("read", |_| Ok(ScriptHostValue::Null)),
                HostExportFunction::new("read", |_| Ok(ScriptHostValue::Null)),
            ]
        ),
        Err(VmError::Operation(message)) if message.contains("duplicate host export callback")
    ));
    assert!(matches!(
        exports.call_with_capabilities("test.host", "read", Vec::new(), &CapabilitySet::default()),
        Err(VmError::Operation(message)) if message.contains("missing capability")
    ));
    let mut function = ScriptHostFunctionDescriptor::new("bad", 0, 0, ScriptHostValueKind::Int);
    function.return_type = ScriptHostTypeRef::new(ScriptHostValueKind::String, "string");
    let mismatched =
        ScriptHostModuleDescriptor::new("test.mismatch", "0.1.0").with_function(function);
    assert!(matches!(
        HostExportRegistry::default().register_module(
            mismatched,
            [HostExportFunction::new("bad", |_| Ok(ScriptHostValue::Null))]
        ),
        Err(VmError::Operation(message)) if message.contains("value kind mismatch")
    ));
}
