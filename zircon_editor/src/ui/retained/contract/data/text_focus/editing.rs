use unicode_segmentation::UnicodeSegmentation;

use super::HostTextInputFocusData;

// Expose EGC boundaries in scalar coordinates for the existing focus/paint contract.
fn grapheme_scalar_boundaries(text: &str) -> impl Iterator<Item = usize> + '_ {
    std::iter::once(0).chain(text.graphemes(true).scan(0, |offset, grapheme| {
        *offset += grapheme.chars().count();
        Some(*offset)
    }))
}

fn floor_grapheme_boundary(text: &str, offset: usize) -> usize {
    grapheme_scalar_boundaries(text)
        .take_while(|&boundary| boundary <= offset)
        .last()
        .unwrap_or(0)
}

fn adjacent_grapheme_boundary(text: &str, caret: usize, backward: bool) -> usize {
    let mut boundaries = grapheme_scalar_boundaries(text);
    if backward {
        boundaries.take_while(|&boundary| boundary < caret).last()
    } else {
        boundaries.find(|&boundary| boundary > caret)
    }
    .unwrap_or(caret)
}

impl HostTextInputFocusData {
    pub(crate) fn resolved_caret_scalar_offset(&self) -> usize {
        let text = self.value_text.as_str();
        self.caret_scalar_offset.map_or_else(
            || text.chars().count(),
            |offset| floor_grapheme_boundary(text, offset),
        )
    }

    pub(crate) fn selected_scalar_range(&self) -> Option<(usize, usize)> {
        self.selected_scalar_range_at(self.resolved_caret_scalar_offset())
    }

    fn resolved_selection_anchor(&self) -> Option<usize> {
        self.selection_anchor_scalar_offset
            .map(|offset| floor_grapheme_boundary(self.value_text.as_str(), offset))
    }

    fn selected_scalar_range_at(&self, caret: usize) -> Option<(usize, usize)> {
        let anchor = self.resolved_selection_anchor()?;
        (anchor != caret).then_some((anchor.min(caret), anchor.max(caret)))
    }

    pub(crate) fn move_text_caret(&mut self, movement: &str, extend: bool) -> bool {
        let previous = (
            self.caret_scalar_offset,
            self.selection_anchor_scalar_offset,
        );
        let caret = self.resolved_caret_scalar_offset();
        let text = self.value_text.as_str();
        let next = match (movement, extend, self.selected_scalar_range_at(caret)) {
            ("left", false, Some((start, _))) => start,
            ("right", false, Some((_, end))) => end,
            ("left", _, _) => adjacent_grapheme_boundary(text, caret, true),
            ("right", _, _) => adjacent_grapheme_boundary(text, caret, false),
            ("home", _, _) => 0,
            ("end" | "all", _, _) => text.chars().count(),
            _ => return false,
        };
        self.selection_anchor_scalar_offset = if movement == "all" {
            Some(0)
        } else if extend {
            Some(self.resolved_selection_anchor().unwrap_or(caret))
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
        let (start, end) = self
            .selected_scalar_range_at(caret)
            .unwrap_or((caret, caret));
        self.replace_scalar_range(start, end, &text);
        true
    }

    pub(crate) fn delete_text(&mut self, backward: bool) -> bool {
        let caret = self.resolved_caret_scalar_offset();
        let (start, end) = self.selected_scalar_range_at(caret).unwrap_or_else(|| {
            let neighbor = adjacent_grapheme_boundary(self.value_text.as_str(), caret, backward);
            (caret.min(neighbor), caret.max(neighbor))
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
        let end_of_insert = start + text.chars().count();
        // A splice can join neighbors. Stay on an EGC edge without altering the bytes.
        let caret = if text.is_empty() {
            floor_grapheme_boundary(&value, end_of_insert)
        } else {
            grapheme_scalar_boundaries(&value)
                .find(|&boundary| boundary >= end_of_insert)
                .unwrap_or(end_of_insert)
        };
        self.value_text = value.into();
        self.caret_scalar_offset = Some(caret);
        self.selection_anchor_scalar_offset = None;
    }
}

#[cfg(test)]
#[path = "editing/tests/cases.rs"]
mod tests;
