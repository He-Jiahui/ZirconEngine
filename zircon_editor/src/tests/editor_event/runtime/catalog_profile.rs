use std::collections::HashMap;
use std::ffi::OsStr;
use std::marker::PhantomData;
use std::path::Path;
use std::rc::Rc;
use std::thread::{self, ThreadId};

use zircon_runtime::core::diagnostics::profiling as p;

const TEST: &str = concat!(
    "tests::editor_event::runtime::animation_assets::first_catalog::",
    "editor202_profile_first_catalog_phases"
);
const MARKER: &str = "editor202_witness";
const NAMES: [&str; 2] = ["open", "explicit_refresh"];
const INPUT_SEQUENCE: &str = "runtime_catalog_input_generation_sequence";
const REBASE: &str = "catalog_generation_rebased_count";
const COUNTER_LIMIT: usize = 32;
const RUNTIME_LIMIT: usize = 12;

pub(super) struct Capture {
    session: String,
    caller: ThreadId,
    running: bool,
}

pub(super) struct Phase {
    // Fields drop in declaration order, so the inner span finishes before its frame.
    _span: p::ProfileScope,
    _frame: p::ProfileFrameScope,
    _same_thread: PhantomData<Rc<()>>,
}

impl Capture {
    pub(super) fn start(fixture_profile: &Path) -> Option<Self> {
        // `start_capture` clears the process recorder. The ordinary witness, even
        // in a profiling build, must return without touching that recorder.
        if !dedicated_test_process() {
            return None;
        }
        if !p::feature_enabled() || p::capture_active() {
            unavailable("both", "recorder_disabled_or_already_active");
            return None;
        }
        let session = format!("editor202-phases-{}", std::process::id());
        let status = p::start_capture(p::ProfileCaptureConfig {
            session_id: session.clone(),
            // No export; even the unused output path belongs to this fixture.
            output_root: fixture_profile
                .join("profile")
                .to_string_lossy()
                .into_owned(),
            max_frames: 64,
            max_spans: p::PROFILE_CAPTURE_MAX_SPANS,
            max_counters: p::PROFILE_CAPTURE_MAX_COUNTERS,
            include_perfetto: false,
            ..Default::default()
        });
        let capture = Self {
            session,
            caller: thread::current().id(),
            running: status.active,
        };
        if status.active && status.feature_enabled {
            Some(capture)
        } else {
            unavailable("both", "capture_did_not_start");
            None
        }
    }

    pub(super) fn enter(&self, name: &'static str) -> Option<Phase> {
        if thread::current().id() != self.caller || !p::capture_active() {
            return None;
        }
        let frame = p::ProfileFrameScope::enter(MARKER, name);
        Some(Phase {
            _span: p::ProfileScope::enter(MARKER, "call", name),
            _frame: frame,
            _same_thread: PhantomData,
        })
    }

    pub(super) fn stop(&mut self) {
        if std::mem::take(&mut self.running) {
            p::stop_capture();
        }
    }

    pub(super) fn report(self) {
        let s = p::snapshot();
        eprintln!(
            "editor202 profile session={:?} caller={:?} active={} feature={} retention={:?}",
            s.session_id, self.caller, s.active, s.feature_enabled, s.recorder_retention
        );
        if self.running
            || s.active
            || !s.feature_enabled
            || s.session_id != self.session
            || self.caller != thread::current().id()
        {
            unavailable("both", "unexpected_recorder_or_caller");
            return;
        }
        let [retention] = s.recorder_retention.as_slice() else {
            unavailable("both", "missing_or_multiple_recorder_retention");
            return;
        };
        if [
            (&retention.frames, s.frames.len()),
            (&retention.spans, s.spans.len()),
            (&retention.counters, s.counters.len()),
        ]
        .iter()
        .any(|(r, n)| {
            r.overwritten != 0
                || r.written != r.retained
                || r.retained != *n as u64
                || r.retained >= r.capacity
        }) {
            unavailable("both", "recorder_loss_or_capacity");
            return;
        }
        // Frame indices are only unique per stream; accept only the two expected frames.
        let [open, refresh] = s.frames.as_slice() else {
            unavailable("both", "missing_or_unexpected_frames");
            return;
        };
        if open.stream != MARKER
            || refresh.stream != MARKER
            || open.name != NAMES[0]
            || refresh.name != NAMES[1]
            || (open.frame_index, refresh.frame_index) != (0, 1)
            || end(open.start_us, open.duration_us) > refresh.start_us
        {
            unavailable("both", "ambiguous_frame_identity_or_window");
            return;
        }
        let by_id: HashMap<_, _> = s.spans.iter().map(|span| (span.id, span)).collect();
        if by_id.len() != s.spans.len() {
            unavailable("both", "ambiguous_span_identity");
            return;
        }
        for frame in [open, refresh] {
            if let Err(reason) = report_phase(&s, &by_id, frame) {
                unavailable(&frame.name, reason);
            }
        }
    }
}

