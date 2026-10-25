//! Memory-only unit values. These are not registered project assets or provider fixtures.

use serde_json::{json, Value};
use zircon_runtime::asset::{AssetUri, DataAsset, DataAssetFormat};

use crate::ui::curve::{CurveInterpolation, CurveModel};

use super::*;

fn source(kind: StandaloneCurveKind) -> StandaloneCurveSource {
    use StandaloneCurveChannelId::*;
    let channels: &[StandaloneCurveChannelId] = match kind {
        StandaloneCurveKind::Float => &[Value],
        StandaloneCurveKind::Vector => &[X, Y, Z],
        StandaloneCurveKind::LinearColor => &[R, G, B, A],
    };
    StandaloneCurveSource {
        version: 1,
        kind,
        precision: StandaloneCurvePrecision::F32,
        input_unit: StandaloneCurveInputUnit::Unitless,
        channels: channels
            .iter()
            .map(|id| StandaloneCurveChannel {
                id: *id,
                display_name: format!("曲线 {id:?}"),
                default_value: None,
                pre_extrapolation: StandaloneCurveExtrapolation::Constant,
                post_extrapolation: StandaloneCurveExtrapolation::Constant,
                keys: vec![key("start", 0.0, 0.0), key("end", 1.0, 1.0)],
            })
            .collect(),
    }
}

fn key(id: &str, input: f32, value: f32) -> StandaloneCurveKey {
    StandaloneCurveKey {
        id: id.into(),
        input,
        value,
        interpolation: StandaloneCurveInterpolation::Linear,
        tangent_mode: None,
        in_tangent: None,
        out_tangent: None,
    }
}

fn data_asset(text: String) -> DataAsset {
    DataAsset {
        uri: AssetUri::parse("res://curve-unit-source.json").unwrap(),
        format: DataAssetFormat::Json,
        canonical_json: serde_json::from_str(&text).unwrap(),
        text,
    }
}

fn asset_for(source: &StandaloneCurveSource) -> DataAsset {
    data_asset(format!(
        "\n{}\n",
        serde_json::to_string_pretty(source).unwrap()
    ))
}

fn decode_value(value: &Value) -> Result<StandaloneCurveSource, StandaloneCurveError> {
    StandaloneCurveCodec::decode(
        &data_asset(serde_json::to_string(value).unwrap()),
        StandaloneCurveKind::Float,
    )
}

fn edit(
    source: &StandaloneCurveSource,
    channel: usize,
    index: usize,
    value: f32,
) -> StandaloneCurveKeyEdit {
    let expected = source.channels[channel].keys[index].clone();
    let mut replacement = expected.clone();
    replacement.value = value;
    StandaloneCurveKeyEdit {
        channel: source.channels[channel].id,
        expected,
        replacement,
    }
}

#[test]
fn standalone_curve_noop_preserves_source_bytes_and_locator() {
    let expected = source(StandaloneCurveKind::Float);
    let mut asset = asset_for(&expected);
    let original = asset.clone();
    let decoded = StandaloneCurveCodec::decode(&asset, expected.kind).unwrap();
    assert_eq!(decoded, expected);
    assert!(!StandaloneCurveCodec::encode_into(&decoded, &mut asset, expected.kind).unwrap());
    assert_eq!(asset, original);

    // An absent canonical cache stays absent on a no-op; it is not an empty document.
    asset.canonical_json = Value::Null;
    let original = asset.clone();
    assert!(!StandaloneCurveCodec::encode_into(&decoded, &mut asset, expected.kind).unwrap());
    assert_eq!(asset, original);
}

#[test]
fn standalone_curve_explicit_kinds_keep_channels_and_independent_keys() {
    for kind in [
        StandaloneCurveKind::Float,
        StandaloneCurveKind::Vector,
        StandaloneCurveKind::LinearColor,
    ] {
        let mut expected = source(kind);
        for (index, channel) in expected.channels.iter_mut().enumerate() {
            // Shared key names in different channels are valid; names are not global IDs.
            channel.keys[0].value = -2.0 - index as f32;
            channel.keys[1].value = 4.0 + index as f32;
            channel.keys[1].input = 1.0 + index as f32;
        }
        expected.channels.reverse();
        let asset = asset_for(&expected);
        let mut decoded = StandaloneCurveCodec::decode(&asset, kind).unwrap();
        assert_eq!(decoded, expected);
        let views = StandaloneCurveAdapter::new(&mut decoded).unwrap().curves();
        assert_eq!(views.len(), expected.channels.len());
        for (view, channel) in views.iter().zip(&expected.channels) {
            assert_eq!(view.id, channel.id);
            assert_eq!(view.display_name, channel.display_name);
            assert_eq!(view.keys[0].id, "start");
            assert_eq!(view.keys[0].point.value, channel.keys[0].value);
            assert_eq!(view.keys[1].point.time, channel.keys[1].input);
        }
        // LinearColor accepts HDR and negative channel values without display clamping.
        assert_eq!(decoded, expected);
    }
}

