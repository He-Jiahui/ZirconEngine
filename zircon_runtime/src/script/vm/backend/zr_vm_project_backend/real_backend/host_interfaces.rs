use crate::core::framework::net::RpcPayloadSchema;
use crate::core::framework::script::ScriptHostValue;
use crate::script::{VmError, VmPluginHostContext, VmSystemStage, VM_HOST_INTERFACE_MODULE};
use zr_vm_rust_binding as zrvm;

use super::errors::{map_zr_error, zr_error};
use super::values::read_host_arguments_for_function;
use super::ZrVmRegistration;

#[derive(Clone, Copy)]
enum RegistrationChannel {
    System,
    BehaviorNode,
    RpcHandler,
    EditorOperation,
}

pub(super) fn register_host_interface_module(
    runtime: &mut zrvm::Runtime,
    host: &VmPluginHostContext,
) -> Result<ZrVmRegistration, VmError> {
    let module = zrvm::ModuleBuilder::new(VM_HOST_INTERFACE_MODULE)
        .module_version("1")
        .documentation(
            "Capability-gated registration of VM systems, behavior nodes, RPC handlers, and editor operations.",
        )
        .add_function(registration_function(
            "register_system",
            &["id", "stage", "module", "function"],
            RegistrationChannel::System,
            host.clone(),
        )?)
        .add_function(registration_function(
            "register_bt_node",
            &["id", "display_name", "module", "function"],
            RegistrationChannel::BehaviorNode,
            host.clone(),
        )?)
        .add_function(registration_function(
            "register_rpc_handler",
            &["id", "payload_schema", "module", "function"],
            RegistrationChannel::RpcHandler,
            host.clone(),
        )?)
        .add_function(registration_function(
            "register_editor_operation",
            &["operation", "module", "function"],
            RegistrationChannel::EditorOperation,
            host.clone(),
        )?)
        .build()
        .map_err(map_zr_error)?;
    runtime.register_native_module(module).map_err(map_zr_error)
}

fn registration_function(
    name: &'static str,
    parameters: &'static [&'static str],
    channel: RegistrationChannel,
    host: VmPluginHostContext,
) -> Result<zrvm::FunctionBuilder, VmError> {
    let arity = u16::try_from(parameters.len()).map_err(|_| {
        VmError::Operation(format!(
            "host interface {name} has too many parameters: {}",
            parameters.len()
        ))
    })?;
    let label = format!("{VM_HOST_INTERFACE_MODULE}.{name}");
    let callback_label = label.clone();
    let mut builder = zrvm::FunctionBuilder::new(name, arity, arity, move |context| {
        let arguments = read_host_arguments_for_function(context, &callback_label)?;
        let arguments = string_arguments(arguments, &callback_label)?;
        register_channel(channel, &host, &arguments)
            .map_err(|error| zr_error(format!("{callback_label} failed: {error}")))?;
        zrvm::Value::new_null()
    })
    .return_type("void")
    .documentation(&format!("Register a VM {} callback.", channel.label()));
    for parameter in parameters {
        builder = builder.parameter(parameter, "string", "");
    }
    Ok(builder)
}

fn string_arguments(
    arguments: Vec<ScriptHostValue>,
    label: &str,
) -> Result<Vec<String>, zrvm::Error> {
    arguments
        .into_iter()
        .enumerate()
        .map(|(index, value)| match value {
            ScriptHostValue::String(value) => Ok(value),
            other => Err(zr_error(format!(
                "{label} argument {index} expected string, received {:?}",
                other.kind()
            ))),
        })
        .collect()
}

fn register_channel(
    channel: RegistrationChannel,
    host: &VmPluginHostContext,
    arguments: &[String],
) -> Result<(), crate::script::VmHostInterfaceError> {
    let caller = host.interface_caller()?;
    match (channel, arguments) {
        (RegistrationChannel::System, [id, stage, module, function]) => {
            let stage = VmSystemStage::parse(stage).ok_or_else(|| {
                crate::script::VmHostInterfaceError::InvalidSystemStage(stage.clone())
            })?;
            host.host_interfaces
                .register_system(&caller, id.clone(), stage, module, function)?;
        }
        (RegistrationChannel::BehaviorNode, [id, display_name, module, function]) => {
            host.host_interfaces.register_behavior_node(
                &caller,
                id.clone(),
                display_name.clone(),
                module,
                function,
            )?;
        }
        (RegistrationChannel::RpcHandler, [id, payload_schema, module, function]) => {
            host.host_interfaces.register_rpc_handler(
                &caller,
                id.clone(),
                RpcPayloadSchema::for_type_path(payload_schema.as_str()),
                module,
                function,
            )?;
        }
        (RegistrationChannel::EditorOperation, [operation, module, function]) => {
            host.host_interfaces.register_editor_operation(
                &caller,
                operation.clone(),
                module,
                function,
            )?;
        }
        (channel, arguments) => {
            return Err(crate::script::VmHostInterfaceError::InvalidArgumentCount {
                channel: channel.label(),
                expected: channel.arity(),
                actual: arguments.len(),
            });
        }
    }
    Ok(())
}

impl RegistrationChannel {
    const fn arity(self) -> usize {
        match self {
            Self::System | Self::BehaviorNode | Self::RpcHandler => 4,
            Self::EditorOperation => 3,
        }
    }

    const fn label(self) -> &'static str {
        match self {
            Self::System => "system",
            Self::BehaviorNode => "behavior node",
            Self::RpcHandler => "RPC handler",
            Self::EditorOperation => "editor operation",
        }
    }
}
