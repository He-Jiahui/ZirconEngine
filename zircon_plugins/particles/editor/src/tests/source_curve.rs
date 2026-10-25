use zircon_editor::ui::curve::{CurveInterpolation, CurveModel, CurvePoint, CurveView};
use zircon_plugin_particles_runtime::{ParticleSystemComponent, ParticlesManager};

use super::{
    ParticleCurveChannel, ParticleSourceCurveAdapter, ParticleSourceCurveEdit,
    ParticleSourceCurveId, ParticleSystemSource,
};

const TEMPLATE: &str = include_str!("../../../templates/cpu_sprite_system.toml");

fn selected_key(source: &ParticleSystemSource) -> String {
    source.emitters[0].key.as_ref().unwrap().clone()
}

fn curve(
    curves: &[CurveView<ParticleSourceCurveId>],
    channel: ParticleCurveChannel,
) -> &CurveView<ParticleSourceCurveId> {
    curves
        .iter()
        .find(|curve| curve.id.channel == channel)
        .unwrap()
}

fn edit_first(
    curves: &[CurveView<ParticleSourceCurveId>],
    channel: ParticleCurveChannel,
    replacement: CurvePoint,
) -> ParticleSourceCurveEdit {
    edit_at(curves, channel, 0, replacement)
}

fn edit_at(
    curves: &[CurveView<ParticleSourceCurveId>],
    channel: ParticleCurveChannel,
    index: usize,
    replacement: CurvePoint,
) -> ParticleSourceCurveEdit {
    let curve = curve(curves, channel);
    ParticleSourceCurveEdit {
        curve_id: curve.id.clone(),
        key_id: curve.keys[index].id.clone(),
        expected: curve.keys[index].point,
        replacement,
    }
}

#[test]
fn real_template_projects_linear_source_curves_with_shared_rgba_keys() {
    let mut source = ParticleSystemSource::from_toml(TEMPLATE).unwrap();
    source.emitters[0].display_name = Some("烟花／a long emitter name".into());
    let original = source.clone();
    let selected = selected_key(&source);
    let model = ParticleSourceCurveAdapter::new(&mut source, &selected).unwrap();
    let curves = model.curves();
    assert_eq!(curves.len(), 5);
    for projected in &curves {
        assert_eq!(projected.id.emitter_key, selected);
        assert!(projected
            .display_name
            .starts_with("烟花／a long emitter name / "));
        assert_eq!(projected.interpolation, CurveInterpolation::Linear);
        assert!(projected
            .keys
            .iter()
            .all(|k| k.in_tangent.is_none() && k.out_tangent.is_none()));
    }
    let red = curve(&curves, ParticleCurveChannel::Red);
    for channel in [
        ParticleCurveChannel::Green,
        ParticleCurveChannel::Blue,
        ParticleCurveChannel::Alpha,
    ] {
        let other = curve(&curves, channel);
        assert_eq!(red.keys[0].id, other.keys[0].id);
        assert_eq!(red.keys[0].point.time, other.keys[0].point.time);
    }
    assert_eq!(source, original);
}

#[test]
fn rgba_time_edit_moves_one_source_key_and_inverse_restores_all_channels() {
    let mut source = ParticleSystemSource::from_toml(TEMPLATE).unwrap();
    let original = source.clone();
    let selected = selected_key(&source);
    let mut model = ParticleSourceCurveAdapter::new(&mut source, &selected).unwrap();
    let before = model.curves();
    let edit = edit_first(
        &before,
        ParticleCurveChannel::Green,
        CurvePoint::new(0.25, 2.5),
    );
    let undo = model.apply(vec![edit.clone()]).unwrap();
    assert_eq!(undo.len(), 1);
    assert_eq!(undo[0].expected, edit.replacement);
    let after = model.curves();
    for channel in [
        ParticleCurveChannel::Red,
        ParticleCurveChannel::Green,
        ParticleCurveChannel::Blue,
        ParticleCurveChannel::Alpha,
    ] {
        let changed = curve(&after, channel);
        let prior = curve(&before, channel);
        assert_eq!(changed.keys[0].id, prior.keys[0].id);
        assert_eq!(changed.keys[0].point.time, 0.25);
        assert_eq!(
            changed.keys[0].point.value,
            if channel == ParticleCurveChannel::Green {
                2.5
            } else {
                prior.keys[0].point.value
            }
        );
    }
    assert_eq!(model.apply(undo).unwrap(), vec![edit]);
    assert_eq!(model.curves(), before);
    assert_eq!(source, original);
}

