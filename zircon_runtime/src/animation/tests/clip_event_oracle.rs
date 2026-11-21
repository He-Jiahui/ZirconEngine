use crate::core::framework::animation::{
    AnimationClipEventSamplingBatch as Batch, AnimationEventTrackAsset,
};

use super::{
    clip_with_events, event_track, sample_clip_events_budgeted, AnimationClipAsset as Clip,
    AnimationClipEvent as Event, AnimationClipEventSamplingCursor as Cursor,
    AnimationClipEventSamplingLimits as Limits, Real,
};

const ENTITY: u64 = 17;
const POINT: Real = 0.01;
const PERIOD: Real = 0.02;
const DECIMAL_FORWARD: [u32; 5] = [
    0x3c23_d70a,
    0x3cf5_c28f,
    0x3d4c_cccc,
    0x3d8f_5c28,
    0x3db8_51ea,
];
const DECIMAL_REVERSE: [u32; 5] = [
    0x3db8_51eb,
    0x3d8f_5c28,
    0x3d4c_cccb,
    0x3cf5_c28c,
    0x3c23_d704,
];

#[derive(Debug, PartialEq, Eq)]
struct Key<'a> {
    playback: u32,
    phase: u32,
    entity: u64,
    name: &'a str,
    payload: Option<&'a str>,
    target: Option<&'a str>,
}

fn keys<'a>(events: impl IntoIterator<Item = &'a Event>) -> Vec<Key<'a>> {
    events
        .into_iter()
        .map(|e| Key {
            playback: e.playback_time_seconds.to_bits(),
            phase: e.clip_time_seconds.to_bits(),
            entity: e.entity,
            name: e.event.as_str(),
            payload: e.payload.as_deref(),
            target: e.target_id.as_deref(),
        })
        .collect()
}

fn anchor(
    playback: u32,
    phase: Real,
    name: &'static str,
    payload: Option<&'static str>,
) -> Key<'static> {
    Key {
        playback,
        phase: phase.to_bits(),
        entity: ENTITY,
        name,
        payload,
        target: None,
    }
}

