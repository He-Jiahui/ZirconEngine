use std::collections::BTreeSet;

use super::{
    ZR_HOST_API_V1_OPTIONAL_FIELD_NAMES, ZR_RUNTIME_API_V8_OPTIONAL_FIELD_NAMES,
    ZR_RUNTIME_API_V8_REQUIRED_FIELD_NAMES,
};

#[test]
fn interface_spec_slot_partitions_match_the_abi_table_fields() {
    assert_eq!(
        crate::runtime_build_set::ZR_RUNTIME_API_V8_REQUIRED_SLOT_NAMES,
        ZR_RUNTIME_API_V8_REQUIRED_FIELD_NAMES,
    );
    assert_eq!(
        crate::runtime_build_set::ZR_RUNTIME_API_V8_OPTIONAL_SLOT_NAMES,
        ZR_RUNTIME_API_V8_OPTIONAL_FIELD_NAMES,
    );
    assert_eq!(
        crate::runtime_build_set::ZR_HOST_API_V1_OPTIONAL_SLOT_NAMES,
        ZR_HOST_API_V1_OPTIONAL_FIELD_NAMES,
    );

    let expected_runtime_slots = crate::runtime_build_set::ZR_RUNTIME_API_V8_REQUIRED_SLOT_NAMES
        .iter()
        .chain(crate::runtime_build_set::ZR_RUNTIME_API_V8_OPTIONAL_SLOT_NAMES.iter())
        .filter(|field| **field != "abi_version" && **field != "size_bytes")
        .map(|field| (*field).to_owned())
        .collect::<BTreeSet<_>>();
    assert_eq!(
        concrete_table_slot_names("ZrRuntimeApiV8"),
        expected_runtime_slots,
    );

    let expected_host_slots = crate::runtime_build_set::ZR_HOST_API_V1_OPTIONAL_SLOT_NAMES
        .iter()
        .map(|field| (*field).to_owned())
        .collect::<BTreeSet<_>>();
    assert_eq!(
        concrete_table_slot_names("ZrHostApiV1"),
        expected_host_slots,
    );
}

fn concrete_table_slot_names(table_name: &str) -> BTreeSet<String> {
    let source = include_str!("../api_table.rs");
    let struct_needle = format!("pub struct {table_name} ");
    let struct_index = source
        .find(&struct_needle)
        .unwrap_or_else(|| panic!("{table_name} must remain a concrete ABI table declaration"));
    let body_start = source[struct_index..]
        .find('{')
        .map(|offset| struct_index + offset + 1)
        .expect("concrete ABI table must have a field body");
    let fields = source[body_start..]
        .lines()
        .map(str::trim)
        .take_while(|line| *line != "}")
        .filter_map(|line| line.strip_prefix("pub "))
        .map(|field| {
            field
                .split_once(':')
                .map(|(field, _)| field.trim().to_owned())
                .expect("concrete ABI table field must have a type")
        })
        .collect::<Vec<_>>();

    assert_eq!(
        fields.first().map(String::as_str),
        Some("abi_version"),
        "{table_name} must begin with the ABI version header"
    );
    assert_eq!(
        fields.get(1).map(String::as_str),
        Some("size_bytes"),
        "{table_name} must retain the byte-size header after its ABI version"
    );
    fields.into_iter().skip(2).collect()
}
