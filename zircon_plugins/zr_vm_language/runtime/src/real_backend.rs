use std::sync::{Mutex, MutexGuard, OnceLock};

use zircon_runtime::core::framework::script::{
    ScriptHostFunctionDescriptor, ScriptHostPrototypeKind, ScriptHostValue,
};
use zircon_runtime::script::{
    CapabilitySet, VmError, VmPluginHostContext, VmPluginInstance, VmPluginManifest,
    VmPluginPackage, VmStateBlob, ZrVmExecutionMode,
};
use zr_vm_rust_binding as zrvm;
use zr_vm_rust_binding_sys as zrvm_sys;

type ZrVmRegistration = zrvm::NativeModuleRegistration;

pub fn load_project_package(
    package: &VmPluginPackage,
    host: &VmPluginHostContext,
) -> Result<Box<dyn VmPluginInstance>, VmError> {
    let project = package
        .zr_vm_project
        .as_ref()
        .ok_or_else(|| VmError::Parse("zr_vm project metadata missing".to_string()))?;
    let _guard = acquire_zr_vm_lock();
    let mut runtime = zrvm::RuntimeBuilder::standard()
        .build()
        .map_err(map_zr_error)?;
    let registrations = register_host_modules(&mut runtime, host)?;
    let workspace = zrvm::ProjectWorkspace::open(&project.project_path).map_err(map_zr_error)?;
    workspace
        .compile(
            &mut runtime,
            &zrvm::CompileOptions {
                emit_intermediate: false,
                incremental: true,
            },
        )
        .map_err(map_zr_error)?;
    let run_options = zrvm::RunOptions {
        execution_mode: match project.execution_mode {
            ZrVmExecutionMode::Interp => zrvm::ExecutionMode::Interp,
            ZrVmExecutionMode::Binary => zrvm::ExecutionMode::Binary,
        },
        // Lifecycle export calls name the target module separately; keeping
        // this empty makes ZrVM load the project entry before resolving it.
        module_name: None,
        program_args: Vec::new(),
    };
    let session = workspace
        .start_session(&mut runtime, &run_options)
        .map_err(map_zr_error)?;

    Ok(Box::new(ZrVmPluginInstance {
        manifest: package.manifest.clone(),
        session,
        _registrations: registrations,
        runtime,
        entry_module: project.entry_module.clone(),
    }))
}

struct ZrVmPluginInstance {
    manifest: VmPluginManifest,
    session: zrvm::ProjectSession,
    _registrations: Vec<ZrVmRegistration>,
    runtime: zrvm::Runtime,
    entry_module: String,
}

unsafe impl Send for ZrVmPluginInstance {}
unsafe impl Sync for ZrVmPluginInstance {}

impl VmPluginInstance for ZrVmPluginInstance {
    fn manifest(&self) -> &VmPluginManifest {
        &self.manifest
    }

    fn activate(&mut self, _host: &VmPluginHostContext) -> Result<(), VmError> {
        let _guard = acquire_zr_vm_lock();
        self.call_optional_export("activate", &[]).map(|_| ())
    }

    fn deactivate(&mut self) -> Result<(), VmError> {
        let _guard = acquire_zr_vm_lock();
        self.call_optional_export("deactivate", &[]).map(|_| ())
    }

    fn save_state(&mut self) -> Result<VmStateBlob, VmError> {
        let _guard = acquire_zr_vm_lock();
        let value = match self.call_optional_export("saveState", &[])? {
            Some(value) => value,
            None => return Ok(VmStateBlob::default()),
        };
        match value.kind() {
            zrvm::ValueKind::String => Ok(VmStateBlob {
                bytes: value.as_string().map_err(map_zr_error)?.into_bytes(),
            }),
            zrvm::ValueKind::Null => Ok(VmStateBlob::default()),
            other => Err(VmError::Operation(format!(
                "zr_vm saveState returned unsupported value kind {other:?}"
            ))),
        }
    }

    fn restore_state(&mut self, state: &VmStateBlob) -> Result<(), VmError> {
        let _guard = acquire_zr_vm_lock();
        let state = String::from_utf8(state.bytes.clone()).map_err(|error| {
            VmError::Operation(format!("zr_vm restoreState requires UTF-8 state: {error}"))
        })?;
        let argument = zrvm::Value::new_string(&state).map_err(map_zr_error)?;
        self.call_optional_export("restoreState", &[argument])
            .map(|_| ())
    }
}

