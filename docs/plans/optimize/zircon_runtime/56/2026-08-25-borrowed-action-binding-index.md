---
title: Runtime56 Borrowed Action Binding Index
category: zircon_runtime
report_id: Runtime56-borrowed-action-binding-index-2026-08-25
date: 2026-08-25
session_id: root-runtime56-bulk-button-release-20260825
implementation_status: implementation_complete
validation_status: managed_validation_queued
validation_ticket: 2a6e907071354f28b114db0c90a9074c
---

# Runtime56 Borrowed Action Binding Index

## Scope

This slice removes action-name string clones from the temporary binding index built by
`ActionEvaluationGeneration::from_action_map`. It preserves the compiled action/binding order,
context slots, unknown-action handling, axis-binding flag, and public input contracts. It does not
claim to close Runtime56's product ingress, action-consumer, focus-arbitration, or replay gaps.

## Implementation

The generation builder previously created `BTreeMap<String, Vec<usize>>` and cloned
`binding.action` for every source binding. The map lives only for the duration of generation
compilation, while the owned `InputActionMap` remains borrowed for the entire build. Its keys now
borrow `&str` directly from the bindings and are discarded before the borrow ends.

The regression builds a mixed-context map with interleaved bindings and an unknown action, then
compares the retired and optimized generations field by field, including each compiled action's
binding slice. A source contract rejects reintroduction of the owned-key map or per-binding clone.

## Performance Contract

| Evidence | Retired path | Optimized gate |
| --- | ---: | ---: |
| Binding action string clones per 4,096-binding generation build | 4,096 | 0 |
| Alternating release benchmark | 11 samples x 64 generation builds | optimized P95 <= 85% of retired P95 |

The benchmark uses unique 120-byte-class action names to expose extension-scale rebind/reload
pressure. It emits `RUNTIME56_BORROWED_ACTION_BINDING_INDEX_BENCH_V1` with both P95 timings,
reduction basis points, sample/iteration/binding counts, and retired/optimized clone counts.

## Validation

Rust 1.94.1 `rustfmt --check`, scoped `git diff --check`, and the production source guards passed
before submission (apart from the repository's existing CRLF notice). One managed Runtime input
batch covers retired/optimized generation equivalence, the borrowed-key source contract, and the
ignored release benchmark. Dynamic P95 evidence, integration SHA, and automatic WeCom performance
delivery remain coordinator-owned and pending.
