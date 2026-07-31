---
record_kind: milestone
status: resolving_failure
created_at: 2026-07-18
plan: docs/plans/zircon_runtime/render/10-renderer-family.md
workflow_node: RF-M1
slice: RF-M1a-renderer-common-contract
session: render10-rf-m1a-renderer-common-contract-r2-20260718
related_code:
  - zircon_runtime/src/core/framework/render/renderer_common.rs
  - zircon_runtime/src/core/framework/render/mod.rs
tests:
  - render_renderer_common_default_preserves_visible_mesh_semantics
  - render_material_override_set_keeps_one_handle_per_sorted_slot
  - render_material_override_set_deserialization_normalizes_unsorted_duplicate_slots
  - render_renderer_common_modes_resolve_shadow_and_velocity_contracts
---

# RF-M1a RendererCommon framework contract

## 完成项目

- 新增 WGPU-free `RendererCommon` framework contract，集中拥有 `enabled`、render layer、queue override、cast/receive shadows、motion vectors、material slot overrides、static 标记和 `LodGroupId`。
- `CastShadowsMode` 固定 `Off/On/TwoSided/ShadowsOnly` 语义；`MotionVectorMode` 固定 `Auto/ForceOn/ForceOff`，并提供面向后续 relevance/velocity 消费端的纯函数解析。
- `MaterialOverrideSet` 将 slot 保持为升序唯一集合，重复 slot 采用最后 authored value；内部存储保持私有，构造与反序列化统一经过 normalize，调用方不能制造未排序状态。
- `RendererCommon::default()` 保持当前 mesh 的可见、可投影、可接收阴影、自动 motion vector 与非静态语义，避免后续迁移改变现有画面。
- 通过 `zircon_runtime::core::framework::render` 固定 facade 导出全部 RF-M1a 类型，没有在 graphics/WGPU 层建立重复 DTO。

## 验证证据

- focused TDD gate：reservation `5af9fc9b126d4f5aae6cd063d1656830`，job `26c8cea505c9424d882da656a8b948f9`，run `30f8107e80ec4389b5ea23607dce1e7d`；4 passed / 0 failed / 8417 filtered，exit 0、released、无 live PIDs。
- canonical package check：reservation `83ec86ac87474739878728de39968f4e`，job `dd020101989e430387ab2632db6f960a`，run `5eb82295897b4f95b38a7854b1aebe9c`；Rust 1.94.1 执行 `cargo check -p zircon_runtime --lib --locked --jobs 1 --color never`，24m40s 后 exit 0、released、无 live PIDs。
- canonical stderr 以 `Finished dev profile` 结束；`error=0`、`renderer_common` 诊断=0。`zircon_runtime` 共报告 514 个既有 warning，本切片不把它们误写成零告警，也不越权修复。
- Rust 1.94.1 rustfmt、scoped `git diff --check` 通过；source re-review 在 deterministic deserialization 修复后为 `Critical 0 / Important 0 / Minor 0`。
- snapshot 504 closeout review 为 `Critical 0 / Important 1 / Minor 2`：源码语义与验证证据成立，但权威计划的 public slots / `RenderLayer` 描述及测试命名不一致。三项均已修复，等待 current-source focused 复跑与最终复核后恢复 accepted 状态。
- snapshot 512 exact5 保持无漂移；current-source reservation `30f046ca115d43d690968d75e94ae2bb`、job `7e3085b81c9c4a6d99b37639bd0b9813`、run `9c7d688523ad40df9350d81f27b7fe70` terminal/released exit 101。4 个 `renderer_common` 测试未执行，最低错误是 Render18 AF-M1 正在新增的 composed-WGSL 测试以 `{error}` 格式化未实现 `Display` 的 `ShaderTemplateValidationError`，共 2 个 E0277、373 warnings。
- 该下层生命周期已复用 node 503072，由 `render18-af-m1-clearcoat-vector-type-fix-20260718` 独立持有 shader/template 路径；本切片不吸收修复。只有 fixed return 后 fresh focused 4/4 通过，才能写入 `M1.1-T testing` 通过行。
- snapshot 553 exact5 current-source 静态终审为 `Critical 0 / Important 0 / Minor 0`；Rust 1.94.1 `rustfmt --check`、scoped `git diff --check` 与 staged=0 同时通过。RF-M1a 当前只剩 node 503072 fixed return 后的 fresh focused Cargo 门，不再存在独立 review 缺口。

## 未完成项目

- RF-M1 后续仍需让 `RenderMeshSnapshot`、`RenderSpriteSnapshot`、ECS 组件和编辑器面板复合该 contract，并硬切散落字段。
- node 503072 的 AF-M1 shader/template fixed return 与随后 current-source focused 复跑仍开放；本次 exit 101 不计 `RendererCommon` 回归结果。
- 计划 04 relevance、计划 05 shadow pass、计划 06 velocity，以及 ShadowsOnly 产品命令数证据尚未接线；本记录不申领 RF-M1 总门。
- RF-M2 batching、RF-M3 LOD Group、RF-M4 renderer registry 和任何 WGPU 产品截图均不属于 RF-M1a。
