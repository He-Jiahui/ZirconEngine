use zircon_runtime_interface::ui::dispatch::{
    UiImePreeditClause, UiTextByteRange, UiTextInputConstraintReceipt,
};

use crate::ui::text::clamp_grapheme_boundary;

/// Maps only platform-referenced boundaries, so memory is proportional to IME metadata, not text.
pub(super) struct TextInputBoundaryMap {
    requested: Vec<(usize, Option<usize>)>,
    next_request: usize,
}

impl TextInputBoundaryMap {
    pub(super) fn new(
        cursor_range: Option<UiTextByteRange>,
        preedit_clauses: &[UiImePreeditClause],
    ) -> Self {
        let capacity = preedit_clauses.len().saturating_mul(2).saturating_add(2);
        let mut requested = Vec::with_capacity(capacity);
        if let Some(range) = cursor_range {
            requested.extend([range.start_byte as usize, range.end_byte as usize]);
        }
        for clause in preedit_clauses {
            requested.extend([
                clause.range.start_byte as usize,
                clause.range.end_byte as usize,
            ]);
        }
        requested.sort_unstable();
        requested.dedup();
        Self {
            requested: requested.into_iter().map(|offset| (offset, None)).collect(),
            next_request: 0,
        }
    }

    pub(super) fn record(&mut self, input_offset: usize, output_offset: usize) {
        if self
            .requested
            .get(self.next_request)
            .is_some_and(|(requested, _)| *requested == input_offset)
        {
            self.requested[self.next_request].1 = Some(output_offset);
            self.next_request += 1;
        }
    }

    pub(super) fn clamp_output(&mut self, output_len: usize) {
        for (_, mapped) in &mut self.requested {
            if let Some(mapped) = mapped {
                *mapped = (*mapped).min(output_len);
            }
        }
    }

    fn map_range(&self, range: UiTextByteRange) -> Option<UiTextByteRange> {
        let start_byte = self.map_offset(range.start_byte)?;
        let end_byte = self.map_offset(range.end_byte)?;
        Some(UiTextByteRange::new(start_byte, end_byte.max(start_byte)))
    }

    fn map_offset(&self, offset: u32) -> Option<u32> {
        let index = self
            .requested
            .binary_search_by_key(&(offset as usize), |(requested, _)| *requested)
            .ok()?;
        u32::try_from(self.requested[index].1?).ok()
    }
}

pub(super) fn remap_preedit_cursor_range(
    boundary_map: &TextInputBoundaryMap,
    text: &str,
    cursor_range: Option<UiTextByteRange>,
    receipt: &mut UiTextInputConstraintReceipt,
) -> Option<UiTextByteRange> {
    let original = cursor_range?;
    let mapped = boundary_map.map_range(original);
    let adjusted = mapped.and_then(|range| {
        let start = clamp_grapheme_boundary(text, range.start_byte as usize);
        let end = clamp_grapheme_boundary(text, range.end_byte as usize).max(start);
        Some(UiTextByteRange::new(
            u32::try_from(start).ok()?,
            u32::try_from(end).ok()?,
        ))
    });
    if adjusted != Some(original) {
        receipt.preedit_cursor_range_adjusted = true;
    }
    adjusted
}

pub(super) fn remap_preedit_clauses(
    boundary_map: &TextInputBoundaryMap,
    clauses: &[UiImePreeditClause],
    receipt: &mut UiTextInputConstraintReceipt,
) -> Vec<UiImePreeditClause> {
    let mut mapped_clauses = Vec::with_capacity(clauses.len());
    for clause in clauses {
        let Some(range) = boundary_map.map_range(clause.range) else {
            receipt.preedit_clause_dropped_count =
                receipt.preedit_clause_dropped_count.saturating_add(1);
            continue;
        };
        if clause.range.start_byte < clause.range.end_byte && range.start_byte == range.end_byte {
            receipt.preedit_clause_dropped_count =
                receipt.preedit_clause_dropped_count.saturating_add(1);
            continue;
        }
        if range != clause.range {
            receipt.preedit_clause_range_adjusted_count = receipt
                .preedit_clause_range_adjusted_count
                .saturating_add(1);
        }
        mapped_clauses.push(UiImePreeditClause::new(range, clause.kind));
    }
    mapped_clauses
}
