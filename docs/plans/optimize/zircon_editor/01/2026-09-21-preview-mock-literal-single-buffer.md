---
title: Editor Preview Mock Literal Single Buffer
category: zircon_editor
report_id: Editor885-preview-mock-literal-single-buffer-2026-09-21
date: 2026-09-21
session_id: root-runtime-editor-async-optimization-20260921
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_target_met
---

# Editor885 Preview Mock Literal Single Buffer

## Finding

UI Asset preview-mock array and table literals recursively formatted every
child into a separate owned `String`, collected those strings into a temporary
vector, joined them, and then wrapped the joined output. Large mock collections
therefore allocated temporary child text and vector storage at every recursive
array level.

## Optimization

- Route array and table formatting through one recursive append owner backed by
  a single output `String`.
- Append array delimiters and child values directly in source order.
- Retain the table's borrowed-entry sort vector so deterministic lexical key
  order is unchanged, while streaming each key/value rendering into the output
  instead of building a second `Vec<String>`.
- Preserve raw top-level string display, quoted inline string escaping, scalar
  formatting, empty array/table spelling, nesting, and all preview-mock callers.

## TDD and deterministic evidence

The Editor885 source/model contract was observed RED at `1/5` and GREEN at
`5/5`. Lower regressions compare exact nested output with the retired recursive
formatter and lock raw/quoted string, lexical table order, empty array, and
empty table behavior.

For a 4,096-scalar array, the deterministic model changes temporary child
strings from `4096` to `0` and temporary vector slots from `4096` to `0`; the
required output buffer remains. Table sort-index storage is intentionally not
claimed as removed. Ignored marker
`EDITOR885_PREVIEW_MOCK_LITERAL_SINGLE_BUFFER_BENCH_V1` emits 101 alternating
p50/p95/p99 sample pairs, checks full text equality, and requires the streamed
p95 to remain within 10% of collect/join formatting.

## Local validation boundary

- Exact-file Rustfmt and scoped `git diff --check` pass.
- Editor879–885 plus adjacent preview/static contracts pass `68/68`.
- Editor885 received no per-task Cargo run and was submitted with Editor884 in
  asynchronous v13 (PID `29732`).
- Local evidence does not establish Windows compilation, allocator behavior,
  or preview-mock product p50/p95/p99 latency.
- Tooling production remains deferred for the later Rust migration.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_editor/src/ui/asset_editor/preview/preview_mock/entries.rs` | `5D9620D765B40CC8F24B39305B105463ED646EB5C8E5F464E6458B0B1E05ECFF` |
| `zircon_editor/src/ui/asset_editor/preview/preview_mock/entries/literal_single_buffer_tests.rs` | `6549E9EF80616E9A2BD2A2C83F296EFFF06A86972D6E0AC281658F7032019334` |
| `tools/tests/test_editor885_preview_mock_literal_single_buffer_performance_contract.py` | `33E8EED02C6A52C139F07C85AFED93691544335D9E4E6C4D224A8E19EAD224D0` |

## Acceptance boundary

Keep this record `implementation_complete` / `managed_validation_pending` until
a combined current-source Windows lane compiles Editor, executes the lower
regression and ignored Release marker, and supplies allocator plus real UI
Asset preview p50/p95/p99 evidence. The deterministic allocation model is not
product acceptance.