#[test]
fn standalone_curve_unknown_semantics_are_rejected_without_overwrite() {
    let original = asset_for(&source(StandaloneCurveKind::Float));
    let mutations = [
        ("root", json!({"weight": 0.5})),
        ("channel", json!({"color_adjustment": 1.0})),
        ("key", json!({"tangent_weight": 0.5})),
        ("precision", json!("f64")),
        ("pre_extrapolation", json!("cycle")),
        ("tangent_mode", json!("auto")),
        ("interpolation", json!("weighted_cubic")),
    ];
    for (field, unsupported) in mutations {
        let mut value = original.canonical_json.clone();
        match field {
            "root" => value
                .as_object_mut()
                .unwrap()
                .extend(unsupported.as_object().unwrap().clone()),
            "channel" => value["channels"][0]
                .as_object_mut()
                .unwrap()
                .extend(unsupported.as_object().unwrap().clone()),
            "key" => value["channels"][0]["keys"][0]
                .as_object_mut()
                .unwrap()
                .extend(unsupported.as_object().unwrap().clone()),
            "precision" => value["precision"] = unsupported,
            "pre_extrapolation" => value["channels"][0][field] = unsupported,
            _ => value["channels"][0]["keys"][0][field] = unsupported,
        }
        let mut unsupported_asset = data_asset(serde_json::to_string(&value).unwrap());
        let before = unsupported_asset.clone();
        assert!(
            StandaloneCurveCodec::decode(&unsupported_asset, StandaloneCurveKind::Float).is_err(),
            "{field}"
        );
        assert!(
            StandaloneCurveCodec::encode_into(
                &source(StandaloneCurveKind::Float),
                &mut unsupported_asset,
                StandaloneCurveKind::Float
            )
            .is_err(),
            "{field}"
        );
        assert_eq!(unsupported_asset, before, "{field}");
    }
}

#[test]
fn standalone_curve_versions_formats_and_expected_kind_require_explicit_admission() {
    let expected = source(StandaloneCurveKind::Float);
    let mut asset = asset_for(&expected);
    for format in [
        DataAssetFormat::Text,
        DataAssetFormat::Toml,
        DataAssetFormat::Yaml,
        DataAssetFormat::Xml,
    ] {
        asset.format = format;
        assert!(
            matches!(StandaloneCurveCodec::decode(&asset, expected.kind), Err(StandaloneCurveError::UnsupportedFormat(actual)) if actual == format)
        );
    }
    asset.format = DataAssetFormat::Json;
    assert!(matches!(
        StandaloneCurveCodec::decode(&asset, StandaloneCurveKind::Vector),
        Err(StandaloneCurveError::KindMismatch { .. })
    ));
    let mut value = asset.canonical_json;
    value["version"] = json!(2);
    assert!(matches!(
        decode_value(&value),
        Err(StandaloneCurveError::UnsupportedVersion(2))
    ));
    for required in ["kind", "precision", "input_unit", "channels"] {
        let mut value = serde_json::to_value(&expected).unwrap();
        value.as_object_mut().unwrap().remove(required);
        assert!(decode_value(&value).is_err(), "{required}");
    }
}

#[test]
fn standalone_curve_raw_duplicate_fields_and_stale_canonical_cache_are_rejected() {
    let expected = source(StandaloneCurveKind::Float);
    let mut asset = asset_for(&expected);
    asset.canonical_json["version"] = json!(99);
    assert!(matches!(
        StandaloneCurveCodec::decode(&asset, expected.kind),
        Err(StandaloneCurveError::CanonicalMismatch)
    ));

    let text = serde_json::to_string(&expected).unwrap();
    let duplicate =
        asset_for(&expected)
            .text
            .replacen("\"version\": 1", "\"version\": 1, \"version\": 1", 1);
    assert_ne!(duplicate, asset_for(&expected).text);
    assert!(matches!(
        StandaloneCurveCodec::decode(&data_asset(duplicate), expected.kind),
        Err(StandaloneCurveError::Encoding(_))
    ));
    let duplicate_key = text.replacen("\"id\":\"start\"", "\"id\":\"start\",\"id\":\"start\"", 1);
    assert_ne!(duplicate_key, text);
    assert!(StandaloneCurveCodec::decode(&data_asset(duplicate_key), expected.kind).is_err());
}

