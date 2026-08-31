---
title: Runtime07 Borrowed Owner Revocation Plugin ID
category: zircon_runtime
report_id: Runtime07-borrowed-owner-revocation-plugin-id-2026-08-29
date: 2026-08-29
session_id: root-runtime07-incremental-capability-set-builder-20260827
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime07 Borrowed Owner Revocation Plugin ID

## Scope

This slice removes a transient plugin-id allocation from extension owner revocation. The module
interner already returns a borrowed module name, and stripping its `.runtime` suffix returns a
borrowed plugin-id prefix. Revocation previously copied that prefix into a `String` before using it
to remove shader sources and asset importers.

## Change

- Keep `plugin_id_from_module_name` as `Option<&str>` throughout owner revocation.
- Reuse the same borrowed prefix for shader-source retention and asset-importer removal.
- Preserve revocation order, owner notification, bridge invalidation/finalization, extension-family
  removal, and empty-result behavior.
- Add a Rust regression proving the derived plugin id starts at the source module-name address.
- Add a Python source performance contract for the borrowed/no-allocation path.

## Deterministic Performance Evidence

The standalone optimized Rust model revokes 16,384 distinct `.runtime` module owners from
prebuilt shader-owner and asset-importer containers over 31 alternating samples. Container setup
is outside the timed and allocation-profiled region, isolating the transient plugin-id ownership
conversion. Both implementations produced checksum `17847271319040491372` in both runs.

| Metric | Before | After | Reduction |
|---|---:|---:|---:|
| Allocation calls per 16,384 revocations | 16,384 | 0 | 100.000% |
| Requested allocation bytes | 196,608 | 0 | 100.000% |
| Run 1 P50 | 7.6701 ms | 5.6857 ms | 25.872% |
| Run 1 P95 | 15.3274 ms | 10.3062 ms | 32.760% |
| Run 2 P50 | 9.0973 ms | 5.9569 ms | 34.520% |
| Run 2 P95 | 19.4504 ms | 13.2258 ms | 32.002% |

Evidence marker: `RUNTIME07_BORROWED_OWNER_REVOCATION_PLUGIN_ID_MODEL_V1`.

## Validation

- `python tools/tests/test_runtime07_borrowed_owner_revocation_plugin_id_performance_contract.py`:
  4 passed after all 4 pre-change checks failed.
- The standalone Rust model preserved suffix parsing and both downstream container operations;
  two runs kept identical allocation profiles and checksums with positive P50/P95 results.
- The Rust regression locks the prefix-borrow identity.
- Python compilation, exact-file Rust/model formatting, the Runtime07 source-contract batch, and
  scoped diff checks are required before snapshot publication.
- Managed Runtime compilation and tests remain pending in the next asynchronous Runtime07 batch.

## Remaining Parent-plan Work

This local allocation removal does not close Runtime07 product acceptance. The parent plan still
requires unified catalog generations, transactional reload, trust/isolation, execution budgets,
real editor/app/export/cook traces, and product-scale comparison.
