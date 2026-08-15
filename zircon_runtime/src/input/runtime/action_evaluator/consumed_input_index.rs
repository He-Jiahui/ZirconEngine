use crate::input::{GamepadAxisInput, InputButton};

/// Reusable sorted indices into the caller-owned UI-consumed input slices.
#[derive(Debug, Default)]
pub(super) struct ConsumedInputIndex {
    button_indices: Vec<usize>,
    axis_indices: Vec<usize>,
    #[cfg(test)]
    source_visits: usize,
}

impl ConsumedInputIndex {
    pub(super) fn load(
        &mut self,
        consumed_buttons: &[InputButton],
        consumed_axes: &[GamepadAxisInput],
    ) {
        self.button_indices.clear();
        self.button_indices.extend(0..consumed_buttons.len());
        self.button_indices
            .sort_unstable_by(|left, right| consumed_buttons[*left].cmp(&consumed_buttons[*right]));

        self.axis_indices.clear();
        self.axis_indices.extend(0..consumed_axes.len());
        self.axis_indices
            .sort_unstable_by(|left, right| consumed_axes[*left].cmp(&consumed_axes[*right]));

        #[cfg(test)]
        {
            self.source_visits = consumed_buttons.len().saturating_add(consumed_axes.len());
        }
    }

    pub(super) fn button_is_consumed(
        &self,
        consumed_buttons: &[InputButton],
        button: &InputButton,
    ) -> bool {
        self.button_indices
            .binary_search_by(|index| consumed_buttons[*index].cmp(button))
            .is_ok()
    }

    pub(super) fn axis_is_consumed(
        &self,
        consumed_axes: &[GamepadAxisInput],
        axis: GamepadAxisInput,
    ) -> bool {
        self.axis_indices
            .binary_search_by(|index| consumed_axes[*index].cmp(&axis))
            .is_ok()
    }

    pub(super) fn clear(&mut self) {
        self.button_indices.clear();
        self.axis_indices.clear();
        #[cfg(test)]
        {
            self.source_visits = 0;
        }
    }

    pub(super) fn storage_capacity(&self) -> usize {
        self.button_indices
            .capacity()
            .saturating_add(self.axis_indices.capacity())
    }

    #[cfg(test)]
    pub(super) fn source_visit_count(&self) -> usize {
        self.source_visits
    }
}
