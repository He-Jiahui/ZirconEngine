use zircon_runtime_interface::ZrRuntimeImeCompositionContextV2;

/// One gate belongs to one RuntimeDynamicSession/window binding. The runtime session handle
/// already scopes delivery; this generation tuple fences delayed events after focus changes and
/// composition cancellation inside that session.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct RuntimeImeCompositionGenerationGate {
    window_generation: u64,
    focus_generation: u64,
    composition_generation: u64,
    active: bool,
}

impl RuntimeImeCompositionGenerationGate {
    pub(super) const fn new(window_generation: u64) -> Self {
        Self {
            window_generation,
            focus_generation: 0,
            composition_generation: 0,
            active: false,
        }
    }

    pub(super) const fn window_generation(&self) -> u64 {
        self.window_generation
    }

    /// Accepts the first update in a new generation and subsequent updates for the active one.
    /// Equal generations after commit/cancel are stale replays and are rejected.
    pub(super) fn begin_or_update(&mut self, context: ZrRuntimeImeCompositionContextV2) -> bool {
        if !self.context_is_valid(context) || self.is_older_than_current(context) {
            return false;
        }
        if self.is_newer_than_current(context) {
            self.advance_to(context);
            self.active = true;
            return true;
        }
        self.active
    }

    /// Completes only the active composition. The high-water tuple remains after completion so a
    /// delayed preedit from the completed composition cannot reopen it.
    pub(super) fn commit(&mut self, context: ZrRuntimeImeCompositionContextV2) -> bool {
        if !self.context_is_valid(context)
            || !self.active
            || self.is_older_than_current(context)
            || self.is_newer_than_current(context)
        {
            return false;
        }
        self.active = false;
        true
    }

    /// Cancels the matching active generation. A newer focus/composition token also installs a
    /// tombstone when the cancel arrives before any preedit, fencing a late update from that cycle.
    pub(super) fn cancel(&mut self, context: ZrRuntimeImeCompositionContextV2) -> bool {
        if !self.context_is_valid(context) || self.is_older_than_current(context) {
            return false;
        }
        if self.is_newer_than_current(context) {
            self.advance_to(context);
            self.active = false;
            return true;
        }
        let was_active = self.active;
        self.active = false;
        was_active
    }

    fn context_is_valid(&self, context: ZrRuntimeImeCompositionContextV2) -> bool {
        context.window_generation == self.window_generation
            && context.window_generation != 0
            && context.focus_generation != 0
            && context.composition_generation != 0
    }

    fn is_older_than_current(&self, context: ZrRuntimeImeCompositionContextV2) -> bool {
        context.focus_generation < self.focus_generation
            || (context.focus_generation == self.focus_generation
                && context.composition_generation < self.composition_generation)
    }

    fn is_newer_than_current(&self, context: ZrRuntimeImeCompositionContextV2) -> bool {
        context.focus_generation > self.focus_generation
            || (context.focus_generation == self.focus_generation
                && context.composition_generation > self.composition_generation)
    }

    fn advance_to(&mut self, context: ZrRuntimeImeCompositionContextV2) {
        self.focus_generation = context.focus_generation;
        self.composition_generation = context.composition_generation;
    }
}

#[cfg(test)]
#[path = "tests/ime_composition_generation.rs"]
mod tests;
