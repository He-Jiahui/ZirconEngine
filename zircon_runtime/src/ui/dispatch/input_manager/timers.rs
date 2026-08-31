use std::{collections::BTreeMap, time::Duration};

use zircon_runtime_interface::ui::{
    dispatch::{UiInputTimestamp, UiPointerId, UiPointerSource},
    event_ui::UiNodeId,
    surface::UiPointerButton,
};

const MICROS_PER_MILLI: u64 = 1_000;
const TOOLTIP_INTRO_FRAME_INTERVAL_MS: u64 = 16;
pub const DEFAULT_DOUBLE_CLICK_TIMEOUT_MS: u64 = 500;
pub const DEFAULT_TOOLTIP_DELAY_MS: u64 = 150;
pub const DEFAULT_TOOLTIP_INTRO_DURATION_MS: u64 = 100;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct UiInputTimerState {
    last_tick: Option<UiInputTimestamp>,
    typeahead_expirations: BTreeMap<UiNodeId, UiInputTimestamp>,
    submenu_hover_expirations: BTreeMap<UiNodeId, UiSubmenuHoverTimerExpiration>,
    tooltip_expirations: BTreeMap<UiNodeId, UiTooltipTimerExpiration>,
    tooltip_intro: Option<UiTooltipIntroTimer>,
    double_click_candidate: Option<UiDoubleClickTimerCandidate>,
    toast_expirations: BTreeMap<UiNodeId, UiToastTimerExpiration>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct UiSubmenuHoverTimerExpiration {
    deadline: UiInputTimestamp,
    option_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct UiTooltipTimerExpiration {
    deadline: UiInputTimestamp,
    tooltip_id: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct UiTooltipIntroTimer {
    started_at: UiInputTimestamp,
    deadline: UiInputTimestamp,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct UiDoubleClickTimerCandidate {
    deadline: UiInputTimestamp,
    target: UiNodeId,
    pointer_id: Option<UiPointerId>,
    pointer_source: UiPointerSource,
    button: Option<UiPointerButton>,
    click_count: u8,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct UiToastTimerExpiration {
    deadline: UiInputTimestamp,
    toast_id: String,
}

impl UiInputTimerState {
    pub fn last_tick(&self) -> Option<UiInputTimestamp> {
        self.last_tick
    }

    pub fn next_frame_visible_delay(&self, now: UiInputTimestamp) -> Option<Duration> {
        let earliest_deadline_micros = self
            .typeahead_expirations
            .values()
            .map(|deadline| deadline.monotonic_micros)
            .chain(
                self.submenu_hover_expirations
                    .values()
                    .map(|expiration| expiration.deadline.monotonic_micros),
            )
            .chain(
                self.tooltip_expirations
                    .values()
                    .map(|expiration| expiration.deadline.monotonic_micros),
            )
            .chain(
                self.tooltip_intro
                    .map(|intro| intro.next_sample_micros(now)),
            )
            .chain(
                self.toast_expirations
                    .values()
                    .map(|expiration| expiration.deadline.monotonic_micros),
            )
            .min()?;

        Some(Duration::from_micros(
            earliest_deadline_micros.saturating_sub(now.monotonic_micros),
        ))
    }

    pub fn record_tick(&mut self, now: UiInputTimestamp) {
        self.last_tick = Some(now);
    }

    pub fn typeahead_expiration(&self, target: UiNodeId) -> Option<UiInputTimestamp> {
        self.typeahead_expirations.get(&target).copied()
    }

    pub fn submenu_hover_expiration(&self, target: UiNodeId) -> Option<UiInputTimestamp> {
        self.submenu_hover_expirations
            .get(&target)
            .map(|expiration| expiration.deadline)
    }

    pub fn submenu_hover_option_id(&self, target: UiNodeId) -> Option<&str> {
        self.submenu_hover_expirations
            .get(&target)
            .map(|expiration| expiration.option_id.as_str())
    }

    pub fn tooltip_expiration(&self, target: UiNodeId) -> Option<UiInputTimestamp> {
        self.tooltip_expirations
            .get(&target)
            .map(|expiration| expiration.deadline)
    }

    pub fn tooltip_id(&self, target: UiNodeId) -> Option<&str> {
        self.tooltip_expirations
            .get(&target)
            .map(|expiration| expiration.tooltip_id.as_str())
    }

    pub fn tooltip_intro_progress(&self, now: UiInputTimestamp) -> Option<f32> {
        self.tooltip_intro.map(|intro| intro.progress(now))
    }

    pub fn toast_expiration(&self, target: UiNodeId) -> Option<UiInputTimestamp> {
        self.toast_expirations
            .get(&target)
            .map(|expiration| expiration.deadline)
    }

    pub fn toast_id(&self, target: UiNodeId) -> Option<&str> {
        self.toast_expirations
            .get(&target)
            .map(|expiration| expiration.toast_id.as_str())
    }

    pub fn double_click_expiration(&self) -> Option<UiInputTimestamp> {
        self.double_click_candidate
            .as_ref()
            .map(|candidate| candidate.deadline)
    }

    pub fn double_click_target(&self) -> Option<UiNodeId> {
        self.double_click_candidate
            .as_ref()
            .map(|candidate| candidate.target)
    }

    pub fn double_click_count(&self) -> Option<u8> {
        self.double_click_candidate
            .as_ref()
            .map(|candidate| candidate.click_count)
    }

    pub fn arm_typeahead_expiration(
        &mut self,
        target: UiNodeId,
        started_at: UiInputTimestamp,
        timeout_ms: u64,
    ) {
        let timeout_micros = timeout_ms.saturating_mul(MICROS_PER_MILLI);
        let deadline = UiInputTimestamp::from_micros(
            started_at.monotonic_micros.saturating_add(timeout_micros),
        );
        self.typeahead_expirations.insert(target, deadline);
    }

    pub fn arm_submenu_hover_expiration(
        &mut self,
        target: UiNodeId,
        option_id: impl Into<String>,
        started_at: UiInputTimestamp,
        delay_ms: u64,
    ) {
        let delay_micros = delay_ms.saturating_mul(MICROS_PER_MILLI);
        let deadline =
            UiInputTimestamp::from_micros(started_at.monotonic_micros.saturating_add(delay_micros));
        self.submenu_hover_expirations.insert(
            target,
            UiSubmenuHoverTimerExpiration {
                deadline,
                option_id: option_id.into(),
            },
        );
    }

    pub fn arm_tooltip_expiration(
        &mut self,
        target: UiNodeId,
        tooltip_id: impl Into<String>,
        started_at: UiInputTimestamp,
        delay_ms: u64,
    ) {
        let delay_micros = delay_ms.saturating_mul(MICROS_PER_MILLI);
        let deadline =
            UiInputTimestamp::from_micros(started_at.monotonic_micros.saturating_add(delay_micros));
        self.tooltip_expirations.insert(
            target,
            UiTooltipTimerExpiration {
                deadline,
                tooltip_id: tooltip_id.into(),
            },
        );
    }

    pub fn clear_submenu_hover_expiration(&mut self, target: UiNodeId) {
        self.submenu_hover_expirations.remove(&target);
    }

    pub fn clear_tooltip_expiration(&mut self, target: UiNodeId) {
        self.tooltip_expirations.remove(&target);
    }

    pub fn clear_tooltip_expirations(&mut self) {
        self.tooltip_expirations.clear();
    }

    pub fn arm_tooltip_intro(&mut self, started_at: UiInputTimestamp) {
        let duration_micros = DEFAULT_TOOLTIP_INTRO_DURATION_MS.saturating_mul(MICROS_PER_MILLI);
        self.tooltip_intro = Some(UiTooltipIntroTimer {
            started_at,
            deadline: UiInputTimestamp::from_micros(
                started_at.monotonic_micros.saturating_add(duration_micros),
            ),
        });
    }

    pub fn clear_tooltip_intro(&mut self) {
        self.tooltip_intro = None;
    }

    pub fn expire_tooltip_intro(&mut self, now: UiInputTimestamp) {
        if self
            .tooltip_intro
            .is_some_and(|intro| intro.deadline <= now)
        {
            self.clear_tooltip_intro();
        }
    }

    pub fn double_click_count_for_release(
        &self,
        target: UiNodeId,
        pointer_id: Option<UiPointerId>,
        pointer_source: UiPointerSource,
        button: Option<UiPointerButton>,
        now: UiInputTimestamp,
    ) -> u8 {
        self.double_click_candidate
            .filter(|candidate| {
                candidate.deadline >= now
                    && candidate.target == target
                    && candidate.pointer_id == pointer_id
                    && candidate.pointer_source == pointer_source
                    && candidate.button == button
            })
            .map(|candidate| candidate.click_count.saturating_add(1).max(1))
            .unwrap_or(1)
    }

    pub fn arm_double_click_candidate(
        &mut self,
        target: UiNodeId,
        pointer_id: Option<UiPointerId>,
        pointer_source: UiPointerSource,
        button: Option<UiPointerButton>,
        click_count: u8,
        started_at: UiInputTimestamp,
    ) {
        let timeout_micros = DEFAULT_DOUBLE_CLICK_TIMEOUT_MS.saturating_mul(MICROS_PER_MILLI);
        let deadline = UiInputTimestamp::from_micros(
            started_at.monotonic_micros.saturating_add(timeout_micros),
        );
        self.double_click_candidate = Some(UiDoubleClickTimerCandidate {
            deadline,
            target,
            pointer_id,
            pointer_source,
            button,
            click_count: click_count.max(1),
        });
    }

    pub fn clear_double_click_candidate(&mut self) {
        self.double_click_candidate = None;
    }

    pub fn expire_double_click_candidate(&mut self, now: UiInputTimestamp) {
        if self
            .double_click_candidate
            .is_some_and(|candidate| candidate.deadline <= now)
        {
            self.clear_double_click_candidate();
        }
    }

    pub fn arm_toast_expiration(
        &mut self,
        target: UiNodeId,
        toast_id: impl Into<String>,
        started_at: UiInputTimestamp,
        timeout_ms: u64,
    ) {
        let timeout_micros = timeout_ms.saturating_mul(MICROS_PER_MILLI);
        let deadline = UiInputTimestamp::from_micros(
            started_at.monotonic_micros.saturating_add(timeout_micros),
        );
        self.toast_expirations.insert(
            target,
            UiToastTimerExpiration {
                deadline,
                toast_id: toast_id.into(),
            },
        );
    }

    pub fn clear_toast_expiration(&mut self, target: UiNodeId) {
        self.toast_expirations.remove(&target);
    }

    pub fn drain_expired_typeahead(&mut self, now: UiInputTimestamp) -> Vec<UiNodeId> {
        let mut expired = Vec::new();
        self.typeahead_expirations.retain(|target, deadline| {
            if *deadline <= now {
                expired.push(*target);
                false
            } else {
                true
            }
        });
        expired
    }

    pub fn drain_expired_submenu_hover(
        &mut self,
        now: UiInputTimestamp,
    ) -> Vec<(UiNodeId, String)> {
        let mut expired = Vec::new();
        self.submenu_hover_expirations.retain(|target, expiration| {
            if expiration.deadline <= now {
                expired.push((*target, std::mem::take(&mut expiration.option_id)));
                false
            } else {
                true
            }
        });
        expired
    }

    pub fn drain_expired_tooltips(&mut self, now: UiInputTimestamp) -> Vec<(UiNodeId, String)> {
        let mut expired = Vec::new();
        self.tooltip_expirations.retain(|target, expiration| {
            if expiration.deadline <= now {
                expired.push((*target, std::mem::take(&mut expiration.tooltip_id)));
                false
            } else {
                true
            }
        });
        expired
    }

    pub fn drain_expired_toasts(&mut self, now: UiInputTimestamp) -> Vec<(UiNodeId, String)> {
        let mut expired = Vec::new();
        self.toast_expirations.retain(|target, expiration| {
            if expiration.deadline <= now {
                expired.push((*target, std::mem::take(&mut expiration.toast_id)));
                false
            } else {
                true
            }
        });
        expired
    }
}

impl UiTooltipIntroTimer {
    fn next_sample_micros(self, now: UiInputTimestamp) -> u64 {
        let frame_interval_micros =
            TOOLTIP_INTRO_FRAME_INTERVAL_MS.saturating_mul(MICROS_PER_MILLI);
        now.monotonic_micros
            .saturating_add(frame_interval_micros)
            .min(self.deadline.monotonic_micros)
    }

    fn progress(self, now: UiInputTimestamp) -> f32 {
        let duration = self
            .deadline
            .monotonic_micros
            .saturating_sub(self.started_at.monotonic_micros);
        if duration == 0 {
            return 1.0;
        }
        let elapsed = now
            .monotonic_micros
            .saturating_sub(self.started_at.monotonic_micros)
            .min(duration);
        (elapsed as f64 / duration as f64) as f32
    }
}

#[cfg(test)]
#[path = "timers/retain_drain_tests.rs"]
mod retain_drain_tests;
