---
title: Editor04 Activity bounded rows, shared generations, and resize admission
category: zircon_editor
date: 2026-10-08
implementation_status: applied_static_review_complete
validation_status: native_and_managed_validation_pending
performance_status: native_product_acceptance_open
---

# Editor04 Activity bounded rows, shared generations, and resize admission

## Scope

The Activity asset surface now keeps the complete logical folder and asset
generation while materializing only a viewport-sized set of physical row
slots.  A slot is rebound to a logical index as the scroll position changes;
folder and asset identity, selection, hover, diagnostics, and pointer routing
remain generation-bound.  The slot budget is derived from the measured panel
height, the existing row stride, and the two-row overscan policy.  A resize
rebuilds the physical budget from the new panel geometry while sharing the
immutable logical generation.

The source slice also shares immutable folder data between Activity paint and
pointer routing.  `AssetWorkspaceSnapshot.visible_folders` is an
`Arc<[AssetFolderSnapshot]>`, and `AssetContentFolderIds` retains that Arc
instead of collecting a second `Vec<String>`.  The test-only `PartialEq<Vec<String>>`
adapter keeps the established pointer fixture expectation without changing
the production owner.

## Applied source shape

- `ActivityAssetVirtualization` owns the Arc folder generation, item
  generation, panel metrics, logical count, content extent, and bounded slot
  count.  `binding` maps each physical slot to the correct logical folder or
  asset, including the mixed-height folder-to-asset boundary.
- Activity layout, retained paint metadata, projector, and pointer layout all
  consume the same logical binding.  Paint and hover use the logical index;
  the physical slot is only a reusable node owner.
- The retained host keeps a complete logical `list_height` from the snapshot's
  folder and asset counts.  Scrollbar extent and virtualization content extent
  read that same value, so row virtualization does not shorten the scroll
  range.
- Resize coverage checks small, grown, and shrunk panels.  The grown panel
  receives the recomputed slot budget, the small and grown views share the
  folder/item generation, stable same-size publication reuses node values, and
  the shrunk view returns to the original budget.
- The 100,000-item product provider case checks bounded physical node count,
  logical scrollbar extent, tail binding/painting, selected and hovered state,
  and the original UUID pointer route.  The mixed folder/item case checks
  cross-boundary geometry and generation replacement while the old immutable
  generation remains readable.

The fixture migration changes the `visible_folders` construction to the Arc
conversion (`m3_asset_folders().into()`), and the pointer regression compares
against the expected `Vec<String>` through its compatibility assertion.  These
are type-owner adaptations, not new asset semantics.

## Source evidence and adoption receipt

The canonical actual-apply receipt is
`E:/cargo-targets/zircon-local/source-repairs/current-main-20261004/editor04-final-review-v2/actual-reviewed-canonical-apply.json`.
It records 17 expected/actual target files, `applyCheck.exitCode: 0`,
`apply.exitCode: 0`, `rustfmtCheck.exitCode: 0`, `diffCheck.exitCode: 0`, and
`controlsUnchanged: true`.  Its status is
`source-applied-native-validation-pending`; `nativeTestsExecuted` is `false`
and `performanceAccepted` is `false`.  The ownership amendment is to keep the
small and shrunk metadata owners alive while borrowing virtualization data.

The current-source extent review is
`E:/cargo-targets/zircon-local/source-repairs/current-main-20261008/editor04-core-extent-v1/core-extent-review.json`
(SHA-256
`c616f26faba46e165204981fcc3e3f6f0be840b49a0be9501410c56bb4340f33`).  It
reviewed the content-layout, geometry/paint metadata, scrollbar descriptor,
virtualization, retained primitives, and Activity product tests and concluded
that the complete logical extent is already preserved; no extent source change
is required.  `ModelRc.metadata<M>()` returns `Option<&M>`; named small/grown/
shrunk metadata locals improve readability and do not represent an `E0716`
repair or an ownership defect.

## Validation gates

| Gate | State |
|---|---|
| Applied source and scoped format/diff checks | complete per actual-apply receipt |
| Logical extent and metadata ownership review | complete; no source correction required |
| Managed Cargo/typecheck and Activity regressions | pending |
| Native 1/100/10k/100k open/close/reopen and interaction runs | pending |
| Release allocation, latency, RSS, and product performance | pending; no result claimed |

The native batch remains bounded to the current 16-command input.  Its
terminal receipt is
`E:/cargo-targets/zircon-local/source-repairs/current-main-20261004/batch-inputs-v1/terminal-receipt.json`:
the first command was `not-invoked-source-changed`, the remaining commands
were `not-invoked-prerequisite-failed`, and `sourceUnchanged` was `false`.
The request therefore produced no Cargo, test, or timing result.  The next
launch must use the repaired current source; this record claims the applied
bounded-row design and static/source evidence only, with native and
performance acceptance still open.

## 最新追加批次终态（2026-10-09）

12 命令批次 `runtime-editor-source-repair-batch-20261009-02` 已结束，首条 `three-package-check` 为真实 Cargo `101` 编译失败，绑定源码在该请求期间未变（`sourceUnchanged=true`），后续 11 条测试或性能命令均为 `not-invoked-prerequisite-failed`。原因已追踪到 Editor 测试迁移后的旧 ZUI include 和 ServiceHandle/Result fixture：24 个缺失 include 诊断与 4 个 E0282 级联构成 28 个 lib-test 错误，另 12 个 integration source diagnostics；JSON 42 条含两个 summary。实际证据 `E:/cargo-targets/zircon-local/source-repairs/current-main-20261008/batch-native-v2/terminal-receipt.json` SHA256 `3ee0c9458fb7e1595a5c904da406981b9b6eaabcf780fc5771c8f5e52972a1ff` 与 `actual-compile-errors.json`。

这一结果没有执行本条的原生功能测试或性能 producer。此前 16 命令 source-drift 拒绝历史完整保留。本次后续控制器 fixture 已接入，UI 测试和生产合同最小修复继续审查；15 命令 successor 草案尚未绑定或启动。后续保持一批运行原测试与原性能矩阵，提交后继续功能修复候选，在实质源码/证据检查点一次核对终态。原测试、性能、产品、Jenkins 和里程碑完成项继续开放。