impl ZrVmPluginInstance {
    fn call_optional_export(
        &mut self,
        export_name: &str,
        arguments: &[zrvm::Value],
    ) -> Result<Option<zrvm::Value>, VmError> {
        let _keep_runtime_alive = &self.runtime;
        match self
            .session
            .call_module_export(&self.entry_module, export_name, arguments)
        {
            Ok(value) => Ok(Some(value)),
            Err(error) if is_optional_export_missing(&error) => Ok(None),
            Err(error) => Err(map_zr_error(error)),
        }
    }
}

fn register_host_modules(
    runtime: &mut zrvm::Runtime,
    host: &VmPluginHostContext,
) -> Result<Vec<ZrVmRegistration>, VmError> {
    let mut registrations = Vec::new();
    for module in host.host_exports.modules() {
        let mut builder = zrvm::ModuleBuilder::new(&module.descriptor.name)
            .module_version(&module.descriptor.version);
        if let Some(documentation) = &module.descriptor.documentation {
            builder = builder.documentation(documentation);
        }

        for type_descriptor in &module.descriptor.types {
            let mut type_builder = zrvm::TypeBuilder::new(
                &type_descriptor.name,
                zr_prototype_type(type_descriptor.prototype_kind),
            )
            .allow_value_construction(type_descriptor.allow_value_construction);
            if let Some(documentation) = &type_descriptor.documentation {
                type_builder = type_builder.documentation(documentation);
            }
            for field in &type_descriptor.fields {
                type_builder = type_builder.field(
                    &field.name,
                    &field.type_ref.type_name,
                    field.documentation.as_deref().unwrap_or(""),
                    0,
                );
            }
            builder = builder.add_type(type_builder);
        }

        for function in &module.descriptor.functions {
            builder = builder.add_function(build_native_function(
                &module.descriptor.name,
                function,
                host.host_exports.clone(),
                host.capabilities.clone(),
            )?);
        }

        let native_module = builder.build().map_err(map_zr_error)?;
        registrations.push(
            runtime
                .register_native_module(native_module)
                .map_err(map_zr_error)?,
        );
    }
    Ok(registrations)
}

fn native_function_label(module_name: &str, function_name: &str) -> String {
    format!("{module_name}.{function_name}")
}

fn validate_native_function_arity(
    module_name: &str,
    function: &ScriptHostFunctionDescriptor,
) -> Result<(u16, u16), VmError> {
    let label = native_function_label(module_name, &function.name);
    let min = u16::try_from(function.min_argument_count)
        .map_err(|_| VmError::Operation(format!("zr_vm function {label} min arity exceeds u16")))?;
    let max = u16::try_from(function.max_argument_count)
        .map_err(|_| VmError::Operation(format!("zr_vm function {label} max arity exceeds u16")))?;
    if function.min_argument_count > function.max_argument_count {
        return Err(VmError::Operation(format!(
            "zr_vm function {label} min arity {} exceeds max arity {}",
            function.min_argument_count, function.max_argument_count
        )));
    }
    if function.parameters.len() > function.max_argument_count {
        return Err(VmError::Operation(format!(
            "zr_vm function {label} declares {} parameters but max arity is {}",
            function.parameters.len(),
            function.max_argument_count
        )));
    }
    Ok((min, max))
}

fn build_native_function(
    module_name: &str,
    function: &ScriptHostFunctionDescriptor,
    exports: zircon_runtime::script::HostExportRegistry,
    capabilities: CapabilitySet,
) -> Result<zrvm::FunctionBuilder, VmError> {
    let function_name = function.name.clone();
    let label = native_function_label(module_name, &function_name);
    let (min, max) = validate_native_function_arity(module_name, function)?;
    let module_name = module_name.to_string();

    let callback_label = label.clone();
    let mut builder = zrvm::FunctionBuilder::new(&function.name, min, max, move |context| {
        let arguments = read_host_arguments_for_function(context, &callback_label)?;
        let value = exports
            .call_with_capabilities(&module_name, &function_name, arguments, &capabilities)
            .map_err(|error| {
                zr_error(format!(
                    "zr_vm host callback {callback_label} failed: {error}"
                ))
            })?;
        to_zr_value_for_function(value, &callback_label)
    })
    .return_type(&function.return_type.type_name);
    if let Some(documentation) = &function.documentation {
        builder = builder.documentation(documentation);
    }
    for parameter in &function.parameters {
        builder = builder.parameter(
            &parameter.name,
            &parameter.type_ref.type_name,
            parameter.documentation.as_deref().unwrap_or(""),
        );
    }
    Ok(builder)
}

