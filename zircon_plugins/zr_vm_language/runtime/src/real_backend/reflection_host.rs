use zircon_runtime::core::framework::script::{
    ScriptHostArguments, ScriptHostError, ScriptHostValueRef,
};
use zircon_runtime::script::VmError;
use zr_vm_rust_binding as zrvm;

use super::errors::{map_zr_error, zr_error};
use super::values::ZrVmScriptHostArgumentSource;
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
                let source =
                    ZrVmScriptHostArgumentSource::new(context, "zircon.reflection.resolve")?;
                let arguments = ScriptHostArguments::new(&source);
                let token =
                    borrow_string(&arguments, 0, "zircon.reflection.resolve", |type_path| {
                        borrow_string(&arguments, 1, "zircon.reflection.resolve", |member_name| {
                            resolve_reflection
                                .resolve(type_path, member_name)
                                .map(|token| token as i64)
                                .map_err(reflection_host_error)
                        })
                    })
                    .map_err(|error| zr_error(error.message))?;
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
                let source = ZrVmScriptHostArgumentSource::new(context, "zircon.reflection.read")?;
                let arguments = ScriptHostArguments::new(&source);
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
                let source = ZrVmScriptHostArgumentSource::new(context, "zircon.reflection.write")?;
                let arguments = ScriptHostArguments::new(&source);
                let token = expect_int(&arguments, 0, "zircon.reflection.write")? as u64;
                let entity = expect_int(&arguments, 1, "zircon.reflection.write")? as u64;
                let changed =
                    borrow_string(&arguments, 2, "zircon.reflection.write", |value_json| {
                        write_reflection
                            .write_json(token, entity, value_json)
                            .map_err(reflection_host_error)
                    })
                    .map_err(|error| zr_error(error.message))?;
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

fn borrow_string<T>(
    arguments: &ScriptHostArguments<'_>,
    index: usize,
    function: &str,
    visitor: impl FnOnce(&str) -> Result<T, ScriptHostError>,
) -> Result<T, ScriptHostError> {
    // The visitor consumes the borrowed view before the argument access returns.
    arguments.with_argument(index, |value| match value {
        ScriptHostValueRef::String(value) => visitor(value),
        value => Err(ScriptHostError::new(format!(
            "{function} argument {index} expected String, received {:?}",
            value.kind()
        ))),
    })
}

fn reflection_host_error(error: crate::ReflectionHostError) -> ScriptHostError {
    ScriptHostError::new(format!("ZrVM reflection host call failed: {error}"))
}

fn expect_int(
    arguments: &ScriptHostArguments<'_>,
    index: usize,
    function: &str,
) -> Result<i64, zrvm::Error> {
    arguments
        .with_argument(index, |value| match value {
            ScriptHostValueRef::Int(value) => Ok(value),
            value => Err(ScriptHostError::new(format!(
                "{function} argument {index} expected Integer, received {:?}",
                value.kind()
            ))),
        })
        .map_err(|error| zr_error(error.message))
}

fn reflection_error(error: crate::ReflectionHostError) -> zrvm::Error {
    zr_error(format!("ZrVM reflection host call failed: {error}"))
}