// Diagnostic only: FNV-1a over little-endian numbers and length-prefixed UTF-8 fields.
fn fingerprint(events: &[Key<'_>]) -> u64 {
    fn feed(hash: &mut u64, bytes: impl IntoIterator<Item = u8>) {
        for byte in bytes {
            *hash = (*hash ^ u64::from(byte)).wrapping_mul(0x0000_0100_0000_01b3);
        }
    }
    let mut hash = 0xcbf2_9ce4_8422_2325;
    for e in events {
        feed(&mut hash, e.playback.to_le_bytes());
        feed(&mut hash, e.phase.to_le_bytes());
        feed(&mut hash, e.entity.to_le_bytes());
        for text in [Some(e.name), e.payload, e.target] {
            feed(
                &mut hash,
                text.map_or(u64::MAX, |s| s.len() as u64).to_le_bytes(),
            );
            if let Some(s) = text {
                feed(&mut hash, s.bytes());
            }
        }
    }
    hash
}

fn trace(label: &str, lane: &str, observed: &[Key<'_>], reference: &[Key<'_>]) {
    let bits: Vec<_> = observed.iter().map(|e| e.playback).collect();
    let first = observed
        .iter()
        .zip(reference)
        .position(|(a, b)| a != b)
        .or_else(|| {
            (observed.len() != reference.len()).then_some(observed.len().min(reference.len()))
        });
    eprintln!(
        "CLIP_ORACLE_V1 {label} observed-production/{lane} count={} hash={:016x} bits={bits:08x?} reference_count={} reference_hash={:016x} first_diff={first:?} actual={:x?} reference={:x?}",
        observed.len(), fingerprint(observed), reference.len(), fingerprint(reference),
        first.and_then(|i| observed.get(i)), first.and_then(|i| reference.get(i)),
    );
}

fn cursor_key(c: &Cursor) -> (u32, Option<&str>, usize) {
    (
        c.playback_time_seconds.to_bits(),
        c.last_event.as_deref(),
        c.last_track_index,
    )
}

struct Observation {
    label: &'static str,
    single: Batch,
    pages: Vec<Batch>,
    expected: Vec<Key<'static>>,
    layout: &'static [usize],
}

impl Observation {
    fn check(&self) {
        let label = self.label;
        let single = keys(&self.single.events);
        let paged = keys(self.pages.iter().flat_map(|p| &p.events));
        assert!(
            self.single.next_cursor.is_none() && !self.single.budget_exhausted,
            "{label}: nonbinding batch"
        );
        assert_eq!(
            single.len(),
            self.expected.len(),
            "{label}: independent count"
        );
        assert_eq!(single, self.expected, "{label}: independent exact anchor");
        assert!(
            self.pages.last().is_some_and(|p| p.next_cursor.is_none()),
            "{label}: did not finish the fixed range"
        );
        assert_eq!(paged, single, "{label}: ordered, bit-exact replay");
        let sizes: Vec<_> = self.pages.iter().map(|p| p.events.len()).collect();
        assert_eq!(sizes.as_slice(), self.layout, "{label}: page counts");
    }
}

fn observe(
    label: &'static str,
    clip: &Clip,
    [from, to]: [Real; 2],
    limits: Limits,
    expected: Vec<Key<'static>>,
    layout: &'static [usize],
) -> Observation {
    // None means fresh. Only production-returned cursors are supplied on later calls,
    // and the range, clip, direction and limits never change in the paged lane.
    let single = sample_clip_events_budgeted(
        clip,
        ENTITY,
        from,
        to,
        true,
        None,
        Limits {
            max_events: 32,
            max_event_bytes: usize::MAX,
            max_playback_span_seconds: Real::MAX,
        },
    );
    let mut cursor: Option<Cursor> = None;
    let mut pages = Vec::new();
    for page in 1..=16 {
        let batch =
            sample_clip_events_budgeted(clip, ENTITY, from, to, true, cursor.clone(), limits);
        let next = batch.next_cursor.as_ref().map(cursor_key);
        let input = cursor.as_ref().map(cursor_key);
        let bits: Vec<_> = batch
            .events
            .iter()
            .map(|e| e.playback_time_seconds.to_bits())
            .collect();
        eprintln!(
            "CLIP_ORACLE_V1 {label} observed-production/page={page} range={:08x}->{:08x} input={input:x?} next={next:x?} count={} bits={bits:08x?} span={:08x} exhausted={}",
            from.to_bits(), to.to_bits(), batch.events.len(), batch.playback_span_seconds.to_bits(), batch.budget_exhausted,
        );
        let done = next.is_none() || next == input;
        cursor = batch.next_cursor.clone();
        pages.push(batch);
        if done {
            break;
        }
    }
    let single_keys = keys(&single.events);
    let paged_keys = keys(pages.iter().flat_map(|p| &p.events));
    trace(label, "nonbinding vs fixed anchor", &single_keys, &expected);
    trace(label, "paged vs nonbinding", &paged_keys, &single_keys);
    trace(label, "paged vs fixed anchor", &paged_keys, &expected);
    Observation {
        label,
        single,
        pages,
        expected,
        layout,
    }
}

fn make_clip(period: Real, event_tracks: Vec<AnimationEventTrackAsset>) -> Clip {
    Clip {
        duration_seconds: period,
        ..clip_with_events(event_tracks)
    }
}

fn limits(max_events: usize, max_playback_span_seconds: Real) -> Limits {
    Limits {
        max_events,
        max_playback_span_seconds,
        ..Limits::default()
    }
}

fn pulse(phase: Real) -> AnimationEventTrackAsset {
    event_track("pulse", phase, Some("cue"))
}

fn pulse_anchors(phase: Real, bits: &[u32]) -> Vec<Key<'static>> {
    bits.iter()
        .map(|&b| anchor(b, phase, "pulse", Some("cue")))
        .collect()
}

#[test]
fn loop_notify_resume_decimal_one_shot_bits() {
    let clip = make_clip(PERIOD, vec![pulse(POINT)]);
    let forward = observe(
        "decimal-count-forward",
        &clip,
        [0.0, 0.09375],
        limits(4, 1.0),
        pulse_anchors(POINT, &DECIMAL_FORWARD),
        &[4, 1],
    );
    // In reverse, the legacy resumed page has the right count but a different final bit.
    let reverse = observe(
        "decimal-count-reverse",
        &clip,
        [0.09375, 0.0],
        limits(4, 1.0),
        pulse_anchors(POINT, &DECIMAL_REVERSE),
        &[4, 1],
    );
    reverse.check();
    forward.check();
}

