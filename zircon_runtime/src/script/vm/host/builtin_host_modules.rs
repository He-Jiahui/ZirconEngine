use std::time::{SystemTime, UNIX_EPOCH};

use crate::core::framework::script::{
    ScriptHostCallFrame, ScriptHostError, ScriptHostFunctionDescriptor, ScriptHostHotPathMetrics,
    ScriptHostModuleDescriptor, ScriptHostParameterDescriptor, ScriptHostValue,
    ScriptHostValueKind, ScriptHostValueRef,
};

use super::super::{CapabilitySet, HostHandle, VmError};
use super::{HostExportFunction, HostExportRegistry, HostRegistry};
use crate::script::register_gameplay_host_module;

const FOUNDATION_MODULE: &str = "zr.zircon.foundation";
const ASSET_MODULE: &str = "zr.zircon.asset";
const SCENE_MODULE: &str = "zr.zircon.scene";
const RENDER_MODULE: &str = "zr.zircon.render";
const MATH_MODULE: &str = "zr.zircon.math";
const HOST_MODULE_VERSION: &str = "0.1.0";
const MATH_MODULE_VERSION: &str = "0.2.0";
const MATH_SCALAR_CAPABILITY: &str = "math.scalar";

pub fn register_builtin_host_modules(
    exports: &HostExportRegistry,
    registry: &HostRegistry,
) -> Result<Vec<HostHandle>, VmError> {
    let mut handles = Vec::new();
    if exports.module(FOUNDATION_MODULE).is_none() {
        handles.push(register_foundation_module(exports)?);
    }
    if exports.module(ASSET_MODULE).is_none() {
        handles.push(register_asset_module(exports)?);
    }
    if exports.module(SCENE_MODULE).is_none() {
        handles.push(register_scene_module(exports, registry)?);
    }
    if exports.module(RENDER_MODULE).is_none() {
        handles.push(register_render_module(exports)?);
    }
    if exports.module(MATH_MODULE).is_none() {
        handles.push(math::register_math_host_module(exports)?);
    }
    if let Some(handle) = register_gameplay_host_module(exports)? {
        handles.push(handle);
    }
    Ok(handles)
}

fn register_foundation_module(exports: &HostExportRegistry) -> Result<HostHandle, VmError> {
    let descriptor = ScriptHostModuleDescriptor::new(FOUNDATION_MODULE, HOST_MODULE_VERSION)
        .with_capability("foundation.log")
        .with_capability("foundation.time")
        .with_capability("foundation.event")
        .with_function(
            ScriptHostFunctionDescriptor::new("time_unix_millis", 0, 0, ScriptHostValueKind::Int)
                .with_required_capability("foundation.time")
                .with_documentation(
                    "Return the current host wall-clock time in Unix milliseconds.",
                ),
        )
        .with_function(
            ScriptHostFunctionDescriptor::new("log_info", 1, 1, ScriptHostValueKind::Null)
                .with_parameter(ScriptHostParameterDescriptor::new(
                    "message",
                    ScriptHostValueKind::String,
                ))
                .with_required_capability("foundation.log")
                .with_documentation("Send an informational message through the host log surface."),
        )
        .with_function(
            ScriptHostFunctionDescriptor::new("event_publish", 2, 2, ScriptHostValueKind::Bool)
                .with_parameter(ScriptHostParameterDescriptor::new(
                    "topic",
                    ScriptHostValueKind::String,
                ))
                .with_parameter(ScriptHostParameterDescriptor::new(
                    "payload",
                    ScriptHostValueKind::String,
                ))
                .with_required_capability("foundation.event")
                .with_documentation("Publish a host event if an event manager is bound."),
        )
        .with_documentation("Runtime foundation helpers exposed through stable VM host calls.");

    exports.register_module(
        descriptor,
        [
            HostExportFunction::new("time_unix_millis", |_| {
                let millis = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map_err(|error| ScriptHostError::new(error.to_string()))?
                    .as_millis();
                Ok(ScriptHostValue::Int(
                    i64::try_from(millis).unwrap_or(i64::MAX),
                ))
            }),
            HostExportFunction::new("log_info", |context| {
                with_string(context, 0, |_| Ok(()))?;
                Ok(ScriptHostValue::Null)
            }),
            HostExportFunction::new("event_publish", |context| {
                with_string(context, 0, |_| Ok(()))?;
                with_string(context, 1, |_| Ok(()))?;
                Ok(ScriptHostValue::Bool(false))
            }),
        ],
    )
}

