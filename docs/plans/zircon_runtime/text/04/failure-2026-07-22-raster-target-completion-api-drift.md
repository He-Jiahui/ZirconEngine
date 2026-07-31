---
handoff_kind: failure
status: open
created_at: 2026-07-22
summary_slug: raster-target-completion-api-drift
origin_plan: docs/plans/zircon_editor/editor/09-editor-asset-management.md
fixing_plan: docs/plans/zircon_runtime/text/04-glyph-atlas-and-rasterization.md
origin_child_dir: docs/plans/zircon_editor/editor/09
fixing_child_dir: docs/plans/zircon_runtime/text/04
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/text/native_bitmap_atlas.rs
  - zircon_runtime/src/text/native_bitmap_atlas/source_cache.rs
  - zircon_runtime/src/text/parallel/raster_pool.rs
---

# Text04 栅格目标与完成结果接口漂移

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/09-editor-asset-management.md`
- 来源执行者：`editor09-runtime-asset-index-projection-r2-20260722`
- 来源执行切片：Editor09 M2.1 runtime asset index projection 精确 7 文件 Windows 门禁
- 修复责任计划：`docs/plans/zircon_runtime/text/04-glyph-atlas-and-rasterization.md`
- 交接原因：错误全部位于 Text04 栅格 owner，Editor09 精确 manifest 不含且不拥有这些源码。
- 修复责任源码当前由 `text01-mvp-foundation-recovery-20260722-r2` 持有；来源 Session 不修改 Text 源码。

## 失败现象与复现证据

Editor09 M2.1 的精确 source-manifest Windows 门禁在运行测试前编译 `zircon_runtime` 失败：

- `native_bitmap_atlas.rs` 与 `source_cache.rs` 导入不存在的 `TextRasterWorkTarget`（E0432）；
- `source_cache.rs` 解构 `TextRasterCompletionDrain` 时要求不存在的
  `stale_page_generation_ids`、`stale_page_generation_count` 字段（E0026）；
- `native_bitmap_atlas.rs` 调用不存在的 `drain_completed_for_target`（E0599），当前 pool
  只暴露 `drain_completed_for_face_epoch`。

受管证据：job `ceb4440353f8477d8c9c4433cf8c562e`，run
`b94d23c4b1694098a2edec57c5c977d3`，基线 HEAD
`6debc3e43aed7ed3ee9c7e25e38388bdd209981a`，exit code 101，测试 0；原始 stderr：
`.codex/state/session-coordinator/cargo-runs/ceb4440353f8477d8c9c4433cf8c562e/b94d23c4b1694098a2edec57c5c977d3/stderr.log`。

## 最低共享层根因

native bitmap atlas 消费端与并行 raster pool 的 target identity、completion drain 字段及
drain 方法没有在同一个 Text04 切片中原子收敛，导致已提交到共享 HEAD 的接口集合内部不可编译。
Editor09 不拥有这些栅格接口，不能用本地兼容 shim、条件导入或复制旧结构规避。

## 架构修复验收

- Text04 明确唯一的 raster target identity 与 completion drain 合同，并原子更新 pool、atlas、
  source cache 及其单元测试；旧 API 硬切删除，不保留双入口。
- target/generation 过滤、stale completion 统计与 face invalidation 保持同一 owner，不能把
  丢弃结果重新归类为普通 unknown completion。
- 在冻结的 current-source manifest 上通过 Text04 聚焦测试及 `zircon_runtime` lib 编译门，
  独立 review 为 0/0/0。
- Text04 形成受管 commit SHA 后，Editor09 重新物化精确验证副本并重跑
  `editor_asset_index_projection`；旧 job 不得作为 Editor09 GREEN 证据。

## 禁止临时方案

- 禁止在 Editor09 或其他消费者中重建 `TextRasterWorkTarget`、补兼容字段或按方法存在性分支。
- 禁止只修导入名而不验证 completion 的 generation/target 过滤语义。
- 禁止使用当前共享工作树上的偶然编译结果替代冻结 source-manifest 证据。

## 修复结果与回传

2026-07-31 current-source 实现复核：Text raster work/completion 已统一携带 `face_epoch`，worker pool 唯一消费入口为 `drain_completed_for_face_epoch(..., TextRasterCompletionDrainBudget)`；native atlas/source cache 已原子消费 `accepted`、`face_invalidated_ids` 与 `face_invalidated_count`。旧 `TextRasterWorkTarget`、`drain_completed_for_target`、completion-side `stale_page_generation_ids/count` 在 Text owner 扫描为 0；真实 page generation 继续由 atlas allocation/staging/upload 校验。7 月 31 日 partial-feature Runtime lib check 已编译这些生产 owner，但没有执行 focused lib-test 或 Editor09 upward gate。

Open state: `target/completion hard cut implementation_complete / managed_validation_pending；等待 Text04 focused current-source gate、独立复审与受管 commit SHA，再由 Editor09 物化新冻结副本重跑原门禁`。

## 产出记录与时间

- 2026-07-22：状态 `open`。已登记 Editor09 受管门禁的 4 条 Text04 编译阻断，保留完整
  job/run/raw stderr 定位，并将修复责任路由至 Text04；等待 Text owner 原子收敛接口、验证、
  复审并回传受管 SHA。
