---
status: completed
owner_plan: docs/plans/zircon_editor/editor/15-build-export-and-publishing.md
milestone: M1
slice: 1.1
updated_at: 2026-07-12 18:10 +08:00
---

# Editor 15 M1.1 ExportPipeline 契约与编排产出

## 完成范围

- 新增 `zircon_runtime_interface::export` 中立契约，统一 `ExportStage`、typed 256-bit
  `ExportDigest`、`ExportArtifactRef`、`ExportStageIo`、stage record/status 与 pipeline report。
- `ExportStage::ALL`、`cli_id`、`report_name` 与 `FromStr` 成为唯一阶段定序和命名入口。
- 新增 headless `zircon_editor::core::export::ExportPipelinePlan`，实现重复/缺失依赖/环检测、
  拓扑执行、BLAKE3 指纹、显式 Skipped、失败 partial report 与 typed source error。
- runtime validate report、pack binary、editor wizard、retained wizard tests、desktop export plugin
  全部直接依赖 interface `ExportStage`。
- 删除 runtime `ExportPipelineStage` 定义与 re-export、pack binary 私有 `ExportStage`、editor
  `wizard/stage.rs` 及解析/命名/全阶段转发 helpers；无旧类型 alias 或兼容 re-export。
- 新增并同步模块文档：`docs/zircon_editor/core/export/pipeline.md`；更新 runtime export build
  plan 文档的契约 owner。

## 验证证据

- Windows 受管独立 core-pipeline test job
  `9c5fcc52e2ef42258d0fead4858e03ac`：4/4 通过，覆盖拓扑拒绝、相同指纹跳过、上游变更
  向下失效、失败记录与续跑。
- 当前 interface test binary：export DTO/stage focused 2/2 通过。
- scoped `rustfmt` 与 `git diff --check` 通过；tracked Rust 全树
  `ExportPipelineStage`、`zircon_runtime::plugin::ExportStage`、旧 stage helper 均为零命中。
- Editor lib no-run 未进入 Editor 15：Runtime 文本 owner 先报 2 组 E0432 与 2 组 E0063；已按
  用户要求归档至 Editor UI 03
  `failure-2026-07-12-rich-table-runtime-export-and-layout-boxes.md`，未在 Editor 15 修复或绕过。

## 后续边界

M1.1 代码切片完成，但 Editor 15 的 M1 里程碑未关闭。下一步执行 M1.2：版本壳 `.zpreset`、
preset 驱动向导以及 CompileHost/PlatformBundle stage executor；之后统一运行 M1 计划声明的
interface、Editor 与 platform-policy 测试阶段。

## 产出记录与时间

| 时间 | 状态 | 完成项目 |
| --- | --- | --- |
| 2026-07-12 18:10 +08:00 | 已完成 | M1.1 shared export DTO、core pipeline、八阶段 hard-cut、测试与模块文档完成；M1.2 和 M1 testing stage 仍待执行。 |