impl Drop for Capture {
    fn drop(&mut self) {
        self.stop(); // Also on unwind, after both caller scopes have dropped.
    }
}

pub(super) fn dedicated_test_process() -> bool {
    let mut args = std::env::args_os().skip(1);
    let (mut filter, mut exact, mut ignored, mut serial, mut output) =
        (false, false, false, false, false);
    while let Some(arg) = args.next() {
        match arg.to_str() {
            Some(TEST) if !filter => filter = true,
            Some("--exact") if !exact => exact = true,
            Some("--ignored") if !ignored => ignored = true,
            Some("--test-threads=1") if !serial => serial = true,
            Some("--test-threads") if !serial => {
                if args.next().as_deref() != Some(OsStr::new("1")) {
                    return false;
                }
                serial = true;
            }
            Some("--show-output" | "--nocapture") if !output => output = true,
            _ => return false,
        }
    }
    filter && exact && ignored && serial && output
}

fn report_phase(
    s: &p::ProfileSnapshot,
    by_id: &HashMap<u64, &p::ProfileSpanSnapshot>,
    frame: &p::ProfileFrameSnapshot,
) -> Result<(), &'static str> {
    let name = frame.name.as_str();
    let frame_window = (frame.start_us, end(frame.start_us, frame.duration_us));
    let frame_index = Some(frame.frame_index);
    let mut anchors = s
        .spans
        .iter()
        .filter(|span| span.stream == MARKER && span.category == "call" && span.name == name);
    let anchor = anchors.next().ok_or("missing_caller_span")?;
    if anchors.next().is_some()
        || anchor.parent_id.is_some()
        || anchor.frame_index != frame_index
        || !encloses(frame_window, window(anchor))
    {
        return Err("ambiguous_caller_span");
    }
    report_runtime_scopes(s, by_id, anchor);
    let mut scoped_sync = s
        .spans
        .iter()
        .filter(|span| is_project_sync(span) && descends_from(by_id, span, anchor.id));
    let sync = scoped_sync.next().ok_or("no_scoped_project_sync")?;
    if scoped_sync.next().is_some()
        || sync.frame_index != frame_index
        || !encloses(window(anchor), window(sync))
    {
        return Err("ambiguous_scoped_project_sync");
    }
    eprintln!(
        "editor202 profile phase={name} entry=scoped frame={} caller_span={} caller_us={:?} sync_span={} sync_parent={:?} sync_us={:?}",
        frame.frame_index, anchor.id, window(anchor), sync.id, sync.parent_id, window(sync)
    );
    if s.spans.iter().any(|other| {
        is_project_sync(other) && other.id != sync.id && overlaps(window(other), window(sync))
    }) {
        return Err("overlapping_project_sync");
    }

    // The recorder has no counter thread/span id. These exact names have one producer:
    // the synchronous sync_from_project body. This headless call has no frame-exporting
    // render extraction that can run it on a worker. Keep only the verified frame and span.
    let (start, stop) = window(sync);
    let samples: Vec<_> = s
        .counters
        .iter()
        .filter_map(|sample| {
            if sample.stream != "editor"
                || sample.frame_index != frame_index
                || !(start..=stop).contains(&sample.timestamp_us)
            {
                return None;
            }
            let name = sample.name.strip_prefix("asset_catalog.")?;
            let relevant = terminal(name).is_some()
                || matches!(
                    name,
                    INPUT_SEQUENCE
                        | REBASE
                        | "runtime_catalog_input_initial_sync_count"
                        | "projection_full_count"
                        | "projection_catalog_record_count"
                        | "shader_ide_refresh_request_count"
                );
            relevant.then_some((name, sample.value, sample.timestamp_us))
        })
        .take(COUNTER_LIMIT + 1)
        .collect();
    if samples.len() > COUNTER_LIMIT {
        return Err("phase_counter_evidence_over_budget");
    }
    eprintln!("editor202 profile phase={name} scoped_samples[name,value,us]={samples:?}");
    let [(INPUT_SEQUENCE, seq, _), rest @ ..] = samples.as_slice() else {
        return Err("missing_input_sequence_sentinel");
    };
    if !(1.0..=9_007_199_254_740_991.0).contains(seq)
        || seq.fract() != 0.0
        || rest.iter().any(|(key, _, _)| *key == INPUT_SEQUENCE)
    {
        return Err("invalid_or_ambiguous_input_sequence_sentinel");
    }

    let mut result = None;
    let mut rebases = 0;
    for (i, &(key, value, _)) in rest.iter().enumerate() {
        if key == REBASE {
            if value != 1.0 {
                return Err("invalid_rebase_marker");
            }
            rebases += 1;
        }
        if let Some(event) = terminal(key) {
            if value != 1.0 || i + 1 != rest.len() {
                return Err("ambiguous_terminal_or_order");
            }
            result = Some(event);
        }
    }
    let (outcome, last_attempt_fence) = result.unwrap_or(("not_attributable", "not_attributed"));
    if rebases != 0 && outcome == "projection_unchanged" {
        return Err("unexpected_rebase_on_unchanged_input");
    }
    let callback = if rebases > 0 || last_attempt_fence == "callback_entered" {
        "observed"
    } else {
        "no_positive_evidence"
    };
    let terminal_note = if result.is_none() {
        "no_terminal_counter"
    } else {
        "scoped_event"
    };
    eprintln!(
        "editor202 profile phase={name} source_sequence={seq} rebase_events={rebases} commit_callback={callback} terminal={outcome} terminal_attempt_fence={last_attempt_fence} evidence={terminal_note}"
    );
    Ok(())
}

