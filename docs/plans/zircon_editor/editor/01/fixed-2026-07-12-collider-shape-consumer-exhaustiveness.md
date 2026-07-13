---
handoff_kind: fixed
status: fixed
created_at: 2026-07-12
summary_slug: collider-shape-consumer-exhaustiveness
origin_plan: docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md
fixing_plan: docs/plans/zircon_plugins/03-physics.md
origin_child_dir: docs/plans/zircon_editor/editor/01
fixing_child_dir: docs/plans/zircon_plugins/03
related_code:
  - zircon_runtime/src/scene/components/scene.rs
  - zircon_runtime/src/scene/world/project_io/physics.rs
  - zircon_runtime/src/scene/world/property_access/entries/physics.rs
  - zircon_runtime/src/scene/world/render_post_process.rs
tests:
  - cargo test -p zircon_editor --lib --locked tests::host::manager::minimal_host_contract::optional_features::editor_manager_plugin_status_lists_owner_optional_feature_dependencies --jobs 1 -- --exact --test-threads=1
resolved_at: 2026-07-12
---


# Physics 03：ColliderShape 新形状未同步共享消费者

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md`
- 来源执行切片：2026-07-11 Editor M1 三份 fixed handoff 当前源码向上精确复验
- 修复责任计划：`docs/plans/zircon_plugins/03-physics.md`
- 交接原因：最低共享故障来自 Physics M2 向 `ColliderShape` 新增五种形状后未同步 Runtime scene 消费者，不属于 Editor provider、字体、ZUI 或 Render 18。

## 失败现象与复现证据

关闭 debug info 后重新构建 Editor provider fully-qualified exact，Render 18 的 OIT include 路径修正后，`zircon_runtime` 在四处产生 `E0004`：`project_io/physics.rs`、`property_access/entries/physics.rs` 两处及 `render_post_process.rs` 仍只覆盖 Box/Sphere/Capsule，没有处理 Cylinder、ConvexHull、TriangleMesh、HeightField、Compound。测试未进入断言。

## 最低共享层根因

`scene/components/scene.rs` 于 2026-07-12 05:29 扩展 `ColliderShape`，但其序列化、属性投影、容量估算与渲染后处理消费者仍停留在三形状闭集。该问题属于 Physics 03 M2 形状族硬切未完成，而非原 Editor M1 fixed handoff 回退。

## 架构修复验收

- 为五种新形状在 project I/O、property projection/capacity 与 render post-process 中定义真实语义并补齐就近回归。
- 不允许用 `_ =>`、`unreachable!()` 或静默降级绕过闭集审查。
- 先通过 Physics 03 形状族聚焦门禁，再返回 Editor 01 重跑 provider/native/HUD/ZUI 向上门禁。

## 禁止临时方案

- 禁止在 Editor 测试上条件编译或跳过 Runtime scene/physics。
- 禁止把新形状压回 Box/Sphere/Capsule，或用 wildcard 吞掉未来形状。
- 禁止削弱原 provider、字体真实 framebuffer 与 ZUI governance 断言。

## 修复结果与回传

- 根因：Physics M2 expanded ColliderShape with Cylinder, ConvexHull, TriangleMesh, HeightField, and Compound without exhaustively updating shared Runtime asset cache, project I/O, property projection/mutation, and render post-process consumers; new tests also used the rejected asset:// locator scheme.
- 架构修复：Added exact recursive shape conversions and projections, split collider-shape property projection into its own bounded module, made the render post-process boundary exhaustive, updated navigation and builtin geometry consumers, added backend-neutral mesh payloads plus registered Jolt TriangleMesh/HeightField conversion, and corrected all new resource fixtures to res://.
- 验证：Windows managed Jolt cargo check passed; physics plugin backend-jolt lib suite passed 21/21; Runtime collider consumer suite passed 10/10; PhysicsMeshAsset JSON round-trip passed 1/1; scoped git diff --check passed; touched production owners are all below 500 lines. Full all-repository structure audit was attempted but timed out at 120 seconds in the concurrently dirty workspace and is not claimed.
- 回传：ColliderShape consumer exhaustiveness is restored across Runtime and plugin boundaries, including recursive Compound and resource-backed Jolt mesh shapes. M2-T1 is ready for parent-plan acceptance.
