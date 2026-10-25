use crate::ui::curve::CurveModel;

use super::super::{
    StandaloneCurveAdapter, StandaloneCurveDelta, StandaloneCurveError,
    StandaloneCurveInterpolation, StandaloneCurveKey, StandaloneCurveKeyEdit, StandaloneCurveKind,
    StandaloneCurveSource, StandaloneCurveTangentMode,
};
use super::{asset_for, edit, source, StandaloneCurveCodec};

fn cubic(kind: StandaloneCurveKind) -> StandaloneCurveSource {
    let mut source = source(kind);
    for channel in &mut source.channels {
        for key in &mut channel.keys {
            key.interpolation = StandaloneCurveInterpolation::Cubic;
            key.tangent_mode = Some(StandaloneCurveTangentMode::User);
            key.in_tangent = Some(0.0);
            key.out_tangent = Some(0.0);
        }
    }
    source
}

fn negative_zero(key: &mut StandaloneCurveKey, field: usize) {
    match field {
        0 => key.value = -0.0,
        1 => key.input = -0.0,
        2 => {
            key.in_tangent = Some(-0.0);
            key.out_tangent = Some(-0.0);
        }
        _ => unreachable!(),
    }
}

#[test]
fn standalone_curve_signed_zero_delta_inverse_and_redo_keep_original_authoring_bits() {
    for field in 0..3 {
        let mut source = cubic(StandaloneCurveKind::Float);
        let before = source.clone();
        let expected = source.channels[0].keys[0].clone();
        let mut replacement = expected.clone();
        negative_zero(&mut replacement, field);
        let delta = StandaloneCurveDelta {
            edits: vec![StandaloneCurveKeyEdit {
                channel: source.channels[0].id,
                expected,
                replacement: replacement.clone(),
            }],
        };
        assert!(
            !delta.is_noop(),
            "signed-zero field {field} is authored data"
        );
        let inverse = StandaloneCurveAdapter::new(&mut source)
            .unwrap()
            .apply(delta.clone())
            .unwrap();
        assert_eq!(source.channels[0].keys[0], replacement);
        assert_ne!(source, before);
        let changed = source.clone();
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
    }
}

#[test]
fn standalone_curve_signed_zero_stale_last_expected_key_does_not_publish_first_edit() {
    for field in 0..4 {
        let mut source = cubic(StandaloneCurveKind::Vector);
        let before = source.clone();
        let valid = edit(&source, 0, 0, 5.0);
        let mut stale = edit(&source, 1, 0, 7.0);
        match field {
            0 => stale.expected.value = -0.0,
            1 => stale.expected.input = -0.0,
            2 => stale.expected.in_tangent = Some(-0.0),
            3 => stale.expected.out_tangent = Some(-0.0),
            _ => unreachable!(),
        }
        assert!(matches!(
            StandaloneCurveAdapter::new(&mut source)
                .unwrap()
                .apply(StandaloneCurveDelta {
                    edits: vec![valid, stale]
                }),
            Err(StandaloneCurveError::StaleKey { .. })
        ));
        assert_eq!(
            source, before,
            "failed field {field} must leave all channels"
        );
    }
}

#[test]
fn standalone_curve_signed_zero_codec_edit_is_not_a_noop_and_reopens_exactly() {
    for field in 0..3 {
        let before = cubic(StandaloneCurveKind::Float);
        let mut changed = before.clone();
        negative_zero(&mut changed.channels[0].keys[0], field);
        let mut asset = asset_for(&before);
        let old_asset = asset.clone();
        assert!(StandaloneCurveCodec::encode_into(&changed, &mut asset, before.kind).unwrap());
        assert_ne!(asset.text, old_asset.text);
        assert_eq!(asset.uri, old_asset.uri);
        assert_eq!(asset.format, old_asset.format);
        assert_eq!(
            StandaloneCurveCodec::decode(&asset, before.kind).unwrap(),
            changed
        );
        let unchanged = asset.clone();
        assert!(!StandaloneCurveCodec::encode_into(&changed, &mut asset, before.kind).unwrap());
        assert_eq!(asset, unchanged);
        assert!(StandaloneCurveCodec::encode_into(&before, &mut asset, before.kind).unwrap());
        assert_eq!(
            StandaloneCurveCodec::decode(&asset, before.kind).unwrap(),
            before
        );
    }
}

