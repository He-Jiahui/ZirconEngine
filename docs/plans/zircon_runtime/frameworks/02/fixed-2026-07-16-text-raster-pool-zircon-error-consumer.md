---
handoff_kind: fixed
status: fixed
created_at: 2026-07-16
summary_slug: text-raster-pool-zircon-error-consumer
origin_plan: docs/plans/zircon_runtime/frameworks/02-module-kernel-and-lifecycle-unification.md
fixing_plan: docs/plans/zircon_runtime/frameworks/05-subsystem-decoupling-contracts.md
origin_child_dir: docs/plans/zircon_runtime/frameworks/02
fixing_child_dir: docs/plans/zircon_runtime/frameworks/05
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/core/runtime/error.rs
  - zircon_runtime/src/text/parallel/raster_pool.rs
  - tools/tests/test_frameworks_02_core_error_single_source.py
tests:
  - python tools/tests/test_frameworks_02_core_error_single_source.py
  - cargo test -p zircon_runtime --lib --locked text_raster_worker_pool
resolved_at: 2026-07-16
---


# Frameworks 05: Text raster pool stale ZirconError consumer

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/frameworks/02-module-kernel-and-lifecycle-unification.md`
- 来源执行切片：M1 current-source `CoreError` single-source hard cut
- 修复责任计划：`docs/plans/zircon_runtime/frameworks/05-subsystem-decoupling-contracts.md`
- 交接原因：Frameworks02 已把内核错误单源硬切到 `CoreError`，而当前 Frameworks05 text hard-cut manifest 中新增的 raster worker 仍直接消费退役错误类型；最低 stale consumer owner 是 Text parallel raster pool。

## 失败现象与复现证据

先写结构守卫并执行 `python tools/tests/test_frameworks_02_core_error_single_source.py`。首次 RED 因当时的并行 framework error owner 仍声明旧错误枚举而失败；Frameworks02 完成生产 owner、task helper、asset worker、root facade 与 prelude 的最小硬切后再次运行，守卫继续按预期失败，并精确报告：

```text
stale_consumers = ['zircon_runtime/src/text/parallel/raster_pool.rs']
```

该文件当前未跟踪、归属活动 Session `frameworks05-m3-text-hardcut-closeout-20260716`，因此来源 Session 不抢改、不把它吸收入 M1 commit。

## 最低共享层根因

`zircon_runtime/src/text/parallel/raster_pool.rs` 的构造路径仍返回退役错误类型；重复请求、队列满和 channel 断开路径直接创建旧 `ChannelSend` variant。Frameworks02 不会为此恢复 root re-export、alias 或兼容 enum；Text consumer 必须直接切到 `CoreError` / `CoreResult`。

## 架构修复验收

- `TextRasterWorkerPool::new(...)` 与 `request(...)` 直接返回 `CoreResult`，所有 channel 失败直接构造 `CoreError::ChannelSend`。
- `python tools/tests/test_frameworks_02_core_error_single_source.py` 通过且 `zircon_runtime/src/**/*.rs` 中退役类型引用为 0。
- Frameworks05 通过 Windows managed job `711bd7035e1f4e62a0def56214a6151b` 运行 `cargo test -p zircon_runtime --lib --locked text_raster_worker_pool --color never`：5 passed / 0 failed / 8173 filtered，exit 0；job 已由 owning R2 fixing session 正常 release。
- Frameworks02 随后重新运行 managed `cargo check -p zircon_runtime --lib --locked` 与 M1 lifecycle/order focused gate。

## 禁止临时方案

- 不得新增 alias、兼容 shim、旧 root re-export、双错误 enum、silent conversion 或 call-site 特判。
- 不得排除 Text 文件、缩窄结构扫描或弱化 M1 / Text03 验收来隐藏失败。

## 修复结果与回传

- 根因：Frameworks05 TextRasterWorkerPool still consumed the retired ZirconError surface after Frameworks02 converged runtime errors to canonical CoreError/CoreResult.
- 架构修复：Hard-cut TextRasterWorkerPool constructors and request paths to CoreResult and CoreError::ChannelSend; no alias, shim, compatibility re-export, or parallel error enum remains. The Text parent batch landed in ad2c6f989cfff927ff5679467ca0cc71e2e20c0e.
- 验证：python tools/tests/test_frameworks_02_core_error_single_source.py: 1/1 passed on current HEAD; Windows managed job 711bd7035e1f4e62a0def56214a6151b: cargo test -p zircon_runtime --lib --locked text_raster_worker_pool, 5 passed / 0 failed / exit 0; current source is tracked in ad2c6f989cfff927ff5679467ca0cc71e2e20c0e.
- 回传：Returned to Frameworks02 after the Frameworks05 parent Text hard-cut commit landed and current-source hard-cut evidence was rechecked.
