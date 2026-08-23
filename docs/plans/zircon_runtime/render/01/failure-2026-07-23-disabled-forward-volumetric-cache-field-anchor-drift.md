---
handoff_kind: failure
status: open
created_at: 2026-07-23
summary_slug: disabled-forward-volumetric-cache-field-anchor-drift
origin_plan: docs/plans/zircon_runtime/text/01-font-resource-faces-and-database.md
fixing_plan: docs/plans/zircon_runtime/render/01-render-graph-rdg-alignment.md
origin_child_dir: docs/plans/zircon_runtime/text/01
fixing_child_dir: docs/plans/zircon_runtime/render/01
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/mesh_pipeline_cache.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/construct.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/forward_shadow_receiver.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/advanced_lighting/froxel/apply_binding.rs
---

# Render01：disabled forward volumetric cache 字段锚点漂移

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/text/01-font-resource-faces-and-database.md`
- 来源执行切片：Text01 current-source default/UI lib-test 的 Render01 structure guard
- 修复责任计划：`docs/plans/zircon_runtime/render/01-render-graph-rdg-alignment.md`
- 交接原因：disabled forward volumetric buffer 的 cache owner 与结构守卫都属于 Render01，Text01 只记录过滤串误命中的 current-source 失败。

## 失败现象与复现证据

Text01 的 current-source default/UI lib-test job `f9f5581fb83b40c2a3cc81aa15f5bcaa`、run
`b98dc769094b4bd9b96fc445fd8a1332` 因过滤串 `text::` 同时命中 `context::`，执行到 Render01
结构守卫 `disabled_forward_volumetric_params_buffer_is_cache_owned`。该测试在
`gpu/tests.rs:18` 失败：`mesh_pipeline_cache.rs` 不再包含
`forward_volumetric_disabled_params_buffer: wgpu::Buffer`。

该 job 于 `2026-07-22T19:24:42.482382+00:00` 自然结束并由协调器 release，exit `101`、
live PIDs 为空；完整批次为 `776 passed / 7 failed / 2 ignored / 8083 filtered`。原始日志位于
`.codex/state/session-coordinator/cargo-runs/f9f5581fb83b40c2a3cc81aa15f5bcaa/b98dc769094b4bd9b96fc445fd8a1332/`。

## 最低共享层根因

生产 owner 没有发生回退：`MeshPipelineCache` 仍持有 disabled buffer，构造器创建一次，disabled binding 继续借用该字段。失败来自 structure guard 把 `forward_volumetric_disabled_params_buffer: wgpu::Buffer` 当作必须位于同一行的源码锚点；rustfmt 将字段名和类型折行后，行为未变但 guard 产生 false RED。当前 guard 修复改为检查相邻字段/类型、构造期创建与 disabled binding 引用，不再依赖单行排版。

## 架构修复验收

- 保留 cache-owned disabled buffer 合同，并让 structure guard 不依赖 rustfmt 单行排版。
- 保持 disabled binding 不在 per-pass 路径创建 GPU buffer。
- 以 Render01 精确 current-source test、原 Text01 向上门禁、独立 review 和 managed commit 回传；Text01 不修改上述 Render 路径。

## 禁止临时方案

不得只恢复旧字段名匹配守卫、删除结构断言，或在每个 render pass 临时创建 disabled volumetric buffer。

## 修复结果与回传

Open state: `生产合同原已存在，structure guard 的 false RED 修复已落地；待 Render01 精确 current-source 与原 Text01 向上门禁取得 managed GREEN`。

- `MeshPipelineCache` 唯一持有 `forward_volumetric_disabled_params_buffer`；构造期通过
  `create_disabled_params_buffer` 创建一次，实际 allocator owner 位于 `advanced_lighting/froxel/apply_binding.rs`。
- disabled forward-volumetric binding 复用该 cache-owned buffer；仅启用体积雾的 shading
  binding 保留其动态参数路径，未在 disabled per-pass 路径分配 GPU buffer。
- `disabled_forward_volumetric_params_buffer_is_cache_owned` 已同时约束字段、构造期创建和
  disabled binding 引用，防止上述漂移回归。
- 本记录保持 `source_complete_dynamic_validation_pending`，因为精确 Render01 test 与原 Text01
  向上门禁尚未取得受管终态；不得将源码审阅替代为 managed GREEN。
