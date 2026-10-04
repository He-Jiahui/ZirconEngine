---
status: in_progress
plan_sources:
  - docs/plans/optimize/zircon_runtime/04-core-resource-asset-serialization-review.md
  - docs/plans/optimize/zircon_runtime/64-runtime-resource-authority-asset-handle-load-request-state-machine-version-lease-cache-dependency-reload-cancellation-product-integration-review.md
---

# Runtime 模块接线修复

## 编译诊断与修复

承接 runtime 合并诊断 run `5e88cd0079d24558a2e427a15b45cee6`：补齐已存在的
场景 AO 类型、RenderFrameScenePayload、动态场景字段转换的模块导出；native
registration replay 从实际声明模块导入错误类型。

辅助源 resolver 改用 `ResolvedProjectPath` 的 operation/display 合同，保留目录
边界检查；IBL staging 使用 typed `From` 保留错误来源。修正测试移动共享数据、
借用诊断数组、旧 include 路径和可变基准闭包问题。

这些属于既有功能和测试的编译修复，不扩展新渲染功能；剩余迁移错误继续合并处理，
不在未运行基准时宣称性能达标。

## 计划完成列表

| 批次 | 内容 | 状态 | 验证证据 |
|---|---|---|---|
| M8 | 模块导出、辅助源路径、typed error、共享数据测试接线 | implemented_pending_validation | 当前编译日志定位，待统一验证 |
