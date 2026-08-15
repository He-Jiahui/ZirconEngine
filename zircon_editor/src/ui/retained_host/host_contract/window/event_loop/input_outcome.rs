#[cfg(any(feature = "profiling", test))]
use std::time::{Duration, Instant};

use zircon_runtime_interface::ui::dispatch::UiInputSequence;

use super::UiHostWindowEventLoop;
use crate::ui::retained_host::host_contract::redraw::HostRedrawRequest;
use crate::ui::retained_host::ui_perf::UiPerfScenario;

#[cfg(any(feature = "profiling", test))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum UiInputOutcomeKind {
    Damaged,
    IntentionallyNoDamage,
    Rejected,
}

#[cfg(any(feature = "profiling", test))]
impl UiInputOutcomeKind {
    #[cfg(feature = "profiling")]
    const fn counter_name(self) -> &'static str {
        match self {
            Self::Damaged => "ui.input.outcome.damaged_sequence",
            Self::IntentionallyNoDamage => "ui.input.outcome.intentionally_no_damage_sequence",
            Self::Rejected => "ui.input.outcome.rejected_sequence",
        }
    }
}

#[cfg(any(feature = "profiling", test))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct UiInputOutcome {
    sequence: UiInputSequence,
    kind: UiInputOutcomeKind,
    input_to_damage: Option<Duration>,
}

#[cfg(any(feature = "profiling", test))]
impl UiInputOutcome {
    #[cfg(test)]
    const fn sequence(self) -> UiInputSequence {
        self.sequence
    }

    #[cfg(test)]
    const fn kind(self) -> UiInputOutcomeKind {
        self.kind
    }

    #[cfg(test)]
    const fn input_to_damage(self) -> Option<Duration> {
        self.input_to_damage
    }
}

#[cfg(any(feature = "profiling", test))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct UiPendingPresentInputBatch {
    first_sequence: UiInputSequence,
    last_sequence: UiInputSequence,
    damaged_count: u64,
    first_damage_started_at: Instant,
}

#[cfg(any(feature = "profiling", test))]
impl UiPendingPresentInputBatch {
    #[cfg(test)]
    const fn first_sequence(self) -> UiInputSequence {
        self.first_sequence
    }

    #[cfg(test)]
    const fn last_sequence(self) -> UiInputSequence {
        self.last_sequence
    }

    #[cfg(test)]
    const fn damaged_count(self) -> u64 {
        self.damaged_count
    }

    #[cfg(test)]
    const fn first_damage_started_at(self) -> Instant {
        self.first_damage_started_at
    }

    fn include(&mut self, sequence: UiInputSequence) {
        self.last_sequence = sequence;
        self.damaged_count = self.damaged_count.saturating_add(1);
    }
}

#[cfg(any(feature = "profiling", test))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct UiActiveInput {
    sequence: UiInputSequence,
    started_at: Instant,
}

#[cfg(any(feature = "profiling", test))]
#[derive(Debug, Default)]
pub(super) struct UiInputOutcomeTracker {
    active: Option<UiActiveInput>,
    pending_present: Option<UiPendingPresentInputBatch>,
}

#[cfg(any(feature = "profiling", test))]
impl UiInputOutcomeTracker {
    fn begin(&mut self, sequence: UiInputSequence, started_at: Instant) -> Option<UiInputOutcome> {
        self.active
            .replace(UiActiveInput {
                sequence,
                started_at,
            })
            .map(|interrupted| UiInputOutcome {
                sequence: interrupted.sequence,
                kind: UiInputOutcomeKind::Rejected,
                input_to_damage: None,
            })
    }