#[test]
fn standalone_curve_numeric_tokens_cannot_hide_precision_or_range_loss() {
    let raw = serde_json::to_string(&source(StandaloneCurveKind::Float)).unwrap();
    assert!(raw.contains("\"value\":0.0"));
    for token in [
        "0.1",
        "1e-1",
        "0.1000",
        "0.100000001490116119384765625",
        "16777216",
        "-16777216",
        "3.4028235e38",
        "1e-45",
    ] {
        let text = raw.replacen("\"value\":0.0", &format!("\"value\":{token}"), 1);
        let decoded =
            StandaloneCurveCodec::decode(&data_asset(text), StandaloneCurveKind::Float).unwrap();
        assert_eq!(
            decoded.channels[0].keys[0].value,
            token.parse::<f32>().unwrap(),
            "{token}"
        );
    }
    for token in [
        "16777217",
        "9223372036854775807",
        "18446744073709551615",
        "1.000000000000000000000001",
        "0.100000000000000000000001",
        "1e40",
        "1e-60",
        "\"0.1\"",
    ] {
        let text = raw.replacen("\"value\":0.0", &format!("\"value\":{token}"), 1);
        assert!(
            StandaloneCurveCodec::decode(&data_asset(text), StandaloneCurveKind::Float).is_err(),
            "{token}"
        );
    }
}

#[test]
fn standalone_curve_invalid_membership_identity_and_order_are_not_normalized() {
    let original = source(StandaloneCurveKind::Float);
    let mut cases = Vec::new();
    let mut invalid = original.clone();
    invalid.channels.clear();
    cases.push(invalid);
    let mut invalid = original.clone();
    invalid.channels.push(invalid.channels[0].clone());
    cases.push(invalid);
    let mut invalid = original.clone();
    invalid.channels[0].id = StandaloneCurveChannelId::X;
    cases.push(invalid);
    for id in ["", " ", " start", "end"] {
        let mut invalid = original.clone();
        invalid.channels[0].keys[0].id = id.into();
        cases.push(invalid);
    }
    for input in [0.0, -1.0, f32::NAN, f32::INFINITY] {
        let mut invalid = original.clone();
        invalid.channels[0].keys[1].input = input;
        cases.push(invalid);
    }
    let mut invalid = original.clone();
    invalid.channels[0].keys[0].value = f32::NEG_INFINITY;
    cases.push(invalid);
    for invalid in cases {
        assert!(invalid.validate().is_err());
    }
}

#[test]
fn standalone_curve_empty_and_explicit_default_are_distinct_without_synthetic_keys() {
    let mut empty = source(StandaloneCurveKind::Float);
    empty.channels[0].keys.clear();
    assert!(StandaloneCurveAdapter::new(&mut empty).unwrap().curves()[0]
        .keys
        .is_empty());
    let mut with_default = empty.clone();
    with_default.channels[0].default_value = Some(0.1);
    let mut asset = asset_for(&empty);
    assert!(StandaloneCurveCodec::encode_into(&with_default, &mut asset, empty.kind).unwrap());
    assert_eq!(
        StandaloneCurveCodec::decode(&asset, empty.kind).unwrap(),
        with_default
    );
    assert!(matches!(
        StandaloneCurveAdapter::new(&mut with_default),
        Err(StandaloneCurveError::UnsupportedSurface { .. })
    ));
    assert!(with_default.channels[0].keys.is_empty());
    assert_eq!(with_default.channels[0].default_value, Some(0.1));
}

#[test]
fn standalone_curve_mixed_modes_remain_in_source_but_cannot_be_flattened_by_surface() {
    let mut mixed = source(StandaloneCurveKind::Float);
    mixed.channels[0].keys[0].interpolation = StandaloneCurveInterpolation::Constant;
    let decoded = StandaloneCurveCodec::decode(&asset_for(&mixed), mixed.kind).unwrap();
    assert_eq!(decoded, mixed);
    let before = mixed.clone();
    assert!(matches!(
        StandaloneCurveAdapter::new(&mut mixed),
        Err(StandaloneCurveError::UnsupportedSurface { .. })
    ));
    assert_eq!(mixed, before);

    for key in &mut mixed.channels[0].keys {
        key.interpolation = StandaloneCurveInterpolation::Constant;
    }
    assert_eq!(
        StandaloneCurveAdapter::new(&mut mixed).unwrap().curves()[0].interpolation,
        CurveInterpolation::Step
    );
}

