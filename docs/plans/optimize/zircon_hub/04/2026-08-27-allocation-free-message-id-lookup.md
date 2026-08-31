---
title: Hub04 Allocation-Free Message ID Lookup
category: zircon_hub
report_id: Hub04-allocation-free-message-id-lookup-2026-08-27
date: 2026-08-27
session_id: root-hub04-two-task-performance-batch-r3-20260831
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Hub04 Allocation-Free Message ID Lookup

## Scope

Structured Hub message deserialization resolves every stable string ID through
`HubMessageId::from_str_id`. The previous implementation materialized `HubMessageId::all()` for
each lookup, allocating and populating a 146-entry vector before searching it linearly.

The lookup now splits the stable ID at its namespace separator and scans only the matching static
category slice. The eight namespaces are unique and already form part of the persisted string-ID
contract, so valid IDs preserve their exact mapping and order is irrelevant to the lookup.
`HubMessageId::all()` remains unchanged for exhaustive inventory and localization tests. Missing
separators, unknown namespaces, and unknown IDs still return `None`, preserving the existing raw
text fallback during deserialization.

## Performance Evidence

The isolated release model mirrors the current 146 IDs across eight namespaces and performs 8,192
lookups. It runs 31 alternating sample pairs. The model was compiled with `rustc -O` on Windows.

| Metric | Materialized full inventory | Static namespace slice | Change |
|---|---:|---:|---:|
| Allocator calls per sample | 8,192 | 0 | -100.000% |
| Cumulative requested bytes per sample | 9,568,256 | 0 | -100.000% |
| P50 | 4,093,900 ns | 580,800 ns | -85.813% |
| P95 | 6,883,300 ns | 1,018,100 ns | -85.209% |

Model source:
`.codex/state/session-coordinator/hub04-allocation-free-message-id-lookup-model.rs`.

The r3 continuation adds a Windows-native release gate in
`zircon_hub/tests/hub04_message_id_lookup_performance.rs`. It invokes the actual
`HubMessageId::from_str_id` production function, compares it with the legacy
`HubMessageId::all()` path for exact result parity, and uses an isolated integration-test
allocator so it does not conflict with the library test executable's allocator. The gate
runs 8,192 lookups over 21 alternating sample pairs, requires zero optimized allocations,
and requires both P50 and P95 to improve by at least 60%. Exact managed measurements are
pending the asynchronous coordinator result.

## Contracts And Validation

- `tools/tests/test_hub04_allocation_free_message_id_lookup_performance_contract.py` locks the
  allocation-free parser, namespace-local static scans, all eight category mappings, the
  existing unknown-ID raw-text fallback, and the native production-path release gate.
- Local source-contract result: 4 tests passed; the combined Hub04 batch passes 8/8.
- Local `rustfmt +1.94.1 --edition 2021 --check` passed for the production file.
- The release model passed zero-allocation and P50/P95 reduction gates.
- Cargo compilation and both ignored release gates remain pending in one managed asynchronous
  coordinator batch; no direct Cargo command was run. Exact command:
  `cargo +1.94.1 test --manifest-path zircon_hub/Cargo.toml --lib --test hub04_message_id_lookup_performance --locked --release --jobs 1 -- hub04_ --include-ignored --nocapture --test-threads=1`.

## Remaining Parent-Plan Work

Hub04 still owns recent-project indexing, launch classification, metadata hydration, sorting,
telemetry, and broader project-page qualification. This slice only removes the per-message full
inventory allocation from stable ID deserialization.
