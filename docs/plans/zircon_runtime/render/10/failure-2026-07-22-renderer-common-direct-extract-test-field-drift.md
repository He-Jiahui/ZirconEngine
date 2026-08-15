---
handoff_kind: failure
status: source_complete_dynamic_validation_pending
created_at: 2026-07-22
summary_slug: renderer-common-direct-extract-test-field-drift
origin_plan: docs/plans/zircon_plugins/01-plugin-architecture-core.md
fixing_plan: docs/plans/zircon_runtime/render/10-renderer-family.md
origin_child_dir: docs/plans/zircon_plugins/01
fixing_child_dir: docs/plans/zircon_runtime/render/10
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/core/framework/render/scene_extract.rs
  - zircon_runtime/src/scene/tests/render_extract/direct_sections.rs
tests:
  - cargo test -p zircon_runtime --lib native_callback_can_reenter_live_host_descriptor_without_deadlock --no-default-features --features core-min --locked --jobs 1 --message-format short --color never -- --test-threads=1 --nocapture
---

# Render10：RendererCommon hard-cut 后 direct extract 测试仍读取旧字段

## 来源执行者

- 来源计划：`docs/plans/zircon_plugins/01-plugin-architecture-core.md`
- 来源执行者：`plugins01-native-callback-stable-owner-r1-20260722`
- 来源执行切片：native callback stable-owner focused Windows lib-test
- 修复责任计划：`docs/plans/zircon_runtime/render/10-renderer-family.md`
- 交接原因：`RenderMeshSnapshot` 的 `RendererCommon` hard-cut、场景提取夹具与字段投影由 Render10 活跃会话持有；Plugins01 不应在插件回调验证中越权改写渲染契约。

## 失败现象与复现证据

托管 reservation `226e5974212640de8a81b5db858f1d5b`、job
`9f186eaafc6946748b1f07ded964d17e`、run `c2b30ba68dca4763bc05ad56cb621ee0`
执行：

```text
cargo test -p zircon_runtime --lib native_callback_can_reenter_live_host_descriptor_without_deadlock --no-default-features --features core-min --locked --jobs 1 --message-format short --color never -- --test-threads=1 --nocapture
```

在 Plugins01 已识别并修复测试支撑导入/可见性错误后，编译尾部仍稳定报告：

```text
zircon_runtime/src/scene/tests/render_extract/direct_sections.rs:84:14:
error[E0609]: no field `render_layer_mask` on type `&scene_extract::RenderMeshSnapshot`
```

当前 `RenderMeshSnapshot` 已只公开 `common: RendererCommon`；同一测试中的 sprite
断言以及其他 render-extract 测试均通过 `common.layer_mask` 读取统一渲染层。第 84 行仍直接读取
hard-cut 前的 `dynamic_row.render_layer_mask`，使所有 `zircon_runtime --lib` 测试目标在执行
focused assertion 前失败。

## 最低共享层根因

RendererCommon 字段收敛已修改生产结构，但 direct extract 夹具中的 mesh 断言未同步到新的
单一字段 owner，导致生产类型与测试镜像漂移。这是 Render10 hard-cut 的最底层测试消费者回归，
不是 native callback 实现错误。

## 架构修复验收

- 将 mesh 层掩码断言切到 `dynamic_row.common.layer_mask`，与 `RenderMeshSnapshot` 当前单一 owner 对齐。
- 保留 scene-schema v1 mask 的 lossy 投影断言值 `0b0010`，不得删减行为覆盖。
- 搜索 RenderMeshSnapshot 消费者，确认没有其他已删除的顶层 renderer-common 字段读取。
- 运行 Render10 focused extract tests，并重跑上述 Plugins01 原始 `zircon_runtime --lib` 向上门。

## 禁止临时方案

- 不得为通过旧测试把 `render_layer_mask` 冗余字段重新加回 `RenderMeshSnapshot`。
- 不得在 Plugins01 测试过滤或 feature gate 中绕过 scene 测试模块编译。
- 不得删除层掩码行为断言。

## 修复结果与回传

Resolving state：Render10 活跃 owner 已把 direct mesh 断言同步为
`dynamic_row.common.layer_mask.to_scene_schema_v1_mask_lossy() == 0b0010`，没有恢复冗余字段，
Rust `1.94.1` scoped rustfmt 与 `git diff --check` 已通过。当前仍待 Render10 focused extract
测试与 Plugins01 原始 focused lib-test 向上门共同 GREEN；完成前本 failure 保持 `open`。
