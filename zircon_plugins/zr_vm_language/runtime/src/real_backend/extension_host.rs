use std::sync::Arc;

use zircon_runtime::core::framework::net::RpcPayloadSchema;
use zircon_runtime::core::framework::script::{
    ScriptHostArguments, ScriptHostError, ScriptHostValueRef,
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
    let caller = Arc::new(host.interface_caller().map_err(|error| {
        VmError::Operation(format!(
            "failed to authenticate ZrVM extension host caller: {error}"
        ))
    })?);
    let registry = host.host_interfaces.clone();
    let system_caller = Arc::clone(&caller);
    let system_registry = registry.clone();
    let behavior_caller = Arc::clone(&caller);
    let behavior_registry = registry.clone();
    let rpc_caller = Arc::clone(&caller);
    let rpc_registry = registry.clone();

    let module = zrvm::ModuleBuilder::new(VM_HOST_INTERFACE_MODULE)
        .module_version(HOST_MODULE_VERSION)
        .documentation("Capability-gated Zircon extension registration channels.")
        .add_function(string_function(
            "register_system",
            4,
            move |arguments, label| {
                borrow_string(arguments, 0, label, |system_id| {
                    borrow_string(arguments, 1, label, |stage_name| {
                        let stage = VmSystemStage::parse(stage_name).ok_or_else(|| {
                            zr_error(format!(
                            "invalid VM system stage '{}'; expected fixed_update, update, or last",
                            stage_name
                        ))
                        })?;
                        borrow_string(arguments, 2, label, |module_name| {
                            borrow_string(arguments, 3, label, |function_name| {
                                system_registry
                                    .register_system(
                                        &system_caller,
                                        system_id,
                                        stage,
                                        module_name,
                                        function_name,
                                    )
                                    .map_err(|error| zr_error(error.to_string()))?;
                                zrvm::Value::new_null()
                            })
                            .map_err(|error| zr_error(error.message))
                        })
                        .map_err(|error| zr_error(error.message))
                    })
                    .map_err(|error| zr_error(error.message))
                })
                .map_err(|error| zr_error(error.message))
            },
        ))
        .add_function(string_function(
            "register_bt_node",
            4,
            move |arguments, label| {
                borrow_string(arguments, 0, label, |node_id| {
                    borrow_string(arguments, 1, label, |display_name| {
                        borrow_string(arguments, 2, label, |module_name| {
                            borrow_string(arguments, 3, label, |function_name| {
                                behavior_registry
                                    .register_behavior_node(
                                        &behavior_caller,
                                        node_id,
                                        display_name,
                                        module_name,
                                        function_name,
                                    )
                                    .map_err(|error| zr_error(error.to_string()))?;
                                zrvm::Value::new_null()
                            })
                            .map_err(|error| zr_error(error.message))
                        })
                        .map_err(|error| zr_error(error.message))
                    })
                    .map_err(|error| zr_error(error.message))
                })
                .map_err(|error| zr_error(error.message))
            },
        ))
        .add_function(string_function(
            "register_rpc_handler",
            4,
            move |arguments, label| {
                borrow_string(arguments, 0, label, |handler_id| {
                    borrow_string(arguments, 1, label, |payload_type| {
                        borrow_string(arguments, 2, label, |module_name| {
                            borrow_string(arguments, 3, label, |function_name| {
                                rpc_registry
                                    .register_rpc_handler(
                                        &rpc_caller,
                                        handler_id,
                                        RpcPayloadSchema::for_type_path(payload_type),
                                        module_name,
                                        function_name,
                                    )
                                    .map_err(|error| zr_error(error.to_string()))?;
                                zrvm::Value::new_null()
                            })
                            .map_err(|error| zr_error(error.message))
                        })
                        .map_err(|error| zr_error(error.message))
                    })
                    .map_err(|error| zr_error(error.message))
                })
                .map_err(|error| zr_error(error.message))
            },
        ))
        .add_function(string_function(
            "register_editor_operation",
            3,
            move |arguments, label| {
                borrow_string(arguments, 0, label, |operation_id| {
                    borrow_string(arguments, 1, label, |module_name| {
                        borrow_string(arguments, 2, label, |function_name| {
                            registry
                                .register_editor_operation(
                                    &caller,
                                    operation_id,
                                    module_name,
                                    function_name,
                                )
                                .map_err(|error| zr_error(error.to_string()))?;
                            zrvm::Value::new_null()
                        })
                        .map_err(|error| zr_error(error.message))
                    })
                    .map_err(|error| zr_error(error.message))
                })
                .map_err(|error| zr_error(error.message))
            },
        ))
        .build()
        .map_err(map_zr_error)?;
    runtime.register_native_module(module).map_err(map_zr_error)
}

fn string_function(
    name: &str,
    arity: u16,
    callback: impl Fn(&ScriptHostArguments<'_>, &str) -> Result<zrvm::Value, zrvm::Error>
        + Send
        + Sync
        + 'static,
) -> zrvm::FunctionBuilder {
    let label = format!("{VM_HOST_INTERFACE_MODULE}.{name}");
    let mut builder = zrvm::FunctionBuilder::new(name, arity, arity, move |context| {
        with_extension_registration_strings(context, &label, &callback)
    })
    .return_type("void");
    for index in 0..arity {
        builder = builder.parameter(&format!("argument{index}"), "string", "");
    }
    builder
}

fn with_extension_registration_strings(
    context: &zrvm::NativeCallContext<'_>,
    label: &str,
    callback: &impl Fn(&ScriptHostArguments<'_>, &str) -> Result<zrvm::Value, zrvm::Error>,
) -> Result<zrvm::Value, zrvm::Error> {
    let source = ZrVmScriptHostArgumentSource::new(context, label)?;
    let host_arguments = ScriptHostArguments::new(&source);
    callback(&host_arguments, label)
}

fn borrow_string<T>(
    arguments: &ScriptHostArguments<'_>,
    index: usize,
    label: &str,
    visitor: impl FnOnce(&str) -> Result<T, zrvm::Error>,
) -> Result<T, zrvm::Error> {
    arguments
        .with_argument(index, |value| match value {
            ScriptHostValueRef::String(value) => {
                visitor(value).map_err(|error| ScriptHostError::new(error.message))
            }
            value => Err(ScriptHostError::new(format!(
                "{label} argument {index} must be a string, received {:?}",
                value.kind()
            ))),
        })
        .map_err(|error| zr_error(error.message))
}
