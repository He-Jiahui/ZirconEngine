# Runtime11B Unterminated Rich-Marker Frontier Optimization Record

- Date: 2026-08-19
- Owner: `runtime11b-linear-rich-tokenizer-r1-01a00797-20260819`
- Source plan: `docs/plans/optimize/zircon_runtime/11b-runtime-text-font-shaping-layout-editing-ime-review.md`, P1-26 / M8
- Status: implementation and 21-pair measurement repair complete; combined managed validation pending

## Problem

The HTML and BBCode parsers called their token scanner at every opener. When a
suffix contained no closing `>` or `]`, every call searched the whole remaining
suffix before advancing by one marker. A corpus of N unterminated openers
therefore performed `N * (N + 1) / 2` delimiter-search visits. Markdown also
performed a failed closing-marker search even when a delimiter could no longer
have a later match.

## Change

- Each parser computes the last possible closing-delimiter start once.
- HTML and BBCode stop token attempts as soon as the monotonic parse cursor is
  beyond that frontier, then preserve the remaining suffix as literal text.
- Markdown precomputes separate `**`, `*`, and backtick frontiers and skips a
  failed suffix search when no later closer can exist.
- Valid token parsing, malformed-token recovery before the frontier, decorator
  behavior, and the public parse result remain unchanged.

## Deterministic Performance Evidence

| Workload | Before | After | Reduction |
|---|---:|---:|---:|
| 100,000 unterminated `<` markers | 5,000,050,000 delimiter-search visits | 100,000 frontier visits; 0 token suffix scans | 99.998% |
| 100,000 unterminated `[` markers | 5,000,050,000 delimiter-search visits | 100,000 frontier visits; 0 token suffix scans | 99.998% |
| 20,000-marker release benchmark fixture | 200,010,000 delimiter-search visits | 20,000 frontier visits | 99.990% |
| Unterminated-suffix delimiter search | O(N^2) | O(N) | one full complexity class |

The visit figures cover closing-delimiter discovery. Literal projection and
grapheme alignment remain linear in the visible output and are intentionally
included in the measured new-parser timing.

## Acceptance

- `text_rich_unterminated_marker_corpus_finishes_at_scale` exercises 100,000
  `<`, `[`, and `*` markers through the public parser surface.
- `text_rich_unterminated_marker_release_benchmark_evidence` runs 21 paired,
  alternating release samples per HTML and BBCode corpus and emits legacy/new
  timing distributions.
- Timing gate: frontier-parser P95 must be no more than 25% of the conservative
  legacy delimiter-scan P95 for both formats.
- `rustfmt +1.94.1 --edition 2021 --check` on the two touched Rust files: passed.
- `git diff --check` on the two touched Rust files: passed.
- Cargo tests and release P50/P95: pending the next multi-task Windows
  coordinator batch; no direct Cargo command was started.

## Remaining M8 Scope

This milestone closes the explicit 100k unterminated-marker complexity path;
it does not close P1-25 or P1-27. Unified input/token/node/depth/time budgets,
bounded structured diagnostics, versioned format feature matrices, deep-tag
limits, and decorator cost isolation remain required before Runtime11B M8 can
be marked complete.
