---
title: Runtime200 Registry Iterator Streaming
category: zircon_runtime
report_id: Runtime200-registry-iterator-streaming-2026-09-13
date: 2026-09-13
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime200 Registry Iterator Streaming

## Scope

The Runtime UI prototype-store builder only needs to visit the authoritative
asset registry; it does not need an owned sorted `Vec` before loading each
artifact. The existing canonical `entries_iter()` API already exposes the
ordered registry view without materializing that temporary collection.

## Change

- Stream `AssetRegistryIndex::entries_iter()` directly in
  `project_ui_prototype_store`.
- Preserve canonical path ordering, type filtering, artifact loading, and
  prototype alias insertion semantics.
- Keep `entries()` available for persistence and callers that explicitly need
  an owned collection.

## Complexity boundary

Prototype-store preparation removes one `O(N)` temporary pointer-vector
allocation and copy from the registry-read path; artifact loading remains the
dominant work. This is a local allocation reduction, not a claim about the
full Runtime UI startup or product latency gate.

## TDD and local evidence

- The source contract was run RED before implementation because the builder
  called `asset_registry().entries()`.
- After implementation the iterator/projection contract passes `2/2`.
- The combined Runtime UI plus Editor asset contract discovery passes
  `620/620` in `15.422s`; the focused index/input/asset slice passes `37/37` in
  `0.039s`.
- The broader Runtime/Editor performance-contract and pressure discovery passes
  `1358/1358` in `9.465s` in one process.
- The later comprehensive non-tooling Runtime/Editor loader covers 548 files
  and passes `2039/2039` in `22.986s`; the earlier receipt is retained for
  traceability.
- Scoped Python compilation, Rustfmt, `git diff --check`, and wiki validation
  pass.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_runtime/src/dynamic_api/session/runtime_ui.rs` | `0EA02048156B82657EF19E46ABE74194F369803A375634813CB286D3E3D6B91D` |
| `tools/tests/test_runtime_editor_registry_iterator_projection_performance_contract.py` | `08F57C9E8D68F85D410C28F7A459CF2C6DA7EC409D9335001411BAD6127A4E7E` |

## Managed gate

This slice joins the existing owner-attributed Runtime/Editor batch. Managed
Cargo/Release allocation and Runtime UI startup p50/p95/p99 evidence remain
pending; no standalone Cargo process or coordinator polling is required.
