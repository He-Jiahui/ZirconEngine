use super::*;

fn register_null_host_export(exports: &HostExportRegistry, module_name: &str, function_name: &str) {
    exports
        .register_module(
            ScriptHostModuleDescriptor::new(module_name, "0.1.0").with_function(
                ScriptHostFunctionDescriptor::new(function_name, 0, 0, ScriptHostValueKind::Null),
            ),
            [HostExportFunction::new(function_name, |_| {
                Ok(ScriptHostValue::Null)
            })],
        )
        .expect("minimal host export should register");
}

fn int_argument(
    context: &crate::core::framework::script::ScriptHostCallFrame<'_>,
    index: usize,
) -> i64 {
    context
        .arguments
        .with_argument(index, |value| match value {
            ScriptHostValueRef::Int(value) => Ok(value),
            value => Err(ScriptHostError::new(format!(
                "argument {index} expected integer, received {:?}",
                value.kind()
            ))),
        })
        .expect("integer fixture must be present")
}

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
                let left = int_argument(context, 0);
                let right = int_argument(context, 1);
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
                seen_context_for_add.lock().unwrap().push((
                    context.module_name.to_owned(),
                    context.function_name.to_owned(),
                ));
                let left = int_argument(context, 0);
                let right = int_argument(context, 1);
                Ok(ScriptHostValue::Int(left + right))
            })],
        )
        .unwrap();

    let call_table = exports.script_call_table();
    let site = call_table.resolve("test.host", "add").unwrap();

    assert_eq!(call_table.len(), 1);
    assert_eq!(site.id().raw(), 0);
    assert_eq!(site.module_name(), "test.host");
    assert_eq!(site.function_name(), "add");
    let arguments = [ScriptHostValue::Int(2), ScriptHostValue::Int(5)];
    let source = ScriptHostOwnedArgumentSource::new(&arguments);
    assert_eq!(
        call_table
            .call(
                site.id(),
                ScriptHostArguments::new(&source),
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
fn runtime13_host_call_frames_borrow_stable_call_site_and_capability_storage() {
    let exports = HostExportRegistry::default();
    let observed_storage = Arc::new(Mutex::new(None));
    let observed_storage_for_callback = Arc::clone(&observed_storage);

    exports
        .register_module(
            ScriptHostModuleDescriptor::new("test.borrowed", "0.1.0")
                .with_capability("test.borrowed.call")
                .with_function(
                    ScriptHostFunctionDescriptor::new("inspect", 2, 2, ScriptHostValueKind::Null)
                        .with_parameter(ScriptHostParameterDescriptor::new(
                            "text",
                            ScriptHostValueKind::String,
                        ))
                        .with_parameter(ScriptHostParameterDescriptor::new(
                            "bytes",
                            ScriptHostValueKind::Bytes,
                        ))
                        .with_required_capability("test.borrowed.call"),
                ),
            [HostExportFunction::new("inspect", move |context| {
                let text_pointer = context.arguments.with_argument(0, |value| match value {
                    ScriptHostValueRef::String(value) => Ok(value.as_ptr() as usize),
                    value => Err(ScriptHostError::new(format!(
                        "expected string, received {:?}",
                        value.kind()
                    ))),
                })?;
                let (bytes_pointer, byte_count) =
                    context.arguments.with_argument(1, |value| match value {
                        ScriptHostValueRef::Bytes(value) => {
                            Ok((value.byte_at(0)? as usize, value.len()?))
                        }
                        value => Err(ScriptHostError::new(format!(
                            "expected bytes, received {:?}",
                            value.kind()
                        ))),
                    })?;
                *observed_storage_for_callback.lock().unwrap() = Some((
                    context.module_name.as_ptr() as usize,
                    context.function_name.as_ptr() as usize,
                    context.granted_capabilities.as_ptr() as usize,
                    text_pointer,
                    bytes_pointer,
                    byte_count,
                ));
                Ok(ScriptHostValue::Null)
            })],
        )
        .expect("borrowed host export should register");

    let call_table = exports.script_call_table();
    let site = call_table
        .resolve("test.borrowed", "inspect")
        .expect("registered call site should resolve");
    let granted = CapabilitySet::default().with("test.borrowed.call");
    let arguments = vec![
        ScriptHostValue::String("call-frame-text".to_string()),
        ScriptHostValue::Bytes(vec![1, 2, 3, 4]),
    ];
    let expected_text_storage = match &arguments[0] {
        ScriptHostValue::String(text) => text.as_ptr() as usize,
        _ => panic!("expected string fixture"),
    };
    let expected_first_byte = match &arguments[1] {
        ScriptHostValue::Bytes(bytes) => usize::from(bytes[0]),
        _ => panic!("expected bytes fixture"),
    };
    let expected_byte_count = match &arguments[1] {
        ScriptHostValue::Bytes(bytes) => bytes.len(),
        _ => panic!("expected bytes fixture"),
    };
    let source = ScriptHostOwnedArgumentSource::new(&arguments);

    assert_eq!(
        call_table
            .call(site.id(), ScriptHostArguments::new(&source), &granted)
            .unwrap(),
        ScriptHostValue::Null
    );
    assert_eq!(
        *observed_storage.lock().unwrap(),
        Some((
            site.module_name().as_ptr() as usize,
            site.function_name().as_ptr() as usize,
            granted.capabilities.as_ptr() as usize,
            expected_text_storage,
            expected_first_byte,
            expected_byte_count,
        ))
    );
}

#[test]
fn runtime13_performance_script_call_table_snapshots_are_generation_owned() {
    let exports = HostExportRegistry::default();
    register_null_host_export(&exports, "test.first", "ping");

    let first = exports.script_call_table();
    let repeated = exports.script_call_table();
    let first_site = first
        .resolve("test.first", "ping")
        .expect("first generation should resolve its registered call site");
    let repeated_site = repeated
        .resolve("test.first", "ping")
        .expect("repeated snapshot should resolve its registered call site");

    assert_eq!(first.generation(), repeated.generation());
    assert!(std::ptr::eq(first_site, repeated_site));

    register_null_host_export(&exports, "test.second", "pong");
    let second = exports.script_call_table();

    assert_eq!(second.generation(), first.generation() + 1);
    assert_eq!(first.len(), 1);
    assert!(first.resolve("test.second", "pong").is_none());
    assert_eq!(second.len(), 2);
    assert!(second.resolve("test.second", "pong").is_some());
}

#[test]
fn runtime13_performance_script_call_table_uses_borrowed_name_index_and_direct_dispatch() {
    let exports = HostExportRegistry::default();
    register_null_host_export(&exports, "test.direct", "run");

    assert_eq!(
        exports
            .call("test.direct", "run", Vec::new())
            .expect("direct registry dispatch should use the compiled call table"),
        ScriptHostValue::Null
    );
    assert!(matches!(
        exports.call("test.missing", "run", Vec::new()),
        Err(VmError::Operation(message))
            if message == "host export module not registered: test.missing"
    ));
    assert!(matches!(
        exports.call("test.direct", "missing", Vec::new()),
        Err(VmError::Operation(message))
            if message == "host export function not registered: test.direct.missing"
    ));

    let table_source = include_str!("../host/script_call_table.rs");
    assert!(table_source
        .contains("by_name: Arc<HashMap<Arc<str>, HashMap<Arc<str>, ScriptCallSiteId>>>"));
    assert!(table_source.contains(".get(module_name)?"));
    assert!(table_source.contains(".get(function_name)?"));
    assert!(table_source.contains("let frame = ScriptHostCallFrame::new("));
    assert!(table_source.contains("arguments,"));
    assert!(!table_source.contains("&arguments,"));
    assert!(table_source.contains("&granted_capabilities.capabilities,"));
    assert!(table_source.contains("with_active_script_runtime_call_context"));
    assert!(!table_source.contains(".to_string()"));
    assert!(!table_source.contains("capabilities.clone()"));

    let registry_source = include_str!("../host/host_export_registry.rs");
    assert!(registry_source.contains("struct HostExportRegistryState"));
    assert!(registry_source.contains("call_table: ScriptCallTable"));
    assert!(registry_source.contains("state.call_table.clone()"));
    assert!(!registry_source.contains("let (descriptor, callback) = {"));
}

#[test]
fn runtime13_performance_host_export_callback_dispatch_releases_registry_lock() {
    let exports = Arc::new(HostExportRegistry::default());
    let weak_exports = Arc::downgrade(&exports);
    exports
        .register_module(
            ScriptHostModuleDescriptor::new("test.reentrant", "0.1.0").with_function(
                ScriptHostFunctionDescriptor::new("ping", 0, 0, ScriptHostValueKind::Null),
            ),
            [HostExportFunction::new("ping", move |_| {
                let exports = weak_exports
                    .upgrade()
                    .expect("registry should remain alive during callback dispatch");
                assert!(exports
                    .script_call_table()
                    .resolve("test.reentrant", "ping")
                    .is_some());
                Ok(ScriptHostValue::Null)
            })],
        )
        .expect("reentrant host export should register");

    let (sender, receiver) = std::sync::mpsc::sync_channel(1);
    let exports_for_call = Arc::clone(&exports);
    let call_thread = std::thread::spawn(move || {
        let result = exports_for_call.call("test.reentrant", "ping", Vec::new());
        let _ = sender.send(result);
    });

    let result = receiver
        .recv_timeout(std::time::Duration::from_secs(5))
        .expect("host callback re-entry must not block on the registry state lock");
    assert_eq!(
        result.expect("reentrant host callback should succeed"),
        ScriptHostValue::Null
    );
    call_thread
        .join()
        .expect("reentrant host callback thread should exit cleanly");
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
                context.arguments.with_argument(0, |value| match value {
                    ScriptHostValueRef::Float(value) => Ok(ScriptHostValue::Float(value)),
                    value => Err(ScriptHostError::new(format!(
                        "expected float, received {:?}",
                        value.kind()
                    ))),
                })
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
