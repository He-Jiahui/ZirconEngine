---
related_code:
  - zircon_app/src/tests
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
tests:
  - zircon_app/src/tests/prelude.rs
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# App tests逐文件性能静态审查（2026-07-19）

`zircon_app/src/tests/**`当前实际 **2/2** 个Rust文件、106行、2条测试已逐文件阅读；旧库存3个已修正。`mod.rs`仅挂接prelude tests，`prelude.rs`验证公开导出、plugin group/profile选择、诊断字符串与CoreRuntime state基础行为。

该目录没有生产热路径，也没有F0窗口创建、首帧、空闲cadence、F2 render或规模预算测试，不新增直接修复。动态入口验收继续由PERF-MVP-005/023/424..428和`zircon_app/src/entry`、`src/bin`证据承担；current-source Cargo验证器仍在Cargo启动前JSON解析失败。完成F0/F2产品运行前本目录保持`pending.md`，不进入`review.md`。