fn terminal(name: &str) -> Option<(&'static str, &'static str)> {
    Some(match name {
        "projection_unchanged_count" => ("projection_unchanged", "not_reached"),
        "source_sync_superseded_count" => ("source_epoch_superseded", "not_reached"),
        "runtime_project_generation_superseded_count" => ("runtime_token_superseded", "rejected"),
        "source_generation_superseded_count" => {
            ("source_generation_superseded", "callback_entered")
        }
        "catalog_generation_superseded_count" => {
            ("catalog_generation_superseded", "callback_entered")
        }
        "catalog_generation_publish_count" => ("editor_catalog_published", "callback_entered"),
        _ => return None,
    })
}

fn report_runtime_scopes(
    s: &p::ProfileSnapshot,
    by_id: &HashMap<u64, &p::ProfileSpanSnapshot>,
    phase: &p::ProfileSpanSnapshot,
) {
    let relevant = s.spans.iter().filter(|span| {
        span.stream == "asset"
            && span.category == "project_generation.phase"
            && matches!(
                span.name.as_str(),
                "file_commit" | "project_install" | "generation_publish"
            )
            && overlaps(window(phase), window(span))
    });
    let mut caller = Vec::new();
    let mut overlap = Vec::new();
    let mut omitted = [0, 0];
    for span in relevant {
        let (list, index) = if span.frame_index == phase.frame_index
            && encloses(window(phase), window(span))
            && descends_from(by_id, span, phase.id)
        {
            (&mut caller, 0)
        } else {
            (&mut overlap, 1)
        };
        if list.len() < RUNTIME_LIMIT {
            list.push((span.name.as_str(), span.id, span.parent_id, window(span)));
        } else {
            omitted[index] += 1;
        }
    }
    eprintln!(
        "editor202 profile phase={} runtime_scopes[name,id,parent,us] caller_descendants={caller:?} unattributed_overlap={overlap:?} omitted[caller,overlap]={omitted:?}",
        phase.name
    );
}

fn is_project_sync(span: &p::ProfileSpanSnapshot) -> bool {
    span.stream == "editor" && span.category == "asset_catalog" && span.name == "sync_from_project"
}

fn descends_from<'a>(
    by_id: &HashMap<u64, &'a p::ProfileSpanSnapshot>,
    mut span: &'a p::ProfileSpanSnapshot,
    ancestor: u64,
) -> bool {
    // The recorder's parent stack is thread-local; IDs grow on entry, not on Drop.
    while let Some(parent) = span.parent_id.filter(|&id| id < span.id) {
        if parent == ancestor {
            return true;
        }
        let Some(&previous) = by_id.get(&parent) else {
            return false;
        };
        span = previous;
    }
    false
}

fn unavailable(phase: &str, reason: &str) {
    eprintln!("editor202 profile phase={phase} attribution=not_attributable reason={reason}");
}

fn end(start: u64, duration: u64) -> u64 {
    start.saturating_add(duration)
}

fn window(span: &p::ProfileSpanSnapshot) -> (u64, u64) {
    (span.start_us, end(span.start_us, span.duration_us))
}

fn encloses((start, end): (u64, u64), (inner_start, inner_end): (u64, u64)) -> bool {
    start <= inner_start && inner_end <= end
}

fn overlaps((start, end): (u64, u64), (other_start, other_end): (u64, u64)) -> bool {
    start <= other_end && other_start <= end
}
