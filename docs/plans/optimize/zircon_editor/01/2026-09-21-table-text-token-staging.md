---
title: Editor Table Text Token Staging
category: zircon_editor
report_id: Editor882-table-text-token-staging-2026-09-21
date: 2026-09-21
session_id: root-runtime-editor-async-optimization-20260921
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_target_met
---

# Editor882 Table Text Token Staging

## Finding

Generic retained table-row paint supports archived rows that encode four cells
inside one whitespace-delimited string. The parser first collected every token
into `Vec<&str>` even though its grammar reads at most six leading tokens. Size
normalization then collected the one- or two-token size cell into a second
temporary vector. Those borrowed-token allocations occurred before creating
the required owned display strings.

## Optimization

- Read the archived grammar into six stack-resident `Option<&str>` slots.
- Preserve the original match priority: six-token explicit-unit revision,
  five-token explicit-unit modified value, revision label, paired modified
  value/unit, four-token fallback, then whole-text fallback.
- Normalize a size cell with three bounded iterator lookaheads, distinguishing
  exactly one token, exactly two tokens, and three-or-more tokens without a
  temporary vector.
- Keep all required output strings, cell order, type/size/revision
  normalization, extra-token tolerance, and fallback text unchanged.

## TDD and deterministic evidence

The Editor882 source/model contract was observed RED at `1/5` and GREEN at
`5/5`. The lower regression covers every parser-priority branch, extra-token
handling, empty/short fallback, compact/explicit sizes, and three-token size
fallback.

For 4,096 visible archived rows, the retired path creates two borrowed-token
vectors per row (`8,192` temporary vectors); bounded lookahead creates zero.
The ignored 101-pair Release marker
`EDITOR882_TABLE_TEXT_TOKEN_STAGING_BENCH_V1` emits alternating p50/p95/p99
samples over the two staging shapes, locks the temporary-vector counts, and
requires bounded-lookahead p95 to remain within 10% of vector staging.

## Local validation boundary

- Exact-file Rustfmt and scoped `git diff --check` pass.
- Editor879–882 plus the adjacent Asset Browser contract batch passes `38/38`.
- Lower Rust execution was submitted with Editor883 in asynchronous v12 (PID
  `28704`); no per-task Cargo run is launched.
- A later one-time v12 receipt read showed Runtime passing, while Editor stopped
  before Cargo because a compile-time include resource was unavailable; no
  Editor Rust diagnostic or acceptance evidence was produced.
- The required owned output strings are intentionally retained.
- Local evidence does not establish Windows compilation, allocator behavior,
  or retained table-paint product p50/p95/p99 latency.
- Tooling production remains deferred for the later Rust migration.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_table_rows/cells/text.rs` | `BE0A3E451E2E5D659957E4955E12756856A0348B015E831B1FA6C69938592D27` |
| `zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_table_rows/cells/text/token_staging_tests.rs` | `73B3282E53395E5D53B77CB51DB240F010541CDDFCAEEBECE7ED26D2F3A40BDD` |
| `tools/tests/test_editor882_table_text_token_staging_performance_contract.py` | `7F82940ABA325D04D253E74A1D26F1803F2C00E7463728A86FF6D74B3603C7B2` |

## Acceptance boundary

Keep this record `implementation_complete` / `managed_validation_pending` until
a combined current-source Windows lane compiles Editor, executes the lower
regression and ignored Release marker, and supplies allocator plus real table
paint product p50/p95/p99 evidence. The deterministic temporary-vector model is
not product acceptance.
