---
title: Runtime73 Resource Dependency Path Buffer
category: zircon_runtime
report_id: Runtime73-resource-dependency-path-buffer-2026-08-26
date: 2026-08-26
session_id: root-runtime-events-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime73 Resource Dependency Path Buffer

## Scope

This slice removes recursive full-path string construction while collecting UI document resource
dependencies. Root, import, token, component, stylesheet, node, slot, array, and table paths;
resource kind inference; first-reference ordering; validation diagnostics; import source attribution;
and dependency output remain unchanged. It supports the dependency work behind `RST-G27` without
claiming completion of style generation, precise invalidation, reload, or product-scale gates.

## Change

- Keep one mutable path buffer for each root or imported document traversal.
- Append dot segments or array indices in place and truncate to the saved prefix after recursion.
- Clone a path only when a validated resource dependency must own it.
- Preserve imported widget/style prefixes and every prior diagnostic path literal.

## Deterministic Performance Evidence

| 512 nested tables, 32 traversals per sample | Before | After |
|---|---:|---:|
| Recursive full-path constructions per sample | 16,384 | 0 |
| Path bytes written by recursive traversal per sample | 46,325,760 | 360,640 |
| Final owned dependency paths per sample | 32 | 32 |
| Traversal-order changes | 0 | 0 |

The ignored release gate runs 17 alternating sample pairs and emits
`RUNTIME73_RESOURCE_DEPENDENCY_PATH_BUFFER_BENCH_V1`. Acceptance requires backtracking traversal
P95 to be at least 80% below recursive full-path construction. Exact Windows timings remain
pending the coordinator run.

## Acceptance

- `optimization_batch_20260826ba_resource_dependency_path_buffer_preserves_paths` covers nested
  table/array traversal, final URI, path restoration, and output ownership.
- `optimization_batch_20260826ba_resource_dependency_uses_backtracking_path_buffer` requires the
  shared append/truncate implementation and rejects recursive table/array path `format!` calls.
- `optimization_batch_20260826ba_resource_dependency_path_buffer_p95` reports paired release
  P50/P95 samples and enforces the 80% P95 reduction gate.

## Remaining Parent-plan Work

Runtime73 still owns typed style schema, compiled cascade authority, component scope, precise
mutation dependency indexes, theme generation/reload, transitions, and Runtime/Editor parity. This
slice only converges resource dependency path traversal.
