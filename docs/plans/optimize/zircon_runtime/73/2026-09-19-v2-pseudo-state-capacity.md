---
title: Runtime v2 pseudo-state collector capacity
category: zircon_runtime
report_id: Runtime810-v2-pseudo-state-capacity-2026-09-19
date: 2026-09-19
session_id: root-runtime-editor-async-optimization-20260919
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_target_met
---

# Runtime810 · v2 pseudo-state collector capacity

## Scope

`collect_pseudo_states` builds the static selector path state list for every
v2 arena node. The authored `props` and `state` maps provide a stable lower
bound on retained entries, while painter resolution appends at least one
resolved alias. The old zero-capacity vector therefore paid geometric growth
for dense authored state maps.

The collector now reserves `props.len() + state.len() + 2` slots before
walking aliases. Sorting and deduplication remain authoritative, so duplicate
aliases, custom state names, resolved painter names, and lexical output order
are unchanged. Runtime-tree state collection and retained-style dirty
classification are intentionally untouched by this narrow slice.

## TDD and deterministic model

- The source contract was RED against `Vec::new()` and GREEN at `3/3` after
  the bounded reservation, lower alias/order regression, and ignored marker
  were added.
- `RUNTIME810_V2_PSEUDO_STATE_CAPACITY_BENCH_V1` is wired for the managed
  Windows Release lane.
- For 4,096 authored entries plus the resolved painter state (4,097 output
slots in the structural model), the legacy collector performs 12 geometric
growth events and the reserved collector performs 0 (`12 -> 0`). This is
  allocation-shape evidence only; it is not allocator, CPU, RSS, or product
  percentile evidence.

## Local validation

- `tools/tests/test_runtime_v2_pseudo_state_capacity_performance_contract.py`:
  `3/3`.
- `python -m py_compile` for the contract: pass.
- Scoped `rustfmt --edition 2021 --check` for
  `zircon_runtime/src/ui/v2/style/runtime_state.rs`: pass.
- Source SHA-256:
  `BBC7F1572E41A65BD1F31484E73F1EAE6959DB91F0AA77A596DCE0477C9B4CD3`.
- Contract SHA-256:
  `6116ADB7839DD4A36DBE25844485D4DABA20EDB24A93140B12E9F4C3C9067B38`.

Managed Windows Cargo/Release execution, lower Rust execution, and product
style p50/p95/p99 evidence remain pending under the shared external-worktree
admission gate. No standalone Cargo process was started and this session does
not poll the coordinator; tooling production remains deferred for the later
Rust migration.
