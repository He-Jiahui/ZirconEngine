use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process;

const INTERFACE_SPEC_PATH: &str = "src/runtime_build_set/interface_spec_v1.json";
const GENERATED_SLOT_CATALOG_FILE: &str = "runtime_build_set_catalog.rs";
const INTERFACE_SPEC_FIELDS: [&str; 7] = [
    "family",
    "spec_version",
    "runtime_api_version",
    "entry_symbol",
    "runtime_api_required_slots",
    "runtime_api_optional_slots",
    "host_api_optional_slots",
];
const EXPECTED_INTERFACE_SPEC_VERSION: u32 = 1;
const EXPECTED_RUNTIME_API_VERSION: u32 = 8;
const EXPECTED_RUNTIME_API_ENTRY_SYMBOL: &str = "zircon_runtime_get_api_v8";

struct InterfaceMetadata<'a> {
    family: &'a str,
    spec_version: u32,
    runtime_api_version: u32,
    entry_symbol: &'a str,
}

fn main() {
    if let Err(error) = generate_slot_catalog() {
        eprintln!("failed to generate Runtime InterfaceSpec slot catalog: {error}");
        process::exit(1);
    }
}

fn generate_slot_catalog() -> Result<(), String> {
    println!("cargo:rerun-if-changed={INTERFACE_SPEC_PATH}");

    let source = fs::read_to_string(INTERFACE_SPEC_PATH)
        .map_err(|error| format!("could not read {INTERFACE_SPEC_PATH}: {error}"))?;
    let generated = generate_slot_catalog_from_source(&source)?;

    let output_directory = env::var_os("OUT_DIR")
        .map(PathBuf::from)
        .ok_or_else(|| "Cargo did not provide OUT_DIR".to_owned())?;
    fs::write(
        output_directory.join(GENERATED_SLOT_CATALOG_FILE),
        generated,
    )
    .map_err(|error| format!("could not write generated Runtime slot catalog: {error}"))
}

fn generate_slot_catalog_from_source(source: &str) -> Result<String, String> {
    let spec: serde_json::Value = serde_json::from_str(source)
        .map_err(|error| format!("could not decode {INTERFACE_SPEC_PATH}: {error}"))?;
    validate_interface_spec_fields(&spec)?;
    let metadata = InterfaceMetadata {
        family: string_field(&spec, "family")?,
        spec_version: u32_field(&spec, "spec_version")?,
        runtime_api_version: u32_field(&spec, "runtime_api_version")?,
        entry_symbol: string_field(&spec, "entry_symbol")?,
    };
    validate_interface_metadata(&metadata)?;
    let required = slot_list(&spec, "runtime_api_required_slots")?;
    let optional = slot_list(&spec, "runtime_api_optional_slots")?;
    let host_optional = slot_list(&spec, "host_api_optional_slots")?;
    validate_slot_partition(&required, &optional, &host_optional)?;
    render_slot_catalog(&metadata, &required, &optional, &host_optional)
}

fn validate_interface_spec_fields(spec: &serde_json::Value) -> Result<(), String> {
    let fields = spec
        .as_object()
        .ok_or_else(|| format!("{INTERFACE_SPEC_PATH} must be a JSON object"))?;
    if fields.len() != INTERFACE_SPEC_FIELDS.len()
        || INTERFACE_SPEC_FIELDS
            .iter()
            .any(|field| !fields.contains_key(*field))
        || fields
            .keys()
            .any(|field| !INTERFACE_SPEC_FIELDS.contains(&field.as_str()))
    {
        return Err(format!(
            "{INTERFACE_SPEC_PATH} must contain exactly {:?}",
            INTERFACE_SPEC_FIELDS
        ));
    }
    Ok(())
}

fn string_field<'a>(spec: &'a serde_json::Value, field: &str) -> Result<&'a str, String> {
    spec.get(field)
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| format!("{INTERFACE_SPEC_PATH} field `{field}` must be a string"))
}

fn u32_field(spec: &serde_json::Value, field: &str) -> Result<u32, String> {
    spec.get(field)
        .and_then(serde_json::Value::as_u64)
        .and_then(|value| u32::try_from(value).ok())
        .ok_or_else(|| format!("{INTERFACE_SPEC_PATH} field `{field}` must be a u32"))
}

