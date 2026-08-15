use zircon_runtime::core::framework::net::RpcPayloadSchema;
use zircon_runtime::core::framework::script::{
    ScriptHostArguments, ScriptHostError, ScriptHostHotPathMetrics, ScriptHostValueRef,
};
use zircon_runtime::script::{
    VmError, VmPluginHostContext, VmSystemStage, VM_HOST_INTERFACE_MODULE,
};
use zr_vm_rust_binding as zrvm;

use super::errors::{map_zr_error, zr_error};
use super::values::ZrVmScriptHostArgumentSource;
use super::ZrVmRegistration;

const HOST_MODULE_VERSION: &str = "0.1.0";

pub(super) fn register_extension_host_module(
    runtime: &mut zrvm::Runtime,
    host: &VmPluginHostContext,
) -> Result<ZrVmRegistration, zircon_runtime::script::VmError> {
    let caller = host.interface_caller().map_err(|error| {
        VmError::Operation(format!(
            "failed to authenticate ZrVM extension host caller: {error}"
        ))
    })?;
    let registry = host.host_interfaces.clone();
    let system_caller = caller.clone();
    let system_registry = registry.clone();
    let behavior_caller = caller.clone();
    let behavior_registry = registry.clone();
    let rpc_caller = caller.clone();
    let rpc_registry = registry.clone();

    let module = zrvm::ModuleBuilder::new(VM_HOST_INTERFACE_MODULE)
        .module_version(HOST_MODULE_VERSION)
        .documentation("Capability-gated Zircon extension registration channels.")
        .add_function(string_function("register_system", 4, move |arguments| {
            let stage = VmSystemStage::parse(&arguments[1]).ok_or_else(|| {
                zr_error(format!(
                    "invalid VM system stage '{}'; expected fixed_update, update, or last",
                    arguments[1]
                ))
            })?;
            system_registry
                .register_system(
                    &system_caller,
                    &arguments[0],
                    stage,
                    &arguments[2],
                    &arguments[3],
                )
                .map_err(|error| zr_error(error.to_string()))?;
            zrvm::Value::new_null()
        }))
        .add_function(string_function("register_bt_node", 4, move |arguments| {
            behavior_registry
                .register_behavior_node(
                    &behavior_caller,
                    &arguments[0],
                    &arguments[1],
                    &arguments[2],
                    &arguments[3],
                )
                .map_err(|error| zr_error(error.to_string()))?;
            zrvm::Value::new_null()
        }))
        .add_function(string_function(
            "register_rpc_handler",
            4,
            move |arguments| {
                rpc_registry
                    .register_rpc_handler(
                        &rpc_caller,
                        &arguments[0],
                        RpcPayloadSchema::for_type_path(&arguments[1]),
                        &arguments[2],
                        &arguments[3],
                    )
                    .map_err(|error| zr_error(error.to_string()))?;
                zrvm::Value::new_null()
            },
        ))
        .add_function(string_function(
            "register_editor_operation",
            3,
            move |arguments| {
                registry
                    .register_editor_operation(&caller, &arguments[0], &arguments[1], &arguments[2])
                    .map_err(|error| zr_error(error.to_string()))?;
                zrvm::Value::new_null()
            },
        ))
        .build()
        .map_err(map_zr_error)?;
    runtime.register_native_module(module).map_err(map_zr_error)
}

fn string_function(
    name: &str,
    arity: u16,
    callback: impl Fn(Vec<String>) -> Result<zrvm::Value, zrvm::Error> + Send + Sync + 'static,
) -> zrvm::FunctionBuilder {
    let label = format!("{VM_HOST_INTERFACE_MODULE}.{name}");
    let mut builder = zrvm::FunctionBuilder::new(name, arity, arity, move |context| {
        let arguments = read_extension_registration_strings_at_business_boundary(
            context,
            &label,
            arity as usize,
        )?;
        callback(arguments)
    })
    .return_type("void");
    for index in 0..arity {
        builder = builder.parameter(&format!("argument{index}"), "string", "");
    }
    builder
}

fn read_extension_registration_strings_at_business_boundary(
    context: &zrvm::NativeCallContext<'_>,
    label: &str,
    count: usize,
) -> Result<Vec<String>, zrvm::Error> {
    let source = ZrVmScriptHostArgumentSource::new(context, label)?;
    let host_arguments = ScriptHostArguments::new(&source);
    let mut registered_arguments = Vec::with_capacity(count);
    for index in 0..count {
        let value = host_arguments
            .with_argument(index, |value| match value {
                ScriptHostValueRef::String(value) => {
                    ScriptHostHotPathMetrics::record_guest_string_copy(value.len());
                    Ok(value.to_owned())
                }
                value => Err(ScriptHostError::new(format!(
                    "{label} argument {index} must be a string, received {:?}",
                    value.kind()
                ))),
            })
            .map_err(|error| zr_error(error.message))?;
        registered_arguments.push(value);
    }
    Ok(registered_arguments)
}
