use zircon_runtime::core::framework::script::{
    ScriptHostFunctionDescriptor, ScriptHostPrototypeKind,
};
use zircon_runtime::script::{CapabilitySet, ScriptCallSite, VmError, VmPluginHostContext};
use zr_vm_rust_binding as zrvm;

use super::errors::{map_zr_error, zr_error};
use super::reflection_host::register_reflection_host_module;
use super::values::{read_host_arguments_for_function, to_zr_value_for_function};
use super::ZrVmRegistration;
use crate::ReflectionHostModule;

pub(super) struct RegisteredHostModules {
    pub(super) registrations: Vec<ZrVmRegistration>,
    pub(super) reflection_host: ReflectionHostModule,
}

pub(super) fn register_host_modules(
    runtime: &mut zrvm::Runtime,
    host: &VmPluginHostContext,
) -> Result<RegisteredHostModules, VmError> {
    let reflection_host = ReflectionHostModule::default();
    let mut registrations = vec![register_reflection_host_module(
        runtime,
        reflection_host.clone(),
    )?];
    let call_table = host.host_exports.script_call_table()?;
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
            let call_site = call_table
                .resolve(&module.descriptor.name, &function.name)
                .ok_or_else(|| {
                    VmError::Operation(format!(
                        "script call table did not contain {}.{}",
                        module.descriptor.name, function.name
                    ))
                })?;
            builder = builder.add_function(build_native_function(
                &module.descriptor.name,
                function,
                call_site,
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
    Ok(RegisteredHostModules {
        registrations,
        reflection_host,
    })
}

pub(super) fn native_function_label(module_name: &str, function_name: &str) -> String {
    format!("{module_name}.{function_name}")
}

pub(super) fn validate_native_function_arity(
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
    call_site: ScriptCallSite,
    capabilities: CapabilitySet,
) -> Result<zrvm::FunctionBuilder, VmError> {
    let function_name = function.name.clone();
    let label = native_function_label(module_name, &function_name);
    let (min, max) = validate_native_function_arity(module_name, function)?;
    let callback_label = label.clone();
    let mut builder = zrvm::FunctionBuilder::new(&function.name, min, max, move |context| {
        let arguments = read_host_arguments_for_function(context, &callback_label)?;
        let value = call_site.call(arguments, &capabilities).map_err(|error| {
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
