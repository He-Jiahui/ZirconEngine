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

fn build_native_function(
    module_name: &str,
    function: &ScriptHostFunctionDescriptor,
    exports: zircon_runtime::script::HostExportRegistry,
    capabilities: CapabilitySet,
) -> Result<zrvm::FunctionBuilder, VmError> {
    let module_name = module_name.to_string();
    let function_name = function.name.clone();
    let min = u16::try_from(function.min_argument_count).map_err(|_| {
        VmError::Operation(format!(
            "zr_vm function {}.{} min arity exceeds u16",
            module_name, function_name
        ))
    })?;
    let max = u16::try_from(function.max_argument_count).map_err(|_| {
        VmError::Operation(format!(
            "zr_vm function {}.{} max arity exceeds u16",
            module_name, function_name
        ))
    })?;

    let mut builder = zrvm::FunctionBuilder::new(&function.name, min, max, move |context| {
        let arguments = read_host_arguments(context)?;
        let value = exports
            .call_with_capabilities(&module_name, &function_name, arguments, &capabilities)
            .map_err(|error| zr_error(error.to_string()))?;
        to_zr_value(value)
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

fn read_host_arguments(
    context: &zrvm::NativeCallContext,
) -> Result<Vec<ScriptHostValue>, zrvm::Error> {
    let count = context.argument_count()?;
    let mut arguments = Vec::with_capacity(count);
    for index in 0..count {
        let value = context.argument(index)?;
        arguments.push(from_zr_value(&value)?);
    }
    Ok(arguments)
}

fn from_zr_value(value: &zrvm::Value) -> Result<ScriptHostValue, zrvm::Error> {
    match value.kind() {
        zrvm::ValueKind::Null => Ok(ScriptHostValue::Null),
        zrvm::ValueKind::Bool => Ok(ScriptHostValue::Bool(value.as_bool()?)),
        zrvm::ValueKind::Int => Ok(ScriptHostValue::Int(value.as_int()?)),
        zrvm::ValueKind::Float => Ok(ScriptHostValue::Float(value.as_float()?)),
        zrvm::ValueKind::String => Ok(ScriptHostValue::String(value.as_string()?)),
        other => Err(zr_error(format!(
            "unsupported zr_vm native argument kind {other:?}"
        ))),
    }
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
