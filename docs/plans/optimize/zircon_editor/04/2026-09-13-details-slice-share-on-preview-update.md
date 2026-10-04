---
title: Editor04 Details Slice Sharing on Preview Update
category: zircon_editor
report_id: Editor04-details-slice-sharing-on-preview-update-2026-09-13
date: 2026-09-13
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor04 Details Slice Sharing on Preview Update

## Scope

Single preview/catalog-row updates already shared immutable catalog records,
but they still copied the complete `details_by_asset_index` pointer slice even
when the target asset's details had never been materialized. Preview completion
is commonly a row-only update, so that copy was avoidable.

## Change

- When the target details slot is empty, retain the existing immutable details
  `Arc` directly for both single-row and batched updates.
- When the slot is materialized, preserve the existing copy-on-write behavior:
  copy the pointer slice once and replace only the target detail row.
- Keep asset/catalog-record replacement, detail contents, generation identity,
  and lookup ordering unchanged.

## Complexity boundary

For an update batch whose targets have no materialized details, the details state
remains shared with `O(1)` work. A materialized target still performs the existing
single `O(N)` pointer-slice copy for `N` assets; this slice
does not claim the larger chunked-generation/P2-03 redesign or product p50/p95/
p99 acceptance.

## TDD and local evidence

- The source contract was run RED before implementation because both single-row
  update paths unconditionally collected the details slice.
- After implementation the focused contract passes `4/4`.
- The change joins the batched Runtime/Editor asset and performance suites;
  the comprehensive non-tooling Runtime/Editor performance-contract and
  pressure loader covers 548 files and passes `2039/2039` in `22.986s`;
  managed Cargo, Release allocation, and product latency gates remain pending.
- Scoped Python compilation, non-recursive Rustfmt, and `git diff --check` pass.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_editor/src/ui/host/editor_asset_manager/generation.rs` | `F9F26FE0042CECDB7FC53C12F998B758BB579417E66CA21701C22AF1A09B3872` |
| `tools/tests/test_editor_catalog_details_cow_capacity_performance_contract.py` | `17DBA710DFDF69F535B53595B25231A080D263A3C8F1C9A2B0255137D6372590` |

## Managed gate

This slice is included in the existing owner-attributed Runtime/Editor batch;
it does not start a standalone Cargo request or coordinator poll. Managed
Windows Cargo/Release allocation and catalog-update p50/p95/p99 evidence remain
pending under the shared admission boundary.
