use super::HostTextInputFocusData;

impl HostTextInputFocusData {
    pub(crate) fn resolved_caret_scalar_offset(&self) -> usize {
        let len = self.value_text.chars().count();
        self.caret_scalar_offset.unwrap_or(len).min(len)
    }

    pub(crate) fn selected_scalar_range(&self) -> Option<(usize, usize)> {
        let caret = self.resolved_caret_scalar_offset();
        let anchor = self
            .selection_anchor_scalar_offset?
            .min(self.value_text.chars().count());
        (anchor != caret).then_some((anchor.min(caret), anchor.max(caret)))
    }

    pub(crate) fn move_text_caret(&mut self, movement: &str, extend: bool) -> bool {
        let previous = (
            self.caret_scalar_offset,
            self.selection_anchor_scalar_offset,
        );
        let caret = self.resolved_caret_scalar_offset();
        let len = self.value_text.chars().count();
        let next = match (movement, extend, self.selected_scalar_range()) {
            ("left", false, Some((start, _))) => start,
            ("right", false, Some((_, end))) => end,
            ("left", _, _) => caret.saturating_sub(1),
            ("right", _, _) => caret.saturating_add(1).min(len),
            ("home", _, _) => 0,
            ("end" | "all", _, _) => len,
            _ => return false,
        };
        self.selection_anchor_scalar_offset = if movement == "all" {
            Some(0)
        } else if extend {
            Some(
                self.selection_anchor_scalar_offset
                    .unwrap_or(caret)
                    .min(len),
            )
        } else {
            None
        };
        self.caret_scalar_offset = Some(next);
        previous
            != (
                self.caret_scalar_offset,
                self.selection_anchor_scalar_offset,
            )
    }

    pub(crate) fn insert_text(&mut self, text: &str) -> bool {
        let text: String = text.chars().filter(|ch| !ch.is_control()).collect();
        if text.is_empty() {
            return false;
        }
        let caret = self.resolved_caret_scalar_offset();
        let (start, end) = self.selected_scalar_range().unwrap_or((caret, caret));
        self.replace_scalar_range(start, end, &text);
        true
    }

    pub(crate) fn delete_text(&mut self, backward: bool) -> bool {
        let caret = self.resolved_caret_scalar_offset();
        let len = self.value_text.chars().count();
        let (start, end) = self.selected_scalar_range().unwrap_or_else(|| {
            if backward {
                (caret.saturating_sub(1), caret)
            } else {
                (caret, caret.saturating_add(1).min(len))
            }
        });
        if start == end {
            return false;
        }
        self.replace_scalar_range(start, end, "");
        true
    }

    fn replace_scalar_range(&mut self, start: usize, end: usize, text: &str) {
        // Scalars become UTF-8 byte positions only at the mutation boundary.
        let mut value = self.value_text.to_string();
        let start_byte = value
            .char_indices()
            .nth(start)
            .map_or(value.len(), |(i, _)| i);
        let end_byte = value
            .char_indices()
            .nth(end)
            .map_or(value.len(), |(i, _)| i);
        value.replace_range(start_byte..end_byte, text);
        self.value_text = value.into();
        self.caret_scalar_offset = Some(start + text.chars().count());
        self.selection_anchor_scalar_offset = None;
    }
}

#[cfg(test)]
#[path = "editing/tests/cases.rs"]
mod tests;
