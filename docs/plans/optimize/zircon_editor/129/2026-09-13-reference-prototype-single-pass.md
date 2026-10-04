---
title: Editor reference prototype single-pass extraction
category: zircon_editor
report_id: Editor129-reference-prototype-single-pass-2026-09-13
date: 2026-09-13
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_target_met
---

# Editor129 reference prototype single-pass extraction

## Scope

References and Used By refreshes locate four retained row prototypes before
removing stale dynamic rows. The former implementation walked the node slice
four independent times with `find_node`, repeating the same prefix scan and
clone boundary.

## Implementation

- `AssetReferenceNodePrototypes::from_nodes` now visits the slice once and
  captures the first panel, name, locator, and kind prototype into four local
  slots.
- The loop stops as soon as all four slots are filled; missing-prototype
  behavior still returns `None` and the first-match precedence is unchanged.
- Dynamic-row filtering, empty-state handling, row ordering, and the
  Editor742 capacity reservation remain unchanged.
- Added a lower Rust duplicate-prototype regression and the ignored Release
  marker `EDITOR743_REFERENCE_PROTOTYPE_SCAN_BENCH_V1`.

## Complexity and deterministic pressure model

For a template prefix of `P` nodes before the four prototypes, the old path
performed up to four prefix walks (`4P`) while the new path performs one (`P`)
and still clones exactly four retained nodes. The 4,096-node model therefore
reduces scan visits from 16,384 to 4,096 (75% fewer). This is a structural
model, not a product CPU/RSS or p50/p95/p99 measurement.

## TDD and local evidence

- The source-pressure contract was intentionally RED against the four
  independent `find_node` calls, then GREEN after the one-pass extractor was
  added.
- The focused prototype/row-capacity contracts pass `6/6`; Rustfmt and Python
  compilation pass.
- The refreshed one-process non-tooling Runtime/Editor performance-plus-pressure
  loader covers `556` modules and passes `2068/2068` tests in `29.116s`,
  including this contract. This is local source/model evidence only.
- The focused Runtime206/Runtime85 plus Editor reference contract invocation
  passes `37/37` in one process (`111.865s`). A broader all-contract
  non-tooling discovery covers `868` modules and `3553` tests in `561.258s`
  with one unrelated WOC dependency failure
  (`test_woc_consumes_runtime_preference_service_without_sync_compat_trait`);
  no Editor/Runtime optimization contract failed.
- The ignored benchmark compares cloned legacy lookups with the one-pass
  extractor and is reserved for the owner-attributed Windows Release lane.

## Source fingerprints

| File | SHA-256 |
| --- | --- |
| `zircon_editor/src/ui/layouts/views/asset_reference_rows.rs` | `B64988B8A95E984CF6D40BA98BF5BCBDFECD41BA18EFABEA840FB840EF07BBC2` |
| `tools/tests/test_editor_asset_reference_prototype_scan_performance_contract.py` | `F0EF6C77BBBD696BD9BFF6A22B7CAB25A05F23E9E40A825804368D3601456BD6` |

## Managed acceptance gate

Keep this slice at `managed_validation_pending` until the batched Windows
Release lane proves compilation, malformed-template parity, allocation
behavior, and reference-panel p50/p95/p99. Tooling production changes remain
deferred.