fn register_asset_module(exports: &HostExportRegistry) -> Result<HostHandle, VmError> {
    let descriptor = ScriptHostModuleDescriptor::new(ASSET_MODULE, HOST_MODULE_VERSION)
        .with_capability("asset.query")
        .with_function(
            ScriptHostFunctionDescriptor::new(
                "locator_identity",
                1,
                1,
                ScriptHostValueKind::String,
            )
            .with_parameter(ScriptHostParameterDescriptor::new(
                "locator",
                ScriptHostValueKind::String,
            ))
            .with_required_capability("asset.query")
            .with_documentation("Return the canonical locator string seen by the host."),
        )
        .with_function(
            ScriptHostFunctionDescriptor::new("status", 1, 1, ScriptHostValueKind::String)
                .with_parameter(ScriptHostParameterDescriptor::new(
                    "locator",
                    ScriptHostValueKind::String,
                ))
                .with_required_capability("asset.query")
                .with_documentation("Return the known asset status for a locator."),
        )
        .with_function(
            ScriptHostFunctionDescriptor::new("revision", 1, 1, ScriptHostValueKind::Int)
                .with_parameter(ScriptHostParameterDescriptor::new(
                    "locator",
                    ScriptHostValueKind::String,
                ))
                .with_required_capability("asset.query")
                .with_documentation("Return the known asset revision for a locator."),
        )
        .with_documentation("Asset lookup host calls that keep VM code on locator strings.");

    exports.register_module(
        descriptor,
        [
            HostExportFunction::new("locator_identity", |context| {
                Ok(ScriptHostValue::String(copy_string_for_host_return(
                    context, 0,
                )?))
            }),
            HostExportFunction::new("status", |context| {
                with_string(context, 0, |_| Ok(()))?;
                Ok(ScriptHostValue::String("unknown".to_string()))
            }),
            HostExportFunction::new("revision", |context| {
                with_string(context, 0, |_| Ok(()))?;
                Ok(ScriptHostValue::Int(0))
            }),
        ],
    )
}

fn register_scene_module(
    exports: &HostExportRegistry,
    registry: &HostRegistry,
) -> Result<HostHandle, VmError> {
    let default_world_handle = registry
        .register_capability("host.scene.world.default")
        .map_err(|error| VmError::Operation(error.to_string()))?;
    let validation_registry = registry.clone();
    let summary_registry = registry.clone();
    let descriptor = ScriptHostModuleDescriptor::new(SCENE_MODULE, HOST_MODULE_VERSION)
        .with_capability("scene.query")
        .with_capability("scene.handle")
        .with_function(
            ScriptHostFunctionDescriptor::new(
                "default_world_handle",
                0,
                0,
                ScriptHostValueKind::HostHandle,
            )
            .with_required_capability("scene.handle")
            .with_documentation("Return a stable host handle for the default runtime world."),
        )
        .with_function(
            ScriptHostFunctionDescriptor::new("handle_is_valid", 1, 1, ScriptHostValueKind::Bool)
                .with_parameter(ScriptHostParameterDescriptor::new(
                    "handle",
                    ScriptHostValueKind::HostHandle,
                ))
                .with_required_capability("scene.query")
                .with_documentation("Check whether a VM-supplied host handle still exists."),
        )
        .with_function(
            ScriptHostFunctionDescriptor::new("summary", 1, 1, ScriptHostValueKind::String)
                .with_parameter(ScriptHostParameterDescriptor::new(
                    "handle",
                    ScriptHostValueKind::HostHandle,
                ))
                .with_required_capability("scene.query")
                .with_documentation("Return a compact summary string for a scene/world handle."),
        )
        .with_documentation("Scene host calls expose stable handles, not direct world pointers.");

    exports.register_module(
        descriptor,
        [
            HostExportFunction::new("default_world_handle", move |_| {
                Ok(ScriptHostValue::HostHandle(default_world_handle.into_raw()))
            }),
            HostExportFunction::new("handle_is_valid", {
                let registry = validation_registry.clone();
                move |context| {
                    let handle = expect_handle(context, 0)?;
                    Ok(ScriptHostValue::Bool(
                        registry.is_valid(HostHandle::from_raw(handle)),
                    ))
                }
            }),
            HostExportFunction::new("summary", move |context| {
                let handle = expect_handle(context, 0)?;
                summary_registry
                    .resolve(HostHandle::from_raw(handle))
                    .map_err(|error| ScriptHostError::new(error.to_string()))?;
                Ok(ScriptHostValue::String(format!("host-handle:{handle}")))
            }),
        ],
    )
}

