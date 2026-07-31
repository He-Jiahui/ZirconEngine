use std::collections::BTreeMap;

use serde_json::json;
use woc_parity::{
    canonical, fnv1a_hex, fnv1a_step_u32, round6, DrawDigest, Mulberry32, TraceValue, FNV_OFFSET,
};

#[test]
fn mulberry32_matches_target_known_vectors() {
    let mut rng = Mulberry32::new(1);
    let values = [
        rng.next_u32(),
        rng.next_u32(),
        rng.next_u32(),
        rng.next_u32(),
    ];
    assert_eq!(values, [2693262067, 11749833, 2265367787, 4213581821]);

    let mut zero = Mulberry32::new(0);
    assert_eq!(zero.state(), 0x9e37_79b9);
    assert_eq!(zero.next_u32(), 1541420728);
}

#[test]
fn fnv_vectors_match_utf16_and_little_endian_u32_folding() {
    assert_eq!(fnv1a_hex("hello"), "4f9f2cab");
    assert_eq!(fnv1a_hex("世界🙂"), "2751bfac");
    assert_eq!(fnv1a_step_u32(FNV_OFFSET, 0x7856_3412), 0x5b1454e5);

    let mut digest = DrawDigest::default();
    digest.observe_u32(0x7856_3412);
    assert_eq!(digest.draws(), 1);
    assert_eq!(digest.hex(), "5b1454e5");
}

#[test]
fn round6_matches_target_nonfinite_and_negative_rounding_rules() {
    assert_eq!(round6(1.0 / 3.0), json!(0.333333));
    assert_eq!(round6(5.0), json!(5));
    assert_eq!(round6(-2.0000004), json!(-2));
    assert_eq!(round6(f64::INFINITY), json!("Infinity"));
    assert_eq!(round6(f64::NEG_INFINITY), json!("-Infinity"));
    assert_eq!(round6(f64::NAN), json!("NaN"));
}

#[test]
fn canonical_sorts_maps_sets_and_object_keys_and_omits_inert_fields() {
    let value = TraceValue::Object(BTreeMap::from([
        ("zero".to_string(), TraceValue::Number(0.0)),
        (
            "empty_object".to_string(),
            TraceValue::Object(BTreeMap::new()),
        ),
        (
            "map".to_string(),
            TraceValue::Map(vec![
                (TraceValue::Number(10.0), TraceValue::String("ten".into())),
                (TraceValue::Number(2.0), TraceValue::String("two".into())),
            ]),
        ),
        (
            "set".to_string(),
            TraceValue::Set(vec![
                TraceValue::String("z".into()),
                TraceValue::String("a".into()),
            ]),
        ),
    ]));

    assert_eq!(
        canonical(&value, true),
        json!({
            "empty_object": {},
            "map": [[2, "two"], [10, "ten"]],
            "set": ["a", "z"]
        })
    );
}