    fn finish_damaged(&mut self, damaged_at: Instant) -> Option<UiInputOutcome> {
        let active = self.active.take()?;
        let outcome = UiInputOutcome {
            sequence: active.sequence,
            kind: UiInputOutcomeKind::Damaged,
            input_to_damage: Some(damaged_at.saturating_duration_since(active.started_at)),
        };
        match self.pending_present.as_mut() {
            Some(batch) => batch.include(active.sequence),
            None => {
                self.pending_present = Some(UiPendingPresentInputBatch {
                    first_sequence: active.sequence,
                    last_sequence: active.sequence,
                    damaged_count: 1,
                    first_damage_started_at: damaged_at,
                });
            }
        }
        Some(outcome)
    }

    fn finish_intentionally_no_damage(&mut self) -> Option<UiInputOutcome> {
        self.finish_without_damage(UiInputOutcomeKind::IntentionallyNoDamage)
    }

    fn reject(&mut self) -> Option<UiInputOutcome> {
        self.finish_without_damage(UiInputOutcomeKind::Rejected)
    }

    fn finish_without_damage(&mut self, kind: UiInputOutcomeKind) -> Option<UiInputOutcome> {
        let active = self.active.take()?;
        Some(UiInputOutcome {
            sequence: active.sequence,
            kind,
            input_to_damage: None,
        })
    }

    fn take_presented_batch(&mut self) -> Option<UiPendingPresentInputBatch> {
        self.pending_present.take()
    }

    #[cfg(test)]
    fn pending_present_batch(&self) -> Option<&UiPendingPresentInputBatch> {
        self.pending_present.as_ref()
    }
}

impl UiHostWindowEventLoop {
    pub(super) fn begin_input_outcome(&mut self, sequence: UiInputSequence) {
        #[cfg(feature = "profiling")]
        if let Some(interrupted) = self.input_outcomes.begin(sequence, Instant::now()) {
            record_input_outcome(interrupted, None);
        }
        #[cfg(not(feature = "profiling"))]
        let _ = sequence;
    }

    pub(super) fn finish_input_outcome(&mut self, redraw: &HostRedrawRequest) {
        #[cfg(feature = "profiling")]
        {
            let kind = redraw_outcome_kind(redraw);
            let outcome = match kind {
                UiInputOutcomeKind::Damaged => self.input_outcomes.finish_damaged(Instant::now()),
                UiInputOutcomeKind::IntentionallyNoDamage => {
                    self.input_outcomes.finish_intentionally_no_damage()
                }
                UiInputOutcomeKind::Rejected => unreachable!("redraw requests are not rejected"),
            };
            if let Some(outcome) = outcome {
                record_input_outcome(
                    outcome,
                    (kind == UiInputOutcomeKind::Damaged).then(|| redraw.scenario()),
                );
            }
        }
        #[cfg(not(feature = "profiling"))]
        let _ = redraw;
    }

    pub(super) fn finish_input_without_damage(&mut self) {
        #[cfg(feature = "profiling")]
        if let Some(outcome) = self.input_outcomes.finish_intentionally_no_damage() {
            record_input_outcome(outcome, None);
        }
    }

    pub(super) fn reject_input_outcome(&mut self) {
        #[cfg(feature = "profiling")]
        if let Some(outcome) = self.input_outcomes.reject() {
            record_input_outcome(outcome, None);
        }
    }

    pub(super) fn reset_input_outcome_tracking(&mut self) {
        #[cfg(feature = "profiling")]
        {
            self.input_outcomes = UiInputOutcomeTracker::default();
        }
    }

    pub(super) fn record_presented_input_batch(&mut self, scenario: UiPerfScenario) {
        #[cfg(feature = "profiling")]
        if let Some(batch) = self.input_outcomes.take_presented_batch() {
            let counters = [
                (
                    "ui.input.present_batch.first_sequence",
                    batch.first_sequence.0 as f64,
                ),
                (
                    "ui.input.present_batch.last_sequence",
                    batch.last_sequence.0 as f64,
                ),
                (
                    "ui.input.present_batch.damaged_count",
                    batch.damaged_count as f64,
                ),
                (
                    damage_to_submit_counter_name(scenario),
                    batch.first_damage_started_at.elapsed().as_secs_f64() * 1_000_000.0,
                ),
            ];
            zircon_runtime::core::diagnostics::profiling::record_counter_batch("editor", &counters);
        }
        #[cfg(not(feature = "profiling"))]
        let _ = scenario;
    }
}