fn read_host_arguments_for_function(
    context: &zrvm::NativeCallContext,
    function_label: &str,
) -> Result<Vec<ScriptHostValue>, zrvm::Error> {
    let count = context.argument_count().map_err(|error| {
        zr_error(format!(
            "failed to read argument count for {function_label}: {error}"
        ))
    })?;
    let mut arguments = Vec::with_capacity(count);
    for index in 0..count {
        let value = context.argument(index).map_err(|error| {
            zr_error(format!(
                "failed to read argument {index} for {function_label}: {error}"
            ))
        })?;
        arguments.push(from_zr_value_for_function(&value, function_label, index)?);
    }
    Ok(arguments)
}

fn from_zr_value_for_function(
    value: &zrvm::Value,
    function_label: &str,
    index: usize,
) -> Result<ScriptHostValue, zrvm::Error> {
    match value.kind() {
        zrvm::ValueKind::Null => Ok(ScriptHostValue::Null),
        zrvm::ValueKind::Bool => Ok(ScriptHostValue::Bool(value.as_bool()?)),
        zrvm::ValueKind::Int => Ok(ScriptHostValue::Int(value.as_int()?)),
        zrvm::ValueKind::Float => Ok(ScriptHostValue::Float(value.as_float()?)),
        zrvm::ValueKind::String => Ok(ScriptHostValue::String(value.as_string()?)),
        other => Err(zr_error(format!(
            "unsupported zr_vm native argument kind {other:?} at {function_label} argument {index}"
        ))),
    }
}

fn to_zr_value_for_function(
    value: ScriptHostValue,
    function_label: &str,
) -> Result<zrvm::Value, zrvm::Error> {
    to_zr_value(value).map_err(|error| {
        zr_error(format!(
            "failed to lower host return value for {function_label}: {error}"
        ))
    })
}

fn to_zr_value(value: ScriptHostValue) -> Result<zrvm::Value, zrvm::Error> {
    match value {
        ScriptHostValue::Null => zrvm::Value::new_null(),
        ScriptHostValue::Bool(value) => zrvm::Value::new_bool(value),
        ScriptHostValue::Int(value) => zrvm::Value::new_int(value),
        ScriptHostValue::Float(value) => zrvm::Value::new_float(value),
        ScriptHostValue::String(value) => zrvm::Value::new_string(&value),
        ScriptHostValue::Bytes(value) => zrvm::Value::new_string(&String::from_utf8_lossy(&value)),
        ScriptHostValue::HostHandle(value) => zrvm::Value::new_int(value as i64),
    }
}

fn zr_prototype_type(kind: ScriptHostPrototypeKind) -> zrvm::PrototypeType {
    match kind {
        ScriptHostPrototypeKind::Module => zrvm::PrototypeType::Module,
        ScriptHostPrototypeKind::Class => zrvm::PrototypeType::Class,
        ScriptHostPrototypeKind::Interface => zrvm::PrototypeType::Interface,
        ScriptHostPrototypeKind::Struct => zrvm::PrototypeType::Struct,
        ScriptHostPrototypeKind::Enum => zrvm::PrototypeType::Enum,
        ScriptHostPrototypeKind::Native => zrvm::PrototypeType::Native,
    }
}

fn map_zr_error(error: zrvm::Error) -> VmError {
    VmError::Operation(format!("zr_vm binding error: {error}"))
}

fn zr_error(message: impl Into<String>) -> zrvm::Error {
    zrvm::Error::new(
        zrvm_sys::ZrRustBindingStatus::ZR_RUST_BINDING_STATUS_INTERNAL_ERROR,
        message,
    )
}

fn is_optional_export_missing(error: &zrvm::Error) -> bool {
    error.status == zrvm_sys::ZrRustBindingStatus::ZR_RUST_BINDING_STATUS_NOT_FOUND
        || error.message.contains("not found")
        || error.message.contains("NOT_FOUND")
}

fn acquire_zr_vm_lock() -> MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
        .lock()
        .expect("zr_vm runtime lock should not be poisoned")
}

#[cfg(test)]
mod tests {
    use super::*;
    use zircon_runtime::core::framework::script::{
        ScriptHostFunctionDescriptor, ScriptHostModuleDescriptor, ScriptHostParameterDescriptor,
        ScriptHostValueKind,
    };
    use zircon_runtime::script::{CapabilitySet, HostExportFunction, HostExportRegistry};

    fn descriptor_with_arity(min: usize, max: usize) -> ScriptHostFunctionDescriptor {
        ScriptHostFunctionDescriptor::new("bad", min, max, ScriptHostValueKind::Null)
    }

    #[test]
    fn validate_native_function_arity_rejects_min_overflow() {
        let descriptor =
            descriptor_with_arity(usize::from(u16::MAX) + 1, usize::from(u16::MAX) + 1);
        let error = validate_native_function_arity("example", &descriptor).unwrap_err();

        assert!(error.to_string().contains("example.bad"));
        assert!(error.to_string().contains("min arity exceeds u16"));
    }

