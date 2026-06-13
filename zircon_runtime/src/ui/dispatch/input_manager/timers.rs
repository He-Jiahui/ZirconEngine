use std::collections::BTreeMap;

use zircon_runtime_interface::ui::{dispatch::UiInputTimestamp, event_ui::UiNodeId};

const MICROS_PER_MILLI: u64 = 1_000;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct UiInputTimerState {
    last_tick: Option<UiInputTimestamp>,
    typeahead_expirations: BTreeMap<UiNodeId, UiInputTimestamp>,
    submenu_hover_expirations: BTreeMap<UiNodeId, UiSubmenuHoverTimerExpiration>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct UiSubmenuHoverTimerExpiration {
    deadline: UiInputTimestamp,
    option_id: String,
}

impl UiInputTimerState {
    pub fn last_tick(&self) -> Option<UiInputTimestamp> {
        self.last_tick
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

    pub fn clear_submenu_hover_expiration(&mut self, target: UiNodeId) {
        self.submenu_hover_expirations.remove(&target);
    }

    pub fn drain_expired_typeahead(&mut self, now: UiInputTimestamp) -> Vec<UiNodeId> {
        let expired = self
            .typeahead_expirations
            .iter()
            .filter_map(|(target, deadline)| (*deadline <= now).then_some(*target))
            .collect::<Vec<_>>();
        for target in &expired {
            self.typeahead_expirations.remove(target);
        }
        expired
    }

    pub fn drain_expired_submenu_hover(
        &mut self,
        now: UiInputTimestamp,
    ) -> Vec<(UiNodeId, String)> {
        let expired = self
            .submenu_hover_expirations
            .iter()
            .filter_map(|(target, expiration)| {
                (expiration.deadline <= now).then(|| (*target, expiration.option_id.clone()))
            })
            .collect::<Vec<_>>();
        for (target, _) in &expired {
            self.submenu_hover_expirations.remove(target);
        }
        expired
    }
}
