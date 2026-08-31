use crate::text::layout::{
    DiscretionaryHyphenDecision, LogicalVirtualFragmentRole, trailing_wrap_space_byte_len,
};
use zircon_runtime_interface::ui::surface::{
    UiResolvedTextRun, UiTextDirection, UiTextRange, UiTextRunKind,
};

use super::direction::resolve_direction;
use super::range_mapping::source_subrange;

#[derive(Clone, Debug)]
pub(super) struct CandidateLine {
    pub text: String,
    pub source_range: UiTextRange,
    pub runs: Vec<UiResolvedTextRun>,
    pub virtual_source_receipts: Vec<VirtualTextSourceReceipt>,
    pub pending_break_suffix: Option<PendingBreakSuffix>,
    pub ellipsized: bool,
}

impl CandidateLine {
    pub(super) fn empty() -> Self {
        Self {
            text: String::new(),
            source_range: UiTextRange::default(),
            runs: Vec::new(),
            virtual_source_receipts: Vec::new(),
            pending_break_suffix: None,
            ellipsized: false,
        }
    }

    pub(super) fn record_virtual_source_receipt(
        &mut self,
        visual_range: UiTextRange,
        style_source_range: UiTextRange,
        replaced_source_range: Option<UiTextRange>,
        virtual_role: LogicalVirtualFragmentRole,
    ) -> bool {
        if visual_range.start >= visual_range.end
            || style_source_range.start >= style_source_range.end
            || replaced_source_range.is_some_and(|range| range.start >= range.end)
        {
            return false;
        }
        self.virtual_source_receipts.push(VirtualTextSourceReceipt {
            visual_range,
            style_source_range,
            replaced_source_range,
            virtual_role,
        });
        true
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct VirtualTextSourceReceipt {
    pub visual_range: UiTextRange,
    pub style_source_range: UiTextRange,
    pub replaced_source_range: Option<UiTextRange>,
    pub virtual_role: LogicalVirtualFragmentRole,
}

#[derive(Clone, Debug)]
pub(super) struct PendingBreakSuffix {
    pub kind: UiTextRunKind,
    pub decision: DiscretionaryHyphenDecision,
}

pub(super) fn append_segment(
    current: &mut CandidateLine,
    kind: UiTextRunKind,
    text: &str,
    source_range: UiTextRange,
) {
    if text.is_empty() {
        return;
    }
    let visual_start = current.text.len();
    current.text.push_str(text);
    let visual_end = current.text.len();
    if current.runs.is_empty() {
        current.source_range.start = source_range.start;
    }
    current.source_range.end = source_range.end;
    current.runs.push(UiResolvedTextRun {
        kind,
        text: text.to_string(),
        source_range,
        visual_range: UiTextRange {
            start: visual_start,
            end: visual_end,
        },
        direction: resolve_direction(text, UiTextDirection::Auto),
    });
}

/// Adds rendered-only text at an existing visual boundary. The virtual run is deliberately
/// zero-width in source space so selection, IME, and hit testing continue to address the source.
pub(super) fn insert_virtual_text(
    line: &mut CandidateLine,
    visual_offset: usize,
    virtual_text: &str,
) -> bool {
    if virtual_text.is_empty()
        || visual_offset > line.text.len()
        || !line.text.is_char_boundary(visual_offset)
    {
        return false;
    }

    let mut text = String::with_capacity(line.text.len() + virtual_text.len());
    let mut runs = Vec::with_capacity(line.runs.len().saturating_add(2));
    let mut inserted = false;
    for run in &line.runs {
        if !inserted
            && run.visual_range.start <= visual_offset
            && visual_offset <= run.visual_range.end
        {
            let local_offset = visual_offset.saturating_sub(run.visual_range.start);
            if !run.text.is_char_boundary(local_offset) {
                return false;
            }
            let source_offset = if run.source_range.start == run.source_range.end {
                run.source_range.start
            } else if run.source_range.end.saturating_sub(run.source_range.start) == run.text.len()
            {
                run.source_range.start + local_offset
            } else {
                return false;
            };
            push_run_fragment(&mut text, &mut runs, run, 0, local_offset);
            let visual_start = text.len();
            text.push_str(virtual_text);
            runs.push(UiResolvedTextRun {
                kind: run.kind,
                text: virtual_text.to_string(),
                source_range: UiTextRange {
                    start: source_offset,
                    end: source_offset,
                },
                visual_range: UiTextRange {
                    start: visual_start,
                    end: text.len(),
                },
                direction: resolve_direction(virtual_text, UiTextDirection::Auto),
            });
            push_run_fragment(&mut text, &mut runs, run, local_offset, run.text.len());
            inserted = true;
        } else {
            push_run_fragment(&mut text, &mut runs, run, 0, run.text.len());
        }
    }
    if !inserted || text.len() != line.text.len().saturating_add(virtual_text.len()) {
        return false;
    }

    let inserted_visual_range = UiTextRange {
        start: visual_offset,
        end: visual_offset + virtual_text.len(),
    };
    for owner in &mut line.virtual_source_receipts {
        if owner.visual_range.start >= visual_offset {
            owner.visual_range.start = owner.visual_range.start.saturating_add(virtual_text.len());
            owner.visual_range.end = owner.visual_range.end.saturating_add(virtual_text.len());
        }
    }
    let style_owner = line.runs.iter().find_map(|run| {
        (run.visual_range.start <= visual_offset
            && visual_offset <= run.visual_range.end
            && run.source_range.start < run.source_range.end)
            .then_some(run.source_range)
    });
    line.text = text;
    line.runs = runs;
    if let Some(style_owner) = style_owner {
        line.record_virtual_source_receipt(
            inserted_visual_range,
            style_owner,
            None,
            LogicalVirtualFragmentRole::Justification,
        );
        line.virtual_source_receipts
            .sort_by_key(|owner| owner.visual_range.start);
    }
    true
}

fn push_run_fragment(
    text: &mut String,
    runs: &mut Vec<UiResolvedTextRun>,
    run: &UiResolvedTextRun,
    start: usize,
    end: usize,
) {
    let Some(fragment) = run.text.get(start..end) else {
        return;
    };
    if fragment.is_empty() {
        return;
    }
    let visual_start = text.len();
    text.push_str(fragment);
    runs.push(UiResolvedTextRun {
        kind: run.kind,
        text: fragment.to_string(),
        source_range: source_subrange(run.source_range, run.text.len(), start, end),
        visual_range: UiTextRange {
            start: visual_start,
            end: text.len(),
        },
        direction: run.direction,
    });
}

pub(super) fn push_current_line(lines: &mut Vec<CandidateLine>, current: &mut CandidateLine) {
    if !current.text.is_empty() || !lines.is_empty() {
        current.pending_break_suffix = None;
        lines.push(std::mem::replace(current, CandidateLine::empty()));
    }
}

pub(super) fn push_wrapped_line(lines: &mut Vec<CandidateLine>, current: &mut CandidateLine) {
    append_pending_break_suffix(current);
    push_current_line(lines, current);
}

pub(super) fn append_pending_break_suffix(current: &mut CandidateLine) {
    let Some(suffix) = current.pending_break_suffix.take() else {
        return;
    };
    append_virtual_discretionary_hyphen(current, suffix.kind, suffix.decision);
}

pub(super) fn append_virtual_discretionary_hyphen(
    current: &mut CandidateLine,
    kind: UiTextRunKind,
    decision: DiscretionaryHyphenDecision,
) {
    let consumed = decision.consumed_source_range();
    let anchor = decision.virtual_anchor();
    let was_empty = current.runs.is_empty();
    let visual_start = current.text.len();
    append_segment(
        current,
        kind,
        decision.marker_text(),
        UiTextRange {
            start: anchor,
            end: anchor,
        },
    );
    current.record_virtual_source_receipt(
        UiTextRange {
            start: visual_start,
            end: current.text.len(),
        },
        consumed.into(),
        Some(consumed.into()),
        LogicalVirtualFragmentRole::DiscretionaryHyphen,
    );
    if was_empty {
        current.source_range.start = consumed.start;
    }
    current.source_range.end = current.source_range.end.max(consumed.end);
}

pub(super) fn trim_word_break_trailing_spaces(line: &mut CandidateLine) {
    let mut spaces_to_trim = trailing_wrap_space_byte_len(&line.text);
    while spaces_to_trim > 0 {
        let Some(last_run) = line.runs.last_mut() else {
            break;
        };
        if !last_run.text.ends_with(' ') {
            break;
        }
        line.text.pop();
        last_run.text.pop();
        last_run.source_range.end = last_run.source_range.end.saturating_sub(1);
        last_run.visual_range.end = last_run.visual_range.end.saturating_sub(1);
        if last_run.text.is_empty() {
            line.runs.pop();
        }
        spaces_to_trim -= 1;
    }
    line.source_range.end = line
        .runs
        .last()
        .map(|run| run.source_range.end)
        .unwrap_or(line.source_range.start);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn virtual_text_keeps_later_candidate_offsets_and_source_anchors() {
        let mut line = CandidateLine::empty();
        append_segment(
            &mut line,
            UiTextRunKind::Plain,
            "سلام",
            UiTextRange { start: 0, end: 8 },
        );

        assert!(insert_virtual_text(&mut line, 2, "ـ"));
        assert!(insert_virtual_text(&mut line, 6, "ـ"));

        assert_eq!(line.text, "سـلـام");
        assert_eq!(line.source_range, UiTextRange { start: 0, end: 8 });
        assert_eq!(line.virtual_source_receipts.len(), 2);
        assert_eq!(
            line.virtual_source_receipts[0].style_source_range,
            UiTextRange { start: 0, end: 8 }
        );
        assert_eq!(
            line.virtual_source_receipts[1].style_source_range,
            UiTextRange { start: 0, end: 8 }
        );
        assert!(
            line.virtual_source_receipts
                .iter()
                .all(|receipt| receipt.replaced_source_range.is_none())
        );
        assert!(
            line.virtual_source_receipts.iter().all(|receipt| {
                receipt.virtual_role == LogicalVirtualFragmentRole::Justification
            })
        );
        assert_eq!(
            line.runs
                .iter()
                .filter(|run| run.source_range.start == run.source_range.end)
                .map(|run| run.source_range.start)
                .collect::<Vec<_>>(),
            vec![2, 4]
        );
    }

    #[test]
    fn discretionary_hyphen_records_a_typed_replacement_at_the_visual_anchor() {
        let mut line = CandidateLine::empty();
        append_segment(
            &mut line,
            UiTextRunKind::Plain,
            "pre",
            UiTextRange { start: 0, end: 3 },
        );
        let decision = crate::text::layout::soft_hyphen_break_suffix_at("pre\u{00ad}", 3)
            .expect("soft hyphen break decision");

        append_virtual_discretionary_hyphen(&mut line, UiTextRunKind::Plain, decision);

        assert_eq!(line.text, "pre-");
        let run = line.runs.last().expect("display-owned hyphen run");
        assert_eq!(run.source_range, UiTextRange { start: 5, end: 5 });
        assert_eq!(line.virtual_source_receipts.len(), 1);
        assert_eq!(
            line.virtual_source_receipts[0].virtual_role,
            LogicalVirtualFragmentRole::DiscretionaryHyphen
        );
        assert_eq!(
            line.virtual_source_receipts[0].replaced_source_range,
            Some(UiTextRange { start: 3, end: 5 })
        );
    }
}
