---
title: Runtime Bin Currentness Protected Plan Routing
date: 2026-08-23
status: routing_only
related_report:
  - docs/plans/performance/01/2026-08-23-runtime-bin-currentness-composite-revalidation.md
protected_targets:
  - docs/plans/performance/review.md
  - docs/plans/performance/pending.md
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
---

# Runtime Bin Currentness Protected Plan Routing

本记录只提供受保护计划的后续归并输入；本轮没有修改 `review.md`、`pending.md` 或编号总计划。

## 建议写入 `review.md` 的模块级条目

暂不写入。`zircon_runtime/src/bin` 的 42/42 Rust 文件已完成当前源码静态组合复验，但 current-source Windows Cargo、真实 cold/warm tool trace、WPR/ETW 和产品 PSO exact-hit 证据均缺失，不满足验收条件。

## 建议保留在 `pending.md` 的模块级条目

`zircon_runtime/src/bin`：静态组合审查完成；等待受管 Windows current-source 构建以及 shader prewarm/export pack 的 cold/warm/scale/RSS/I/O/worker/PSO exact-hit 动态证据。不要把 `zircon_export_validate` 的 JSON 微调当作该模块关闭条件。

## 建议归并到现有计划 owner

| 问题 | 目标计划 |
|---|---|
| shader source/module/permutation/prewarm/PSO authority、scheduler 与 exact-hit | Runtime91 |
| source/import/build graph/DDC/cook/chunk/pack/delta | Runtime85 |
| Validate/Build/Cook/Pack/Bundle stage truth 与 receipt | Tooling03 |
| binary target identity、CLI protocol、artifact/process/qualification receipt | Tooling18 |

`PERF-MVP-448/449` 只保留为历史性能 finding/路由别名，不新增并行 implementation owner。总计划下次由其 owner 更新时，应纠正“prewarm 没有 bounded inventory/SCC/source validation dedup”的旧描述，并把开放问题改为“串行 worker、无共享 admission/cancel/priority、无 runtime exact PSO hit 证明”。

## 晋级门

只有同时满足以下条件，模块级条目才可从 pending 晋级 review：

1. 受管 Windows Cargo 对当前 source fingerprint 生成 artifact/build receipt。
2. prewarm 与 pack 规模矩阵完成 cold/warm/增量/取消测试，输出 wall time、CPU、RSS、I/O、队列和 cache/chunk/PSO hit 数据。
3. WPR/ETW 证明主线程/worker/文件 I/O 无异常堆积；Renderer 产品链另用 RenderDoc 证明 pipeline/cache marker 和帧行为。
4. Runtime85/Runtime91/Tooling03/Tooling18 的 owner 归属保持唯一，未新增 facade、compat 路径或第二套 manifest truth。
5. 所有证据绑定同一 current-source artifact identity；旧二进制、静态推断或退出码 0 不得代替动态资格。

