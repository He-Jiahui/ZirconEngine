---
title: Runtime200 Dynamic Pointer State Hash Lookup
category: zircon_runtime
report_id: Runtime200-dynamic-pointer-state-hash-2026-09-13
date: 2026-09-13
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime200 Dynamic Pointer State Hash Lookup

## Scope

`RuntimeUiSurfaceSet` keeps cross-surface pointer capture and last-position
state keyed by `Option<u64>`. The tables are lookup/update stores; no caller
observes key ordering. Their `BTreeMap` implementation therefore adds ordered
tree work to every pointer move, capture, and cleanup probe.

## Change

- Replace the private capture and position tables with `HashMap`.
- Update the capture helper signatures and behavior fixtures to use the same
  exact-key map type.
- Preserve `None` as the ordinary mouse key, capture cleanup on Down/Up/Cancel,
  and all fallback/dispatch ordering.

## Complexity boundary

Pointer capture and position probes move from `O(log P)` ordered-map lookup to
expected `O(1)` hashing for `P` active pointer keys. Surface routing, hit-test
publication, pointer identity qualification, and product p50/p95/p99 gates
remain unchanged and still require managed validation.

## TDD and local evidence

- The source contract was run RED before implementation: both private tables,
  helper signatures, and the behavior fixture still used `BTreeMap`.
- After implementation the focused hash-state contract passes `3/3`; the
  adjacent pointer/input contract batch passes `23/23`.
- The refreshed broader Runtime/Editor performance-contract and pressure
  discovery passes `1364/1364` in `8.303s` in one process; the combined UI/asset
  batch passes `623/623` in `15.573s`.
- The later comprehensive non-tooling Runtime/Editor loader covers 548 files
  and passes `2039/2039` in `22.986s`; the earlier receipts are retained for
  traceability.
- Scoped Python compilation, Rustfmt, `git diff --check`, and wiki validation
  remain required before record finalization.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_runtime/src/dynamic_api/session/runtime_ui.rs` | `0EA02048156B82657EF19E46ABE74194F369803A375634813CB286D3E3D6B91D` |
| `zircon_runtime/src/dynamic_api/session/runtime_ui/input_routing.rs` | `831FCD0C8A55CD00BB1C0336616E1785286FF9DE8C6A00F740F50C38CAD43D5F` |
| `zircon_runtime/src/dynamic_api/session/runtime_ui/tests.rs` | `E5AC5AFA45C0DB3BC18A618AF6662E16515F20D15D81C49E23325FD716377846` |
| `tools/tests/test_runtime_dynamic_pointer_state_hash_performance_contract.py` | `A4D8F295C170CF0FF5C3C55863E3A9AB87EFB4C915E3E33472D01DEDC234DE27` |

## Managed gate

This slice joins the existing owner-attributed Runtime/Editor batch. Managed
Cargo/Release allocation and pointer-input p50/p95/p99 evidence remain
pending; no standalone Cargo process or coordinator polling is required.