#[test]
fn an_edit_survives_codec_reopen_and_lowers_into_real_runtime_values() {
    let mut source = ParticleSystemSource::from_toml(TEMPLATE).unwrap();
    let selected = selected_key(&source);
    let mut model = ParticleSourceCurveAdapter::new(&mut source, &selected).unwrap();
    let curves = model.curves();
    model
        .apply(vec![edit_first(
            &curves,
            ParticleCurveChannel::Red,
            CurvePoint::new(0.2, 3.0),
        )])
        .unwrap();
    let reopened = ParticleSystemSource::from_toml(&source.to_toml().unwrap()).unwrap();
    assert_eq!(reopened, source);
    let runtime = reopened.to_cpu_asset().unwrap();
    assert_eq!(runtime.emitters[0].color_over_lifetime[0].t, 0.2);
    assert_eq!(runtime.emitters[0].color_over_lifetime[0].value.x, 3.0);
}

#[test]
fn edited_curve_changes_real_cpu_output_and_inverse_restores_it() {
    fn simulated_red(source: &ParticleSystemSource) -> f32 {
        let manager = ParticlesManager::default();
        manager
            .instantiate(ParticleSystemComponent::new(
                1,
                source.to_cpu_asset().unwrap(),
            ))
            .unwrap();
        manager.tick(0.1).unwrap();
        let frame = manager.snapshot();
        assert_eq!(frame.sprites.len(), 1);
        frame.sprites[0].color.x
    }

    let mut source = ParticleSystemSource::from_toml(TEMPLATE).unwrap();
    source.emitters[0].spawn_rate_per_second = 10.0;
    source.emitters[0].lifetime.min = 1.0;
    source.emitters[0].lifetime.max = 1.0;
    let original_red = simulated_red(&source);
    let selected = selected_key(&source);
    let undo = {
        let mut model = ParticleSourceCurveAdapter::new(&mut source, &selected).unwrap();
        let curves = model.curves();
        model
            .apply(vec![edit_first(
                &curves,
                ParticleCurveChannel::Red,
                CurvePoint::new(0.2, 3.0),
            )])
            .unwrap()
    };
    assert!(simulated_red(&source) > original_red);
    ParticleSourceCurveAdapter::new(&mut source, &selected)
        .unwrap()
        .apply(undo)
        .unwrap();
    assert_eq!(simulated_red(&source), original_red);
}

#[test]
fn crossing_a_key_preserves_identity_and_inverse_restores_source_order() {
    let mut source = ParticleSystemSource::from_toml(TEMPLATE).unwrap();
    let mut middle = source.emitters[0].size_over_lifetime[0].clone();
    middle.key = None;
    middle.t = 0.5;
    source.emitters[0].size_over_lifetime.insert(1, middle);
    source.assign_missing_keys().unwrap();
    let original = source.clone();
    let selected = selected_key(&source);
    let mut model = ParticleSourceCurveAdapter::new(&mut source, &selected).unwrap();
    let curves = model.curves();
    let edit = edit_first(
        &curves,
        ParticleCurveChannel::SizeMultiplier,
        CurvePoint::new(0.75, 0.5),
    );
    let undo = model.apply(vec![edit.clone()]).unwrap();
    let moved = model.curves();
    let keys = &curve(&moved, ParticleCurveChannel::SizeMultiplier).keys;
    assert_eq!(keys[1].id, edit.key_id);
    assert_eq!(keys[1].point, edit.replacement);
    model.apply(undo).unwrap();
    assert_eq!(source, original);
}