#[test]
fn loop_notify_resume_advancing_empty_span_retains_origin() {
    let clip = make_clip(PERIOD, vec![pulse(POINT)]);
    let forward = observe(
        "decimal-span-forward",
        &clip,
        [0.0, 0.09375],
        limits(8, 1.0 / 64.0),
        pulse_anchors(POINT, &DECIMAL_FORWARD),
        &[1, 1, 0, 1, 1, 1],
    );
    let reverse = observe(
        "decimal-span-reverse",
        &clip,
        [0.09375, 0.0],
        limits(8, 1.0 / 64.0),
        pulse_anchors(POINT, &DECIMAL_REVERSE),
        &[1, 1, 1, 0, 1, 1],
    );
    for (observation, ends) in [(&forward, [1, 2, 3, 4, 5]), (&reverse, [5, 4, 3, 2, 1])] {
        observation.check();
        for (page, n) in observation.pages.iter().zip(ends) {
            assert_eq!(
                page.next_cursor.as_ref().map(cursor_key),
                Some(((n as Real / 64.0).to_bits(), None, 0))
            );
            assert!(!page.budget_exhausted);
        }
    }
}

#[test]
fn loop_notify_resume_distinguishes_tie_slots_and_ulp_neighbors() {
    let tracks = [
        ("m", POINT, None),
        ("a", POINT, Some("z-slot")),
        ("a", POINT, Some("a-slot")),
        ("z-edge", Real::from_bits(0x3c23_d709), None),
        ("y-edge", Real::from_bits(0x3c23_d70b), None),
    ];
    let clip = make_clip(
        PERIOD,
        tracks
            .map(|(name, time, payload)| event_track(name, time, payload))
            .into(),
    );
    // At the first crossing, the adjacent f32 times must not join the name/index tie group.
    let forward = [
        (3, 0x3c23_d709),
        (1, 0x3c23_d70a),
        (2, 0x3c23_d70a),
        (0, 0x3c23_d70a),
        (4, 0x3c23_d70b),
    ];
    let reverse = [
        (4, 0x3c23_d70b),
        (1, 0x3c23_d70a),
        (2, 0x3c23_d70a),
        (0, 0x3c23_d70a),
        (3, 0x3c23_d709),
    ];
    // At the later crossing, the three phases really round to the *same* playback bit.
    let rounded = [1, 2, 0, 4, 3].map(|i| (i, 0x3d8f_5c29));
    let cases = [
        ("identity-forward", [0.0, 0.015625], forward),
        ("identity-reverse", [0.015625, 0.0], reverse),
        ("identity-rounded-tie", [0.0625, 0.078125], rounded),
    ];
    let observations = cases.map(|(label, range, order)| {
        let expected = order
            .into_iter()
            .map(|(index, bits)| {
                let (name, phase, payload) = tracks[index];
                anchor(bits, phase, name, payload)
            })
            .collect();
        observe(label, &clip, range, limits(1, 1.0), expected, &[1; 5])
    });
    for ((_, _, order), observation) in cases.into_iter().zip(observations) {
        observation.check();
        for (page, (index, bits)) in observation.pages.iter().take(4).zip(order) {
            assert_eq!(
                page.next_cursor.as_ref().map(cursor_key),
                Some((bits, Some(tracks[index].0), index))
            );
        }
    }
}

