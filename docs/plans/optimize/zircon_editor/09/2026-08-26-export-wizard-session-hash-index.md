---
title: Editor09 Export Wizard Session Hash Index
category: zircon_editor
report_id: Editor09-export-wizard-session-hash-index-2026-08-26
date: 2026-08-26
session_id: root-runtime-events-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor09 Export Wizard Session Hash Index

## Scope

This slice replaces the retained export wizard's profile-session owner with `HashMap`. View-model
projection, action dispatch, plan regeneration, and mutable session access now resolve profile
names through expected constant-time lookup.

`poll_all` still streams mutable sessions without cloning a key list. Because its returned update
order was previously induced by `BTreeMap`, it now sorts only the changed update rows by profile
name before returning. Job lifecycle, profile isolation, projection invalidation, and command
execution semantics are unchanged.

## Performance Workload

The release workload fills 4,096 long shared-prefix profile names and performs 4,096 stable hits for
the final profile.

| Work per workload | Before | After |
|---|---:|---:|
| Ordered profile-session lookups | 4,096 | 0 |
| Hash profile-session lookups | 0 | 4,096 |
| Poll result order changes | 0 | 0 |
| Allocations on profile-session hits | 0 | 0 |

The ignored release gate runs 17 alternating sample pairs and emits
`EDITOR09_EXPORT_WIZARD_SESSION_HASH_INDEX_BENCH_V1`. Acceptance requires hash lookup P95 to be at
least 30% below the legacy `BTreeMap` path. Exact Windows P50/P95 timings remain pending the
coordinator run.

## Acceptance

- `optimization_batch_20260826ca_export_wizard_session_hash_index_isolates_profiles` covers
  independent profile creation and lookup.
- `optimization_batch_20260826ca_export_wizard_session_hash_index_preserves_poll_order` locks the
  hash owner plus explicit changed-update ordering contract.
- `optimization_batch_20260826ca_export_wizard_session_hash_index_p95` reports paired release
  P50/P95 samples and enforces the 30% P95 reduction gate.

## Remaining Parent-plan Work

Editor09 still owns global admission, cancellation, resource budgets, progress aggregation,
shutdown, persistence, and product-wide background-job integration. This slice only converges
retained export-profile session lookup.
