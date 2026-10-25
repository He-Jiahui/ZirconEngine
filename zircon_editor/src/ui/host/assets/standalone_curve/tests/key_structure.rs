//! Immutable memory-only source values; no registered asset, schema, host or clock fixture.

use serde_json::{json, Value};
use zircon_runtime::asset::DataAssetFormat;

use super::super::{
    StandaloneCurveChannelId as Channel, StandaloneCurveCodec, StandaloneCurveError,
    StandaloneCurveInterpolation as Interpolation, StandaloneCurveKind as Kind,
    StandaloneCurveTangentMode,
};
use super::{asset_for, data_asset, key, source};

#[test]
fn standalone_curve_structure_insert_sorts_whole_vector_batch_and_preserves_other_channels() {
    let expected = source(Kind::Vector);
    let asset = asset_for(&expected);
    let original = asset.clone();
    let inserted = vec![
        (Channel::Z, key("later", 2.0, 7.0)),
        (Channel::X, key("mid", 0.5, -0.0)),
        (Channel::Z, key("earlier", -1.0, -4.0)),
    ];
    let candidate = StandaloneCurveCodec::prepare_keys_insert(&asset, expected.kind, &inserted)
        .unwrap()
        .unwrap();
    let changed = StandaloneCurveCodec::decode(&candidate, expected.kind).unwrap();
    assert_eq!(asset, original);
    assert_eq!(candidate.uri, asset.uri);
    assert_eq!(candidate.format, asset.format);
    assert_eq!(changed.channels[1], expected.channels[1]);
    assert_eq!(
        changed.channels[0]
            .keys
            .iter()
            .map(|key| key.id.as_str())
            .collect::<Vec<_>>(),
        vec!["start", "mid", "end"]
    );
    assert_eq!(
        changed.channels[2]
            .keys
            .iter()
            .map(|key| key.id.as_str())
            .collect::<Vec<_>>(),
        vec!["earlier", "start", "end", "later"]
    );
    assert_eq!(
        changed.channels[0].keys[1].value.to_bits(),
        (-0.0_f32).to_bits()
    );
    assert_eq!(
        changed.channels[0].display_name,
        expected.channels[0].display_name
    );
}

#[test]
fn standalone_curve_structure_insert_color_batch_retains_hdr_values_and_channel_local_ids() {
    let expected = source(Kind::LinearColor);
    let asset = asset_for(&expected);
    let inserted = vec![
        (Channel::A, key("new", 0.5, -0.0)),
        (Channel::B, key("new", 0.5, -2.5)),
        (Channel::G, key("new", 0.5, 8.0)),
        (Channel::R, key("new", 0.5, 16.0)),
    ];
    let candidate = StandaloneCurveCodec::prepare_keys_insert(&asset, expected.kind, &inserted)
        .unwrap()
        .unwrap();
    let changed = StandaloneCurveCodec::decode(&candidate, expected.kind).unwrap();
    for (channel, new_key) in &inserted {
        assert_eq!(
            changed
                .channels
                .iter()
                .find(|item| item.id == *channel)
                .unwrap()
                .keys[1],
            *new_key
        );
    }
    assert_eq!(changed.input_unit, expected.input_unit);
    assert_eq!(changed.precision, expected.precision);
}

#[test]
fn standalone_curve_structure_insert_rejects_identity_collisions_and_absent_channel() {
    let asset = asset_for(&source(Kind::Float));
    let original = asset.clone();
    for inserted in [
        vec![(Channel::Value, key("start", 0.5, 7.0))],
        vec![
            (Channel::Value, key("new", 0.5, 7.0)),
            (Channel::Value, key("new", 0.75, 8.0)),
        ],
        vec![
            (Channel::Value, key("new", 0.5, 7.0)),
            (Channel::X, key("wrong", 0.5, 8.0)),
        ],
        vec![(Channel::Value, key(" padded ", 0.5, 1.0))],
    ] {
        assert!(matches!(
            StandaloneCurveCodec::prepare_keys_insert(&asset, Kind::Float, &inserted),
            Err(StandaloneCurveError::InvalidSource { .. })
        ));
        assert_eq!(asset, original);
    }
}

