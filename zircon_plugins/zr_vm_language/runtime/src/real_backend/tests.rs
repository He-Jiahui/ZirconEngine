use zircon_runtime::core::framework::script::{
    ScriptHostFunctionDescriptor, ScriptHostModuleDescriptor, ScriptHostParameterDescriptor,
    ScriptHostValue, ScriptHostValueKind,
};
use zircon_runtime::script::{CapabilitySet, HostExportFunction, HostExportRegistry};

use super::errors::zr_error;
use super::host_modules::{native_function_label, validate_native_function_arity};
use super::instance::lower_zr_arguments;
use super::lock::acquire_zr_vm_lock;
use super::values::{from_zr_return_value_for_export, to_zr_value, to_zr_value_for_function};

#[test]
fn zr_vm_real_backend_runtime_lock_recovers_after_poison() {
    std::thread::spawn(|| {
        let _guard = acquire_zr_vm_lock();
        panic!("poison ZrVM runtime lock for recovery coverage");
    })
    .join()
    .expect_err("lock poison worker should panic");

    drop(acquire_zr_vm_lock());
}

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
        to_zr_value(&ScriptHostValue::Null).unwrap().kind(),
        zr_vm_rust_binding::ValueKind::Null
    ));
    assert!(to_zr_value(&ScriptHostValue::Bool(true))
        .unwrap()
        .as_bool()
        .unwrap());
    assert_eq!(
        to_zr_value(&ScriptHostValue::Int(7))
            .unwrap()
            .as_int()
            .unwrap(),
        7
    );
    assert_eq!(
        to_zr_value(&ScriptHostValue::Float(1.5))
            .unwrap()
            .as_float()
            .unwrap(),
        1.5
    );
    assert_eq!(
        to_zr_value(&ScriptHostValue::String("ok".to_string()))
            .unwrap()
            .as_string()
            .unwrap(),
        "ok"
    );
    let bytes = to_zr_value(&ScriptHostValue::Bytes(vec![0, 104, 128, 255])).unwrap();
    assert_eq!(bytes.kind(), zr_vm_rust_binding::ValueKind::Array);
    assert_eq!(bytes.array_len().unwrap(), 4);
    assert_eq!(bytes.array_get(0).unwrap().as_int().unwrap(), 0);
    assert_eq!(bytes.array_get(1).unwrap().as_int().unwrap(), 104);
    assert_eq!(bytes.array_get(2).unwrap().as_int().unwrap(), 128);
    assert_eq!(bytes.array_get(3).unwrap().as_int().unwrap(), 255);
    assert_eq!(
        to_zr_value(&ScriptHostValue::HostHandle(42))
            .unwrap()
            .as_int()
            .unwrap(),
        42
    );
}

#[test]
fn byte_arrays_round_trip_losslessly_across_the_owned_zr_vm_return_boundary() {
    let source = ScriptHostValue::Bytes(vec![0, 104, 128, 255]);
    let value = to_zr_value(&source).expect("lower bytes as a ZrVM array");

    assert_eq!(
        from_zr_return_value_for_export(&value, "example", "bytes")
            .expect("raise byte array from an export"),
        source
    );
}

#[test]
fn strings_remain_owned_by_the_host_after_borrowed_zr_vm_lowering() {
    let source = ScriptHostValue::String("borrowed input".to_owned());
    let value = to_zr_value(&source).expect("lower a borrowed host string");

    assert_eq!(value.as_string().unwrap(), "borrowed input");
    assert_eq!(source, ScriptHostValue::String("borrowed input".to_owned()));
}

#[test]
fn lower_zr_arguments_reuses_capacity_and_clears_failed_values() {
    let _guard = acquire_zr_vm_lock();
    let mut lowered_arguments = Vec::new();
    lower_zr_arguments(
        &mut lowered_arguments,
        &[
            ScriptHostValue::Int(7),
            ScriptHostValue::String("first".to_owned()),
        ],
    )
    .unwrap();
    let capacity = lowered_arguments.capacity();
    let storage = lowered_arguments.as_ptr();

    lower_zr_arguments(
        &mut lowered_arguments,
        &[ScriptHostValue::Bool(true), ScriptHostValue::Int(9)],
    )
    .unwrap();
    assert_eq!(lowered_arguments.capacity(), capacity);
    assert_eq!(lowered_arguments.as_ptr(), storage);
    assert_eq!(lowered_arguments.len(), 2);

    let error = lower_zr_arguments(
        &mut lowered_arguments,
        &[
            ScriptHostValue::Int(11),
            ScriptHostValue::String("bad\0value".to_owned()),
        ],
    )
    .unwrap_err();
    assert!(error.message.contains("string contains interior NUL"));
    assert!(lowered_arguments.is_empty());
    assert_eq!(lowered_arguments.capacity(), capacity);
}

#[test]
fn host_handle_i64_transport_preserves_packed_generation_bits() {
    let handle = zircon_runtime::script::HostHandle::from_parts(17, u32::MAX);
    let raw = handle.into_raw();
    let lowered = to_zr_value(&ScriptHostValue::HostHandle(raw)).unwrap();
    let transported = lowered.as_int().unwrap();

    assert!(transported < 0);
    assert_eq!(transported as u64, raw);
    assert_eq!(
        zircon_runtime::script::HostHandle::from_raw(transported as u64),
        handle
    );
}

#[test]
fn from_zr_return_value_for_export_rejects_non_byte_array_elements_with_context() {
    let mut value = zr_vm_rust_binding::Value::new_array().unwrap();
    value
        .array_push(&zr_vm_rust_binding::Value::new_string("not-a-byte").unwrap())
        .unwrap();
    let error = from_zr_return_value_for_export(&value, "example", "unsupported").unwrap_err();

    assert!(error.message.contains("export example.unsupported"));
    assert!(error.message.contains("expected byte integer"));
}

#[test]
fn from_zr_return_value_for_export_rejects_out_of_range_byte_array_elements() {
    let mut value = zr_vm_rust_binding::Value::new_array().unwrap();
    value
        .array_push(&zr_vm_rust_binding::Value::new_int(256).unwrap())
        .unwrap();
    let error = from_zr_return_value_for_export(&value, "example", "bytes").unwrap_err();

    assert!(error.message.contains("export example.bytes"));
    assert!(error.message.contains("outside 0..=255: 256"));
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
