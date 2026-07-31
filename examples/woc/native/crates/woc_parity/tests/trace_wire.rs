use serde_json::json;
use woc_parity::{
    decode_vm_trace, resolve_trace_symbol, trace_symbol_id, VmTraceDecodeLimits, VmTraceWireError,
    FNV_OFFSET, TRACE_SYMBOL_FINGERPRINT,
};

const TAG_UNSIGNED: u8 = 3;
const TAG_FIXED6: u8 = 5;
const TAG_SYMBOL: u8 = 9;
const TAG_ARRAY: u8 = 10;
const TAG_OBJECT: u8 = 11;

#[test]
fn vm_trace_wire_decodes_authoritative_state_and_computes_canonical_digests() {
    let empty = trace_symbol_id("").expect("empty string is a real canonical map value");
    assert_ne!(empty, 0);
    assert_eq!(resolve_trace_symbol(empty), Some(""));

    let bytes = minimal_trace();
    let actual = decode_vm_trace(&bytes, VmTraceDecodeLimits::default()).unwrap();
    assert_eq!(
        actual,
        json!({
            "scenario": "entity_roster",
            "seed": 1012,
            "sampleEvery": 2,
            "ticks": 0,
            "coverage": ["addEntity roster + spatial grids"],
            "draws": 0,
            "drawDigest": "811c9dc5",
            "frames": [{
                "tick": 0,
                "time": 0,
                "nextId": 408,
                "state": "e44ef6e3",
                "events": "741638a5",
                "rng": { "draws": 0, "digest": "811c9dc5" },
                "label": "init",
                "players": [],
                "entities": [{ "hp": 90, "id": 408, "kind": "player" }]
            }]
        })
    );
}

#[test]
fn vm_trace_wire_accepts_nonempty_event_digest_without_event_symbols() {
    let mut bytes = minimal_trace();
    let digest_offset = bytes.len() - std::mem::size_of::<u32>();
    bytes[digest_offset..].copy_from_slice(&0xae7a_3896_u32.to_le_bytes());

    let actual = decode_vm_trace(&bytes, VmTraceDecodeLimits::default()).unwrap();
    assert_eq!(actual["frames"][0]["events"], "ae7a3896");
}

#[test]
fn vm_trace_wire_rejects_dictionary_drift_trailing_bytes_and_duplicate_keys() {
    let mut drifted = minimal_trace();
    drifted[6..14].copy_from_slice(&(TRACE_SYMBOL_FINGERPRINT ^ 1).to_le_bytes());
    assert!(matches!(
        decode_vm_trace(&drifted, VmTraceDecodeLimits::default()),
        Err(VmTraceWireError::DictionaryMismatch { .. })
    ));

    let mut unknown_scenario = minimal_trace();
    unknown_scenario[14..16].copy_from_slice(&0_u16.to_le_bytes());
    assert_eq!(
        decode_vm_trace(&unknown_scenario, VmTraceDecodeLimits::default()).unwrap_err(),
        VmTraceWireError::UnknownSymbol(0)
    );

    let mut non_numeric_time = minimal_trace();
    non_numeric_time[50] = 0;
    assert_eq!(
        decode_vm_trace(&non_numeric_time, VmTraceDecodeLimits::default()).unwrap_err(),
        VmTraceWireError::ExpectedNumber {
            context: "frame.time"
        }
    );

    let mut trailing = minimal_trace();
    trailing.push(0);
    assert_eq!(
        decode_vm_trace(&trailing, VmTraceDecodeLimits::default()).unwrap_err(),
        VmTraceWireError::TrailingBytes { remaining: 1 }
    );

    let mut duplicate = trace_prefix(0);
    push_u16(&mut duplicate, 1);
    push_frame_prefix(&mut duplicate, false);
    push_array(&mut duplicate, 0);
    push_array_header(&mut duplicate, 1);
    duplicate.push(TAG_OBJECT);
    push_u32(&mut duplicate, 2);
    let id = symbol("id");
    push_u16(&mut duplicate, id);
    push_unsigned(&mut duplicate, 1);
    push_u16(&mut duplicate, id);
    push_unsigned(&mut duplicate, 2);
    push_u32(&mut duplicate, 0x7416_38a5);
    assert!(matches!(
        decode_vm_trace(&duplicate, VmTraceDecodeLimits::default()),
        Err(VmTraceWireError::DuplicateObjectKey { symbol }) if symbol == id
    ));
}

