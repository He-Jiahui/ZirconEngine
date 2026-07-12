# Physics M6 编辑器与调试产出记录

## 归属

- 父计划：`docs/plans/zircon_plugins/03-physics.md`
- 里程碑：M6-T1 Physics Debug Overlay；M6-T2 Physics Diagnostics；M6-T3 Ragdoll Profile Editor
- 当前状态：`plugins_03_m6_editor_debug_complete`
- 记录日期：2026-07-12

## 完成项目

- Physics editor 注册 `physics.debug_overlay` view、viewport tool mode、toggle command 与精确菜单路径 `View/Debug Overlays/Physics`；overlay DTO 将同步碰撞体投影为 wireframe primitive，并对 solid collider 与 trigger 使用不同颜色。
- 新增 `physics.diagnostics` view 与 ZUI 模板；`PhysicsRuntimeSystem` 在每次 fixed step 边界将 `physics.step.duration_ms`、frame index、毫秒单位及 `physics/step` tags 写入 Core diagnostic store，manager 缺失路径同样留有耗时样本。
- 新增 `physics.ragdoll_profile` asset editor、创建命令和 ZUI 模板；初始 profile 生成器按 skeleton bone 拓扑生成 capsule、质量与父子映射，并复用 runtime `RagdollProfile::validate` 作为唯一合法性门。
- 补齐 `authoring.zui`、`debug_overlay.zui`、`diagnostics.zui`、`ragdoll_profile.zui` 四个可分发 UI 资产，保持 authoring macro 和 runtime manifest mirror 路径不变。
- 编辑器功能拆分为 `overlay.rs`、`ragdoll_profile_editor.rs`、`extension_ids.rs` 与声明式 `plugin.rs`，最大新增生产 owner 低于 200 行，没有继续堆叠根文件。

## 测试与验证

- `overlay_registration_snapshot_matches_physics_debug_contract`：锁定 view/template/tool/command/menu/diagnostics/ragdoll editor 注册合同。
- `generated_profile_covers_all_mapped_bones`：验证生成 profile 覆盖 skeleton 映射并通过 runtime 验证。
- `physics_overlay_colors_triggers_separately_from_solid_colliders`：验证 overlay primitive 与颜色语义。
- `physics_step_duration_is_published_to_diagnostic_store`：验证 fixed-step 耗时进入共享诊断存储。
- Windows editor managed Cargo job `1a7dfa0bdaf5491a9a116d41b4b793c3` 在 Physics editor 最后一次行为改动后通过 4/4；最新 default runtime job `acb783ff253a48c48c6b776e809f355b` 通过 59/59；最新 Jolt job `808fd60e6cca4b20b425b22cb205f872` 通过 73/73。
- 后续仅格式化 Physics 源码后的 editor 重跑 jobs `b786e6a90a2945fe842bd6871fb5e61c`、`f4401e6bba854e55bce0cc5912d582e8` 均在编译外部 `zircon_editor` 时因共享工作区缺失 `zircon_runtime::plugin::ExportBuildPlanError` 退出 101，未到达 Physics editor crate；该跨会话基础层漂移不推翻既有 4/4 产品证据，也未在本会话越权修复。
- 四个 ZUI 资产逐项通过 `validate_plugin_distribution_zui_asset`；全插件结构审计报告零违规；scoped rustfmt、`git diff --check` 与 owner 行数预算通过。

| 里程碑 | 测试阶段 | 状态 |
|---|---|---|
| M6 | M6-Testing：overlay、diagnostics、ragdoll editor、ZUI/结构审计 | 通过 |

## 验收结论

- M6-T1、M6-T2、M6-T3 与 M6-Testing 均完成。
- 编辑器只消费 Physics/runtime 中立 DTO 和 SDK 扩展注册接口；没有把 UI 类型带入 runtime，也没有制造兼容 facade。
