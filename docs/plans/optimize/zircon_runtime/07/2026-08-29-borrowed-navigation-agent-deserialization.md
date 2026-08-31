---
title: Runtime07 Borrowed Navigation Agent Deserialization
category: zircon_runtime
report_id: Runtime07-borrowed-navigation-agent-deserialization-2026-08-29
date: 2026-08-29
session_id: root-runtime07-incremental-capability-set-builder-20260827
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime07 Borrowed Navigation Agent Deserialization

## Scope

This slice removes a deep `serde_json::Value` clone from the guest navigation movement call. The
existing dynamic component remains borrowed from the World while serde constructs the same owned
`NavMeshAgentDescriptor` used for navigation updates.

## Change

- Deserialize `NavMeshAgentDescriptor` directly from the borrowed component JSON value.
- Reuse the same borrowed serde pattern already used by the navigation World projection.
- Preserve invalid-component fallback, descriptor mutation, writeback, and navigation tick behavior.
- Add a Rust descriptor round-trip guard plus a Python source contract that rejects reintroducing
  the deep JSON clone.

## Deterministic Performance Evidence

The standalone optimized Rust model uses the descriptor's actual 19-field serialized shape: one
object container, 19 field-name strings, three string-valued fields, numeric and Boolean fields,
and the destination array. It compares deep-cloning that JSON-shaped value before decoding with
decoding directly from the borrowed value. Each sample performs 32,768 decodes, alternates legacy
and optimized order across 31 samples, counts allocations for one decode, and verifies identical
descriptor checksums.

| Metric | Before | After | Reduction |
|---|---:|---:|---:|
| Decode allocation calls | 24 | 1 | 95.833% |
| Decode requested bytes | 1,194 | 24 | 97.990% |
| Descriptor decode P50 | 78.1087 ms | 4.6830 ms | 94.005% |
| Descriptor decode P95 | 126.7943 ms | 9.4845 ms | 92.520% |

Evidence marker: `RUNTIME07_BORROWED_NAVIGATION_AGENT_DESERIALIZATION_MODEL_V1`.

A second complete run remained favorable: P50 improved 93.483% and P95 improved 91.200%.
Both paths produced checksum `12977619570418913792`.

## Validation

- `python tools/tests/test_runtime07_borrowed_navigation_agent_deserialization_performance_contract.py`:
  3 passed after the pre-change contract failed 3 of 3 checks.
- The standalone Rust 1.94.1 model compiled and passed two complete 31-sample runs with identical
  checksums.
- The Rust guard serializes a default descriptor, deserializes it from `&Value`, and verifies exact
  value round-trip behavior.
- Exact-file Rust formatting, Python bytecode compilation, and scoped diff checks are required
  before snapshot freeze.
- Managed Rust compilation and focused tests remain pending in an asynchronous Runtime07 batch.

Managed batch request: `runtime07-borrowed-gameplay-seven-task-batch-20260830-v1`.

Validation attempt: ticket `a9dc9a55e9044c239cc7dfda8bbc64b6` failed before Cargo at
coordinator artifact governance for `D:\ZirconBuilds\mvp-test-fixtures-36724`; the 22 local contract
checks remain green while integrated acceptance and success publication remain pending.

## Remaining Parent-plan Work

Runtime07 still owns deterministic resolver and catalog generations, package trust constraints,
transactional lifecycle, isolation, execution budgets, and product-scale editor/app/export/cook
acceptance in the canonical review.
