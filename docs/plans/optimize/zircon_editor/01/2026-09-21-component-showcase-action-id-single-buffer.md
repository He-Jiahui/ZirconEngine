---
title: Editor Component Showcase Action ID Single Buffer
category: zircon_editor
report_id: Editor889-component-showcase-action-id-single-buffer-2026-09-21
date: 2026-09-21
session_id: root-runtime-editor-async-optimization-20260921
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_target_met
---

# Editor889 Component Showcase Action ID Single Buffer

## Finding

Component-showcase fallback action IDs normalized every non-empty path segment
into a separate owned `String`, collected the strings into a temporary vector,
and joined the vector. The prefixed route also created a normalized child
string before formatting the final output. Dense binding paths therefore paid
per-segment allocations and vector slots even though only one action ID survives.

## Optimization

- Allocate one action-ID buffer from a saturating input-length bound and append
  both prefixed and fallback routes into it.
- Share an append-oriented camel-to-snake helper with the existing owned helper,
  retaining its prior in-place trailing-separator behavior.
- Keep delimiter placement tied to the raw non-empty segment position, rather
  than output emptiness, so a segment that normalizes to empty still retains the
  exact legacy dot produced by `join`.
- Preserve prefix recognition, separator set, empty-segment filtering, ASCII
  normalization, Unicode behavior, source order, and empty output exactly.

## TDD and deterministic evidence

The combined Editor889/Runtime870 source-model batch was observed RED at `2/10`
and GREEN at `10/10`. Lower regressions compare prefixed, punctuation-heavy,
mixed-separator, repeated-separator, empty-normalized-segment, empty, and Unicode
inputs against the retired collect/join implementation. Existing Editor87
camel-to-snake coverage remains wired through the shared append owner.

Across 4,096 dense fallback renders with 64 segments, the deterministic model
changes temporary child strings and vector slots from `262144/262144` to `0/0`;
the required final action-ID string remains. Ignored marker
`EDITOR889_COMPONENT_SHOWCASE_ACTION_ID_SINGLE_BUFFER_BENCH_V1` emits 101
alternating p50/p95/p99 sample pairs and requires single-buffer p95 to remain
within 10% of collect/join.

## Local validation boundary

- Exact-file Rustfmt, Python bytecode compilation, and scoped
  `git diff --check` pass.
- Editor889 and Runtime870 contracts pass `10/10`; the combined batch with the
  adjacent diagnostic-log M0 and Editor test-infrastructure contracts passes
  `25/25`.
- Editor889 received no per-task Cargo run and was submitted with Runtime870 in
  asynchronous v16 (PID `34696`) at `2026-09-21T21:30:54.1471688+08:00`.
- No v13-v16 receipt was read or monitored after submission.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_editor/src/ui/retained_host/app/pane_surface_actions/component_showcase/bindings.rs` | `FF12259D7FC8F2450E318E20D75EC8447A01BB6EB2540B54C663D67EB1CEB18A` |
| `zircon_editor/src/ui/retained_host/app/pane_surface_actions/component_showcase/bindings/action_id_single_buffer_tests.rs` | `C661F6D08A65681A112706FE36E6B821BED89599EC7AECBFE832BF7B014FF83F` |
| `tools/tests/test_editor889_component_showcase_action_id_single_buffer_performance_contract.py` | `B71F8A8F0A75424679F67BEADDB8951F46AA5CD22581987ECCC1B48588831871` |

## Acceptance boundary

Keep this record `implementation_complete` / `managed_validation_pending` until
a combined current-source Windows lane compiles Editor, executes the lower
regression and ignored Release marker, and supplies allocator plus real
component-showcase action-routing p50/p95/p99 evidence. The deterministic
allocation model is not product acceptance.