#[test]
fn vm_trace_wire_enforces_total_bytes_depth_and_collection_limits() {
    let bytes = minimal_trace();
    let limits = VmTraceDecodeLimits {
        max_total_bytes: bytes.len() - 1,
        ..VmTraceDecodeLimits::default()
    };
    assert!(matches!(
        decode_vm_trace(&bytes, limits),
        Err(VmTraceWireError::LimitExceeded {
            context: "trace bytes",
            ..
        })
    ));

    let mut nested = trace_prefix(0);
    push_u16(&mut nested, 1);
    push_frame_prefix(&mut nested, false);
    nested.push(TAG_ARRAY);
    push_u32(&mut nested, 1);
    nested.push(TAG_ARRAY);
    push_u32(&mut nested, 1);
    nested.push(TAG_ARRAY);
    push_u32(&mut nested, 0);
    push_array(&mut nested, 0);
    push_u32(&mut nested, 0x7416_38a5);
    let limits = VmTraceDecodeLimits {
        max_value_depth: 1,
        ..VmTraceDecodeLimits::default()
    };
    assert!(matches!(
        decode_vm_trace(&nested, limits),
        Err(VmTraceWireError::LimitExceeded {
            context: "value depth",
            ..
        })
    ));
}

fn minimal_trace() -> Vec<u8> {
    let mut bytes = trace_prefix(1);
    push_u16(&mut bytes, 1);
    push_frame_prefix(&mut bytes, true);
    push_array(&mut bytes, 0);
    push_array_header(&mut bytes, 1);
    bytes.push(TAG_OBJECT);
    push_u32(&mut bytes, 3);
    push_u16(&mut bytes, symbol("hp"));
    push_unsigned(&mut bytes, 90);
    push_u16(&mut bytes, symbol("id"));
    push_unsigned(&mut bytes, 408);
    push_u16(&mut bytes, symbol("kind"));
    bytes.push(TAG_SYMBOL);
    push_u16(&mut bytes, symbol("player"));
    push_u32(&mut bytes, 0x7416_38a5);
    bytes
}

fn trace_prefix(coverage_count: u16) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"WTR1");
    push_u16(&mut bytes, 1);
    bytes.extend_from_slice(&TRACE_SYMBOL_FINGERPRINT.to_le_bytes());
    push_u16(&mut bytes, symbol("entity_roster"));
    push_u32(&mut bytes, 1012);
    push_u32(&mut bytes, 2);
    push_u32(&mut bytes, 0);
    push_u16(&mut bytes, coverage_count);
    if coverage_count > 0 {
        push_u16(&mut bytes, symbol("addEntity roster + spatial grids"));
    }
    push_u32(&mut bytes, 0);
    push_u32(&mut bytes, FNV_OFFSET);
    bytes
}

fn push_frame_prefix(bytes: &mut Vec<u8>, full: bool) {
    bytes.extend_from_slice(&0_u64.to_le_bytes());
    bytes.push(TAG_FIXED6);
    bytes.push(0);
    bytes.extend_from_slice(&0_u64.to_le_bytes());
    bytes.extend_from_slice(&408_u64.to_le_bytes());
    push_u16(bytes, symbol("init"));
    bytes.push(u8::from(full));
    push_u32(bytes, 0);
    push_u32(bytes, FNV_OFFSET);
}

fn push_array(bytes: &mut Vec<u8>, length: u32) {
    push_array_header(bytes, length);
}

fn push_array_header(bytes: &mut Vec<u8>, length: u32) {
    bytes.push(TAG_ARRAY);
    push_u32(bytes, length);
}

fn push_unsigned(bytes: &mut Vec<u8>, value: u64) {
    bytes.push(TAG_UNSIGNED);
    bytes.extend_from_slice(&value.to_le_bytes());
}

fn push_u16(bytes: &mut Vec<u8>, value: u16) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

fn push_u32(bytes: &mut Vec<u8>, value: u32) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

fn symbol(value: &str) -> u16 {
    trace_symbol_id(value).unwrap_or_else(|| panic!("missing trace symbol {value}"))
}
