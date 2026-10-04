---
title: Editor04 Catalog Generation Capacity
category: zircon_editor
report_id: Editor04-catalog-generation-capacity-2026-09-13
date: 2026-09-13
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor04 Catalog Generation Capacity

## Scope

The full editor asset catalog projection already knows the number of catalog
records before building details, public asset rows, and retained catalog-record
rows. This slice applies that bound to all record-backed vectors without
changing sorting, row identity, detail projection, or folder generation.

## Change

- Reserve the catalog record count before collecting and sorting record refs.
- Build details, public assets, and catalog-record rows with explicit bounded
  vectors and one pass each.
- Preserve locator ordering, `build_details_generation` behavior, and the
  existing `EditorAssetCatalogGeneration::from_parts` contract.

## Complexity boundary

The projection remains `O(N log N)` because locator sorting is required, while
the three record-backed vectors no longer grow geometrically during a full
catalog build. This is allocation-layout evidence only; incremental structural
sharing and Runtime registry authority remain parent-plan work.

## TDD and local evidence

- The source contract was run RED before implementation because the builder
  used three implicit `collect::<Vec<_>>()` paths.
- After implementation the focused source contract passes `3/3`.
- The combined Runtime UI plus Editor asset contract discovery passes `620/620`
  in `15.422s`; the focused index/input/asset subset passes `37/37` in `0.039s`.
- The broader Runtime/Editor performance-contract and pressure discovery passes
  `1358/1358` in `9.465s` in one process.
- Scoped Python compilation, Rustfmt, `git diff --check`, and wiki validation pass.
- Existing Editor asset/catalog/reference regressions remain the adjacent
  batch scope; scoped rustfmt and Python compilation are required before the
  next combined validation invocation.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_editor/src/ui/host/editor_asset_manager/manager/catalog_generation/build.rs` | `1C4A3E436415615A20F823002E9763A9A9CA19EE0896AA6839C878655B0D1610` |
| `tools/tests/test_editor_catalog_generation_capacity_performance_contract.py` | `67D10FE97E6A66F1FB917459AA9BBB7724A2AD7656577E1571BB32B6BBE9919D` |

## Managed gate

This slice joins the existing owner-attributed Runtime/Editor batch. Managed
Cargo/Release allocation and catalog-build p50/p95/p99 evidence remain pending;
no standalone Cargo process or coordinator polling is required.