fn register_render_module(exports: &HostExportRegistry) -> Result<HostHandle, VmError> {
    let descriptor = ScriptHostModuleDescriptor::new(RENDER_MODULE, HOST_MODULE_VERSION)
        .with_capability("render.query")
        .with_function(
            ScriptHostFunctionDescriptor::new("backend_name", 0, 0, ScriptHostValueKind::String)
                .with_required_capability("render.query")
                .with_documentation("Return the current read-only render backend label."),
        )
        .with_function(
            ScriptHostFunctionDescriptor::new("frame_index", 0, 0, ScriptHostValueKind::Int)
                .with_required_capability("render.query")
                .with_documentation("Return the latest frame index known to the host surface."),
        )
        .with_documentation("Read-only render host metadata exposed to VM code.");

    exports.register_module(
        descriptor,
        [
            HostExportFunction::new("backend_name", |_| {
                Ok(ScriptHostValue::String("unavailable".to_string()))
            }),
            HostExportFunction::new("frame_index", |_| Ok(ScriptHostValue::Int(0))),
        ],
    )
}

mod math {
    use super::*;

    #[derive(crate::ZirconScriptType)]
    #[zircon_script(
        name = "Vec3",
        value_kind = ScriptHostValueKind::Float,
        prototype = crate::core::framework::script::ScriptHostPrototypeKind::Struct,
        allow_value_construction = true,
        documentation = "Pure Vec3 value descriptor for VM reflection."
    )]
    struct Vec3 {
        #[zircon_script(type_name = "float")]
        x: f64,
        #[zircon_script(type_name = "float")]
        y: f64,
        #[zircon_script(type_name = "float")]
        z: f64,
    }

    #[derive(crate::ZirconScriptType)]
    #[zircon_script(
        name = "ColorRgba",
        value_kind = ScriptHostValueKind::Float,
        prototype = crate::core::framework::script::ScriptHostPrototypeKind::Struct,
        allow_value_construction = true,
        documentation = "Pure RGBA color value descriptor for VM reflection."
    )]
    struct ColorRgba {
        #[zircon_script(type_name = "float")]
        r: f64,
        #[zircon_script(type_name = "float")]
        g: f64,
        #[zircon_script(type_name = "float")]
        b: f64,
        #[zircon_script(type_name = "float")]
        a: f64,
    }

    // These descriptor-only structs are reflected by the math module; keep their
    // field layout visible without adding VM-visible host calls.
    const _: ((f64, f64, f64), (f64, f64, f64, f64)) = {
        let vec3 = Vec3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        };
        let color = ColorRgba {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        };
        (
            (vec3.x, vec3.y, vec3.z),
            (color.r, color.g, color.b, color.a),
        )
    };

    #[crate::zircon_host_function(
        name = "vec3_length",
        return_type_name = "float",
        documentation = "Return sqrt(x*x + y*y + z*z)."
    )]
    fn vec3_length(x: f64, y: f64, z: f64) -> f64 {
        (x * x + y * y + z * z).sqrt()
    }

    #[crate::zircon_host_function(
        name = "vec3_dot",
        return_type_name = "float",
        documentation = "Return the dot product for two Vec3 values."
    )]
    fn vec3_dot(ax: f64, ay: f64, az: f64, bx: f64, by: f64, bz: f64) -> f64 {
        ax * bx + ay * by + az * bz
    }

    pub fn math_host_module_descriptor() -> Result<ScriptHostModuleDescriptor, VmError> {
        Ok(
            ScriptHostModuleDescriptor::new(MATH_MODULE, MATH_MODULE_VERSION)
                .with_capability(MATH_SCALAR_CAPABILITY)
                .with_type(
                    <Vec3 as crate::core::framework::script::ZirconScriptType>::script_host_type_descriptor()
                        .map_err(|error| VmError::Operation(error.to_string()))?,
                )
                .with_type(
                    <ColorRgba as crate::core::framework::script::ZirconScriptType>::script_host_type_descriptor()
                        .map_err(|error| VmError::Operation(error.to_string()))?,
                )
                .with_function(__zircon_host_function_descriptor_vec3_length())
                .with_function(__zircon_host_function_descriptor_vec3_dot())
                .with_function(scalar_descriptor(
                    "abs",
                    &["value"],
                    "Return the absolute value. Non-finite input or result is rejected.",
                ))
                .with_function(scalar_descriptor(
                    "atan2",
                    &["y", "x"],
                    "Return atan2(y, x) using libm. Non-finite input or result is rejected.",
                ))
                .with_function(scalar_descriptor(
                    "ceil",
                    &["value"],
                    "Round upward using libm. Non-finite input or result is rejected.",
                ))
                .with_function(scalar_descriptor(
                    "cos",
                    &["value"],
                    "Return cosine using libm. Non-finite input or result is rejected.",
                ))
                .with_function(scalar_descriptor(
                    "exp",
                    &["value"],
                    "Return e raised to value using libm. Non-finite input or result is rejected.",
                ))
                .with_function(scalar_descriptor(
                    "floor",
                    &["value"],
                    "Round downward using libm. Non-finite input or result is rejected.",
                ))
                .with_function(scalar_descriptor(
                    "sin",
                    &["value"],
                    "Return sine using libm. Non-finite input or result is rejected.",
                ))
                .with_function(scalar_descriptor(
                    "sqrt",
                    &["value"],
                    "Return the square root using libm. Negative and non-finite results are rejected.",
                ))
                .with_function(scalar_descriptor(
                    "pow",
                    &["base", "exponent"],
                    "Return base raised to exponent using libm. Non-finite input or result is rejected.",
                ))
                .with_documentation(
                    "Deterministic scalar ABI backed by libm 0.2.16. Every scalar argument and result must be finite on every supported target.",
                ),
        )
    }

    pub fn register_math_host_module(exports: &HostExportRegistry) -> Result<HostHandle, VmError> {
        exports.register_module(
            math_host_module_descriptor()?,
            [
                __zircon_host_export_function_vec3_length(),
                __zircon_host_export_function_vec3_dot(),
                HostExportFunction::new("abs", |context| scalar_unary(context, "abs", libm::fabs)),
                HostExportFunction::new("atan2", scalar_atan2),
                HostExportFunction::new("ceil", |context| {
                    scalar_unary(context, "ceil", libm::ceil)
                }),
                HostExportFunction::new("cos", |context| scalar_unary(context, "cos", libm::cos)),
                HostExportFunction::new("exp", |context| scalar_unary(context, "exp", libm::exp)),
                HostExportFunction::new("floor", |context| {
                    scalar_unary(context, "floor", libm::floor)
                }),
                HostExportFunction::new("sin", |context| scalar_unary(context, "sin", libm::sin)),
                HostExportFunction::new("sqrt", |context| {
                    scalar_unary(context, "sqrt", libm::sqrt)
                }),
                HostExportFunction::new("pow", scalar_pow),
            ],
        )
    }

    fn scalar_descriptor(
        name: &str,
        parameters: &[&str],
        documentation: &str,
    ) -> ScriptHostFunctionDescriptor {
        let mut descriptor = ScriptHostFunctionDescriptor::new(
            name,
            parameters.len(),
            parameters.len(),
            ScriptHostValueKind::Float,
        )
        .with_required_capability(MATH_SCALAR_CAPABILITY)
        .with_documentation(documentation);
        for parameter in parameters {
            descriptor = descriptor.with_parameter(ScriptHostParameterDescriptor::new(
                *parameter,
                ScriptHostValueKind::Float,
            ));
        }
        descriptor
    }

    fn scalar_unary(
        context: &ScriptHostCallFrame<'_>,
        name: &str,
        operation: impl FnOnce(f64) -> f64,
    ) -> Result<ScriptHostValue, ScriptHostError> {
        scalar_result(name, operation(scalar_argument(context, name, 0)?))
    }

    fn scalar_atan2(context: &ScriptHostCallFrame<'_>) -> Result<ScriptHostValue, ScriptHostError> {
        let y = scalar_argument(context, "atan2", 0)?;
        let x = scalar_argument(context, "atan2", 1)?;
        scalar_result("atan2", libm::atan2(y, x))
    }

    fn scalar_pow(context: &ScriptHostCallFrame<'_>) -> Result<ScriptHostValue, ScriptHostError> {
        let base = scalar_argument(context, "pow", 0)?;
        let exponent = scalar_argument(context, "pow", 1)?;
        scalar_result("pow", libm::pow(base, exponent))
    }

    fn scalar_argument(
        context: &ScriptHostCallFrame<'_>,
        function: &str,
        index: usize,
    ) -> Result<f64, ScriptHostError> {
        let value = context
            .arguments
            .with_argument(index, |value| match value {
                ScriptHostValueRef::Float(value) => Ok(value),
                value => Err(ScriptHostError::new(format!(
                    "{function} argument {index} expected finite float, received {:?}",
                    value.kind()
                ))),
            })?;
        if value.is_finite() {
            Ok(value)
        } else {
            Err(ScriptHostError::new(format!(
                "{function} argument {index} must be finite"
            )))
        }
    }

    fn scalar_result(name: &str, value: f64) -> Result<ScriptHostValue, ScriptHostError> {
        if value.is_finite() {
            Ok(ScriptHostValue::Float(value))
        } else {
            Err(ScriptHostError::new(format!(
                "{name} produced a non-finite result"
            )))
        }
    }
}

