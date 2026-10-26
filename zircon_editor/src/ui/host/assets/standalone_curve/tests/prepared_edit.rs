//! Memory-only preparation contracts; these do not publish assets or grant document writes.

use serde_json::Value;

use super::super::{
    StandaloneCurveCodec, StandaloneCurveDelta, StandaloneCurveError, StandaloneCurveInterpolation,
    StandaloneCurveKeyEdit, StandaloneCurveKind,
};
use super::{asset_for, data_asset, edit, source};

#[test]
fn standalone_curve_prepare_batch_returns_complete_candidate_without_mutating_original() {
    let expected = source(StandaloneCurveKind::Vector);
    let mut asset = asset_for(&expected);
    asset.canonical_json = Value::Null;
    let original = asset.clone();
    let delta = StandaloneCurveDelta {
        edits: vec![
            edit(&expected, 0, 0, -7.0),
            edit(&expected, 1, 1, 4.0),
            edit(&expected, 2, 0, -0.0),
        ],
    };
    let candidate = StandaloneCurveCodec::prepare_edit(&asset, expected.kind, delta)
        .unwrap()
        .unwrap();
    assert_eq!(asset, original);
    assert_eq!(candidate.uri, original.uri);
    assert_eq!(candidate.format, original.format);
    assert_eq!(
        candidate.canonical_json,
        serde_json::from_str::<Value>(&candidate.text).unwrap()
    );
    let mut changed = expected.clone();
    changed.channels[0].keys[0].value = -7.0;
    changed.channels[1].keys[1].value = 4.0;
    changed.channels[2].keys[0].value = -0.0;
    assert_eq!(
        StandaloneCurveCodec::decode(&candidate, expected.kind).unwrap(),
        changed
    );
    assert_eq!(
        StandaloneCurveCodec::decode(&asset, expected.kind).unwrap(),
        expected
    );
}

#[test]
fn standalone_curve_prepare_noop_preserves_raw_spelling_and_empty_or_populated_cache() {
    let expected = source(StandaloneCurveKind::Float);
    let raw = format!(
        "\n\t{}\n \n",
        serde_json::to_string_pretty(&expected)
            .unwrap()
            .replace("\"value\": 1.0", "\"value\": 1.0000")
    );
    for empty_cache in [false, true] {
        let mut asset = data_asset(raw.clone());
        if empty_cache {
            asset.canonical_json = Value::Null;
        }
        let before = asset.clone();
        for delta in [
            StandaloneCurveDelta::default(),
            StandaloneCurveDelta {
                edits: vec![edit(&expected, 0, 1, 1.0)],
            },
        ] {
            assert!(
                StandaloneCurveCodec::prepare_edit(&asset, expected.kind, delta)
                    .unwrap()
                    .is_none()
            );
            assert_eq!(asset, before);
        }
    }
}

#[test]
fn standalone_curve_prepare_stale_last_key_and_stale_noop_reject_the_whole_batch() {
    let expected = source(StandaloneCurveKind::Float);
    let asset = asset_for(&expected);
    let original = asset.clone();
    let mut stale = edit(&expected, 0, 1, 7.0);
    stale.expected.value = 42.0;
    let stale_noop = StandaloneCurveKeyEdit {
        channel: stale.channel,
        expected: stale.expected.clone(),
        replacement: stale.expected.clone(),
    };
    for edits in [vec![edit(&expected, 0, 0, 8.0), stale], vec![stale_noop]] {
        assert!(matches!(
            StandaloneCurveCodec::prepare_edit(
                &asset,
                expected.kind,
                StandaloneCurveDelta { edits }
            ),
            Err(StandaloneCurveError::StaleKey { .. })
        ));
        assert_eq!(asset, original);
    }
}

#[test]
fn standalone_curve_prepare_invalid_final_keys_and_duplicate_targets_leave_source_intact() {
    let expected = source(StandaloneCurveKind::Float);
    let asset = asset_for(&expected);
    let original = asset.clone();
    let valid = edit(&expected, 0, 0, 2.0);
    let mut cases = Vec::new();
    for value in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        cases.push(vec![edit(&expected, 0, 0, value)]);
    }
    let mut crossed = valid.clone();
    crossed.replacement.input = 2.0;
    cases.push(vec![crossed]);
    let mut tangent_on_linear = valid.clone();
    tangent_on_linear.replacement.in_tangent = Some(0.0);
    cases.push(vec![tangent_on_linear]);
    let mut renamed = valid.clone();
    renamed.replacement.id = "renamed".into();
    cases.push(vec![renamed]);
    cases.push(vec![valid.clone(), valid]);
    for edits in cases {
        assert!(matches!(
            StandaloneCurveCodec::prepare_edit(
                &asset,
                expected.kind,
                StandaloneCurveDelta { edits }
            ),
            Err(StandaloneCurveError::InvalidSource { .. })
        ));
        assert_eq!(asset, original);
    }
}

