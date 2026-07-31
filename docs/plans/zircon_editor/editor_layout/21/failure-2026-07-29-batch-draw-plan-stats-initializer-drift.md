---
handoff_kind: failure
status: open
created_at: 2026-07-29
summary_slug: batch-draw-plan-stats-initializer-drift
origin_plan: docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md
origin_workflow_node: M2
fixing_plan: docs/plans/zircon_editor/editor_layout/21-gpu-submission-and-draw-pipeline.md
origin_child_dir: docs/plans/zircon_editor/editor/01
fixing_child_dir: docs/plans/zircon_editor/editor_layout/21
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/rhi_wgpu/ui_surface/batching.rs
tests:
  - cargo test -p zircon_editor --lib gateway:: --locked --jobs 1 --color never -- --test-threads=1
---

# Layout21: BatchDrawPlanStats initializer drift

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md`
- 来源执行切片：M2 runtime-frame-demand hard cut 的 `gateway::` current-source compile gate
- 来源执行者：`editor01-runtime-frame-demand-hardcut-r1-20260729`
- 来源受管运行：job `640dc354cc38475daa1bd25e7217baf6` / run `bb226267623f4322839092a6f7365c15`
- 修复责任计划：`docs/plans/zircon_editor/editor_layout/21-gpu-submission-and-draw-pipeline.md`
- 交接原因：UI batch plan 的构造、缓存命中和统计口径属于 Layout21 的 GPU 提交边界；Editor01 只消费运行时编译结果，不能在 gateway 层补字段或改变统计语义。

## 失败现象与复现证据

2026-07-29 11:05 CST，source-manifest fingerprint `d3510f7e2cd5a4ade996a4887d77103aee3b1c710e29b2714427666e9503eed5` 的受管 focused gate 自然终态为 `exit 101`，尚未进入 `gateway::` 测试。Rust 编译 `zircon_runtime` 时报告：

```text
error[E0063]: missing fields `batch_plan_build_count` and `batch_plan_cache_hit_count`
in initializer of `BatchDrawPlanStats`
  --> zircon_runtime/src/rhi_wgpu/ui_surface/batching.rs:150:16
```

原始 stderr：`.codex/state/session-coordinator/cargo-runs/640dc354cc38475daa1bd25e7217baf6/bb226267623f4322839092a6f7365c15/stderr.log`。该运行还同时暴露 Render17 图执行导出问题，已另行路由，不应混入本工单。

## 最低共享层根因

`BatchDrawPlanStats` 已把 batch-plan build/cache-hit 指标纳入共享统计契约，但 `batch_draw_plan` 的直接构造初始化器没有同步迁移。`CompiledUiBatchPlanCache::resolve` 已在其结果上区分 build 与 cache hit；direct builder 的统计初始化和 resolve 层的指标合成缺少单一、完整的语义边界，导致 Rust 的结构体完整性检查在任何依赖 `zircon_runtime` 的上层门禁前失败。

## 架构修复验收

- Layout21 为 direct `BatchDrawPlan` 构造与 `CompiledUiBatchPlanCache::resolve` 定义一致的 build/cache-hit 统计归属，所有 `BatchDrawPlanStats` 初始化器完整表达该契约。
- 覆盖 cache miss、稳定 generation cache hit、damage/无 generation bypass 的计数测试，证明计数不被重复累计且不依赖调用方填补字段。
- `cargo test -p zircon_runtime --lib ui_surface --locked` 的受管 current-source 验证通过；随后来源命令能够编译越过本 E0063 后才可继续判定 Editor01 gateway 结果。
- 记录 immutable manifest、独立 review 和 fixed return；本 failure 在上述证据齐备前保持 `open`。

## 禁止临时方案

- 不得在 Editor01、Cargo 包装器或任一上层调用点以结构体默认值、条件编译或忽略统计的方式掩盖此错误。
- 不得删除 build/cache-hit 字段来恢复旧统计形状，也不得把 cache 指标伪装为 draw-call 指标。
- 不得将本次 `exit 101` 当作 gateway 失败或把 source-polluted 后续运行作为 Layout21 验收。

## 修复结果与回传

Open。该工单由 Layout21 接收后，需在 batch-plan owner 范围内修复、执行受管 current-source 验证并经独立审查，再以 `fixed-*` lifecycle record 回传来源计划。来源的 Editor01 网关切片保持冻结，等待外部编译链恢复后创建新的 source-bound reservation，绝不复用 job `640dc354cc38475daa1bd25e7217baf6`。

2026-07-29 12:50 CST，Layout21 已通过 failure-priority reservation
`6c56e3b131884a2788f0419544d9be78` 运行当前源 focused lib gate：job
`daf0b16f577d49d0aa8dc747d972f702` / run
`d8da72004daf439085e99efac3999c9a` 自然释放为 `exit 101`、live PIDs `[]`。
该运行未再报告 `BatchDrawPlanStats`、`batching.rs` 或 E0063，但不构成验收：完整
Runtime 输入指纹从启动前 `3126939cdea97bea6c293a2dc1e70247f22a6c159bc5b70f1aa1e79cd581779a`
变化为终态 `bcce51a05a74f42f56acc34753eb8680224b67946065954194c58a676487ca60`。
终态八个编译错误全部属于已经登记的下层责任：Render17
`scene-viewport-surface-projection-drift` 两项，以及 Runtime11 operation bounded-service
六项。原始日志保留在
`.codex/state/session-coordinator/cargo-runs/daf0b16f577d49d0aa8dc747d972f702/d8da72004daf439085e99efac3999c9a/`。
在两个下层 lifecycle fixed return 且 Runtime 输入重新冻结前，不创建或复用 Layout21
验收作业。

## 产出记录与时间

| 时间 | 范围 | 状态 | 完成项与后续门禁 |
| --- | --- | --- | --- |
| 2026-07-29 11:05 CST | Layout21 batch-plan cache statistics | failure open | 已从 Editor01 受管 job `640dc354cc38475daa1bd25e7217baf6` 的终态日志提取 E0063，并确认根因位于 `BatchDrawPlanStats` 构造/缓存统计契约。等待 Layout21 在 owned scope 完成语义修复、受管验证、独立复审和 fixed return。 |
| 2026-07-29 12:50 CST | Layout21 current-source focused lib diagnostic | diagnostic RED / source-raced | failure-priority job `daf0b16f577d49d0aa8dc747d972f702` / run `d8da72004daf439085e99efac3999c9a` 自然释放 `exit 101`、无存活 PID；本 owner E0063 已消失，但 pre/post 全输入指纹不一致，八项终态错误均已路由至既有 Render17 surface 与 Runtime11 operation lifecycle。本运行不得验收、不得复用。 |
