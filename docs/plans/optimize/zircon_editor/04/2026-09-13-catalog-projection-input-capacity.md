---
title: Editor04 Catalog Projection Input Capacity
category: zircon_editor
report_id: Editor04-catalog-projection-input-capacity-2026-09-13
date: 2026-09-13
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor04 Catalog Projection Input Capacity

## Scope

The full Runtime-to-Editor catalog projection creates two hash maps for every
non-unchanged refresh and grows them geometrically while walking the catalog.
Each projected record also appends metadata and reference-repair diagnostics to
a vector without reserving the known upper bound.

## Change

- Reserve both catalog maps from the authoritative Runtime asset count before
  the single-pass projection.
- Reserve diagnostic output from metadata diagnostics plus reference repairs,
  preserving the existing ordering and message text.
- Keep the Runtime registry and catalog-input generation as the only authorities;
  no second scan or count pass is introduced.

## Complexity boundary

Projection remains `O(N)` for `N` catalog inputs. The map and diagnostic output
paths no longer pay geometric rehash/reallocation growth on the common full
refresh path. This does not claim incremental catalog structural sharing or
product Asset Browser p50/p95/p99 acceptance.

## TDD and local evidence

- The source contract was run RED before implementation because both maps used
  `HashMap::new()` and diagnostics used an unreserved collect.
- After implementation the focused contract passes `3/3`.
- The Runtime UI plus Editor asset/projection batch (82 modules) passes
  `421/421` in `3.136s`; the broader current Runtime/Editor
  performance-contract and pressure batch passes `1364/1364` in `9.538s` in one
  process. These are local source/model receipts; managed Cargo, Release
  allocation, and product latency gates remain pending.
- The later comprehensive non-tooling Runtime/Editor loader covers 548 files
  and passes `2039/2039` in `22.986s`; this supersedes the narrower count while
  preserving the earlier receipt for traceability.
- Scoped Python compilation, non-recursive Rustfmt, `git diff --check`, and wiki
  validation are required evidence for this slice.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_editor/src/ui/host/editor_asset_manager/manager/project_sync/sync_from_project.rs` | `95D94B7A1E9FB0A998DA65A6B60628A20A354D20E5117FFFF5DE392AD5CFC4E1` |
| `zircon_editor/src/ui/host/editor_asset_manager/manager/project_sync/record_projection.rs` | `DD95FD480A2CC9A11898B8EC9013F04CFF991A494FDFA665C826EC97EBD585D6` |
| `tools/tests/test_editor_catalog_projection_input_capacity_performance_contract.py` | `1EA1286AAA3B956EF19A1801EFB0429018BC479B1F74082AD893F292B040D931` |

## Managed gate

This slice is included in the existing owner-attributed Runtime/Editor batch;
it does not start a standalone Cargo request or coordinator poll. Managed
Windows Cargo/Release allocation and catalog projection p50/p95/p99 evidence
remain pending under the shared admission boundary.