#[test]
fn standalone_curve_prepare_validates_the_final_channel_mode_before_encoding() {
    let expected = source(StandaloneCurveKind::Vector);
    let asset = asset_for(&expected);
    let original = asset.clone();
    let edits = expected.channels[0]
        .keys
        .iter()
        .map(|key| {
            let mut replacement = key.clone();
            replacement.interpolation = StandaloneCurveInterpolation::Constant;
            StandaloneCurveKeyEdit {
                channel: expected.channels[0].id,
                expected: key.clone(),
                replacement,
            }
        })
        .collect::<Vec<_>>();
    assert!(matches!(
        StandaloneCurveCodec::prepare_edit(
            &asset,
            expected.kind,
            StandaloneCurveDelta {
                edits: vec![edits[0].clone()]
            }
        ),
        Err(StandaloneCurveError::UnsupportedSurface { .. })
    ));
    let candidate =
        StandaloneCurveCodec::prepare_edit(&asset, expected.kind, StandaloneCurveDelta { edits })
            .unwrap()
            .unwrap();
    let reopened = StandaloneCurveCodec::decode(&candidate, expected.kind).unwrap();
    assert!(reopened.channels[0]
        .keys
        .iter()
        .all(|key| key.interpolation == StandaloneCurveInterpolation::Constant));
    assert_eq!(reopened.channels[1..], expected.channels[1..]);
    assert_eq!(asset, original);
}

#[test]
fn standalone_curve_prepare_unrepresentable_source_rejects_even_an_empty_delta() {
    for with_default in [false, true] {
        let mut expected = source(StandaloneCurveKind::Float);
        if with_default {
            expected.channels[0].default_value = Some(-0.0);
        } else {
            expected.channels[0].keys[0].interpolation = StandaloneCurveInterpolation::Constant;
        }
        let asset = asset_for(&expected);
        let before = asset.clone();
        assert_eq!(
            StandaloneCurveCodec::decode(&asset, expected.kind).unwrap(),
            expected
        );
        assert!(matches!(
            StandaloneCurveCodec::prepare_edit(
                &asset,
                expected.kind,
                StandaloneCurveDelta::default()
            ),
            Err(StandaloneCurveError::UnsupportedSurface { .. })
        ));
        assert_eq!(asset, before);
    }
}

#[test]
fn standalone_curve_prepare_kind_cache_and_unknown_source_fail_before_a_candidate() {
    let expected = source(StandaloneCurveKind::Float);
    let asset = asset_for(&expected);
    let before = asset.clone();
    assert!(matches!(
        StandaloneCurveCodec::prepare_edit(
            &asset,
            StandaloneCurveKind::Vector,
            StandaloneCurveDelta::default()
        ),
        Err(StandaloneCurveError::KindMismatch { .. })
    ));
    assert_eq!(asset, before);
    let mut cached = asset.clone();
    cached.canonical_json = serde_json::json!({"stale": true});
    let before = cached.clone();
    assert!(matches!(
        StandaloneCurveCodec::prepare_edit(&cached, expected.kind, StandaloneCurveDelta::default()),
        Err(StandaloneCurveError::CanonicalMismatch)
    ));
    assert_eq!(cached, before);
    let mut unknown = serde_json::to_value(&expected).unwrap();
    unknown["unknown"] = Value::Null;
    let asset = data_asset(serde_json::to_string(&unknown).unwrap());
    let before = asset.clone();
    assert!(matches!(
        StandaloneCurveCodec::prepare_edit(&asset, expected.kind, StandaloneCurveDelta::default()),
        Err(StandaloneCurveError::Encoding(_))
    ));
    assert_eq!(asset, before);
}

#[test]
fn standalone_curve_prepare_signed_zero_is_a_change_and_old_expected_is_stale_on_reopen() {
    let expected = source(StandaloneCurveKind::Float);
    let asset = asset_for(&expected);
    let before = asset.clone();
    let delta = StandaloneCurveDelta {
        edits: vec![edit(&expected, 0, 0, -0.0)],
    };
    let candidate = StandaloneCurveCodec::prepare_edit(&asset, expected.kind, delta.clone())
        .unwrap()
        .unwrap();
    let reopened = StandaloneCurveCodec::decode(&candidate, expected.kind).unwrap();
    assert_eq!(
        reopened.channels[0].keys[0].value.to_bits(),
        (-0.0_f32).to_bits()
    );
    let prepared = candidate.clone();
    assert!(matches!(
        StandaloneCurveCodec::prepare_edit(&candidate, expected.kind, delta),
        Err(StandaloneCurveError::StaleKey { .. })
    ));
    assert!(StandaloneCurveCodec::prepare_edit(
        &candidate,
        expected.kind,
        StandaloneCurveDelta {
            edits: vec![edit(&reopened, 0, 0, -0.0)]
        }
    )
    .unwrap()
    .is_none());
    assert_eq!(candidate, prepared);
    assert_eq!(asset, before);
}
