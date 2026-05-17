use std::time::{SystemTime, UNIX_EPOCH};

use crate::core::framework::script::{
    ScriptHostCallContext, ScriptHostError, ScriptHostFunctionDescriptor,
    ScriptHostModuleDescriptor, ScriptHostParameterDescriptor, ScriptHostValue,
    ScriptHostValueKind,
};

use super::super::{CapabilitySet, HostHandle, VmError};
use super::{HostExportFunction, HostExportRegistry, HostRegistry};

const FOUNDATION_MODULE: &str = "zr.zircon.foundation";
const ASSET_MODULE: &str = "zr.zircon.asset";
const SCENE_MODULE: &str = "zr.zircon.scene";
const RENDER_MODULE: &str = "zr.zircon.render";
const MATH_MODULE: &str = "zr.zircon.math";
const HOST_MODULE_VERSION: &str = "0.1.0";

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
                let _message = expect_string(context, 0)?;
                Ok(ScriptHostValue::Null)
            }),
            HostExportFunction::new("event_publish", |context| {
                let _topic = expect_string(context, 0)?;
                let _payload = expect_string(context, 1)?;
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
                Ok(ScriptHostValue::String(expect_string(context, 0)?))
            }),
            HostExportFunction::new("status", |context| {
                let _locator = expect_string(context, 0)?;
                Ok(ScriptHostValue::String("unknown".to_string()))
            }),
            HostExportFunction::new("revision", |context| {
                let _locator = expect_string(context, 0)?;
                Ok(ScriptHostValue::Int(0))
            }),
        ],
    )
}

fn register_scene_module(
    exports: &HostExportRegistry,
    registry: &HostRegistry,
) -> Result<HostHandle, VmError> {
    let default_world_handle = registry.register_capability("host.scene.world.default");
    let validation_registry = registry.clone();
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
                Ok(ScriptHostValue::HostHandle(default_world_handle.get()))
            }),
            HostExportFunction::new("handle_is_valid", {
                let registry = validation_registry.clone();
                move |context| {
                    let handle = expect_handle(context, 0)?;
                    Ok(ScriptHostValue::Bool(
                        registry.is_valid(HostHandle::new(handle)),
                    ))
                }
            }),
            HostExportFunction::new("summary", move |context| {
                let handle = expect_handle(context, 0)?;
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

#[crate::zircon_host_module(
    name = "zr.zircon.math",
    version = "0.1.0",
    documentation = "Pure math value descriptors and deterministic helper functions."
)]
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
    #[allow(dead_code)]
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
    #[allow(dead_code)]
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
}

fn expect_string(context: &ScriptHostCallContext, index: usize) -> Result<String, ScriptHostError> {
    match context.arguments.get(index) {
        Some(ScriptHostValue::String(value)) => Ok(value.clone()),
        Some(value) => Err(ScriptHostError::new(format!(
            "argument {index} expected string, received {:?}",
            value.kind()
        ))),
        None => Err(ScriptHostError::new(format!(
            "argument {index} was not provided"
        ))),
    }
}

fn expect_handle(context: &ScriptHostCallContext, index: usize) -> Result<u64, ScriptHostError> {
    match context.arguments.get(index) {
        Some(ScriptHostValue::HostHandle(value)) => Ok(*value),
        Some(ScriptHostValue::Int(value)) if *value >= 0 => Ok(*value as u64),
        Some(value) => Err(ScriptHostError::new(format!(
            "argument {index} expected host handle, received {:?}",
            value.kind()
        ))),
        None => Err(ScriptHostError::new(format!(
            "argument {index} was not provided"
        ))),
    }
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
}
