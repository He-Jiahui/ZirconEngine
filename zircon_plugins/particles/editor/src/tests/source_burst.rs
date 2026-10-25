use super::{ParticleSystemSource, TEMPLATE};
use crate::{
    parse_particle_burst_count, ParticleSourceBurst, ParticleSourceBurstEdit,
    ParticleSourceBurstValue,
};
use zircon_plugin_particles_runtime::{ParticleSystemComponent, ParticlesManager};
use zircon_runtime::core::math::Real;

fn source_with_bursts() -> ParticleSystemSource {
    let mut source = ParticleSystemSource::from_toml(TEMPLATE).unwrap();
    source.emitters[0].spawn_rate_per_second = 0.0;
    // Preserve authored order even when chronological order differs; runtime sorts its own asset.
    source.emitters[0].bursts = vec![
        ParticleSourceBurst {
            key: Some("late-burst".into()),
            time_seconds: 0.75,
            count: 2,
        },
        ParticleSourceBurst {
            key: Some("early-burst".into()),
            time_seconds: 0.25,
            count: 3,
        },
    ];
    source.validate().unwrap();
    source
}

fn edit(source: &ParticleSystemSource) -> ParticleSourceBurstEdit {
    ParticleSourceBurstEdit {
        emitter_key: source.emitters[0].key.clone().unwrap(),
        burst_key: "early-burst".into(),
        expected: ParticleSourceBurstValue {
            time_seconds: 0.25,
            count: 3,
        },
        replacement: ParticleSourceBurstValue {
            time_seconds: 0.5,
            count: 7,
        },
    }
}

fn live_after(source: &ParticleSystemSource, ticks: usize) -> usize {
    let manager = ParticlesManager::default();
    manager
        .instantiate(ParticleSystemComponent::new(
            1,
            source.to_cpu_asset().unwrap(),
        ))
        .unwrap();
    for _ in 0..ticks {
        manager.tick(0.25).unwrap();
    }
    manager.snapshot().emitters[0].live_particles as usize
}

#[test]
fn particle_burst_edit_changes_real_cpu_occurrence_and_inverse_restores_source() {
    let mut source = source_with_bursts();
    let original = source.clone();
    assert_eq!(live_after(&source, 1), 3);
    let forward = edit(&source);
    assert!(!forward.is_noop());
    source.validate_burst_edit(&forward).unwrap();
    assert_eq!(source, original);
    let inverse = source.apply_burst_edit(forward.clone()).unwrap();
    assert_eq!(live_after(&source, 1), 0);
    assert_eq!(live_after(&source, 2), 7);
    assert_eq!(live_after(&source, 3), 9);
    assert_eq!(source.emitters[0].bursts[0], original.emitters[0].bursts[0]);
    assert_eq!(source.emitters[0].bursts[1].key, Some("early-burst".into()));
    assert_eq!(source.apply_burst_edit(inverse).unwrap(), forward);
    assert_eq!(source, original);
    assert_eq!(live_after(&source, 1), 3);
}

#[test]
fn particle_burst_edit_round_trip_keeps_equal_times_order_and_full_u32_count() {
    assert_eq!(parse_particle_burst_count("16777217").unwrap(), 16_777_217);
    assert_eq!(parse_particle_burst_count("4294967295").unwrap(), u32::MAX);
    assert_eq!(parse_particle_burst_count("0").unwrap(), 0);
    for invalid in ["", "-1", "+1", " 1", "1 ", "1.0", "1e3", "4294967296"] {
        assert!(parse_particle_burst_count(invalid).is_err());
    }
    let mut source = source_with_bursts();
    let original = source.clone();
    let mut forward = edit(&source);
    forward.replacement = ParticleSourceBurstValue {
        time_seconds: 0.75,
        count: u32::MAX,
    };
    let inverse = source.apply_burst_edit(forward.clone()).unwrap();
    let encoded = source.to_toml().unwrap();
    assert!(encoded.contains("4294967295"));
    let mut reopened = ParticleSystemSource::from_toml(&encoded).unwrap();
    assert_eq!(reopened, source);
    let asset = reopened.to_cpu_asset().unwrap();
    assert_eq!(asset.emitters[0].bursts[0].count, 2);
    assert_eq!(asset.emitters[0].bursts[1].count, u32::MAX);
    assert_eq!(reopened.apply_burst_edit(inverse).unwrap(), forward);
    assert_eq!(reopened, original);
}

