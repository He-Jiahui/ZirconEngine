---
status: in_progress
plan_sources:
  - docs/plans/optimize/zircon_editor/269-editor-build-export-preset-pipeline-cook-pack-platform-bundle-publishing-resume-determinism-current-working-tree-review.md
---

# 导出失败与恢复报告

## 当前源码确认

`ED-EXPORT-P0-003` 在当前源码仍可复现：向导 adapter 把非零退出包装为成功，core
pipeline 因而会保存 Passed，旧产物仍在时下一次 resume 可跳过失败的编译。
报告解码失败也被 `.ok()` 静默当作无历史记录，且失败分支丢弃了 core 的 Failed 报告。

## 实施与验收

- CompileHost adapter 只接受 `Some(0)`，非零或未知退出码返回既有 typed stage failure。
- core 执行失败时保存 Failed 报告，避免磁盘继续保留前一次 Passed 状态。
- 产物执行前先持久化不可复用的空记录，写入失败时不启动子进程；失败报告写入再出错
  时保留原始 typed failure 和 I/O 原因。取消继续返回 Cancelled。
- 私有 core receipt 增加版本 1；旧版无版本记录保留在磁盘但不参与 resume，避免
  复用历史误报。新成功报告仍使用既有 fingerprint 与磁盘产物复核。
- 损坏报告和未来版本返回 InvalidData；读取失败保留 I/O 错误，只有 NotFound 为首次运行。
- 回归覆盖非零/未知/成功退出、失败报告重试、旧版失效、损坏/未来版本及原子替换。
- 生产 CompileHost 和 PlatformBundle 均覆盖，另注入取消与 failure receipt 写入故障。
- 与 runtime/editor 前批任务合并验证，不逐项启动 Cargo。本项是正确性修复，性能
  指标沿用根目录校验与进度缓冲基准；不能宣称已关闭完整 Runtime ExportService 缺口。

## 计划完成列表

| 批次 | 内容 | 状态 | 验证证据 |
|---|---|---|---|
| M3 | 导出退出码传播、失败报告持久化、旧版恢复失效与损坏拒绝 | implemented_pending_validation | 复审确认取消、PlatformBundle 与 I/O 故障问题已修正，静态格式检查通过；合并验证见 [M1-M4 交接](03-atlas-manifest-lookup.md) |
