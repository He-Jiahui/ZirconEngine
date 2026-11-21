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

fn clip(duration_seconds: Real, tracks: Vec<AnimationEventTrackAsset>) -> Clip {
    Clip {
        duration_seconds,
        ..clip_with_events(tracks)
    }
}

fn key(c: &Cursor) -> (u32, Option<&str>, usize) {
    let bits = c.playback_time_seconds.to_bits();
    (bits, c.last_event.as_deref(), c.last_track_index)
}

fn event_key(e: &Event) -> (u32, &str, Option<&str>) {
    let bits = e.playback_time_seconds.to_bits();
    (bits, e.event.as_str(), e.payload.as_deref())
}

fn pages(case: &str, clip: &Clip, [from, to]: [Real; 2], limits: Limits, cap: usize) -> Vec<Batch> {
    let start = Cursor::at_range_start(from);
    let mut batches = Vec::new();
    let mut cursor = None;
    for page in 1..=cap {
        let batch =
            sample_clip_events_budgeted(clip, ENTITY, from, to, true, cursor.clone(), limits);
        let old = cursor.as_ref().unwrap_or(&start);
        let next = batch.next_cursor.as_ref();
        let advances = next.is_none_or(|n| {
            let dt = n.playback_time_seconds - old.playback_time_seconds;
            let directed = if from < to { dt } else { -dt };
            let (_, a, i) = key(old);
            let (_, b, j) = key(n);
            // At equal time a span cursor (no name) is after the whole tie group.
            directed > 0.0 || dt == 0.0 && a.is_some() && (b.is_none() || (b, j) > (a, i))
        });
        let events: Vec<_> = batch.events.iter().map(event_key).collect();
        eprintln!("CLIP_PAGES_V1 {case} {from}->{to} page={page}/{cap} in={:x?} len={} events={events:x?} bytes={} oversized={} span={} exhausted={} next={:x?} advances={advances}",
            key(old), events.len(), batch.emitted_event_bytes, batch.oversized_event_count,
            batch.playback_span_seconds, batch.budget_exhausted, next.map(key));
        cursor = batch.next_cursor.clone();
        batches.push(batch);
        if !advances || cursor.is_none() {
            break;
        }
    }
    batches
}

fn assert_pages(batches: &[Batch], expected: &[usize]) {
    let pending = batches.last().and_then(|b| b.next_cursor.as_ref());
    assert!(
        pending.is_none(),
        "cursor stalled or page limit reached: {pending:?}"
    );
    let lengths: Vec<_> = batches.iter().map(|p| p.events.len()).collect();
    assert_eq!(lengths, expected);
}

fn events(batches: &[Batch]) -> impl Iterator<Item = &Event> {
    batches.iter().flat_map(|page| &page.events)
}

fn one_at_a_time() -> Limits {
    Limits {
        max_events: 1,
        ..Limits::default()
    }
}

#[test]
fn loop_notify_resume_equal_time_name_then_track_index() {
    let cases = [
        ([0.0625, 0.078125], POINT, 0x3d8f_5c29),
        ([0.0625, 0.046875], POINT, 0x3d4c_cccc),
        // The reverse inverse at the represented crossing is just below 6, not exactly 6.
        ([0.140625, 0.125], Real::from_bits(0x3c23_d70c), 0x3e05_1eb8),
    ];
    let observations = cases.map(|(range, point, _)| {
        let tracks = [("z", None), ("a", Some("z-slot")), ("a", Some("a-slot"))];
        let clip = clip(
            PERIOD,
            tracks
                .map(|(name, payload)| event_track(name, point, payload))
                .into(),
        );
        pages("equal-time", &clip, range, one_at_a_time(), 4)
    });
    for ((_, point, nominal), batches) in cases.into_iter().zip(observations) {
        assert_pages(&batches, &[1, 1, 1]);
        let bits = batches[0].events[0].playback_time_seconds.to_bits();
        assert!(
            bits.abs_diff(nominal) <= 1,
            "{bits:08x} is not near {nominal:08x}"
        );
        let expected = [("a", Some("z-slot")), ("a", Some("a-slot")), ("z", None)];
        for (e, (name, payload)) in events(&batches).zip(expected) {
            assert_eq!(e.clip_time_seconds, point);
            assert_eq!(event_key(e), (bits, name, payload));
        }
        for (page, index) in batches.iter().take(2).zip([1, 2]) {
            let cursor = page.next_cursor.as_ref().map(key);
            assert_eq!(cursor, Some((bits, Some("a"), index)));
        }
    }
}