#[test]
fn standalone_curve_signed_zero_default_remains_distinct_from_unset_and_surface_rejects_it() {
    let mut source = source(StandaloneCurveKind::Float);
    source.channels[0].keys.clear();
    source.channels[0].default_value = Some(-0.0);
    let mut asset = asset_for(&source);
    let mut positive = source.clone();
    positive.channels[0].default_value = Some(0.0);
    assert_ne!(source, positive);
    assert!(StandaloneCurveCodec::encode_into(&positive, &mut asset, source.kind).unwrap());
    let mut reopened = StandaloneCurveCodec::decode(&asset, source.kind).unwrap();
    assert_eq!(reopened.channels[0].default_value.unwrap().to_bits(), 0);
    let before = reopened.clone();
    assert!(matches!(
        StandaloneCurveAdapter::new(&mut reopened),
        Err(StandaloneCurveError::UnsupportedSurface { .. })
    ));
    assert_eq!(reopened, before);
    source.channels[0].default_value = None;
    assert!(StandaloneCurveCodec::encode_into(&source, &mut asset, source.kind).unwrap());
    assert_eq!(
        StandaloneCurveCodec::decode(&asset, source.kind)
            .unwrap()
            .channels[0]
            .default_value,
        None
    );
}

#[test]
fn standalone_curve_signed_zero_true_noop_keeps_source_spelling_and_missing_cache() {
    let mut expected = cubic(StandaloneCurveKind::Float);
    let key = &mut expected.channels[0].keys[0];
    key.input = -0.0;
    key.value = -0.0;
    key.in_tangent = Some(-0.0);
    // Numerically linked slopes retain their individual authored sign bits.
    key.out_tangent = Some(0.0);
    let mut asset = asset_for(&expected);
    assert!(asset.text.contains("-0.0"));
    asset.text = asset.text.replacen("-0.0", "-0e0", 1);
    asset.canonical_json = serde_json::Value::Null;
    let original = asset.clone();
    let mut decoded = StandaloneCurveCodec::decode(&asset, expected.kind).unwrap();
    assert_eq!(decoded, expected);
    assert!(!StandaloneCurveCodec::encode_into(&decoded, &mut asset, expected.kind).unwrap());
    assert_eq!(asset, original);
    let key = decoded.channels[0].keys[0].clone();
    let delta = StandaloneCurveDelta {
        edits: vec![StandaloneCurveKeyEdit {
            channel: decoded.channels[0].id,
            expected: key.clone(),
            replacement: key,
        }],
    };
    assert!(delta.is_noop());
    let inverse = StandaloneCurveAdapter::new(&mut decoded)
        .unwrap()
        .apply(delta)
        .unwrap();
    assert!(inverse.is_noop());
    assert_eq!(decoded, expected);
}

#[test]
fn standalone_curve_lossless_mode_change_requires_explicit_complete_channel_batch() {
    let mut source = source(StandaloneCurveKind::Vector);
    let before = source.clone();
    let channel = before.channels[0].id;
    let edits: Vec<_> = before.channels[0]
        .keys
        .iter()
        .map(|expected| {
            let mut replacement = expected.clone();
            replacement.interpolation = StandaloneCurveInterpolation::Cubic;
            replacement.tangent_mode = Some(StandaloneCurveTangentMode::User);
            replacement.in_tangent = Some(-0.0);
            replacement.out_tangent = Some(-0.0);
            StandaloneCurveKeyEdit {
                channel,
                expected: expected.clone(),
                replacement,
            }
        })
        .collect();
    assert!(matches!(
        StandaloneCurveAdapter::new(&mut source)
            .unwrap()
            .apply(StandaloneCurveDelta {
                edits: vec![edits[0].clone()]
            }),
        Err(StandaloneCurveError::UnsupportedSurface { .. })
    ));
    assert_eq!(source, before);
    let inverse = StandaloneCurveAdapter::new(&mut source)
        .unwrap()
        .apply(StandaloneCurveDelta { edits })
        .unwrap();
    assert_ne!(source, before);
    assert_eq!(&source.channels[1..], &before.channels[1..]);
    StandaloneCurveAdapter::new(&mut source)
        .unwrap()
        .apply(inverse)
        .unwrap();
    assert_eq!(source, before);
}
