---
title: Editor08 default command registry direct append
category: zircon_editor
report_id: Editor811-default-command-registry-direct-append-2026-09-19
date: 2026-09-19
session_id: root-runtime-editor-async-optimization-20260919
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_target_met
---

# Editor811 · default command registry direct append

## Scope

`default_workbench_commands` assembled each built-in command group into a
temporary vector and then extended a second vector. The registry is created
at editor startup and contains a fixed 66-command catalog, so the old shape
paid for six heap-backed group vectors plus repeated growth of the outer
collector.

## Implementation

- Reserve the fixed built-in registry bound once with
  `DEFAULT_WORKBENCH_COMMAND_CAPACITY`.
- Convert file, edit, selection, runtime, view, window, and animation groups
  to append directly into the caller-owned vector.
- Preserve command order, IDs, menu paths, key chords, predicates, event
  payloads, and all existing command descriptor construction.
- Add a lower Rust cardinality/capacity regression and the ignored
  `EDITOR811_DEFAULT_COMMAND_DIRECT_APPEND_BENCH_V1` Release marker.

## TDD and deterministic model

The Python source/model contract was intentionally RED against the previous
`vec!`/`extend(group())` shape and GREEN after the direct-append refactor. A
66-command outer collector would cross six geometric growth boundaries under
the old empty-vector model; the reserved bound models zero outer growth and
removes six temporary heap-backed group vectors. This is allocation-shape
evidence only, not allocator, RSS, CPU, or product p50/p95/p99 evidence.

## Local evidence

- Focused source/model contract:
  `tools/tests/test_editor_default_command_registry_direct_append_performance_contract.py`
  (`4/4`).
- Combined command-boundary static batch (new registry contract plus existing
  command palette, toolkit, settings-wiring, and product-interaction
  contracts) passes `29/29` with zero failures, errors, or skips.
- The strict non-tooling performance/pressure batch (tooling, export, and
  coordinator paths excluded) loads `680` files and passes `2635/2635` tests
  in `43.429s`, with zero failures, errors, or skips. The merged batch includes
  the Runtime804/807/808 and Editor805-811 contracts.
- The subsequent Editor812-inclusive rerun loads `681` files and passes
  `2639/2639` tests in `47.359s`, with zero failures, errors, or skips; the
  ten-slice focused Runtime/Editor batch is now `44/44`.
- Exact-file Rustfmt, Python compilation, and scoped diff checks pass.
- Lower Rust source regression and ignored Release marker are wired in
  `zircon_editor/src/core/commands/defaults.rs`.
- Managed Cargo/Release and command-registry product percentile evidence remain
  pending; tooling production remains deferred for the later Rust migration.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_editor/src/core/commands/defaults.rs` | `96367D04650EF79EC7EB9EA28172BB7B6052F6598E300172839732A72607EB27` |
| `tools/tests/test_editor_default_command_registry_direct_append_performance_contract.py` | `D538F93834A6A8FDD66423728AC38A31BE65308D2AC348C4E6B4B3BB5354B317` |

## Acceptance boundary

Keep this record `implementation_complete` / `managed_validation_pending`
until the owner-attributed Windows Release batch compiles the current Editor
tree, runs the lower regression and ignored marker, and supplies command
registry allocation and product p50/p95/p99 evidence. No coordinator status is
polled by this session.