#[test]
fn standalone_curve_structure_insert_rejects_duplicate_times_nonfinite_and_invalid_final_modes() {
    let asset = asset_for(&source(Kind::Float));
    let original = asset.clone();
    let mut cases = Vec::new();
    for input in [0.0, -0.0, f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        cases.push(key("invalid", input, 3.0));
    }
    cases.push(key("invalid", 0.75, f32::NAN));
    let mut tangent = key("invalid", 0.75, 3.0);
    tangent.in_tangent = Some(1.0);
    cases.push(tangent);
    let mut mixed = key("invalid", 0.75, 3.0);
    mixed.interpolation = Interpolation::Constant;
    cases.push(mixed);
    for invalid in cases {
        assert!(StandaloneCurveCodec::prepare_keys_insert(
            &asset,
            Kind::Float,
            &[
                (Channel::Value, key("valid-first", 0.5, 2.0)),
                (Channel::Value, invalid)
            ],
        )
        .is_err());
        assert_eq!(asset, original);
    }
    assert!(StandaloneCurveCodec::prepare_keys_insert(
        &asset,
        Kind::Float,
        &[
            (Channel::Value, key("one", 0.5, 2.0)),
            (Channel::Value, key("two", 0.5, 3.0))
        ],
    )
    .is_err());
}

#[test]
fn standalone_curve_structure_remove_whole_color_batch_keeps_empty_authored_channels() {
    let expected = source(Kind::LinearColor);
    let asset = asset_for(&expected);
    let original = asset.clone();
    let mut removed: Vec<_> = expected
        .channels
        .iter()
        .flat_map(|channel| channel.keys.iter().map(|key| (channel.id, key.clone())))
        .collect();
    removed.reverse();
    let candidate = StandaloneCurveCodec::prepare_keys_remove(&asset, expected.kind, &removed)
        .unwrap()
        .unwrap();
    let mut empty = expected.clone();
    for channel in &mut empty.channels {
        channel.keys.clear();
    }
    assert_eq!(
        StandaloneCurveCodec::decode(&candidate, expected.kind).unwrap(),
        empty
    );
    assert_eq!(asset, original);
    assert_eq!(candidate.uri, asset.uri);
    assert_eq!(candidate.format, asset.format);
}

#[test]
fn standalone_curve_structure_remove_stale_missing_duplicate_or_wrong_channel_is_atomic() {
    let expected = source(Kind::Vector);
    let asset = asset_for(&expected);
    let original = asset.clone();
    let first = (Channel::X, expected.channels[0].keys[0].clone());
    let mut stale = expected.channels[2].keys[1].clone();
    stale.value = 42.0;
    let missing = key("missing", 1.0, 1.0);
    for last in [
        (Channel::Z, stale),
        (Channel::Z, missing),
        (Channel::Value, expected.channels[0].keys[1].clone()),
    ] {
        assert!(matches!(
            StandaloneCurveCodec::prepare_keys_remove(
                &asset,
                expected.kind,
                &[first.clone(), last]
            ),
            Err(StandaloneCurveError::StaleKey { .. })
        ));
        assert_eq!(asset, original);
    }
    assert!(matches!(
        StandaloneCurveCodec::prepare_keys_remove(&asset, expected.kind, &[first.clone(), first]),
        Err(StandaloneCurveError::InvalidSource { .. })
    ));
}

#[test]
fn standalone_curve_structure_remove_checks_signed_zero_in_complete_expected_key() {
    let mut expected = source(Kind::Float);
    expected.channels[0].keys[0].value = -0.0;
    let asset = asset_for(&expected);
    let mut stale = expected.channels[0].keys[0].clone();
    stale.value = 0.0;
    assert!(matches!(
        StandaloneCurveCodec::prepare_keys_remove(
            &asset,
            expected.kind,
            &[(Channel::Value, stale)]
        ),
        Err(StandaloneCurveError::StaleKey { .. })
    ));
    let candidate = StandaloneCurveCodec::prepare_keys_remove(
        &asset,
        expected.kind,
        &[(Channel::Value, expected.channels[0].keys[0].clone())],
    )
    .unwrap()
    .unwrap();
    assert_eq!(
        StandaloneCurveCodec::decode(&candidate, expected.kind)
            .unwrap()
            .channels[0]
            .keys,
        vec![expected.channels[0].keys[1].clone()]
    );
}

#[test]
fn standalone_curve_structure_empty_batches_validate_and_preserve_raw_text_and_cache() {
    let expected = source(Kind::Float);
    let raw = format!("\n\t{} \n", serde_json::to_string(&expected).unwrap());
    for empty_cache in [false, true] {
        let mut asset = data_asset(raw.clone());
        if empty_cache {
            asset.canonical_json = Value::Null;
        }
        let original = asset.clone();
        assert!(
            StandaloneCurveCodec::prepare_keys_insert(&asset, expected.kind, &[])
                .unwrap()
                .is_none()
        );
        assert!(
            StandaloneCurveCodec::prepare_keys_remove(&asset, expected.kind, &[])
                .unwrap()
                .is_none()
        );
        assert_eq!(asset, original);
    }
}