#[cfg(any(feature = "profiling", test))]
fn redraw_outcome_kind(redraw: &HostRedrawRequest) -> UiInputOutcomeKind {
    if redraw.requires_present() {
        UiInputOutcomeKind::Damaged
    } else {
        UiInputOutcomeKind::IntentionallyNoDamage
    }
}

#[cfg(feature = "profiling")]
fn record_input_outcome(outcome: UiInputOutcome, scenario: Option<UiPerfScenario>) {
    let outcome_counter = (outcome.kind.counter_name(), outcome.sequence.0 as f64);
    if let (Some(scenario), Some(input_to_damage)) = (scenario, outcome.input_to_damage) {
        let counters = [
            outcome_counter,
            (
                input_to_damage_counter_name(scenario),
                input_to_damage.as_secs_f64() * 1_000_000.0,
            ),
        ];
        zircon_runtime::core::diagnostics::profiling::record_counter_batch("editor", &counters);
    } else {
        zircon_runtime::core::diagnostics::profiling::record_counter_batch(
            "editor",
            &[outcome_counter],
        );
    }
}

#[cfg(feature = "profiling")]
const fn input_to_damage_counter_name(scenario: UiPerfScenario) -> &'static str {
    match scenario {
        UiPerfScenario::Startup => "ui.startup.input_to_damage_us",
        UiPerfScenario::IdleHover => "ui.idle_hover.input_to_damage_us",
        UiPerfScenario::Click => "ui.click.input_to_damage_us",
        UiPerfScenario::Drag => "ui.drag.input_to_damage_us",
        UiPerfScenario::DrawerResize => "ui.drawer_resize.input_to_damage_us",
        UiPerfScenario::WindowResize => "ui.window_resize.input_to_damage_us",
        UiPerfScenario::AssetRefresh => "ui.asset_refresh.input_to_damage_us",
        UiPerfScenario::ViewportImage => "ui.viewport_image.input_to_damage_us",
        UiPerfScenario::ShellContent => "ui.shell_content.input_to_damage_us",
    }
}