fn validate_interface_metadata(metadata: &InterfaceMetadata<'_>) -> Result<(), String> {
    if metadata.family.is_empty()
        || !metadata.family.bytes().all(|character| {
            character.is_ascii_lowercase() || character.is_ascii_digit() || character == b'.'
        })
    {
        return Err(format!(
            "{INTERFACE_SPEC_PATH} field `family` must be a non-empty lowercase dotted identifier"
        ));
    }
    if metadata.spec_version != EXPECTED_INTERFACE_SPEC_VERSION {
        return Err(format!(
            "{INTERFACE_SPEC_PATH} field `spec_version` must be {EXPECTED_INTERFACE_SPEC_VERSION}"
        ));
    }
    if metadata.runtime_api_version != EXPECTED_RUNTIME_API_VERSION {
        return Err(format!(
            "{INTERFACE_SPEC_PATH} field `runtime_api_version` must be {EXPECTED_RUNTIME_API_VERSION}"
        ));
    }
    if metadata.entry_symbol != EXPECTED_RUNTIME_API_ENTRY_SYMBOL {
        return Err(format!(
            "{INTERFACE_SPEC_PATH} field `entry_symbol` must be `{EXPECTED_RUNTIME_API_ENTRY_SYMBOL}`"
        ));
    }
    Ok(())
}

fn slot_list<'a>(spec: &'a serde_json::Value, field: &str) -> Result<Vec<&'a str>, String> {
    let values = spec
        .get(field)
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| format!("{INTERFACE_SPEC_PATH} field `{field}` must be an array"))?;

    let mut slots = Vec::with_capacity(values.len());
    for value in values {
        let slot = value.as_str().ok_or_else(|| {
            format!("{INTERFACE_SPEC_PATH} field `{field}` must contain only strings")
        })?;
        validate_slot_name(slot, field)?;
        slots.push(slot);
    }
    Ok(slots)
}

fn validate_slot_name(slot: &str, field: &str) -> Result<(), String> {
    let mut characters = slot.bytes();
    let Some(first) = characters.next() else {
        return Err(format!(
            "{INTERFACE_SPEC_PATH} field `{field}` contains an empty slot name"
        ));
    };
    if !first.is_ascii_lowercase() && first != b'_' {
        return Err(format!(
            "{INTERFACE_SPEC_PATH} field `{field}` contains invalid slot `{slot}`"
        ));
    }
    if !characters.all(|character| {
        character.is_ascii_lowercase() || character.is_ascii_digit() || character == b'_'
    }) {
        return Err(format!(
            "{INTERFACE_SPEC_PATH} field `{field}` contains invalid slot `{slot}`"
        ));
    }
    Ok(())
}

fn validate_slot_partition(
    required: &[&str],
    optional: &[&str],
    host_optional: &[&str],
) -> Result<(), String> {
    let mut runtime_slots = BTreeSet::new();
    for (field, slots) in [
        ("runtime_api_required_slots", required),
        ("runtime_api_optional_slots", optional),
    ] {
        for slot in slots {
            if !runtime_slots.insert(*slot) {
                return Err(format!(
                    "{INTERFACE_SPEC_PATH} field `{field}` repeats slot `{slot}`"
                ));
            }
        }
    }

    let mut host_slots = BTreeSet::new();
    for slot in host_optional {
        if !host_slots.insert(*slot) {
            return Err(format!(
                "{INTERFACE_SPEC_PATH} field `host_api_optional_slots` repeats slot `{slot}`"
            ));
        }
    }
    Ok(())
}

fn render_slot_catalog(
    metadata: &InterfaceMetadata<'_>,
    required: &[&str],
    optional: &[&str],
    host_optional: &[&str],
) -> Result<String, String> {
    let mut output = String::from(
        "// @generated by build.rs from src/runtime_build_set/interface_spec_v1.json.\n\n",
    );
    render_interface_constants(&mut output, metadata)?;
    render_slot_list(
        &mut output,
        "ZR_RUNTIME_API_V8_REQUIRED_SLOT_NAMES",
        required,
    )?;
    render_slot_list(
        &mut output,
        "ZR_RUNTIME_API_V8_OPTIONAL_SLOT_NAMES",
        optional,
    )?;
    render_slot_list(
        &mut output,
        "ZR_HOST_API_V1_OPTIONAL_SLOT_NAMES",
        host_optional,
    )?;
    Ok(output)
}

fn render_interface_constants(
    output: &mut String,
    metadata: &InterfaceMetadata<'_>,
) -> Result<(), String> {
    let family = serde_json::to_string(metadata.family)
        .map_err(|error| format!("could not encode InterfaceSpec family: {error}"))?;
    output.push_str(&format!(
        "pub const ZR_RUNTIME_INTERFACE_FAMILY_V1: &str = {family};\n"
    ));
    output.push_str(&format!(
        "pub const ZR_RUNTIME_INTERFACE_SPEC_VERSION_V1: u32 = {};\n",
        metadata.spec_version
    ));
    output.push_str(&format!(
        "pub const ZIRCON_RUNTIME_API_VERSION_V8: u32 = {};\n",
        metadata.runtime_api_version
    ));
    output.push_str(&format!(
        "pub const ZR_RUNTIME_GET_API_SYMBOL_V8: &[u8] = b\"{}\\0\";\n\n",
        metadata.entry_symbol
    ));
    Ok(())
}