    #[test]
    fn validate_native_function_arity_rejects_max_overflow() {
        let descriptor = descriptor_with_arity(0, usize::from(u16::MAX) + 1);
        let error = validate_native_function_arity("example", &descriptor).unwrap_err();

        assert!(error.to_string().contains("example.bad"));
        assert!(error.to_string().contains("max arity exceeds u16"));
    }

    #[test]
    fn validate_native_function_arity_rejects_min_greater_than_max() {
        let descriptor = descriptor_with_arity(3, 2);
        let error = validate_native_function_arity("example", &descriptor).unwrap_err();

        assert!(error.to_string().contains("example.bad"));
        assert!(error
            .to_string()
            .contains("min arity 3 exceeds max arity 2"));
    }

    #[test]
    fn validate_native_function_arity_rejects_parameter_count_above_max() {
        let descriptor = descriptor_with_arity(0, 1)
            .with_parameter(ScriptHostParameterDescriptor::new(
                "left",
                ScriptHostValueKind::Float,
            ))
            .with_parameter(ScriptHostParameterDescriptor::new(
                "right",
                ScriptHostValueKind::Float,
            ));
        let error = validate_native_function_arity("example", &descriptor).unwrap_err();

        assert!(error.to_string().contains("example.bad"));
        assert!(error
            .to_string()
            .contains("declares 2 parameters but max arity is 1"));
    }

    #[test]
    fn to_zr_value_lowers_supported_host_values() {
        assert!(matches!(
            to_zr_value(ScriptHostValue::Null).unwrap().kind(),
            zrvm::ValueKind::Null
        ));
        assert!(to_zr_value(ScriptHostValue::Bool(true))
            .unwrap()
            .as_bool()
            .unwrap());
        assert_eq!(
            to_zr_value(ScriptHostValue::Int(7))
                .unwrap()
                .as_int()
                .unwrap(),
            7
        );
        assert_eq!(
            to_zr_value(ScriptHostValue::Float(1.5))
                .unwrap()
                .as_float()
                .unwrap(),
            1.5
        );
        assert_eq!(
            to_zr_value(ScriptHostValue::String("ok".to_string()))
                .unwrap()
                .as_string()
                .unwrap(),
            "ok"
        );
        assert_eq!(
            to_zr_value(ScriptHostValue::Bytes(vec![104, 105]))
                .unwrap()
                .as_string()
                .unwrap(),
            "hi"
        );
        assert_eq!(
            to_zr_value(ScriptHostValue::HostHandle(42))
                .unwrap()
                .as_int()
                .unwrap(),
            42
        );
    }

    #[test]
    fn from_zr_value_for_function_rejects_unsupported_argument_kind_with_context() {
        let value = zrvm::Value::new_array().unwrap();
        let error = from_zr_value_for_function(&value, "example.unsupported", 2).unwrap_err();

        assert!(error.message.contains("example.unsupported"));
        assert!(error.message.contains("argument 2"));
        assert!(error.message.contains("Array"));
    }

    #[test]
    fn to_zr_value_for_function_wraps_return_lowering_errors_with_context() {
        let error = match to_zr_value_for_function(
            ScriptHostValue::String("bad\0value".to_string()),
            "example.return_value",
        ) {
            Ok(_) => panic!("expected return lowering to reject interior NUL strings"),
            Err(error) => error,
        };

        assert!(error
            .message
            .contains("failed to lower host return value for example.return_value"));
        assert!(error.message.contains("string contains interior NUL"));
    }

    #[test]
    fn callback_dispatch_errors_include_function_context() {
        let exports = HostExportRegistry::default();
        exports
            .register_module(
                ScriptHostModuleDescriptor::new("example", "0.1.0")
                    .with_capability("allowed")
                    .with_function(
                        ScriptHostFunctionDescriptor::new(
                            "secure",
                            0,
                            0,
                            ScriptHostValueKind::Null,
                        )
                        .with_required_capability("allowed"),
                    ),
                [HostExportFunction::new("secure", |_| {
                    Ok(ScriptHostValue::Null)
                })],
            )
            .unwrap();

        let label = native_function_label("example", "secure");
        let error = exports
            .call_with_capabilities("example", "secure", Vec::new(), &CapabilitySet::default())
            .map_err(|error| zr_error(format!("zr_vm host callback {label} failed: {error}")))
            .unwrap_err();

        assert!(error.message.contains("example.secure"));
        assert!(error.message.contains("capability"));
    }
}