#[test]
fn particle_burst_edit_rejects_invalid_stale_missing_and_cross_emitter_without_writes() {
    let source = source_with_bursts();
    let forward = edit(&source);
    let mut invalid = Vec::new();
    for time_seconds in [Real::NAN, Real::INFINITY, Real::NEG_INFINITY, -0.25] {
        let mut value = forward.clone();
        value.replacement.time_seconds = time_seconds;
        invalid.push(value);
    }
    let mut stale = forward.clone();
    stale.expected.count += 1;
    invalid.push(stale);
    let mut stale = forward.clone();
    stale.expected.time_seconds += 0.25;
    invalid.push(stale);
    let mut missing = forward.clone();
    missing.burst_key = "deleted-burst".into();
    invalid.push(missing);
    let mut wrong_emitter = forward;
    wrong_emitter.emitter_key = "other-emitter".into();
    invalid.push(wrong_emitter);
    for invalid in invalid {
        let mut current = source.clone();
        assert!(current.validate_burst_edit(&invalid).is_err());
        assert!(current.apply_burst_edit(invalid).is_err());
        assert_eq!(current, source);
    }
    let mut invalid_source = source;
    invalid_source.emitters[0].bursts[0].key = Some("early-burst".into());
    let before = invalid_source.clone();
    assert!(invalid_source.apply_burst_edit(edit(&before)).is_err());
    assert_eq!(invalid_source, before);
}

#[test]
fn particle_burst_edit_follows_stable_key_across_reorder_and_rechecks_after_preflight() {
    let mut source = source_with_bursts();
    let mut other = crate::ParticleSourceEmitter::default();
    other.id = "other-emitter".into();
    source.emitters.push(other);
    source.assign_missing_keys().unwrap();
    let forward = edit(&source);
    source.validate_burst_edit(&forward).unwrap();
    source.emitters[0].bursts.swap(0, 1);
    source.emitters[0].display_name = Some("粒子发射器／很长的重命名标签".into());
    let before = source.clone();
    let inverse = source.apply_burst_edit(forward.clone()).unwrap();
    assert_eq!(source.emitters[0].bursts[0].count, 7);
    assert_eq!(source.emitters[0].bursts[1], before.emitters[0].bursts[1]);
    assert_eq!(source.emitters[1], before.emitters[1]);
    assert!(source.apply_burst_edit(forward.clone()).is_err());
    source.apply_burst_edit(inverse).unwrap();
    assert_eq!(source, before);
    source.emitters[0].bursts.remove(0);
    let deleted = source.clone();
    assert!(source.apply_burst_edit(forward).is_err());
    assert_eq!(source, deleted);
}

#[test]
fn particle_burst_edit_zero_time_count_and_noop_keep_identity() {
    let mut source = source_with_bursts();
    let original = source.clone();
    let mut forward = edit(&source);
    forward.replacement = ParticleSourceBurstValue {
        time_seconds: 0.0,
        count: 0,
    };
    let inverse = source.apply_burst_edit(forward).unwrap();
    assert_eq!(live_after(&source, 1), 0);
    assert_eq!(source.emitters[0].bursts.len(), 2);
    let zero = ParticleSourceBurstEdit {
        expected: inverse.expected,
        replacement: inverse.expected,
        ..inverse.clone()
    };
    assert!(zero.is_noop());
    let before_noop = source.clone();
    assert_eq!(source.apply_burst_edit(zero.clone()).unwrap(), zero);
    assert_eq!(source, before_noop);
    source.apply_burst_edit(inverse).unwrap();
    assert_eq!(source, original);
}

#[test]
fn particle_burst_typed_capture_uses_stable_key_and_preserves_complete_expected_pair() {
    let mut source = source_with_bursts();
    source.emitters[0].bursts[0].time_seconds = 0.25;
    let original = source.clone();
    let emitter = source.emitters[0].key.as_deref().unwrap();
    let mut captured = source.capture_burst_edit(emitter, "early-burst").unwrap();
    assert_eq!(captured.emitter_key, emitter);
    assert_eq!(captured.burst_key, "early-burst");
    assert_eq!(captured.expected.time_seconds, 0.25);
    assert_eq!(captured.expected.count, 3);
    assert!(captured.is_noop());
    captured
        .set_replacement_from_text("5e-1", "16777217")
        .unwrap();
    assert_eq!(captured.replacement.time_seconds, 0.5);
    assert_eq!(captured.replacement.count, 16_777_217);
    captured
        .set_replacement_from_text("0.5", "4294967295")
        .unwrap();
    assert_eq!(captured.replacement.count, u32::MAX);
    assert_eq!(captured.expected.time_seconds, 0.25);
    assert_eq!(captured.expected.count, 3);
    assert_eq!(source, original);
}