fn render_slot_list(output: &mut String, constant: &str, slots: &[&str]) -> Result<(), String> {
    output.push_str(&format!("pub const {constant}: &[&str] = &[\n"));
    for slot in slots {
        let literal = serde_json::to_string(slot)
            .map_err(|error| format!("could not encode slot `{slot}`: {error}"))?;
        output.push_str("    ");
        output.push_str(&literal);
        output.push_str(",\n");
    }
    output.push_str("];\n\n");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::generate_slot_catalog_from_source;

    const CURRENT_INTERFACE_SPEC: &str =
        include_str!("src/runtime_build_set/interface_spec_v1.json");

    fn interface_spec_with_slots(required: &str, optional: &str, host_optional: &str) -> String {
        format!(
            r#"{{
                "family": "zircon.runtime.internal",
                "spec_version": 1,
                "runtime_api_version": 8,
                "entry_symbol": "zircon_runtime_get_api_v8",
                "runtime_api_required_slots": {required},
                "runtime_api_optional_slots": {optional},
                "host_api_optional_slots": {host_optional}
            }}"#
        )
    }

    #[test]
    fn slot_catalog_generator_emits_the_current_interface_inventory() {
        let catalog = generate_slot_catalog_from_source(CURRENT_INTERFACE_SPEC)
            .expect("current InterfaceSpec must generate a Rust slot catalog");

        assert!(catalog.starts_with("// @generated by build.rs"));
        assert!(catalog.contains("pub const ZR_RUNTIME_INTERFACE_FAMILY_V1"));
        assert!(catalog.contains("pub const ZIRCON_RUNTIME_API_VERSION_V8: u32 = 8;"));
        assert!(catalog.contains("pub const ZR_RUNTIME_GET_API_SYMBOL_V8"));
        assert!(catalog.contains("pub const ZR_RUNTIME_API_V8_REQUIRED_SLOT_NAMES"));
        assert!(catalog.contains("pub const ZR_RUNTIME_API_V8_OPTIONAL_SLOT_NAMES"));
        assert!(catalog.contains("pub const ZR_HOST_API_V1_OPTIONAL_SLOT_NAMES"));
        assert!(catalog.contains("\"create_session\""));
        assert!(catalog.contains("\"present_viewport\""));
        assert!(catalog.contains("\"diagnostics_sink\""));
    }

    #[test]
    fn slot_catalog_generator_rejects_duplicate_runtime_slots_across_partitions() {
        let source =
            interface_spec_with_slots("[\"create_session\"]", "[\"create_session\"]", "[]");
        let error = generate_slot_catalog_from_source(&source)
            .expect_err("a V8 slot cannot be both required and optional");

        assert!(error.contains("runtime_api_optional_slots"));
        assert!(error.contains("create_session"));
    }

    #[test]
    fn slot_catalog_generator_keeps_host_slots_in_a_separate_namespace() {
        let source =
            interface_spec_with_slots("[\"release_allocation\"]", "[]", "[\"release_allocation\"]");
        let catalog = generate_slot_catalog_from_source(&source)
            .expect("host and runtime tables may reuse a field label");

        assert_eq!(catalog.matches("\"release_allocation\"").count(), 2);
    }

    #[test]
    fn slot_catalog_generator_rejects_non_rust_slot_identifiers() {
        let source = interface_spec_with_slots("[\"1bad\"]", "[]", "[]");
        let error = generate_slot_catalog_from_source(&source)
            .expect_err("a generated Rust catalog must reject invalid identifiers");

        assert!(error.contains("runtime_api_required_slots"));
        assert!(error.contains("1bad"));
    }

    #[test]
    fn slot_catalog_generator_rejects_metadata_that_does_not_describe_v8() {
        let source = CURRENT_INTERFACE_SPEC
            .replace("\"runtime_api_version\": 8", "\"runtime_api_version\": 9");
        let error = generate_slot_catalog_from_source(&source)
            .expect_err("the V8 Rust table cannot be generated from a V9 InterfaceSpec");

        assert!(error.contains("runtime_api_version"));
        assert!(error.contains("must be 8"));
    }
}
