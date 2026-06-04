use zircon_runtime::core::framework::script::{
    ScriptHostFunctionDescriptor, ScriptHostModuleDescriptor, ScriptHostParameterDescriptor,
    ScriptHostValue, ScriptHostValueKind,
};
use zircon_runtime::script::{CapabilitySet, HostExportFunction, HostExportRegistry};

use super::errors::zr_error;
use super::host_modules::{native_function_label, validate_native_function_arity};
use super::values::{from_zr_value_for_function, to_zr_value, to_zr_value_for_function};

fn descriptor_with_arity(min: usize, max: usize) -> ScriptHostFunctionDescriptor {
    ScriptHostFunctionDescriptor::new("bad", min, max, ScriptHostValueKind::Null)
}

#[test]
fn validate_native_function_arity_rejects_min_overflow() {
    let descriptor = descriptor_with_arity(usize::from(u16::MAX) + 1, usize::from(u16::MAX) + 1);
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
        zr_vm_rust_binding::ValueKind::Null
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
    let value = zr_vm_rust_binding::Value::new_array().unwrap();
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
                    ScriptHostFunctionDescriptor::new("secure", 0, 0, ScriptHostValueKind::Null)
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