#[test]
fn standalone_curve_manual_cubic_projects_both_slopes_and_rejects_missing_tangents() {
    let mut cubic = source(StandaloneCurveKind::Float);
    for key in &mut cubic.channels[0].keys {
        key.interpolation = StandaloneCurveInterpolation::Cubic;
        key.tangent_mode = Some(StandaloneCurveTangentMode::User);
        key.in_tangent = Some(0.1);
        key.out_tangent = Some(0.1);
    }
    let mut decoded = StandaloneCurveCodec::decode(&asset_for(&cubic), cubic.kind).unwrap();
    assert_eq!(decoded, cubic);
    let views = StandaloneCurveAdapter::new(&mut decoded).unwrap().curves();
    assert_eq!(views[0].interpolation, CurveInterpolation::Hermite);
    for key in &views[0].keys {
        assert_eq!(key.in_tangent, Some(0.1));
        assert_eq!(key.out_tangent, Some(0.1));
    }
    cubic.channels[0].keys[1].in_tangent = Some(-0.1);
    assert!(cubic.validate().is_err());
    cubic.channels[0].keys[1].in_tangent = None;
    assert!(cubic.validate().is_err());
    let mut linear = source(StandaloneCurveKind::Float);
    linear.channels[0].keys[0].in_tangent = Some(0.0);
    assert!(linear.validate().is_err());
}

#[test]
fn standalone_curve_edit_inverse_redo_and_codec_reopen_keep_stable_identity() {
    let mut source = source(StandaloneCurveKind::Vector);
    let before = source.clone();
    let delta = StandaloneCurveDelta {
        edits: vec![edit(&source, 0, 0, -3.0), edit(&source, 2, 1, 8.0)],
    };
    let inverse = StandaloneCurveAdapter::new(&mut source)
        .unwrap()
        .apply(delta.clone())
        .unwrap();
    let changed = source.clone();
    assert_ne!(changed, before);
    let redo = StandaloneCurveAdapter::new(&mut source)
        .unwrap()
        .apply(inverse)
        .unwrap();
    assert_eq!(source, before);
    assert_eq!(redo, delta);
    StandaloneCurveAdapter::new(&mut source)
        .unwrap()
        .apply(redo)
        .unwrap();
    assert_eq!(source, changed);
    let mut asset = asset_for(&before);
    let uri = asset.uri.clone();
    assert!(StandaloneCurveCodec::encode_into(&source, &mut asset, source.kind).unwrap());
    assert_eq!(asset.uri, uri);
    assert_eq!(asset.format, DataAssetFormat::Json);
    assert_eq!(
        StandaloneCurveCodec::decode(&asset, source.kind).unwrap(),
        changed
    );
    assert_eq!(
        changed.channels[0].keys[0].id,
        before.channels[0].keys[0].id
    );
    assert_eq!(changed.channels[2].id, before.channels[2].id);
}

#[test]
fn standalone_curve_batch_stale_or_invalid_last_edit_does_not_publish_first_edit() {
    let original = source(StandaloneCurveKind::Vector);
    let valid = edit(&original, 0, 0, 9.0);
    let mut stale = edit(&original, 1, 1, 5.0);
    stale.expected.value = 99.0;
    let mut missing = edit(&original, 1, 1, 5.0);
    missing.expected.id = "removed".into();
    missing.replacement.id = "removed".into();
    let mut renamed = edit(&original, 1, 1, 5.0);
    renamed.replacement.id = "new-id".into();
    let mut collided = edit(&original, 1, 1, 5.0);
    collided.replacement.input = 0.0;
    let mut nonfinite = edit(&original, 1, 1, 5.0);
    nonfinite.replacement.value = f32::NAN;
    for last in [stale, missing, renamed, collided, nonfinite, valid.clone()] {
        let mut source = original.clone();
        assert!(StandaloneCurveAdapter::new(&mut source)
            .unwrap()
            .apply(StandaloneCurveDelta {
                edits: vec![valid.clone(), last]
            })
            .is_err());
        assert_eq!(source, original);
    }
}

#[test]
fn standalone_curve_noop_delta_and_failed_encode_leave_original_unchanged() {
    let mut source = source(StandaloneCurveKind::Float);
    let before = source.clone();
    let noop = StandaloneCurveDelta {
        edits: vec![edit(&source, 0, 0, 0.0)],
    };
    assert!(noop.is_noop());
    let inverse = StandaloneCurveAdapter::new(&mut source)
        .unwrap()
        .apply(noop)
        .unwrap();
    assert!(inverse.is_noop());
    assert_eq!(source, before);
    let inverse = StandaloneCurveAdapter::new(&mut source)
        .unwrap()
        .apply(StandaloneCurveDelta::default())
        .unwrap();
    assert!(inverse.is_noop());
    let mut asset = asset_for(&source);
    let original = asset.clone();
    source.channels[0].keys[0].value = f32::NAN;
    assert!(StandaloneCurveCodec::encode_into(&source, &mut asset, source.kind).is_err());
    assert_eq!(asset, original);
}

#[path = "tests/signed_zero.rs"]
mod signed_zero;

#[path = "tests/boundaries.rs"]
mod boundaries;

#[path = "tests/prepared_edit.rs"]
mod prepared_edit;

#[path = "tests/key_structure.rs"]
mod key_structure;
