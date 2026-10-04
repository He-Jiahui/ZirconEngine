---
title: Editor123 template binding-ID projection capacity
category: zircon_editor
report_id: Editor837-template-binding-id-capacity-2026-09-19
date: 2026-09-19
session_id: root-runtime-editor-async-optimization-20260919
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_target_met
---

# Editor837 · template binding-ID projection capacity

## Scope

The retained template projection already knows the exact authored binding/event
count for each node, but both `project_node` and `project_v2_binding_ids`
started their returned ID vectors at zero capacity. Dense templates therefore
paid geometric growth and copied the same ID strings while preserving the
existing binding-resolution and traversal semantics.

## Implementation

- Reserve `node.bindings.len()` before collecting authored binding IDs.
- Reserve `node.events.len()` before collecting V2 event binding IDs.
- Keep binding resolution order, returned ID order, global binding projection
  order, error propagation, and empty-node zero-capacity behavior unchanged.
- Wire a lower source/count regression and the ignored
  `EDITOR837_TEMPLATE_BINDING_ID_CAPACITY_BENCH_V1` Release marker.

## TDD and deterministic model

The Python source/model contract was intentionally RED against the two
`Vec::new()` collectors and GREEN after the exact input-length reservations
were added. A dense 4,096-entry model changes the geometric growth count from
`11` to `0` for either collector. This is allocation-shape evidence only; it
is not allocator, CPU, RSS, or product p50/p95/p99 evidence.

## Local evidence

- Focused source/model contract:
  `tools/tests/test_editor_template_binding_id_capacity_performance_contract.py`
  (`3/3`).
- Lower order/capacity regression and ignored Release marker:
  `zircon_editor/src/ui/template_runtime/runtime/projection/binding_capacity_tests.rs`.
- Exact-file Rustfmt and Python compilation pass. A batched Runtime/Editor
  source/model recheck covering this slice and twelve adjacent contracts passes
  `49/49` with zero failures, errors, or skips in one process. Managed
  Cargo/Release and product percentile evidence remain pending.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_editor/src/ui/template_runtime/runtime/projection.rs` | `111477312FDAB4487E65E214B5EC95110D79FFBF5B8D71A3C6875E1501289971` |
| `zircon_editor/src/ui/template_runtime/runtime/projection/binding_capacity_tests.rs` | `0DC54E34E42635B2345084AD555A382B0688907AC4820987D50CCA95AFEBA330` |
| `tools/tests/test_editor_template_binding_id_capacity_performance_contract.py` | `CBDC8A5BAA6D8D7D6D25F803BF2EE50CC90F1A313B3C83057666439E8B6BCD4B` |

## Acceptance boundary

Keep this record `implementation_complete` / `managed_validation_pending` until
the owner-attributed Windows Release batch compiles the current Runtime/Editor
tree, executes the lower regression and ignored marker, and supplies Editor
allocation plus product p50/p95/p99 evidence. Tooling production remains
deferred for the later Rust migration.
