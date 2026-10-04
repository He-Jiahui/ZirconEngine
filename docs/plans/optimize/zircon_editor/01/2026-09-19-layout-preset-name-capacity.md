---
title: Editor01 layout preset name capacity
category: zircon_editor
report_id: Editor830-layout-preset-name-capacity-2026-09-19
date: 2026-09-19
session_id: root-runtime-editor-async-optimization-20260919
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_target_met
---

# Editor830 · layout preset name capacity

## Scope

`EditorUiHost::preset_names` merges names discovered from project asset URIs
with persisted layout-preset keys, then sorts and deduplicates them. The two
source lengths are known before publication, but the result vector previously
started at zero capacity and grew geometrically for dense projects.

## Implementation

- Materialize the asset URI list and persisted preset map once.
- Reserve their combined length as a safe upper bound before extending either
  source.
- Preserve source order before the existing sort, final lexical order, and
  deduplication semantics; empty inputs retain zero capacity.
- Add a lower order/capacity regression and the ignored
  `EDITOR830_LAYOUT_PRESET_NAME_CAPACITY_BENCH_V1` Release marker.

The bound is a local Editor projection capacity, not a shared Runtime contract;
the constant-free length expression stays at the owning helper boundary.

## TDD and deterministic model

The Python source/model contract was intentionally RED against the old
zero-capacity collector and GREEN after the combined-bound reservation was
added (`4/4`). For 4,096 asset names plus 4,096 persisted names, the
zero-capacity model changes `12` geometric growth events to `0`; sorting,
deduplication, and the empty path remain unchanged. This is allocation-shape
evidence only, not allocator, CPU, RSS, or product percentile evidence.

## Local evidence

- `tools/tests/test_editor_layout_preset_name_capacity_performance_contract.py`:
  `4/4`.
- Lower Rust module
  `zircon_editor/src/ui/host/optimization_batch_editor830_layout_preset_capacity_tests.rs`
  covers merged ordering/deduplication, empty capacity, and the ignored Release
  marker.
- Exact-file `rustfmt --edition 2021 --check` passes for the production and
  lower Rust files; Python compilation passes for the source contract.
- The current one-process non-tooling Runtime/Editor contract batch loads
  `627` modules and passes `2239/2239` tests in `5.505s` with zero failures,
  errors, or skips; the nine-slice focused loader passes `33/33` in `0.017s`.
- Managed Windows Cargo/Release compilation, lower Rust execution, allocator
  evidence, and layout-preset product p50/p95/p99 remain pending behind the
  external dirty-worktree admission gate. Tooling production remains deferred
  for the later Rust migration.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_editor/src/ui/host/layout_persistence.rs` | `6C7F0DB2ACFAC532D901904137D5DC4A0CC365F74B07A74A00EED8F21B68BE77` |
| `zircon_editor/src/ui/host/optimization_batch_editor830_layout_preset_capacity_tests.rs` | `D8468D89BD8C5EFB82E7023A8796E3B72888E9616F99C9C637B8B1B383A9CFA5` |
| `tools/tests/test_editor_layout_preset_name_capacity_performance_contract.py` | `BFBE8FE058D5522221AF761C209F17936806B02BD80B09BF5CEDDB0EB36A18C3` |

## Acceptance boundary

Keep this record `implementation_complete` / `managed_validation_pending` until
the owner-attributed Windows Release batch compiles the current Runtime/Editor
tree, executes the lower regression and ignored marker, and supplies allocator
plus layout-preset product p50/p95/p99 evidence. Local source/model evidence
does not claim final performance acceptance.
