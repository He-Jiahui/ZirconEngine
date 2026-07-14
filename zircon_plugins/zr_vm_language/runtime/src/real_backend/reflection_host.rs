use zircon_runtime::core::framework::script::ScriptHostValue;
use zircon_runtime::script::VmError;
use zr_vm_rust_binding as zrvm;

use super::errors::{map_zr_error, zr_error};
use super::values::read_host_arguments_for_function;
use super::ZrVmRegistration;
use crate::ReflectionHostModule;

const REFLECTION_MODULE_NAME: &str = "zircon.reflection";
const REFLECTION_MODULE_VERSION: &str = "1";

pub(super) fn register_reflection_host_module(
    runtime: &mut zrvm::Runtime,
    reflection: ReflectionHostModule,
) -> Result<ZrVmRegistration, VmError> {
    let resolve_reflection = reflection.clone();
    let read_reflection = reflection.clone();
    let write_reflection = reflection;
    let module = zrvm::ModuleBuilder::new(REFLECTION_MODULE_NAME)
        .module_version(REFLECTION_MODULE_VERSION)
        .documentation("Numeric VM-to-World reflection bridge compiled from public type schemas.")
        .add_function(
            zrvm::FunctionBuilder::new("resolve", 2, 2, move |context| {
                let arguments =
                    read_host_arguments_for_function(context, "zircon.reflection.resolve")?;
                let type_path = expect_string(&arguments, 0, "zircon.reflection.resolve")?;
                let member_name = expect_string(&arguments, 1, "zircon.reflection.resolve")?;
                let token = resolve_reflection
                    .resolve(type_path, member_name)
                    .map_err(reflection_error)?;
                zrvm::Value::new_int(token as i64)
            })
            .parameter(
                "type_path",
                "String",
                "Fully-qualified public reflected type path",
            )
            .parameter("member_name", "String", "Public reflected field name")
            .return_type("Integer")
            .documentation(
                "Resolves names once during package loading and returns an opaque token.",
            ),
        )
        .add_function(
            zrvm::FunctionBuilder::new("read", 2, 2, move |context| {
                let arguments =
                    read_host_arguments_for_function(context, "zircon.reflection.read")?;
                let token = expect_int(&arguments, 0, "zircon.reflection.read")? as u64;
                let entity = expect_int(&arguments, 1, "zircon.reflection.read")? as u64;
                let value = read_reflection
                    .read_json(token, entity)
                    .map_err(reflection_error)?;
                zrvm::Value::new_string(&value)
            })
            .parameter(
                "call_site",
                "Integer",
                "Opaque numeric token returned by resolve",
            )
            .parameter("entity", "Integer", "World entity identifier")
            .return_type("String")
            .documentation(
                "Reads a reflected value by dense numeric slots and returns tagged JSON.",
            ),
        )
        .add_function(
            zrvm::FunctionBuilder::new("write", 3, 3, move |context| {
                let arguments =
                    read_host_arguments_for_function(context, "zircon.reflection.write")?;
                let token = expect_int(&arguments, 0, "zircon.reflection.write")? as u64;
                let entity = expect_int(&arguments, 1, "zircon.reflection.write")? as u64;
                let value_json = expect_string(&arguments, 2, "zircon.reflection.write")?;
                let changed = write_reflection
                    .write_json(token, entity, value_json)
                    .map_err(reflection_error)?;
                zrvm::Value::new_bool(changed)
            })
            .parameter(
                "call_site",
                "Integer",
                "Opaque numeric token returned by resolve",
            )
            .parameter("entity", "Integer", "World entity identifier")
            .parameter("value_json", "String", "Tagged ReflectedValue JSON payload")
            .return_type("Bool")
            .documentation("Writes a reflected value by dense numeric slots."),
        )
        .build()
        .map_err(map_zr_error)?;
    runtime.register_native_module(module).map_err(map_zr_error)
}

fn expect_string<'a>(
    arguments: &'a [ScriptHostValue],
    index: usize,
    function: &str,
) -> Result<&'a str, zrvm::Error> {
    match arguments.get(index) {
        Some(ScriptHostValue::String(value)) => Ok(value),
        Some(value) => Err(zr_error(format!(
            "{function} argument {index} expected String, received {:?}",
            value.kind()
        ))),
        None => Err(zr_error(format!(
            "{function} argument {index} was not provided"
        ))),
    }
}

fn expect_int(
    arguments: &[ScriptHostValue],
    index: usize,
    function: &str,
) -> Result<i64, zrvm::Error> {
    match arguments.get(index) {
        Some(ScriptHostValue::Int(value)) => Ok(*value),
        Some(value) => Err(zr_error(format!(
            "{function} argument {index} expected Integer, received {:?}",
            value.kind()
        ))),
        None => Err(zr_error(format!(
            "{function} argument {index} was not provided"
        ))),
    }
}

fn reflection_error(error: crate::ReflectionHostError) -> zrvm::Error {
    zr_error(format!("ZrVM reflection host call failed: {error}"))
}
