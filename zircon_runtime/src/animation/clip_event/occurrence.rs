use crate::core::math::Real;

const SIGN_BIT: u32 = 1 << 31;
const FRACTION_MASK: u32 = (1 << 23) - 1;

// Both the heap and the locator use the same native rounding and stop rule.
#[inline]
pub(super) fn step(time: Real, duration: Real, reverse: bool) -> Option<Real> {
    #[cfg(test)]
    STEP_PROBES.with(|probes| probes.set(probes.get() + 1));

    let next = if reverse {
        time - duration
    } else {
        time + duration
    };
    let advances = if reverse { next < time } else { next > time };
    (next.is_finite() && advances).then_some(next)
}

/// Locate the first nonnegative occurrence at or after the cursor in playback order.
/// The seed is already finite and in the original range.
pub(super) fn seek(mut time: Real, duration: Real, cursor: Real, reverse: bool) -> Option<Real> {
    if !cursor.is_finite() {
        return None;
    }
    let goal = playback_key(cursor, reverse);
    loop {
        let key = playback_key(time, reverse);
        if key >= goal {
            return Some(time);
        }

        let next = step(time, duration, reverse)?;
        if next < 0.0 {
            return None; // A descending track cannot reenter the range.
        }
        let next_key = playback_key(next, reverse);
        if next_key >= goal {
            return Some(next);
        }

        // Strict native progress makes both key differences positive. Only
        // extrapolate when both probes stay in this binade's strict interior.
        let stride = next_key - key;
        let room = interior_room(time, reverse);
        if stride <= room {
            let second_key = playback_key(step(next, duration, reverse)?, reverse);
            if second_key - key <= room && second_key - next_key == stride {
                // In the uniform interior, a fixed duration rounds to a fixed
                // stride, except for the first adjustment to even on a tie.
                // Two equal strides establish the steady stride here.
                let steps = (goal - key).div_ceil(stride).min(room / stride);
                let offset = steps * stride; // At most room, so bits stay in-band.
                let bits = time.to_bits();
                time = Real::from_bits(if reverse {
                    bits - offset
                } else {
                    bits + offset
                });
                continue;
            }
        }
        time = next;
    }
}

// Unsigned encoding of total_cmp order, increasing in the playback direction.
fn playback_key(time: Real, reverse: bool) -> u32 {
    let bits = time.to_bits();
    let key = if bits & SIGN_BIT == 0 {
        bits ^ SIGN_BIT
    } else {
        !bits
    };
    if reverse {
        !key
    } else {
        key
    }
}

// Leave both ends of each exponent band to native arithmetic. The rounding
// cell at a band boundary can include a neighbor with a different spacing.
fn interior_room(time: Real, reverse: bool) -> u32 {
    let bits = time.to_bits();
    let fraction = bits & FRACTION_MASK;
    if bits & SIGN_BIT != 0 || fraction == 0 || fraction == FRACTION_MASK {
        return 0;
    }
    if reverse {
        fraction - 1
    } else {
        FRACTION_MASK - 1 - fraction
    }
}

#[cfg(test)]
thread_local! {
    static STEP_PROBES: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

#[cfg(test)]
#[path = "occurrence/tests.rs"]
mod tests;
