---
title: Runtime58 Bridge Dependency Traversal Scratch Capacity
category: zircon_runtime
report_id: Runtime802-bridge-dependency-scratch-capacity-2026-09-18
date: 2026-09-18
session_id: root-runtime-editor-async-optimization-20260918
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_target_met
---

# Runtime802 · Bridge dependency traversal scratch capacity

## Scope

Runtime plugin catalog construction projects bridge dependency diagnostics for
every registered package. The graph already removes every current root from its
DFS `visiting` set before returning, but the caller previously allocated a new
set for each root. Graph-owned registration indexes are populated for every
catalog build, while issue-only reachability and DFS buffers should remain lazy
for clean catalogs.

## Implementation

- Retain one `visiting` `HashSet` for the complete root-diagnostic pass, clear
  it before each root, and pass it through the existing DFS only after the
  existing no-issue short circuit.
- Reserve the registration-count upper bound for the graph's package order,
  registered-plugin set, and provider/dependency maps.
- After the existing no-issue short circuit, reserve the package-order bound
  for the reachable set and closure cache before issuing root diagnostics.
- Preserve diagnostic text, source order, duplicate suppression, recursive
  cycle behavior, and cache authority.

## Deterministic performance model

For 1,024 diagnostic roots, the former call site creates 1,024 DFS visiting
sets; the retained scratch creates one, removing 1,023 root-level set
allocations. Clean catalogs still avoid all issue-only scratch allocation. The
registration-bound reservations remove geometric growth from the graph's
always-populated top-level indexes. This is allocation shape evidence, not a
measured CPU, RSS, or percentile result.

## Local evidence

- TDD source contract intentionally RED with two missing allocation-shape
  assertions, then GREEN (`3/3`).
- A lower Rust regression is wired to verify capacity bounds and that a reused
  scratch set is empty after cyclic diagnostic traversals in the managed batch.
- Exact-file Rustfmt and Python AST checks pass.
- Managed Windows Cargo, Release allocation, and catalog-build p50/p95/p99
  evidence remain pending; tooling production remains deferred.

## Acceptance boundary

Keep this record `managed_validation_pending` until a single owner-attributed
Windows batch validates the Runtime58/Runtime200/205 slices together. Do not
treat the deterministic allocation model as product performance acceptance.