fn with_string<T>(
    context: &ScriptHostCallFrame<'_>,
    index: usize,
    visitor: impl for<'value> FnOnce(&'value str) -> Result<T, ScriptHostError>,
) -> Result<T, ScriptHostError>
where
    T: Sized,
{
    context.arguments.with_argument(index, |value| match value {
        ScriptHostValueRef::String(value) => visitor(value),
        value => Err(ScriptHostError::new(format!(
            "argument {index} expected string, received {:?}",
            value.kind()
        ))),
    })
}

fn copy_string_for_host_return(
    context: &ScriptHostCallFrame<'_>,
    index: usize,
) -> Result<String, ScriptHostError> {
    with_string(context, index, |value| {
        ScriptHostHotPathMetrics::record_guest_string_copy(value.len());
        Ok(value.to_owned())
    })
}

fn expect_handle(context: &ScriptHostCallFrame<'_>, index: usize) -> Result<u64, ScriptHostError> {
    context.arguments.with_argument(index, |value| match value {
        ScriptHostValueRef::HostHandle(value) => Ok(value),
        // ZrVM carries the neutral u64 payload through i64 while preserving its bits.
        ScriptHostValueRef::Int(value) => Ok(value as u64),
        value => Err(ScriptHostError::new(format!(
            "argument {index} expected host handle, received {:?}",
            value.kind()
        ))),
    })
}

pub fn builtin_host_capabilities() -> CapabilitySet {
    CapabilitySet::default()
        .with("foundation.log")
        .with("foundation.time")
        .with("foundation.event")
        .with("asset.query")
        .with("scene.query")
        .with("scene.handle")
        .with("render.query")
        .with(MATH_SCALAR_CAPABILITY)
        .with("gameplay.input")
        .with("gameplay.entity")
        .with("gameplay.navigation")
        .with("gameplay.scene_transition")
}
