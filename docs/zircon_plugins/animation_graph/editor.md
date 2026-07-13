---
related_code:
  - zircon_plugins/animation_graph/editor/src/plugin.rs
  - zircon_plugins/animation_graph/editor/src/tests.rs
  - zircon_plugins/timeline_sequence/editor/src/plugin.rs
implementation_files:
  - zircon_plugins/animation_graph/editor/src/plugin.rs
plan_sources:
  - user: 2026-07-11 按 docs/plans/zircon_plugins 完整实现插件架构
  - docs/plans/zircon_plugins/04-animation.md
tests:
  - zircon_plugins/animation_graph/editor/src/tests.rs
  - zircon_plugins/timeline_sequence/editor/src/tests.rs
doc_type: module-detail
---

# Animation Graph Editor Plugin

`zircon_plugins/animation_graph/editor` 是 Animation M6 的 graph/state-machine authoring owner。它注册 `animation.graph` 和 `animation.state_machine` 两种 graph editor、compile/validate/open operations、asset editor 以及 node palette，不在基础 Animation editor 插件中复制第二套编辑器。

## M6 扩展面

- State-machine palette 包含 state/transition/condition，并与 `animation.state_machine` asset editor 共用 compile/validate operation。
- Animation-graph palette 包含 clip/blend/output，并新增 `blend_space_1d` 和 `blend_space_2d` authoring node，对应 runtime 的排序插值和 Delaunay 采样内核。
- Avatar Mask 骨骼树通过 `animation.Asset.AvatarMask` component drawer 和 `plugins://animation_graph/editor/avatar_mask_bone_tree.zui` 文档锚点注册，使 mask 编辑与 graph authoring 属于同一 Animation 工具域。
- `zircon_plugins/timeline_sequence/editor` 已是 M6-T3 的独立 owner：它注册 `animation.sequence` timeline editor 及 transform/component-property/event-marker 三类 track，不在 Animation Graph 插件中重复注册。

## 验证状态

Animation Graph 依赖链在当前共享 Windows Cargo 通道中长时间等待 Editor 链接/构建锁；本次新增注册的 source-level 四个锚点与 rustfmt/diff hygiene 通过，但未将超时命令计为 Cargo 通过。正式行为复跑列入 Animation 测试阶段。
