//! Memory-only boundary values; these neither register an asset nor prove Toolkit admission.

use serde_json::json;

use crate::ui::curve::{CurveInterpolation, CurveModel};

use super::super::{
    StandaloneCurveAdapter, StandaloneCurveCodec, StandaloneCurveDelta, StandaloneCurveError,
    StandaloneCurveInterpolation, StandaloneCurveKeyEdit, StandaloneCurveKind,
    StandaloneCurveTangentMode,
};
use super::{asset_for, data_asset, key, source};

#[test]
fn standalone_curve_boundary_channels_keep_independent_modes_and_key_inputs() {
    let mut expected = source(StandaloneCurveKind::Vector);
    for key in &mut expected.channels[0].keys {
        key.interpolation = StandaloneCurveInterpolation::Constant;
    }
    expected.channels[1].keys[1].input = 2.5;
    for key in &mut expected.channels[2].keys {
        key.interpolation = StandaloneCurveInterpolation::Cubic;
        key.tangent_mode = Some(StandaloneCurveTangentMode::User);
        key.in_tangent = Some(-2.0);
        key.out_tangent = Some(-2.0);
        key.value = 4.0;
    }
    expected.channels[2].keys[1].input = 3.25;
    let asset = asset_for(&expected);
    let mut decoded = StandaloneCurveCodec::decode(&asset, expected.kind).unwrap();
    let views = StandaloneCurveAdapter::new(&mut decoded).unwrap().curves();
    assert_eq!(views[0].interpolation, CurveInterpolation::Step);
    assert_eq!(views[1].interpolation, CurveInterpolation::Linear);
    assert_eq!(views[2].interpolation, CurveInterpolation::Hermite);
    for (view, channel) in views.iter().zip(&expected.channels) {
        assert_eq!(view.id, channel.id);
        for (projected, original) in view.keys.iter().zip(&channel.keys) {
            assert_eq!(projected.id, original.id);
            assert_eq!(projected.point.time.to_bits(), original.input.to_bits());
            assert_eq!(projected.point.value.to_bits(), original.value.to_bits());
            assert_eq!(projected.in_tangent, original.in_tangent);
            assert_eq!(projected.out_tangent, original.out_tangent);
        }
    }
    assert_eq!(decoded, expected);
}

#[test]
fn standalone_curve_boundary_rich_modes_weights_and_extrapolation_reject_before_overwrite() {
    let expected = source(StandaloneCurveKind::Float);
    let base = serde_json::to_value(&expected).unwrap();
    let mut cases = Vec::new();
    for mode in ["auto", "auto_clamped", "break", "weighted", "unknown"] {
        let mut value = base.clone();
        value["channels"][0]["keys"][0]["tangent_mode"] = json!(mode);
        cases.push(value);
    }
    for field in [
        "arrive_tangent_weight",
        "leave_tangent_weight",
        "in_tangent_weight",
        "out_tangent_weight",
        "tangent_weight_mode",
    ] {
        for weight in [json!(0), serde_json::Value::Null] {
            let mut value = base.clone();
            value["channels"][0]["keys"][0][field] = weight;
            cases.push(value);
        }
    }
    for field in ["pre_extrapolation", "post_extrapolation"] {
        for mode in ["cycle", "cycle_with_offset", "oscillate", "linear", "none"] {
            let mut value = base.clone();
            value["channels"][0][field] = json!(mode);
            cases.push(value);
        }
    }
    for value in cases {
        let mut asset = data_asset(serde_json::to_string(&value).unwrap());
        let original = asset.clone();
        assert!(matches!(
            StandaloneCurveCodec::decode(&asset, expected.kind),
            Err(StandaloneCurveError::Encoding(_))
        ));
        assert!(StandaloneCurveCodec::encode_into(&expected, &mut asset, expected.kind).is_err());
        assert_eq!(asset, original);
    }
}

#[test]
fn standalone_curve_boundary_gradient_and_seven_color_adjustments_are_not_silently_dropped() {
    let expected = source(StandaloneCurveKind::LinearColor);
    let base = serde_json::to_value(&expected).unwrap();
    for field in [
        "gradient",
        "adjust_hue",
        "adjust_saturation",
        "adjust_brightness",
        "adjust_brightness_curve",
        "adjust_vibrance",
        "adjust_min_alpha",
        "adjust_max_alpha",
    ] {
        let mut value = base.clone();
        value[field] = json!(0.0);
        let mut asset = data_asset(serde_json::to_string(&value).unwrap());
        let original = asset.clone();
        assert!(matches!(
            StandaloneCurveCodec::decode(&asset, expected.kind),
            Err(StandaloneCurveError::Encoding(_))
        ));
        assert!(StandaloneCurveCodec::encode_into(&expected, &mut asset, expected.kind).is_err());
        assert_eq!(
            asset, original,
            "unsupported field {field} must remain intact"
        );
    }
}

#[test]
fn standalone_curve_boundary_full_snapshot_codec_does_not_create_an_insert_delete_command() {
    let original = source(StandaloneCurveKind::Float);
    let mut asset = asset_for(&original);
    let mut replacement = original.clone();
    replacement.channels[0].keys.remove(0);
    replacement.channels[0].keys.push(key("new", 2.0, 4.0));
    assert!(StandaloneCurveCodec::encode_into(&replacement, &mut asset, original.kind).unwrap());
    let mut reopened = StandaloneCurveCodec::decode(&asset, original.kind).unwrap();
    assert_eq!(reopened, replacement);
    let before = reopened.clone();
    assert!(matches!(
        StandaloneCurveAdapter::new(&mut reopened)
            .unwrap()
            .apply(StandaloneCurveDelta {
                edits: vec![StandaloneCurveKeyEdit {
                    channel: original.channels[0].id,
                    expected: original.channels[0].keys[0].clone(),
                    replacement: original.channels[0].keys[0].clone(),
                }]
            }),
        Err(StandaloneCurveError::StaleKey { .. })
    ));
    assert_eq!(reopened, before);
}
