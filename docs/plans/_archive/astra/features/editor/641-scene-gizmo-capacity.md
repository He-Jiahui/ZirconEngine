---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/641/2026-09-01-preallocated-scene-gizmos.md
related_records:
  - docs/plans/astra/features/editor/30-recent-optimization-batch-completion.md
  - docs/plans/astra/features/editor/664-20260911-editor-ui-optimization-batch-completion.md
implementation_files:
  - zircon_editor/src/scene/viewport/render_packet.rs
  - zircon_editor/src/scene/viewport/render_packet/reused_overlay_storage_tests.rs
tests:
  - tools/tests/test_editor641_scene_gizmo_capacity_performance_contract.py
---

# Editor641 · scene-gizmo capacity evidence refresh

## 计划完成列表

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Editor641 scene viewport extraction | Keep the exact camera/directional-light capacity bound and strengthen the ignored Release probe from 17 to 101 alternating pairs. Emit raw sorted series plus nearest-rank P50/P95. | TDD RED→GREEN source contract `2/2`; Rustfmt and scoped diff checks pass; the existing node-kind capacity regression remains in place. Managed Editor Cargo/Release and product viewport percentile evidence remain pending. | implemented_pending_validation |

## Implementation evidence

`build_scene_gizmos` still reserves only the bounded camera/directional-light admission count;
inactive nodes and failed gizmo construction remain fail-closed. The benchmark-only change makes
the P95 gate reproducible and auditable without changing scene ordering or overlay semantics.

## Source fingerprints

| File | SHA-256 |
| --- | --- |
| `zircon_editor/src/scene/viewport/render_packet.rs` | `A19A0E5FC2680FA70374588D67D579B47D6AF59A2E4640840BF2DAE22E49018D` |
| `zircon_editor/src/scene/viewport/render_packet/reused_overlay_storage_tests.rs` | `C1D02A982A3991089A6B307EF5D11061D871CA948A1CB48A05BAF51513F390B8` |
| `tools/tests/test_editor641_scene_gizmo_capacity_performance_contract.py` | `DD1A46E1016DF69D83B0F5FD342AAE1D1002ED0AC9962855F5E61086EE5B194F` |

The focused source contract passes `2/2`. The current post-capacity batched local validation
ran this contract with Runtime170 and four adjacent Runtime/Editor suites: `45`
tests passed in `94.665s` with zero failures or errors. The one-process all-surface
`test_*performance_contract.py` batch previously passed `2443/2443` in `405.162s`.
These are local static receipts only. No managed timing or Cargo result is inferred,
and tooling production remains out of scope.

The later non-tooling Runtime/Editor loader covers 57 modules and passes `203/203` in `0.171s`,
including this Editor641 contract after the Runtime08c source-contract matcher repair. This
remains local source evidence; managed Cargo/Release and product viewport percentile gates are
unchanged.

After the production-file import-order normalization and Runtime170 capacity change, the focused
Runtime170 plus Editor641 contract batch passed `7/7`; Editor641 production/test Rustfmt and scoped
diff checks passed again. This is a source-format/contract receipt, not managed
Cargo/Release performance evidence.

## 性能与受管验证边界

The 101-pair marker improves sample stability and independent P50/P95 recomputation, but it does
not claim a product frame-time or allocation result. Keep this record
`implemented_pending_validation` until the asynchronous Editor Windows batch validates the source
and records Release percentile evidence.
