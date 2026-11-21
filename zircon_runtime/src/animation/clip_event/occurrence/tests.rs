use crate::core::math::Real;

use super::{seek, step, STEP_PROBES};

#[test]
fn loop_occurrence_locator_bounds_native_steps() {
    let far: Real = 16_777_216.0;
    for (seed, cursor, reverse, expected) in [(1.0, far, false, far), (far, 0.0, true, 0.0)] {
        STEP_PROBES.with(|probes| probes.set(0));
        let found = seek(seed, 1.0, cursor, reverse);
        let probes = STEP_PROBES.with(|probes| probes.get());
        eprintln!(
            "LOOP_OCCURRENCE_LOCATOR_STEPS_V1 seed={:08x} cursor={:08x} reverse={reverse} steps={probes}",
            seed.to_bits(),
            cursor.to_bits(),
        );
        assert_eq!(found.map(Real::to_bits), Some(expected.to_bits()));
        // Either query skips over fifteen million prior occurrences.
        assert!(
            (1..=192).contains(&probes),
            "{probes} native steps (reverse={reverse})",
        );
    }
}

#[test]
fn loop_occurrence_locator_does_not_jump_to_a_binade_boundary() {
    let seed = Real::from_bits(0x4a80_0003);
    let boundary = Real::from_bits(0x4a80_0000);
    // The first two steps are one ULP each, but the third skips the boundary.
    assert_eq!(
        seek(seed, 0.6875, boundary, true).map(Real::to_bits),
        Some(0x4a7f_ffff),
    );
}

#[test]
fn loop_occurrence_locator_keeps_rounding_stalls_and_float_edges() {
    let odd = Real::from_bits(0x4a80_0001);
    let even = Real::from_bits(0x4a80_0002);
    let beyond = Real::from_bits(0x4a80_0003);
    assert_eq!(seek(odd, 0.25, even, false), Some(even));
    assert_eq!(seek(odd, 0.25, beyond, false), None);
    assert_eq!(
        seek(odd, 0.75, beyond, false).map(Real::to_bits),
        Some(0x4a80_0004),
    );

    let tiny = Real::from_bits(1);
    assert_eq!(seek(-0.0, tiny, 0.0, false).map(Real::to_bits), Some(1));
    assert_eq!(seek(tiny, tiny, 0.0, true).map(Real::to_bits), Some(0));
    assert_eq!(seek(tiny, tiny, -0.0, true), None);
    assert_eq!(
        seek(Real::from_bits(8), tiny, Real::from_bits(3), true).map(Real::to_bits),
        Some(3),
    );

    let before_max = Real::from_bits(Real::MAX.to_bits() - 1);
    let ulp = Real::from_bits(0x7380_0000);
    assert_eq!(seek(before_max, ulp, Real::MAX, false), Some(Real::MAX));
    assert_eq!(step(Real::MAX, ulp, false), None);
}
