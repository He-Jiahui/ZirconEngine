---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/01/2026-08-30-runtime-authored-geometry-delta-publication.md
  - docs/plans/optimize/zircon_editor/01/2026-08-25-ui-input-paint-style-static-candidates.md
  - docs/plans/optimize/zircon_runtime/11a-runtime-ui-architecture-tree-layout-input-accessibility-review.md
related_code:
  - zircon_runtime/src/ui/surface/render/cache.rs
  - zircon_runtime/src/ui/surface/popup_stack.rs
tests:
  - zircon_runtime/src/ui/surface/render/cache/tests/geometry_patch.rs
  - zircon_runtime/src/ui/tests/popup_declarative_state.rs
---

# Runtime Render Cache And Popup Scratch Reduction

## Scope

The geometry-only and local re-extract cache patch paths validate one staged
replacement per changed owner before mutating the published extract. Their
staging vectors now reserve the exact `changed_node_ids` upper bound. Geometry
patching creates its damage set only after staging succeeds and reserves the
validated patch count: each eligible owner has exactly one command, so that
count is a strict damage-frame upper bound.

The local re-extract damage set intentionally remains lazy. An accepted local
re-extract can have a dynamic number of commands per owner and a no-op patch
has no damage frames; reserving by command count would turn a stable patch into
an unnecessary large allocation.

`popup_branch_closures` no longer collects the popup-stack tail into a temporary
vector before traversing it. It now directly reverse-iterates the stack slice,
preserving LIFO close order and the same shared visited set for stack-backed and
unstacked descendants.

## Plan Completion List

| Batch | Work | Status | Validation evidence |
| --- | --- | --- | --- |
| Runtime11A / RUI-671 | Bound local render-cache patch scratch by validated changed owners; remove popup stack-tail temporary materialization | implemented_pending_validation | Lower Rust source regressions cover capacity/lazy-damage invariants and direct stack-tail streaming. Scoped Rustfmt, scoped diff check, a direct source probe, and the combined 95-test Runtime/Editor static-contract batch passed. Managed Runtime Cargo and Windows Release allocation/p50/p95/p99 evidence remain pending. |

## Static Evidence

- `render_cache_patch_staging_reserves_only_known_owner_bounds` prevents the
  geometry and local-reextract staging vectors from returning to geometric
  growth, and prevents dynamic local-reextract damage from becoming eagerly
  allocated.
- Existing popup branch-order coverage still requires unstacked deep children
  to close before their parent. The new source regression requires direct
  reverse iteration and rejects restoration of the `stack_tail` collection.
- Scoped `rustfmt --edition 2021 --check --config skip_children=true` passed
  for `cache.rs`, its focused regression, and the popup regression. A full-file
  check of `popup_stack.rs` reports only pre-existing unrelated formatting at
  lines outside this change; that file was not mechanically rewritten.
- Scoped `git diff --check` passed. Git reported only the repository's existing
  line-ending notices.
- The direct Runtime cache/popup source probe passed.
- The combined Runtime/Editor static-contract batch passed `95/95` in `0.180s`.
  It covers Runtime navigation, incremental rebuild, render-cache authority,
  and Editor asset-browser, console, and watcher contracts.

## Source Snapshot

- `render/cache.rs`: `CEAC069A2CBFE29E34A49A58A46892DA964EFB7652A330FFDC9014FD61C1119A`
- `render/cache/tests/geometry_patch.rs`: `2BFFB0555D23D8545FFC943CE5983E2C6CF6EFAB9A814CDC388B6EE33E4A8090`
- `popup_stack.rs`: `B6790DF0E552DD2EE56800F0B1EB9F56D65D5A580BD3D59EF9B808331D91E5BD`
- `popup_declarative_state.rs`: `EA9B363072DC4D994165054FFF11009F91B322CE33CE3D2C553177D3255A692D`

## Managed Gate

No direct Cargo command or coordinator status query was run for this slice.
The next immutable multi-task Runtime/Editor validation input must compile the
two focused Rust test modules and collect a release allocation/time comparison.
Until that batch succeeds, this record remains `implemented_pending_validation`
and does not claim product performance acceptance.
