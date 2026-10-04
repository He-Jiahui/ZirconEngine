---
title: Editor hierarchy projection single-pass selection state
category: zircon_editor
report_id: Editor01-hierarchy-projection-single-pass-2026-09-13
date: 2026-09-13
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor01 hierarchy projection single-pass selection state

## Scope

The retained hierarchy pane needs a `SceneNodeData` vector for publication. The
previous projection separately scanned the logical rows for `selected` and then
walked the same rows again to build that required vector. This slice combines
those operations without changing filtering, ordering, identity, depth
conversion, or template-state behavior.

## Implementation

`hierarchy_template_projection` now constructs `hierarchy_nodes` once. The map
closure updates `has_selection` as each source row is converted, and the same
collected vector is passed to `model_rc` after template state is applied.

## Deterministic work model

For `N` rows where selection is absent or appears at the end, the retired path
performed `2N` source-row visits (`any` plus DTO projection). The current path
performs `N` visits and retains the same one owned DTO per row. This is a
source-operation comparison, not a CPU, allocator, RSS, or product-latency
measurement.

## Validation

- The focused source contract was RED against the separate `.any` and `.map`
  traversals, then GREEN after the merged projection.
- The focused new contracts pass `4/4`; the single-process non-tooling
  Runtime/Editor performance-plus-pressure batch covers 351 modules and passes
  `1345/1345` tests in `39.080s`.
- A subsequent same-source rerun of that shared non-tooling batch covers 351
  modules and passes `1346/1346` tests in `8.890s`.
- Scoped Rustfmt passes. Managed Cargo, product startup, and hierarchy
  allocation/CPU/RSS/p50/p95/p99 evidence remain pending under the existing
  external `E:\Git\zr_vm` dirty-worktree admission blocker.

## Acceptance boundary

This slice is static/source complete only. Keep the completion status pending
until the owner-attributed Windows Release batch verifies compile, row-model
parity, and the declared allocation/latency gates.