#[test]
fn loop_notify_resume_same_track_crosses_binary_exponents() {
    const L: Real = 4_194_304.0;
    const PHASE: Real = 0.125;
    let edge = make_clip(0.6875, vec![pulse(PHASE)]);
    let forward = observe(
        "binade-forward",
        &edge,
        [L - 2.0, L + 1.5],
        limits(2, 4.0),
        pulse_anchors(
            PHASE,
            &[
                0x4a7f_fffb,
                0x4a7f_fffe,
                0x4a80_0000,
                0x4a80_0001,
                0x4a80_0002,
                0x4a80_0003,
            ],
        ),
        &[2, 2, 2],
    );
    // The first reverse seed is reachable from L+2: floor gives 6_100_808;
    // n*d is L+1.5 exactly and adding the phase rounds back to L+1.5.
    let reverse = observe(
        "binade-reverse",
        &edge,
        [L + 2.0, L - 1.0],
        limits(3, 4.0),
        pulse_anchors(
            PHASE,
            &[
                0x4a80_0003,
                0x4a80_0002,
                0x4a80_0001,
                0x4a7f_ffff,
                0x4a7f_fffc,
            ],
        ),
        &[3, 2],
    );
    let small = make_clip(0.75, vec![pulse(PHASE)]);
    // This single track actually straddles 1, 2 and 4 in each traversal.
    let bits = [
        0x3f60_0000,
        0x3fd0_0000,
        0x4018_0000,
        0x4048_0000,
        0x4078_0000,
        0x4094_0000,
    ];
    let spanning_forward = observe(
        "several-binades-forward",
        &small,
        [0.5, 5.0],
        limits(1, 8.0),
        pulse_anchors(PHASE, &bits),
        &[1; 6],
    );
    let mut reverse_bits = bits;
    reverse_bits.reverse();
    let spanning_reverse = observe(
        "several-binades-reverse",
        &small,
        [5.0, 0.5],
        limits(1, 8.0),
        pulse_anchors(PHASE, &reverse_bits),
        &[1; 6],
    );
    for observation in [forward, spanning_forward, spanning_reverse, reverse] {
        observation.check();
    }
}

#[test]
fn loop_notify_resume_stopped_track_stays_stopped() {
    const L: Real = 4_194_304.0;
    const PHASE: Real = 0.125;
    let clip = make_clip(0.25, vec![pulse(PHASE)]);
    // d is half the upper binade's spacing. An even point cannot advance, but
    // the half-second span itself advances and visits odd cursors on later pages.
    let forward = observe(
        "rounded-stop-forward",
        &clip,
        [L - 0.5, L + 2.0],
        limits(4, 0.5),
        pulse_anchors(PHASE, &[0x4a7f_ffff, 0x4a80_0000]),
        &[2, 0, 0, 0, 0],
    );
    let reverse = observe(
        "rounded-stop-reverse",
        &clip,
        [L + 2.5, L + 0.5],
        limits(4, 0.5),
        pulse_anchors(PHASE, &[0x4a80_0004]),
        &[1, 0, 0, 0],
    );
    forward.check();
    reverse.check();
}

#[test]
fn loop_notify_resume_overflowed_first_seed_stays_absent() {
    const PHASE: Real = 0.125;
    let clip = make_clip(0.6875, vec![pulse(PHASE)]);
    let from = Real::from_bits(0x7f40_0000);
    let edge = Real::from_bits(0x7ec0_0001);
    let end = Real::from_bits(0x7ec0_0000);
    // At the original start the exact quotient is (24/11)*2^127, which overflows.
    // The second page's cursor is reachable from the first, empty page.
    let overflow = observe(
        "seed-overflow-reverse",
        &clip,
        [from, end],
        limits(4, Real::from_bits(0x7ebf_ffff)),
        vec![],
        &[0, 0],
    );
    // New fresh call, not a continuation of the wide range: its seed is finite.
    let tail = observe(
        "seed-finite-fresh-control",
        &clip,
        [edge, end],
        limits(4, Real::MAX),
        pulse_anchors(PHASE, &[0x7ec0_0000]),
        &[1],
    );
    // Just above the admitted duration threshold, with a strictly interior phase.
    let tiny = make_clip(
        Real::from_bits(0x3400_0001),
        vec![pulse(Real::from_bits(0x3380_0001))],
    );
    let forward = observe(
        "seed-overflow-forward",
        &tiny,
        [Real::from_bits(0x7f7f_fffe), Real::MAX],
        limits(4, Real::MAX),
        vec![],
        &[0],
    );
    tail.check();
    forward.check();
    overflow.check();
    assert_eq!(
        overflow.pages[0].next_cursor.as_ref().map(cursor_key),
        Some((edge.to_bits(), None, 0))
    );
}
