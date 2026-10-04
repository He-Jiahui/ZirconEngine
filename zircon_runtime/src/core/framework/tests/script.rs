use super::{ScriptHostIntoValue, ScriptHostTypeRef, ScriptHostValue, ScriptHostValueKind};

#[test]
fn bytes_default_to_the_zr_vm_byte_array_type() {
    assert_eq!(
        ScriptHostValueKind::Bytes.default_zr_type_name(),
        "container.Array<uint>"
    );
    assert_eq!(
        ScriptHostTypeRef::from_value_kind(ScriptHostValueKind::Bytes).type_name,
        "container.Array<uint>"
    );
}

#[test]
fn byte_vectors_encode_through_the_owned_return_value_contract() {
    let bytes = vec![0, 104, 128, 255];
    let host_value = bytes.clone().into_script_host_value();

    assert_eq!(host_value, ScriptHostValue::Bytes(bytes));
}
