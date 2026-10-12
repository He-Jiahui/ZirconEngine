use std::hint::black_box;

use super::BuiltinRenderFeature;

const LOOKUP_ROUNDS: usize = 65_536;

#[test]
fn optimization_batch_20260830dm_authoring_name_lookup_round_trips_every_feature() {
    for feature in BuiltinRenderFeature::ALL {
        assert_eq!(
            BuiltinRenderFeature::from_authoring_name(feature.authoring_name()),
            Some(*feature)
        );
    }
    assert_eq!(BuiltinRenderFeature::from_authoring_name(""), None);
    assert_eq!(
        BuiltinRenderFeature::from_authoring_name("UnknownFeature"),
        None
    );
}

#[test]
#[ignore = "deterministic candidate-check model for the managed optimization batch"]
fn optimization_batch_20260830dm_authoring_name_lookup_evidence() {
    let mut legacy_candidate_checks = 0_u64;
    let mut direct_lookup_calls = 0_u64;

    for round in 0..LOOKUP_ROUNDS {
        let feature = BuiltinRenderFeature::ALL[round % BuiltinRenderFeature::ALL.len()];
        let name = black_box(feature.authoring_name());
        let legacy = BuiltinRenderFeature::ALL.iter().copied().find(|candidate| {
            legacy_candidate_checks += 1;
            candidate.authoring_name() == name
        });
        direct_lookup_calls += 1;
        let optimized = BuiltinRenderFeature::from_authoring_name(name);
        assert_eq!(optimized, legacy);
    }

    let reduction_basis_points = legacy_candidate_checks
        .saturating_sub(direct_lookup_calls)
        .saturating_mul(10_000)
        / legacy_candidate_checks;
    println!(
        "RUNTIME524_BUILTIN_FEATURE_NAME_LOOKUP_BENCH_V1 rounds={LOOKUP_ROUNDS} legacy_candidate_checks={legacy_candidate_checks} direct_lookup_calls={direct_lookup_calls} candidate_check_reduction_basis_points_model={reduction_basis_points}"
    );
    assert!(direct_lookup_calls < legacy_candidate_checks);
}
