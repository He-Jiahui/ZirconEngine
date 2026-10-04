---
title: Runtime82 Grapheme Aligned Edit Replacement
category: zircon_runtime
report_id: Runtime82-grapheme-aligned-edit-replace-2026-09-26
date: 2026-09-26
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: release_profile_pending
---

# Runtime82 grapheme aligned edit replacement

## Scope

The editable-state replacement helper received already aligned byte offsets from
selection insert, caret insert, Backspace, and Delete. It then called a second
helper that ran `clamp_grapheme_boundary` on both offsets again. On a Unicode
field, each clamp enumerates graphemes from the start, so selection replacement
performed four scans where two were sufficient.

The four callers are local to `edit_state.rs`: selection insert clamps both
range endpoints; caret insert clamps the caret; Backspace clamps the caret and
gets the prior boundary from the shared grapheme navigator; Delete clamps the
caret and gets the next boundary from that navigator. These paths now pass the
verified offsets to an aligned replacement helper. The composition restore and
preedit paths keep the defensive `replace_range_preserving_composition` clamp
because their stored or external ranges can be stale. Text, caret, selection,
composition clearing, and committed edit intents retain their prior behavior.

## Evidence and target

Tests compare selected and caret insert against a test-local copy of the former
double-clamp path using mixed Prepend, combining, CRLF, and ASCII text; they
also check Backspace and Delete ranges on a combining cluster. The ignored
Release marker `RUNTIME82_UNICODE_SELECTION_REPLACE_SINGLE_CLAMP_BENCH_V1`
compares old and new selection replacement with exact output parity, five
warmups, 31 alternating paired samples, and 128/1,024/8,192 combining
graphemes. Each timed operation replaces the selected trailing nonempty
combining grapheme with another equal-byte-length combining grapheme and
alternates the two values; odd iteration counts leave a changed final value
that is checked outside timing. It retains old/new raw nanosecond samples and reports nearest-rank
p50/p95/p99 plus OS, architecture, package version, and source byte count. The
1,024 and 8,192 cases require optimized p95 at most 75% of the old path. The
128 case is diagnostic because fixed dispatch cost dominates it.

This is a local editing helper target. It does not establish million-character
product editing latency, allocations, RSS, IME session authority, App/Editor
integration, or the Runtime82 qualification gates.

## Validation manifest

Include the non-ignored `unicode_selection_and_caret_replacement_match_the_legacy_clamped_path`
and `unicode_backspace_and_delete_keep_exact_grapheme_ranges` tests in the
grouped managed Runtime lib batch. Run the ignored Release marker in the
grouped performance batch and retain its printed per-scale raw samples and
p50/p95/p99. Static checks
alone do not satisfy this acceptance boundary.
