---
doc_type: feature-completion
status: applied_static_review_complete_native_validation_pending
validation_status: source_receipt_and_extent_review_complete
performance_status: native_product_acceptance_open
date: 2026-10-08
plan_sources:
  - docs/plans/optimize/zircon_editor/04/2026-10-08-activity-virtualization-and-resize.md
---

# Editor04 Activity completion list

This list records the bounded Activity row adoption and its source review.  It
does not close managed Cargo, native product, or performance gates.

| Work item | Evidence | State |
|---|---|---|
| Bound physical Activity rows to viewport plus overscan | `ActivityAssetVirtualization` maps reusable slots to logical folder/asset indices and preserves complete logical count and extent | applied; static review complete |
| Preserve mixed folder/asset geometry and logical pointer identity | Activity layout/projector use generation bindings; the 100k tail paints and routes by original UUID in the reviewed product case | applied; execution pending |
| Share immutable folder generation | `AssetWorkspaceSnapshot.visible_folders` and `AssetContentFolderIds` use `Arc`; same-generation resize cases retain pointer identity | applied; static review complete |
| Recompute physical budget on resize | Small, grown, and shrunk panel cases check slot-budget recomputation, stable node values, and shared logical generation | applied; execution pending |
| Preserve full logical scrollbar extent | Core extent review confirms `list_height` comes from complete folder/asset counts and is consumed by metadata, scrollbar, and virtualization | reviewed; no source correction required |
| Update fixtures for the Arc owner | Screenshot fixture uses `m3_asset_folders().into()`; pointer regression retains the expected `Vec<String>` comparison through the test adapter | applied; static review complete |
| Actual source adoption | `actual-reviewed-canonical-apply.json` records 17 targets, apply/check/rustfmt/diff exit code 0, and unchanged controls | complete |
| Managed Cargo/typecheck and focused regressions | No Cargo or coordinator test result is present in the adoption receipt or extent review | pending |
| Native 1/100/10k/100k product runs | Open/close/reopen, click/pointer/resize interaction, and native frame evidence | pending |
| Release performance and memory gates | P50/P95/P99, allocation, RSS, latency, and successful-present evidence | pending; acceptance open |

## Evidence references

- Actual adoption: `E:/cargo-targets/zircon-local/source-repairs/current-main-20261004/editor04-final-review-v2/actual-reviewed-canonical-apply.json` (`source-applied-native-validation-pending`; native tests and performance both false).
- Bounded 16-command terminal request: `E:/cargo-targets/zircon-local/source-repairs/current-main-20261004/batch-inputs-v1/terminal-receipt.json` (`not-invoked-source-changed` on command 1, prerequisite-failed for the rest, `sourceUnchanged: false`; no Cargo result).
- Logical extent review: `E:/cargo-targets/zircon-local/source-repairs/current-main-20261008/editor04-core-extent-v1/core-extent-review.json`, SHA-256 `c616f26faba46e165204981fcc3e3f6f0be840b49a0be9501410c56bb4340f33`.

The extent review explicitly records that `ModelRc.metadata` borrows
`Option<&M>` and that the metadata owner remains alive.  No `E0716` bug or
extent regression is claimed.  The bounded terminal request was invalidated by
source drift before Cargo invocation; the remaining acceptance work must use
the repaired current source and produce fresh managed and native receipts
before this list can be closed.

## 最新追加批次终态（2026-10-09）

12 命令批次 `runtime-editor-source-repair-batch-20261009-02` 已结束，首条 `three-package-check` 为真实 Cargo `101` 编译失败，绑定源码在该请求期间未变（`sourceUnchanged=true`），后续 11 条测试或性能命令均为 `not-invoked-prerequisite-failed`。原因已追踪到 Editor 测试迁移后的旧 ZUI include 和 ServiceHandle/Result fixture：24 个缺失 include 诊断与 4 个 E0282 级联构成 28 个 lib-test 错误，另 12 个 integration source diagnostics；JSON 42 条含两个 summary。实际证据 `E:/cargo-targets/zircon-local/source-repairs/current-main-20261008/batch-native-v2/terminal-receipt.json` SHA256 `3ee0c9458fb7e1595a5c904da406981b9b6eaabcf780fc5771c8f5e52972a1ff` 与 `actual-compile-errors.json`。

这一结果没有执行本条的原生功能测试或性能 producer。此前 16 命令 source-drift 拒绝历史完整保留。本次后续控制器 fixture 已接入，UI 测试和生产合同最小修复继续审查；15 命令 successor 草案尚未绑定或启动。后续保持一批运行原测试与原性能矩阵，提交后继续功能修复候选，在实质源码/证据检查点一次核对终态。原测试、性能、产品、Jenkins 和里程碑完成项继续开放。