#[test]
fn particle_burst_typed_invalid_pair_keeps_prior_delta_and_nonzero_time_cannot_underflow() {
    let source = source_with_bursts();
    let emitter = source.emitters[0].key.as_deref().unwrap();
    let mut captured = source.capture_burst_edit(emitter, "early-burst").unwrap();
    captured.set_replacement_from_text("0.5", "7").unwrap();
    let before = captured.clone();
    for (time, count) in [
        ("", "7"),
        ("NaN", "7"),
        ("inf", "7"),
        ("-0.25", "7"),
        ("1e1000", "7"),
        ("1e-1000", "7"),
        ("-1e-1000", "7"),
        ("0.75", ""),
        ("0.75", "+7"),
        ("0.75", "7.0"),
        ("0.75", "1e3"),
        ("0.75", " 7"),
        ("0.75", "4294967296"),
    ] {
        assert!(captured.set_replacement_from_text(time, count).is_err());
        assert_eq!(captured, before);
    }
    captured.set_replacement_from_text("0e-1000", "0").unwrap();
    assert_eq!(captured.replacement.time_seconds, 0.0);
    assert_eq!(captured.replacement.count, 0);
    let smallest = Real::from_bits(1);
    captured
        .set_replacement_from_text(&smallest.to_string(), "7")
        .unwrap();
    assert_eq!(captured.replacement.time_seconds, smallest);
    assert_eq!(captured.expected, before.expected);
}

#[test]
fn particle_burst_typed_pending_draft_does_not_rebase_expected_onto_a_new_source_value() {
    let mut source = source_with_bursts();
    let emitter = source.emitters[0].key.clone().unwrap();
    let mut captured = source.capture_burst_edit(&emitter, "early-burst").unwrap();
    let concurrent = edit(&source);
    source.apply_burst_edit(concurrent).unwrap();
    let changed = source.clone();
    // The proposal now equals the actual source, but its original expected pair remains stale.
    captured.set_replacement_from_text("0.5", "7").unwrap();
    assert_eq!(captured.expected.time_seconds, 0.25);
    assert_eq!(captured.expected.count, 3);
    assert!(source.validate_burst_edit(&captured).is_err());
    assert!(source.apply_burst_edit(captured).is_err());
    assert_eq!(source, changed);
}

#[test]
fn particle_burst_typed_capture_rejects_missing_identities_and_noop_still_needs_preflight() {
    let mut source = source_with_bursts();
    let original = source.clone();
    let emitter = source.emitters[0].key.clone().unwrap();
    assert!(source
        .capture_burst_edit("missing-emitter", "early-burst")
        .is_err());
    assert!(source
        .capture_burst_edit(&emitter, "deleted-burst")
        .is_err());
    assert_eq!(source, original);
    let captured = source.capture_burst_edit(&emitter, "early-burst").unwrap();
    assert!(captured.is_noop());
    source.validate_burst_edit(&captured).unwrap();
    let concurrent = edit(&source);
    source.apply_burst_edit(concurrent).unwrap();
    let changed = source.clone();
    assert!(captured.is_noop());
    assert!(source.validate_burst_edit(&captured).is_err());
    assert_eq!(source, changed);
    source.emitters[0].bursts[1].key = None;
    let missing_identity = source.clone();
    assert!(source.capture_burst_edit(&emitter, "early-burst").is_err());
    assert_eq!(source, missing_identity);
}

#[test]
fn particle_burst_typed_pair_round_trip_inverse_and_redo_use_existing_source_and_cpu_path() {
    let mut source = source_with_bursts();
    let original = source.clone();
    let original_toml = source.to_toml().unwrap();
    let emitter = source.emitters[0].key.clone().unwrap();
    let mut captured = source.capture_burst_edit(&emitter, "early-burst").unwrap();
    captured.set_replacement_from_text("0.5", "7").unwrap();
    source.validate_burst_edit(&captured).unwrap();
    assert!(!captured.is_noop());
    let inverse = source.apply_burst_edit(captured.clone()).unwrap();
    assert_eq!(live_after(&source, 1), 0);
    assert_eq!(live_after(&source, 2), 7);
    let committed = source.clone();
    let encoded = source.to_toml().unwrap();
    let mut reopened = ParticleSystemSource::from_toml(&encoded).unwrap();
    assert_eq!(reopened, committed);
    let redo = reopened.apply_burst_edit(inverse).unwrap();
    assert_eq!(redo, captured);
    assert_eq!(reopened, original);
    assert_eq!(reopened.to_toml().unwrap(), original_toml);
    assert_eq!(live_after(&reopened, 1), 3);
    reopened.apply_burst_edit(redo).unwrap();
    assert_eq!(reopened, committed);
    assert_eq!(live_after(&reopened, 2), 7);
}
