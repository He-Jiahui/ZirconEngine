---
status: in_progress
plan_sources:
  - docs/plans/optimize/zircon_runtime/58-runtime-plugin-interface-bridge-slot-generation-strong-weak-native-vm-lifecycle-diagnostics-product-integration-review.md
---

# Native plugin discovery 测试入口

## 编译诊断

合并诊断 run `5e88cd0079d24558a2e427a15b45cee6` 中 native plugin 测试的多数错误
来自 `NativePluginLoader` 未从 `plugin::native` 公开导出，导致后续 report 闭包类型
全部无法推断。测试改用现有 `native::discovery` 函数入口，保留原有报告断言；registration replay
错误类型已从实际 `registration_manifest` 模块导入。

## 计划完成列表

| 批次 | 内容 | 状态 | 验证证据 |
|---|---|---|---|
| M9 | native discovery 测试入口迁移及级联类型错误修复 | implemented_pending_validation | Runtime06/Editor12 及 native authority 静态回归批次 `45/45` 通过；native public-surface audit 覆盖 86 symbols、2 locations，lifecycle source files 20、App call sites 3，`risks=[]`。Rustfmt/diff-check 通过；受管 runtime/editor Cargo 仍待合并批次。 |