#[test]
fn loop_notify_resume_adjacent_times_are_not_ties() {
    let adjacent = Real::from_bits(POINT.to_bits() + 1);
    let cases = [
        ([0.0, 0.015625], [POINT, adjacent]),
        ([0.015625, 0.0], [adjacent, POINT]),
    ];
    let observations = cases.map(|(range, [first, second])| {
        // Put name order opposite to playback order in both directions.
        let tracks = [("a", second), ("z", first)].map(|(n, t)| event_track(n, t, None));
        pages(
            "adjacent-f32",
            &clip(PERIOD, tracks.into()),
            range,
            one_at_a_time(),
            3,
        )
    });
    for ((_, times), batches) in cases.into_iter().zip(observations) {
        assert_pages(&batches, &[1, 1]);
        for (e, (time, name)) in events(&batches).zip(times.into_iter().zip(["z", "a"])) {
            assert_eq!(e.clip_time_seconds.to_bits(), time.to_bits());
            assert_eq!(event_key(e), (time.to_bits(), name, None));
        }
    }
}

#[test]
fn loop_notify_resume_default_cap_splits_a_tie_group() {
    use std::hash::{DefaultHasher, Hash, Hasher};

    const TRACKS: usize = 20;
    type ExactKey<'a> = (u64, u32, u32, &'a str, Option<&'a str>, Option<&'a str>);

    fn exact_key(e: &Event) -> ExactKey<'_> {
        let (playback, name, payload) = event_key(e);
        (
            e.entity,
            e.clip_time_seconds.to_bits(),
            playback,
            name,
            payload,
            e.target_id.as_deref(),
        )
    }

    fn first_difference(a: &[ExactKey<'_>], b: &[ExactKey<'_>]) -> Option<usize> {
        a.iter()
            .zip(b)
            .position(|(a, b)| a != b)
            .or_else(|| (a.len() != b.len()).then_some(a.len().min(b.len())))
    }

    fn footprint(keys: &[ExactKey<'_>]) -> u64 {
        let mut hash = DefaultHasher::new();
        keys.hash(&mut hash);
        hash.finish()
    }

    let names: [String; TRACKS] = std::array::from_fn(|i| format!("point-{i:02}"));
    let tracks = names
        .iter()
        .rev()
        .map(|name| event_track(name, POINT, None));
    let clip = clip(PERIOD, tracks.collect());
    let ranges = [[0.0, 0.1], [0.1, 0.0]];
    // Legacy fresh seeds, then four f32 adds/subtracts. The reverse trajectory
    // is not the forward array reversed, nor the nearest f32 to each nominal time.
    let anchors = [
        [
            0x3c23_d70a,
            0x3cf5_c28f,
            0x3d4c_cccc,
            0x3d8f_5c28,
            0x3db8_51ea,
        ],
        [
            0x3db8_51eb,
            0x3d8f_5c28,
            0x3d4c_cccb,
            0x3cf5_c28c,
            0x3c23_d704,
        ],
    ];
    let observed = ranges.map(|range @ [from, to]| {
        let single = sample_clip_events_budgeted(
            &clip,
            ENTITY,
            from,
            to,
            true,
            None,
            Limits {
                max_events: 5 * TRACKS + 1,
                ..Limits::default()
            },
        );
        let batches = pages("partial-tie", &clip, range, Limits::default(), 4);
        (single, batches)
    });
    let compared = [0, 1].map(|i| {
        let [from, to] = ranges[i];
        let anchor = anchors[i];
        let (single, batches) = &observed[i];
        // Exactly five occurrences of each name, ordered by time and then name.
        let expected: Vec<ExactKey<'_>> = anchor.into_iter().flat_map(|tick| {
            names.iter().map(move |name| (ENTITY, POINT.to_bits(), tick, name.as_str(), None, None))
        }).collect();
        let fresh: Vec<_> = single.events.iter().map(exact_key).collect();
        let paged: Vec<_> = events(batches).map(exact_key).collect();
        let ticks: Vec<_> = fresh.chunks(TRACKS).map(|group| group[0].2).collect();
        let counts: Vec<_> = batches.iter().map(|page| page.events.len()).collect();
        let fresh_diff = first_difference(&fresh, &expected);
        let paged_diff = first_difference(&paged, &fresh);
        eprintln!(
            "CLIP_PARTIAL_REFERENCE {from}->{to} anchor={anchor:08x?} anchor_hash={:016x} observed_unpaged_count={} observed_unpaged_bits={ticks:08x?} observed_unpaged_hash={:016x} unpaged_exhausted={} unpaged_next={:x?} unpaged_first_diff={fresh_diff:?} actual={:x?} anchor_item={:x?}",
            footprint(&expected), fresh.len(), footprint(&fresh), single.budget_exhausted,
            single.next_cursor.as_ref().map(key), fresh_diff.and_then(|j| fresh.get(j)), fresh_diff.and_then(|j| expected.get(j)),
        );
        eprintln!(
            "CLIP_PARTIAL_REFERENCE {from}->{to} observed_pages={counts:?} paged_count={} paged_hash={:016x} paged_first_diff={paged_diff:?} paged={:x?} unpaged={:x?}",
            paged.len(), footprint(&paged), paged_diff.and_then(|j| paged.get(j)), paged_diff.and_then(|j| fresh.get(j)),
        );
        (expected, fresh, paged)
    });

    // Both real fresh references are checked before a known-RED page can panic.
    for ((single, _), (expected, fresh, _)) in observed.iter().zip(&compared) {
        assert!(single.next_cursor.is_none() && !single.budget_exhausted);
        assert_eq!(fresh.len(), 5 * TRACKS);
        assert_eq!(fresh, expected, "legacy one-shot trajectory and tie order");
    }
    for (i, ((_, batches), (_, fresh, paged))) in observed.iter().zip(&compared).enumerate() {
        assert_pages(batches, &[64, 36]);
        assert_eq!(paged, fresh, "fixed range must replay every event exactly");
        let cursor = batches[0].next_cursor.as_ref().map(key);
        assert_eq!(cursor, Some((anchors[i][3], Some("point-03"), 16)));
        assert!(batches[0].budget_exhausted && !batches[1].budget_exhausted);
    }
}

