# Runtime11B Rich Active-Tag Index Optimization Record

- Date: 2026-08-19
- Owner: `runtime11b-linear-rich-tokenizer-r1-01a00797-20260819`
- Source plan: `docs/plans/optimize/zircon_runtime/11b-runtime-text-font-shaping-layout-editing-ime-review.md`, P1-25 / M8
- Status: implementation and 21-pair measurement repair complete; combined managed validation pending

## Problem

HTML and BBCode kept active style tags in a `Vec`. Every close token used a
reverse linear search. With D active tags and C mismatched close tokens, the
parser performed `D * C` tag-name comparisons while leaving the stack
unchanged. Deep hostile input could therefore turn close resolution into
O(D * C) work in addition to tokenization.

## Change

- `ActiveTagStack` retains the allocation-free vector-only path through 32
  active tags.
- When depth exceeds 32, it builds one `tag name -> positions` index and uses
  O(1)-average lookup for subsequent close tokens.
- Push, duplicate-name close, and truncation update the same index. Removing a
  deep suffix visits each removed tag once, so total maintenance is amortized
  linear rather than repeated full-stack search.
- Style/link selection still reads the last active tag and public parse output
  is unchanged.

## Deterministic Performance Evidence

| Workload | Before | After | Reduction |
|---|---:|---:|---:|
| Depth 10,000 plus 10,000 mismatched closes | 100,000,000 tag comparisons | 10,000 one-time index-build visits + 10,000 hash lookups | 99.98% fewer linear tag visits |
| Depth 5,000 release benchmark fixture | 25,000,000 tag comparisons | 5,000 build visits + 5,000 hash lookups | 99.96% fewer linear tag visits |
| Mismatched close resolution above threshold | O(depth) per close | O(1) average per close | one full complexity class |
| Common active depth 0-32 | vector reverse search | vector reverse search | no index allocation |

## Acceptance

- `text_rich_mismatched_close_corpus_keeps_the_active_style_at_scale` runs
  10,000 opens and 10,000 mismatched closes for both HTML and BBCode.
- `text_rich_deep_duplicate_closes_keep_the_active_tag_index_consistent`
  exercises indexed duplicate positions and repeated truncation.
- `text_rich_active_tag_index_release_benchmark_evidence` emits 21-pair,
  alternating legacy and indexed timing distributions for both formats.
- Timing gate: indexed parser P95 must be no more than 25% of the conservative
  legacy reverse-search P95 for HTML and BBCode.
- Exact-file Rustfmt and scoped `git diff --check`: passed.
- Cargo tests and release P50/P95: pending the combined Runtime11B Windows
  coordinator batch; no per-task Cargo command was started.

## Remaining Scope

The index bounds lookup complexity, not nesting memory. The M8
`RichParseBudget`, typed budget diagnostics, maximum nesting policy, and
decorator execution limits remain open and must be implemented before P1-25 is
closed end to end.
