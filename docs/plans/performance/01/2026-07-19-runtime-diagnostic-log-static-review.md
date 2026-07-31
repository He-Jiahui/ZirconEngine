---
related_code:
  - zircon_runtime/src/diagnostic_log
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md
  - docs/plans/zircon_runtime/render/17-performance-and-profiling.md
reference_sources:
  - dev/bevy/crates/bevy_log/src/lib.rs
  - dev/godot/core/io/logger.cpp
tests:
  - zircon_runtime/src/diagnostic_log
  - current-source Windows Cargo and log-storm product traces pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime diagnostic_log逐文件性能静态审查（2026-07-19）

## 范围与结论

`zircon_runtime/src/diagnostic_log/**`当前源 **7/7** 个Rust文件、**1,290** 行、**16** 条测试已逐文件阅读，覆盖filter/env、platform path、settings、diagnostic snapshot schedule、console/file sink与timestamp。

主要热点是每条允许日志都在caller线程执行`chrono::Local::now().format().to_string()`、整行format与newline replace，然后竞争一个`Mutex<Option<File>>`，同步`write_all`并立即`flush`。所有线程和所有scope共享该锁；慢盘会直接进入runtime/editor关键路径。`filter_for_scope`又为每条日志线性扫描全部module rules。调用方若先`format!`再进入filter，disabled路径仍会分配。

## 本轮直接止损

`write_diagnostic_store_snapshot`保留公开`format_diagnostic_store_snapshot` API，但写sink时直接对series执行`filter_map(format_diagnostic_series)`，删除一次完整`Vec<String>`中间所有权。RED→GREEN源码守卫、`rustfmt`与`git diff --check`通过。

## 参考与计划

Bevy在启动时构建`EnvFilter`并通过tracing layer过滤；Godot `RotatedFileLogger`明确只在error或配置要求时flush，并注释说明不应每条stdout都flush以避免性能损失。Zircon不复制API，但需要compiled filter、lazy message gate、bounded sink owner和批量flush，详见PERF-MVP-434及Runtime07 failure记录。

## 动态验收

1/1k/100k logs/s、1/64 callers、0/10/1k scopes与0/10/100ms慢sink下记录caller allocation、timestamp、mutex wait/hold、write/flush、queue age/depth/drop与p95；验证error durability、顺序、rotation、shutdown和crash flush。current-source Cargo与F0/F2产品trace完成前留在`pending.md`。
