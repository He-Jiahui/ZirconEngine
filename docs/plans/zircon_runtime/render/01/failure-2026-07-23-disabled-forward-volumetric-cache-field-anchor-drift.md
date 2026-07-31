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

Render01 的 disabled binding cache ownership 与测试锚点发生漂移，当前证据无法判定 buffer 已迁移到等价 owner，还是生产路径重新引入 per-pass allocation。

## 架构修复验收

- Render01 判定 cache-owned disabled buffer 合同仍应保留，还是守卫应迁到新的等价 owner/字段。
- 保持 disabled binding 不在 per-pass 路径创建 GPU buffer。
- 以 Render01 精确 current-source test、向上门禁、独立 review 和 managed commit 回传；Text01 不修改上述 Render 路径。

## 禁止临时方案

不得只恢复旧字段名匹配守卫、删除结构断言，或在每个 render pass 临时创建 disabled volumetric buffer。

## 修复结果与回传

Open state: `待 Render01 确认 current cache owner、修正生产或守卫，并取得精确 current-source 与向上门禁证据`。
