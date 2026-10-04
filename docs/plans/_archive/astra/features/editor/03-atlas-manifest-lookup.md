---
status: in_progress
plan_sources:
  - docs/plans/optimize/zircon_editor/211-editor-sprite-atlas-tileset-tilemap-canvas2d-animation-collision-preview-current-source-review.md
---

# 图集候选查找与共享缓存

## 当前源码确认

Editor211 的 multi-manifest resolver 错误仍存在：循环中的 `?` 在第一个损坏或
无对应 entry 的 manifest 结束整个查找。缓存还在每次不同 entry 解析时 clone
整份 SpriteAtlasAsset，entries 越多越昂贵。

## 实施与验收

- 按原排序继续扫描损坏和不含 entry 的候选，保留第一份真正匹配 manifest 的优先级。
- manifest 缓存持有不可变 Arc；命中只增加共享引用，保留负缓存、64 项上限及原淘汰顺序。
- 回归使用真实 TOML、图像与生产解析器，覆盖多个不匹配/损坏候选、第一匹配和全不匹配。
- 独立 cache 实例验证命中共享同一 manifest allocation，避免全局缓存清理造成并发测试干扰。
- release 基准对比 1/1k/10k entries 的旧版 clone 命中与当前生产 cache 命中，预热 8
  次、101 个交错样本，报告 p50/p95/p99。1 项 p95 回退不得超过 5%，1k/10k 目标
  为旧版的 20% 以内；不将结构上的减少复制当作实测。

## 计划完成列表

| 批次 | 内容 | 状态 | 验证证据 |
|---|---|---|---|
| M4 | 多图集查找修复与共享 manifest 缓存 | implemented_pending_validation | 独立复核无阻断项，rustfmt 与 diff-check 通过；M1-M4 合并验证输入已准备，尚未运行到测试与性能采样 |

## 批量验证交接

输入保存在 `.codex/state/astra-m1-m4-validation-20260905.json`：16 个本轮修改路径和
33 个既有 tasks 依赖路径，共 49 项 SHA-256。依赖 overlay 不代表本轮创作或可提交所有权。
后续重提前必须重新核对当前 hash 和租约归属。

批次同时覆盖 panic 诊断、进度缓冲、asset/UI root、导出 failure receipt 和 atlas
cache，并包含 host `profile_files_preflight` 回归。命令：
`cargo test --locked --release --no-default-features -p zircon_runtime -p zircon_editor -p zircon_runtime_host --lib -- --include-ignored --nocapture --test-threads=1 astra_m profile_files_preflight`。

M1/M2 诊断曾报 host visitor E0277，缺失约束已在源码修复；前一个 managed job
`f95a4c5f25e549a49c21bec5bb43b694` 未取得终态回执。完成 M3、M4 后申请新合并批次
时，兼容 pool 仍占用，随后协调器 descriptor 不可用。合并 scope 注册请求
`43533c9d09ac4782ad03e904d1370a6c` 待对账；`astra-m1-m4-runtime-editor-20260905-v2`
只是预留请求号，尚未创建 validation ticket。

不等待、不持续监控编译；继续独立 runtime/editor 工作，下一次处理此批次时对账、
取得租约并提交一次。当前无实测性能数据，不把源码复核或格式检查标为性能达标。

2026-09-05 后续：49 项输入核对一致；已有 session scope 不能通过重复注册扩展，
该注册请求的终态为 `session_write_scope_immutable`。通过 coordinator 的当前工作树
诊断入口完成 M1-M4 整批提交：job `d82b2e0f13904008a48300a5a5957f87`，run
`aa0b976d340e4042908b9ffe5e3ed23a`，提交回执 `running`，复用前批缓存。
日志路径已写入上述 JSON。它仍不是不可变快照验收；提交后继续独立导出终态修复。
