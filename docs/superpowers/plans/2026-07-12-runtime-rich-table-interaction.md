# Runtime Rich-Table Interaction Implementation Plan

**Goal:** Close RT-M8 rich-table link interaction through the shared resolved text hit-test owner.

**Architecture:** Add full physical-frame candidate selection before the existing writing-mode fallback. Keep parser link ranges, affinity resolution, surface effects, and host requests unchanged.

## RT-M8 S0: Failing contracts

- [x] Add HorizontalTb and VerticalRl table-link tests to `ui/text/rich_text/tests.rs`.
- [x] Add a negative padding/background click contract.
- [x] Add one surface primary-release table-link dispatch contract.
- [x] Run only the new exact filters and record the expected failures before production code.

## RT-M8 S1: Shared hit-test repair

- [x] Add a private full-frame line candidate lookup in `ui/text/hit_test.rs`.
- [x] Use it before the existing nearest horizontal row / VerticalRl column fallback.
- [x] Keep affinity, grapheme, visual/source mapping, and non-table behavior unchanged.
- [x] Do not inspect table boxes, BBCode syntax, or `RichTable` DTOs in hit testing.

## RT-M8-T: Validation and records

- [x] Run exact-file rustfmt and scoped diff checks.
- [x] Run current-source production check and focused table-link tests on Windows. (Focused current-source binary is green; the later production retry is externally blocked by concurrent Environment API drift.)
- [x] Run the existing ordinary rich-link regressions.
- [x] Record any external lib-test compile blocker separately from focused results.
- [x] Update Text07 output records, detailed module docs, concise plan status, and the active session note.
- [x] Keep the overall Goal active for Text03/Text05 gaps.

## Status

| Slice | Status | Evidence |
|---|---|---|
| RT-M8 S0 | completed | Two table-link contracts failed on the old single-axis candidate selection; padding/background negative control passed. |
| RT-M8 S1 | completed | Full physical line-frame containment now precedes the unchanged nearest-axis caret fallback. |
| RT-M8-T | completed-focused | Horizontal/Vertical table links 2/2, padding 1/1, surface table host request 1/1, and two ordinary rich-link regressions pass on a current-source Windows binary. Later production retry is externally blocked by concurrent Environment API drift. |