#[test]
fn invalid_points_duplicates_and_stale_values_are_rejected_before_mutation() {
    let mut source = ParticleSystemSource::from_toml(TEMPLATE).unwrap();
    let original = source.clone();
    let selected = selected_key(&source);
    let mut model = ParticleSourceCurveAdapter::new(&mut source, &selected).unwrap();
    let curves = model.curves();
    for point in [
        CurvePoint::new(f32::NAN, 1.0),
        CurvePoint::new(0.1, f32::INFINITY),
        CurvePoint::new(-0.1, 1.0),
        CurvePoint::new(1.1, 1.0),
        CurvePoint::new(0.1, -1.0),
        CurvePoint::new(1.0, 1.0),
    ] {
        assert!(model
            .apply(vec![edit_first(
                &curves,
                ParticleCurveChannel::SizeMultiplier,
                point
            )])
            .is_err());
        assert_eq!(model.curves(), curves);
    }
    let mut stale = edit_first(
        &curves,
        ParticleCurveChannel::Red,
        CurvePoint::new(0.1, 1.0),
    );
    stale.expected.time = 0.5;
    assert!(model.apply(vec![stale]).is_err());
    assert_eq!(model.curves(), curves);
    assert_eq!(source, original);
}

#[test]
fn wrong_emitter_or_deleted_key_cannot_mutate_the_selected_curve() {
    let mut source = ParticleSystemSource::from_toml(TEMPLATE).unwrap();
    let original = source.clone();
    let selected = selected_key(&source);
    let mut model = ParticleSourceCurveAdapter::new(&mut source, &selected).unwrap();
    let curves = model.curves();
    let mut edit = edit_first(
        &curves,
        ParticleCurveChannel::Blue,
        CurvePoint::new(0.1, 1.0),
    );
    edit.curve_id.emitter_key = "another emitter".into();
    assert!(model.apply(vec![edit]).is_err());
    let mut edit = edit_first(
        &curves,
        ParticleCurveChannel::Blue,
        CurvePoint::new(0.1, 1.0),
    );
    edit.key_id = "deleted key".into();
    assert!(model.apply(vec![edit]).is_err());
    assert_eq!(source, original);
}

#[test]
fn empty_curves_stay_empty_and_invalid_source_is_not_normalized_by_projection() {
    let mut source = ParticleSystemSource::from_toml(TEMPLATE).unwrap();
    source.emitters[0].color_over_lifetime.clear();
    source.emitters[0].size_over_lifetime.clear();
    let original = source.clone();
    let selected = selected_key(&source);
    let model = ParticleSourceCurveAdapter::new(&mut source, &selected).unwrap();
    assert!(model.curves().iter().all(|curve| curve.keys.is_empty()));
    assert_eq!(source, original);
    source.emitters[0].drag = f32::NAN;
    assert!(ParticleSourceCurveAdapter::new(&mut source, &selected).is_err());
    assert!(source.emitters[0].drag.is_nan());
    let mut empty = ParticleSystemSource::from_toml("[system]\nid = \"empty\"\n").unwrap();
    assert!(ParticleSourceCurveAdapter::new(&mut empty, &selected).is_err());
    assert!(empty.emitters.is_empty());
}

