use crate::core::framework::text::TextLayoutError;
use crate::text::{ShapedGlyphRun, TextRange};

#[derive(Default)]
pub(super) struct GraphemeProjectionCursor {
    first: usize,
    after_last: usize,
    previous: Option<(usize, usize)>,
}

impl GraphemeProjectionCursor {
    pub(super) fn overlap_bounds(
        &mut self,
        graphemes: &[(usize, usize)],
        cluster_start: usize,
        cluster_end: usize,
    ) -> (usize, usize) {
        let monotonic = self
            .previous
            .map_or(true, |(previous_start, previous_end)| {
                cluster_start >= previous_start && cluster_end >= previous_end
            });

        if !monotonic {
            let first = graphemes.partition_point(|&(_, end)| end <= cluster_start);
            let after_last = graphemes.partition_point(|&(start, _)| start < cluster_end);
            self.first = first;
            self.after_last = after_last;
            self.previous = Some((cluster_start, cluster_end));
            return (first, after_last);
        }

        while self.first < graphemes.len() && graphemes[self.first].1 <= cluster_start {
            self.first += 1;
        }
        if self.after_last < self.first {
            self.after_last = self.first;
        }
        while self.after_last < graphemes.len() && graphemes[self.after_last].0 < cluster_end {
            self.after_last += 1;
        }
        self.previous = Some((cluster_start, cluster_end));
        (self.first, self.after_last)
    }
}

pub(super) fn aligned_grapheme_span(
    graphemes: &[(usize, usize)],
    first: usize,
    after_last: usize,
    range: TextRange,
) -> Option<f32> {
    if first >= after_last {
        return None;
    }
    let last = after_last.checked_sub(1)?;
    let &(first_start, _) = graphemes.get(first)?;
    let &(_, last_end) = graphemes.get(last)?;
    if first_start != range.start || last_end != range.end {
        return None;
    }
    Some((after_last - first) as f32)
}

pub(super) fn validate_shaped_geometry_source(
    shaped: &ShapedGlyphRun,
    text: &str,
) -> Result<(), TextLayoutError> {
    let source_range = shaped.source_range;
    let Some(source_span) = source_range.end.checked_sub(source_range.start) else {
        return Err(TextLayoutError::BidiInvariant);
    };
    if source_span != shaped.source_text.len() || text != shaped.source_text.as_ref() {
        return Err(TextLayoutError::BidiInvariant);
    }
    if source_range.start > source_range.end {
        return Err(TextLayoutError::BidiInvariant);
    }
    let mut previous_line_end = source_range.start;
    for line in &shaped.lines {
        if line.source_range.start < source_range.start
            || line.source_range.end > source_range.end
            || line.source_range.start > line.source_range.end
            || line.source_range.start < previous_line_end
        {
            return Err(TextLayoutError::LayoutFailed);
        }
        let Some(line_start) = line.source_range.start.checked_sub(source_range.start) else {
            return Err(TextLayoutError::LayoutFailed);
        };
        let Some(line_end) = line.source_range.end.checked_sub(source_range.start) else {
            return Err(TextLayoutError::LayoutFailed);
        };
        if !shaped.source_text.is_char_boundary(line_start)
            || !shaped.source_text.is_char_boundary(line_end)
        {
            return Err(TextLayoutError::LayoutFailed);
        }
        previous_line_end = line.source_range.end;
        for glyph in &line.glyphs {
            let range = glyph.source_range;
            if range.start < line.source_range.start
                || range.end > line.source_range.end
                || range.start > range.end
            {
                return Err(TextLayoutError::LayoutFailed);
            }
            let Some(start) = range.start.checked_sub(source_range.start) else {
                return Err(TextLayoutError::LayoutFailed);
            };
            let Some(end) = range.end.checked_sub(source_range.start) else {
                return Err(TextLayoutError::LayoutFailed);
            };
            if !shaped.source_text.is_char_boundary(start)
                || !shaped.source_text.is_char_boundary(end)
            {
                return Err(TextLayoutError::LayoutFailed);
            }
        }
    }
    Ok(())
}

#[cfg(test)]
#[path = "tests/grapheme_projection.rs"]
mod tests;
