---
title: Runtime07 Single-pass Linked Runtime Projection
category: zircon_runtime
report_id: Runtime07-single-pass-linked-runtime-projection-2026-08-28
date: 2026-08-28
session_id: root-runtime07-incremental-capability-set-builder-20260827
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime07 Single-pass Linked Runtime Projection

## Scope

This slice removes a duplicate enabled-plugin scan and duplicate crate-name allocations from linked
runtime plugin projection. It preserves eligibility rules, first-crate link ordering, package-id
membership for plugins sharing a crate, generated crate paths, and later feature-link extension.

## Change

- Project linked runtime crate names, package ids, and crate links in one helper and one enabled-plugin
  scan instead of repeating the full eligibility predicate.
- Preallocate both hash sets and the link vector from the enabled-plugin upper bound.
- Check duplicate crate names through the borrowed runtime-crate string before materializing the
  owned crate name, eliminating allocations for duplicate mappings.
- Keep package-id insertion outside crate-name deduplication so every eligible plugin remains visible
  to runtime availability even when multiple plugins share one crate.
- Add a Rust regression with two package ids sharing one runtime crate and a Python source contract
  for the single-pass, preallocated, borrowed-dedup shape.

## Deterministic Performance Evidence

The standalone optimized Rust model projects 32,768 eligible plugins where each pair shares a crate,
yielding 16,384 ordered crate links and 32,768 package ids across 17 alternating samples. Both paths
first compare all sets and links. Both produced checksum `1835008`.

| Metric | Before | After | Reduction |
|---|---:|---:|---:|
| Allocation calls | 131,114 | 98,307 | 25.022% |
| Requested allocation bytes | 9,699,400 | 6,946,848 | 28.379% |
| Projection P50 | 59.6965 ms | 32.9863 ms | 44.743% |
| Projection P95 | 111.1226 ms | 97.4268 ms | 12.325% |

Evidence marker: `RUNTIME07_SINGLE_PASS_LINKED_RUNTIME_PROJECTION_MODEL_V1`.

## Validation

- `python tools/tests/test_runtime07_single_pass_linked_runtime_projection_performance_contract.py`:
  3 passed after the pre-change contract failed 3 of 3 checks.
- The standalone Rust model asserts equality of crate-name membership, package-id membership, ordered
  crate links, generated paths, and checksum.
- A Rust regression asserts that duplicate crate mappings produce one link but retain both package
  ids.
- Exact-file Rust formatting, Python bytecode compilation, and scoped diff checks passed.
- Managed Rust compilation and focused tests remain pending in a later asynchronous Runtime07 batch;
  this candidate will not be validated alone.

## Remaining Parent-plan Work

Runtime07 still owns deterministic resolver and catalog generations, package trust constraints,
transactional lifecycle, isolation, execution budgets, and product-scale editor/app/export/cook
acceptance in the canonical review.
