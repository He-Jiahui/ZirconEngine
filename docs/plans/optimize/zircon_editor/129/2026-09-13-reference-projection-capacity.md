---
title: Editor129 Asset Reference Projection Capacity
category: zircon_editor
report_id: Editor129-reference-projection-capacity-2026-09-13
date: 2026-09-13
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor129 Asset Reference Projection Capacity

## Scope

`build_details_generation` materializes direct and reverse asset-reference rows
when the selected asset details generation is published. The result order,
unknown-reference fallback, and Runtime registry authority remain unchanged.

## Change

- Reserve `record.direct_references.len()` before projecting direct rows.
- Retain the single Runtime referencer UUID collection and reserve its length
  before filtering unknown catalog entries into `referenced_by` rows.
- Use explicit bounded loops so the known lower bound is applied without a
  second intermediate projection vector.
- Keep deterministic `reference_order` sorting and
  `known_project_asset: false` fallback semantics intact.

## Complexity boundary

The projection remains `O(R log R)` because stable reference ordering is still
required, but geometric `Vec` growth is removed for `R` direct/reverse rows and
the filtered reverse projection no longer relies on an implicit collector
capacity. This is a local allocation-layout optimization; typed search-provider
and field-path contracts from Editor129 remain open.

## TDD and local evidence

- The new source contract failed before implementation because both projections
  used implicit `collect::<Vec<_>>()` growth.
- After implementation the focused contract passes `3/3`.
- Scoped rustfmt, Python compilation, and diff checks pass. The next combined
  Editor/Runtime batch should include the existing catalog/reference regressions;
  no per-task Cargo run was started.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_editor/src/ui/host/editor_asset_manager/manager/catalog_generation/details.rs` | `4DA999FE03CBE806ED1625CEF0442CF00F491D5B1D273810D7411084FA286A1E` |
| `tools/tests/test_editor_asset_reference_projection_capacity_performance_contract.py` | `A5047317428150EC3A16CB4D4E5C09098AD6F7999885D3F9B697DE0F2E3B1B81` |

## Managed gate

This slice joins the existing owner-attributed multi-task Runtime/Editor batch.
Managed Cargo/Release validation remains pending because prior admission was
rejected by the external `E:\\Git\\zr_vm` dirty-worktree and static-overlay
ownership gates. No coordinator status polling was performed.