#[test]
fn particle_curve_batch_rgba_gesture_returns_a_complete_inverse_and_redo() {
    let mut source = ParticleSystemSource::from_toml(TEMPLATE).unwrap();
    let original = source.clone();
    let selected = selected_key(&source);
    let values = [3.0, -0.5, 2.0, 0.75];
    let (edits, undo) = {
        let mut model = ParticleSourceCurveAdapter::new(&mut source, &selected).unwrap();
        let before = model.curves();
        let edits: Vec<_> = [
            ParticleCurveChannel::Red,
            ParticleCurveChannel::Green,
            ParticleCurveChannel::Blue,
            ParticleCurveChannel::Alpha,
        ]
        .into_iter()
        .zip(values)
        .map(|(channel, value)| edit_first(&before, channel, CurvePoint::new(0.25, value)))
        .collect();
        let undo = CurveModel::apply(&mut model, edits.clone()).unwrap();
        assert_eq!(undo.len(), edits.len());
        for (inverse, edit) in undo.iter().zip(&edits) {
            assert_eq!(inverse.curve_id, edit.curve_id);
            assert_eq!(inverse.key_id, edit.key_id);
            assert_eq!(inverse.expected, edit.replacement);
            assert_eq!(inverse.replacement, edit.expected);
        }
        (edits, undo)
    };
    assert_eq!(source.emitters[0].color_over_lifetime[0].t, 0.25);
    assert_eq!(source.emitters[0].color_over_lifetime[0].value, values);
    let committed = source.clone();
    let reopened = ParticleSystemSource::from_toml(&source.to_toml().unwrap()).unwrap();
    assert_eq!(reopened, committed);
    let runtime = reopened.to_cpu_asset().unwrap();
    assert_eq!(runtime.emitters[0].color_over_lifetime[0].t, 0.25);
    assert_eq!(runtime.emitters[0].color_over_lifetime[0].value.y, -0.5);
    assert_eq!(runtime.emitters[0].color_over_lifetime[0].value.x, 3.0);
    let redo = ParticleSourceCurveAdapter::new(&mut source, &selected)
        .unwrap()
        .apply(undo)
        .unwrap();
    assert_eq!(source, original);
    assert_eq!(redo, edits);
    ParticleSourceCurveAdapter::new(&mut source, &selected)
        .unwrap()
        .apply(redo)
        .unwrap();
    assert_eq!(source, committed);
}

#[test]
fn particle_curve_batch_checks_final_times_and_allows_two_keys_to_swap() {
    let mut source = ParticleSystemSource::from_toml(TEMPLATE).unwrap();
    let original = source.clone();
    let selected = selected_key(&source);
    {
        let mut model = ParticleSourceCurveAdapter::new(&mut source, &selected).unwrap();
        let before = model.curves();
        let mut edits = Vec::new();
        for channel in [
            ParticleCurveChannel::Red,
            ParticleCurveChannel::SizeMultiplier,
        ] {
            for index in 0..2 {
                let point = curve(&before, channel).keys[index].point;
                edits.push(edit_at(
                    &before,
                    channel,
                    index,
                    CurvePoint::new(1.0 - point.time, point.value),
                ));
            }
        }
        let undo = CurveModel::apply(&mut model, edits.clone()).unwrap();
        let after = model.curves();
        for channel in [
            ParticleCurveChannel::Red,
            ParticleCurveChannel::Green,
            ParticleCurveChannel::Blue,
            ParticleCurveChannel::Alpha,
            ParticleCurveChannel::SizeMultiplier,
        ] {
            let before = &curve(&before, channel).keys;
            let after = &curve(&after, channel).keys;
            assert_eq!(after[0].id, before[1].id);
            assert_eq!(after[1].id, before[0].id);
            assert_eq!(after[0].point.time, 0.0);
            assert_eq!(after[1].point.time, 1.0);
            assert_eq!(after[0].point.value, before[1].point.value);
            assert_eq!(after[1].point.value, before[0].point.value);
        }
        assert_eq!(model.apply(undo).unwrap(), edits);
    }
    assert_eq!(source, original);
}