#[test]
fn standalone_curve_structure_source_and_surface_admission_precedes_even_empty_batch() {
    let expected = source(Kind::Float);
    let asset = asset_for(&expected);
    let mut explicit_default = expected.clone();
    explicit_default.channels[0].default_value = Some(-0.0);
    let mut mixed = expected.clone();
    mixed.channels[0].keys[1].interpolation = Interpolation::Constant;
    for unsupported in [asset_for(&explicit_default), asset_for(&mixed)] {
        assert!(matches!(
            StandaloneCurveCodec::prepare_keys_insert(&unsupported, expected.kind, &[]),
            Err(StandaloneCurveError::UnsupportedSurface { .. })
        ));
        assert!(matches!(
            StandaloneCurveCodec::prepare_keys_remove(&unsupported, expected.kind, &[]),
            Err(StandaloneCurveError::UnsupportedSurface { .. })
        ));
    }
    let mut wrong_format = asset.clone();
    wrong_format.format = DataAssetFormat::Toml;
    let mut stale_cache = asset.clone();
    stale_cache.canonical_json = json!({});
    for invalid in [wrong_format, stale_cache] {
        assert!(StandaloneCurveCodec::prepare_keys_insert(&invalid, expected.kind, &[]).is_err());
        assert!(StandaloneCurveCodec::prepare_keys_remove(&invalid, expected.kind, &[]).is_err());
    }
    assert!(matches!(
        StandaloneCurveCodec::prepare_keys_insert(&asset, Kind::Vector, &[]),
        Err(StandaloneCurveError::KindMismatch { .. })
    ));
    assert!(matches!(
        StandaloneCurveCodec::prepare_keys_remove(&asset, Kind::Vector, &[]),
        Err(StandaloneCurveError::KindMismatch { .. })
    ));
}

#[test]
fn standalone_curve_structure_cubic_user_slopes_keep_independent_zero_bits() {
    let mut expected = source(Kind::Float);
    for key in &mut expected.channels[0].keys {
        key.interpolation = Interpolation::Cubic;
        key.tangent_mode = Some(StandaloneCurveTangentMode::User);
        key.in_tangent = Some(-0.0);
        key.out_tangent = Some(0.0);
    }
    let asset = asset_for(&expected);
    let mut inserted = expected.channels[0].keys[0].clone();
    inserted.id = "middle".into();
    inserted.input = 0.5;
    inserted.value = -0.0;
    let candidate = StandaloneCurveCodec::prepare_keys_insert(
        &asset,
        expected.kind,
        &[(Channel::Value, inserted.clone())],
    )
    .unwrap()
    .unwrap();
    let changed = StandaloneCurveCodec::decode(&candidate, expected.kind).unwrap();
    assert_eq!(changed.channels[0].keys[1], inserted);
    assert_eq!(
        changed.channels[0].keys[1].in_tangent.unwrap().to_bits(),
        (-0.0_f32).to_bits()
    );
    assert_eq!(
        changed.channels[0].keys[1].out_tangent.unwrap().to_bits(),
        0.0_f32.to_bits()
    );
    let restored = StandaloneCurveCodec::prepare_keys_remove(
        &candidate,
        expected.kind,
        &[(Channel::Value, inserted)],
    )
    .unwrap()
    .unwrap();
    assert_eq!(
        StandaloneCurveCodec::decode(&restored, expected.kind).unwrap(),
        expected
    );
}

#[test]
fn standalone_curve_structure_typed_reciprocal_is_not_original_byte_history() {
    let expected = source(Kind::Float);
    let original = data_asset(format!(
        " \t{}  \n",
        serde_json::to_string(&expected).unwrap()
    ));
    let inserted = key("new", 0.5, -3.0);
    let candidate = StandaloneCurveCodec::prepare_keys_insert(
        &original,
        expected.kind,
        &[(Channel::Value, inserted.clone())],
    )
    .unwrap()
    .unwrap();
    let restored = StandaloneCurveCodec::prepare_keys_remove(
        &candidate,
        expected.kind,
        &[(Channel::Value, inserted)],
    )
    .unwrap()
    .unwrap();
    assert_eq!(
        StandaloneCurveCodec::decode(&restored, expected.kind).unwrap(),
        expected
    );
    assert_ne!(restored.text, original.text);
    assert!(original.text.starts_with(" \t"));
}
