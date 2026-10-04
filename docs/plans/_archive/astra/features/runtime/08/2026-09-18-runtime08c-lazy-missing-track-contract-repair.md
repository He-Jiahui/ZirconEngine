---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/08c/2026-08-26-lazy-missing-track-path.md
related_records:
  - docs/plans/astra/features/runtime/170-compiled-sequence-channel-sampling.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_runtime/src/animation/sequence/compiled.rs
tests:
  - tools/tests/test_runtime08c_lazy_missing_track_path_performance_contract.py
---

# Runtime08c · lazy missing-track contract repair

## 计划完成列表

| 批次 | 内容 | 状态 | 验证证据 |
| --- | --- | --- | --- |
| M08c-contract | 在 Runtime170 的 owned source/track-capacity hard cut 后，更新 Runtime08c source contract 的 Rust block 边界与 `missing_tracks` 局部 owner 断言；保持成功 writer 不构造诊断路径、缺失 writer 保留路径的语义 | implemented_pending_validation | 相邻 Runtime/Editor source-contract 批次 `95/95` 通过；最新 57 模块 Runtime/Editor loader `203/203` 通过；Python AST、Rustfmt、scoped diff 与 Wiki 检查通过 |

## 变更边界

这是一项测试合同维护，不新增 tooling 生产代码，也不改变 Runtime 编译器行为。旧合同依赖
固定缩进、`Ok(compiled)` 结束锚点和已移除的 `compiled.missing_tracks` 字段前缀；当前合同改用
平衡 Rust block 提取与空白容忍的正则断言，继续锁定 lazy diagnostic-path 的顺序关系。

## 受管验证边界

本地源合同和结构证据已完成；Cargo 编译、忽略的 Windows Release benchmark、分配器结果以及
产品 p50/p95/p99 仍由统一异步 Runtime/Editor 批次负责，当前不作性能验收声明。
