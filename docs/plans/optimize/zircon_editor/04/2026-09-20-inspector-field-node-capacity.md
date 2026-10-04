---
title: Editor857 inspector field node capacity
category: zircon_editor
report_id: Editor857-inspector-field-node-capacity-2026-09-20
date: 2026-09-20
session_id: root-runtime-editor-async-optimization-20260920
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_target_met
---

# Editor857 - inspector field node capacity

## Scope

The retained Editor Inspector projection always emits a fixed base set of
editable nodes and may append a header, optional diagnostic, and dynamic
property nodes for every plugin component. The outer and per-component vectors
previously started at zero capacity despite those bounds being available from
the view model.

## Optimization

- Reserve the fixed nine-node Inspector base plus the exact plugin-component
  node upper bound and one optional fallback/empty row for the outer projection.
- Reserve each plugin-component projection from one header, its optional
  diagnostic, and its property count before direct append.
- Preserve node order, fallback/empty-row exclusivity, control identities,
  dynamic property payloads, and action-button placement.

## TDD and deterministic evidence

The Python source/model contract was run RED before the capacity declarations
and lower owner existed (two structural/wiring failures), then GREEN at `4/4`.
The lower Rust owner checks component order, empty projection behavior, and
carries the ignored `EDITOR857_INSPECTOR_FIELD_NODE_CAPACITY_BENCH_V1` marker. A
1,024-component dense model changes the modeled inspector growth events from
`18` to `0`.

## Local validation

- Standalone `rustc --edition 2021 --test` execution of the lower owner passes
  `2/2` non-ignored tests after correcting the modeled plugin-node count; the
  single benchmark marker remains intentionally ignored for the managed Windows
  Release lane.
- `tools/tests/test_editor_inspector_field_node_capacity_performance_contract.py`:
  `4/4`.
- The combined Runtime/Editor focused batch covering the preceding V2/render/
  material slices plus Runtime857, Runtime858, and Editor857 passes `38/38`
  tests in `0.021s`, with zero failures, errors, or skips.
- After Runtime859 and Editor858 joined the batch, the current combined focused
  invocation passes `46/46` tests in `0.020s`, with zero failures, errors, or
  skips.
- Exact-file `rustfmt --edition 2021 --check` passes for the production and
  lower Rust owners; `python -m py_compile` passes for all three new contracts.
- The expanded non-tooling source-contract loader covers `965` files and passes
  `4084/4084` tests in `137.924s` under the explicit performance-or-contract
  filename filter (tooling, export, and coordinator files excluded), with zero
  load errors, failures, errors, or skips.
- The post-Runtime859/Editor858 expanded receipt covers `967` files and passes
  `4092/4092` tests in `375.582s`; two shader-prewarm Cargo command lines were
  printed by fixture tests and are not managed Windows Release/Cargo acceptance.
- No managed Windows Cargo/Release validation command was started locally.
  Allocator and Inspector product p50/p95/p99 evidence remain pending behind the
  shared external worktree gate; isolated contract fixtures are not a managed
  acceptance result.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_editor/src/ui/retained_host/ui/pane_data_conversion/inspector_fields.rs` | `85BE9FC23025175F1002904D3D0D17FDC9C8139D87DFBD7FFC46AD2826E5CA90` |
| `zircon_editor/src/ui/retained_host/ui/pane_data_conversion/inspector_fields/capacity_tests.rs` | `BE40CDC87CECF2FE300EB1CD5E25A96C6EF9143AC2C12A3C722CC4F7CD8E7A87` |
| `tools/tests/test_editor_inspector_field_node_capacity_performance_contract.py` | `D96B8178C70361088D3F196BB353AB9F3A5A6F2B223DFFABD3F19BC7FF33811B` |

## Acceptance boundary

Keep this record `implementation_complete` / `managed_validation_pending` until
the owner-attributed Windows Release lane compiles the current Editor tree,
executes the lower regression and ignored marker, and supplies allocator plus
Inspector product p50/p95/p99 measurements. Tooling production remains deferred
for the later Rust migration; coordinator status is not polled here.