#[test]
fn particle_curve_batch_late_rejection_preserves_every_source_field_and_key_order() {
    let mut source = ParticleSystemSource::from_toml(TEMPLATE).unwrap();
    let original = source.clone();
    let selected = selected_key(&source);
    let before = ParticleSourceCurveAdapter::new(&mut source, &selected)
        .unwrap()
        .curves();
    let red = edit_first(
        &before,
        ParticleCurveChannel::Red,
        CurvePoint::new(0.25, 3.0),
    );
    let size = edit_first(
        &before,
        ParticleCurveChannel::SizeMultiplier,
        CurvePoint::new(0.25, 0.8),
    );
    let mut stale = size.clone();
    stale.expected.value = 99.0;
    let mut missing = size.clone();
    missing.key_id = "deleted-size-key".into();
    let mut wrong_emitter = size.clone();
    wrong_emitter.curve_id.emitter_key = "different-document-emitter".into();
    let mut nonfinite = size.clone();
    nonfinite.replacement.time = f32::NAN;
    let mut negative_size = size.clone();
    negative_size.replacement.value = -1.0;
    let mut duplicate_time = size.clone();
    duplicate_time.replacement.time = 1.0;
    let conflicting_time = edit_first(
        &before,
        ParticleCurveChannel::Green,
        CurvePoint::new(0.5, 2.0),
    );
    for edits in [
        vec![red.clone(), stale],
        vec![red.clone(), missing],
        vec![red.clone(), wrong_emitter],
        vec![red.clone(), nonfinite],
        vec![red.clone(), negative_size],
        vec![red.clone(), duplicate_time],
        vec![red.clone(), conflicting_time],
        vec![red.clone(), size, red.clone()],
    ] {
        let mut model = ParticleSourceCurveAdapter::new(&mut source, &selected).unwrap();
        assert!(CurveModel::apply(&mut model, edits).is_err());
        assert_eq!(model.curves(), before);
        assert_eq!(source, original);
    }
}

#[test]
fn particle_curve_batch_order_does_not_change_grouped_components_or_other_emitters() {
    let mut original = ParticleSystemSource::from_toml(TEMPLATE).unwrap();
    let mut other = original.emitters[0].clone();
    other.key = None;
    other.id = "isolated-emitter".into();
    for key in &mut other.color_over_lifetime {
        key.key = None;
    }
    for key in &mut other.size_over_lifetime {
        key.key = None;
    }
    original.emitters.push(other);
    original.assign_missing_keys().unwrap();
    let mut forward = original.clone();
    let selected = selected_key(&original);
    let before = ParticleSourceCurveAdapter::new(&mut forward, &selected)
        .unwrap()
        .curves();
    let mut edits = vec![
        edit_first(
            &before,
            ParticleCurveChannel::Red,
            CurvePoint::new(0.25, 4.0),
        ),
        edit_at(
            &before,
            ParticleCurveChannel::Green,
            1,
            CurvePoint::new(0.75, 2.0),
        ),
        edit_first(
            &before,
            ParticleCurveChannel::Alpha,
            CurvePoint::new(0.25, 0.5),
        ),
    ];
    ParticleSourceCurveAdapter::new(&mut forward, &selected)
        .unwrap()
        .apply(edits.clone())
        .unwrap();
    edits.reverse();
    let mut reverse = original.clone();
    ParticleSourceCurveAdapter::new(&mut reverse, &selected)
        .unwrap()
        .apply(edits)
        .unwrap();
    assert_eq!(forward, reverse);
    assert_eq!(forward.system, original.system);
    assert_eq!(forward.emitters[1], original.emitters[1]);
    assert_eq!(
        forward.emitters[0].color_over_lifetime[0].value,
        [4.0, 0.72, 0.25, 0.5]
    );
    assert_eq!(
        forward.emitters[0].color_over_lifetime[1].value,
        [1.0, 2.0, 0.05, 0.0]
    );
}

#[test]
fn particle_curve_batch_empty_is_unchanged_and_a_stale_gesture_cannot_reapply() {
    let mut source = ParticleSystemSource::from_toml(TEMPLATE).unwrap();
    let original = source.clone();
    let selected = selected_key(&source);
    let (edits, undo) = {
        let mut model = ParticleSourceCurveAdapter::new(&mut source, &selected).unwrap();
        assert!(model.apply(Vec::new()).unwrap().is_empty());
        let before = model.curves();
        let edits = vec![edit_first(
            &before,
            ParticleCurveChannel::Blue,
            CurvePoint::new(0.25, 2.0),
        )];
        let undo = model.apply(edits.clone()).unwrap();
        (edits, undo)
    };
    let committed = source.clone();
    assert!(ParticleSourceCurveAdapter::new(&mut source, &selected)
        .unwrap()
        .apply(edits)
        .is_err());
    assert_eq!(source, committed);
    ParticleSourceCurveAdapter::new(&mut source, &selected)
        .unwrap()
        .apply(undo)
        .unwrap();
    assert_eq!(source, original);
}