#[test]
fn loop_notify_resume_span_bytes_and_oversized_event() {
    let tracks = ["z", "bbbb", "a"].map(|name| event_track(name, 0.25, None));
    let clip = clip(0.5, tracks.into());
    let limits = Limits {
        max_events: 2,
        max_event_bytes: 2,
        max_playback_span_seconds: 0.125,
    };
    let ranges = [[0.0, 0.75], [1.0, 0.25]];
    let observed = ranges.map(|range| pages("span-bytes", &clip, range, limits, 10));
    let positions: [[Real; 2]; 2] = [[0.25, 0.75], [0.75, 0.25]];
    for (batches, times) in observed.into_iter().zip(positions) {
        assert_pages(&batches, &[0, 1, 1, 1, 0, 0, 1, 1, 1]);
        let expected = times
            .into_iter()
            .flat_map(|t| ["a", "bbbb", "z"].map(|n| (t, n)));
        for (e, (time, name)) in events(&batches).zip(expected) {
            assert_eq!(event_key(e), (time.to_bits(), name, None));
        }
        let bytes = [0, 1, 4, 1, 0, 0, 1, 4, 1];
        for (i, page) in batches.iter().enumerate() {
            assert_eq!(page.emitted_event_bytes, bytes[i]);
            assert_eq!(page.oversized_event_count, usize::from(bytes[i] == 4));
            assert_eq!(page.budget_exhausted, matches!(i, 1 | 2 | 6 | 7));
            assert_eq!(page.playback_span_seconds, if i < 7 { 0.125 } else { 0.0 });
        }
    }
}