#[cfg(feature = "profiling")]
const fn damage_to_submit_counter_name(scenario: UiPerfScenario) -> &'static str {
    match scenario {
        UiPerfScenario::Startup => "ui.startup.damage_to_submit_us",
        UiPerfScenario::IdleHover => "ui.idle_hover.damage_to_submit_us",
        UiPerfScenario::Click => "ui.click.damage_to_submit_us",
        UiPerfScenario::Drag => "ui.drag.damage_to_submit_us",
        UiPerfScenario::DrawerResize => "ui.drawer_resize.damage_to_submit_us",
        UiPerfScenario::WindowResize => "ui.window_resize.damage_to_submit_us",
        UiPerfScenario::AssetRefresh => "ui.asset_refresh.damage_to_submit_us",
        UiPerfScenario::ViewportImage => "ui.viewport_image.damage_to_submit_us",
        UiPerfScenario::ShellContent => "ui.shell_content.damage_to_submit_us",
    }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use zircon_runtime_interface::ui::dispatch::UiInputSequence;

    use super::*;

    #[test]
    fn damaged_inputs_coalesce_into_one_bounded_present_batch() {
        let mut tracker = UiInputOutcomeTracker::default();
        let started_at = Instant::now();

        tracker.begin(UiInputSequence::new(1), started_at);
        let first = tracker
            .finish_damaged(started_at + Duration::from_micros(10))
            .expect("first input outcome");
        tracker.begin(UiInputSequence::new(2), started_at);
        let quiet = tracker
            .finish_intentionally_no_damage()
            .expect("quiet input outcome");
        tracker.begin(UiInputSequence::new(3), started_at);
        let third = tracker
            .finish_damaged(started_at + Duration::from_micros(30))
            .expect("third input outcome");

        assert_eq!(first.kind(), UiInputOutcomeKind::Damaged);
        assert_eq!(first.input_to_damage(), Some(Duration::from_micros(10)));
        assert_eq!(quiet.kind(), UiInputOutcomeKind::IntentionallyNoDamage);
        assert_eq!(third.kind(), UiInputOutcomeKind::Damaged);
        let batch = tracker.take_presented_batch().expect("present batch");
        assert_eq!(batch.first_sequence(), UiInputSequence::new(1));
        assert_eq!(batch.last_sequence(), UiInputSequence::new(3));
        assert_eq!(batch.damaged_count(), 2);
        assert_eq!(
            batch.first_damage_started_at(),
            started_at + Duration::from_micros(10)
        );
        assert!(tracker.take_presented_batch().is_none());
    }

    #[test]
    fn retry_does_not_consume_the_pending_present_batch() {
        let mut tracker = UiInputOutcomeTracker::default();
        let started_at = Instant::now();
        tracker.begin(UiInputSequence::new(8), started_at);
        tracker.finish_damaged(started_at).expect("input outcome");

        let before_retry = tracker.pending_present_batch().copied();
        let after_retry = tracker.pending_present_batch().copied();

        assert_eq!(before_retry, after_retry);
        assert_eq!(
            tracker
                .take_presented_batch()
                .expect("retry retained batch")
                .damaged_count(),
            1
        );
    }

    #[test]
    fn rejected_and_quiet_inputs_never_enter_the_present_batch() {
        let mut tracker = UiInputOutcomeTracker::default();
        let started_at = Instant::now();
        tracker.begin(UiInputSequence::new(11), started_at);
        let quiet = tracker
            .finish_intentionally_no_damage()
            .expect("quiet outcome");
        tracker.begin(UiInputSequence::new(12), started_at);
        let rejected = tracker.reject().expect("rejected outcome");

        assert_eq!(quiet.sequence(), UiInputSequence::new(11));
        assert_eq!(rejected.sequence(), UiInputSequence::new(12));
        assert_eq!(rejected.kind(), UiInputOutcomeKind::Rejected);
        assert!(tracker.pending_present_batch().is_none());
    }

    #[test]
    fn beginning_an_input_rejects_an_unfinished_prior_input() {
        let mut tracker = UiInputOutcomeTracker::default();
        let started_at = Instant::now();
        assert!(tracker
            .begin(UiInputSequence::new(21), started_at)
            .is_none());

        let interrupted = tracker
            .begin(UiInputSequence::new(22), started_at)
            .expect("unfinished input must fail closed");

        assert_eq!(interrupted.sequence(), UiInputSequence::new(21));
        assert_eq!(interrupted.kind(), UiInputOutcomeKind::Rejected);
        assert_eq!(
            tracker.reject().expect("current input").sequence(),
            UiInputSequence::new(22)
        );
    }

    #[test]
    fn input_outcome_authority_has_no_unbounded_sequence_collection() {
        let production = include_str!("input_outcome.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("production input outcome source");

        assert!(!production.contains("Vec<"));
        assert!(!production.contains("VecDeque<"));
    }

    #[test]
    fn frame_update_wake_is_not_misclassified_as_present_damage() {
        let frame_update = HostRedrawRequest::FrameUpdate {
            scenario: UiPerfScenario::Click,
        };
        let present = HostRedrawRequest::Full {
            frame_update: true,
            scenario: UiPerfScenario::Click,
        };

        assert_eq!(
            redraw_outcome_kind(&frame_update),
            UiInputOutcomeKind::IntentionallyNoDamage
        );
        assert_eq!(redraw_outcome_kind(&present), UiInputOutcomeKind::Damaged);
    }
}
