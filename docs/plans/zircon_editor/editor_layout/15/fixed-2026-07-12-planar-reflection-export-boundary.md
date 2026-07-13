---
handoff_kind: fixed
status: fixed
created_at: 2026-07-12
resolved_at: 2026-07-12
summary_slug: planar-reflection-export-boundary
origin_plan: docs/plans/zircon_editor/editor_layout/15-component-standardization-from-primitives.md
fixing_plan: docs/plans/zircon_runtime/render/18-advanced-lighting-features.md
origin_child_dir: docs/plans/zircon_editor/editor_layout/15
fixing_child_dir: docs/plans/zircon_runtime/render/18
related_code:
  - zircon_runtime/src/core/framework/render/mod.rs
  - zircon_runtime/src/core/framework/render/advanced_lighting/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/probe_buffer/mod.rs
tests:
  - cargo test -p zircon_editor --lib --locked --no-run --jobs 1 --message-format short
---


# Render 18：平面反射导出边界断裂

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor_layout/15-component-standardization-from-primitives.md`
- 来源执行切片：Layout 15 Blend Space 原生组件绘制与截图验收
- 修复责任计划：`docs/plans/zircon_runtime/render/18-advanced-lighting-features.md`
- 交接原因：失败发生在 Runtime advanced-lighting 平面反射的父模块导出边界，属于 Render 18，不属于 Editor UI；Layout 15 不在消费端添加别名或绕过 Runtime 唯一真源。

## 失败现象与复现证据

2026-07-12 受管 Windows job `011f41975ab047d19186a038465a7400` 执行：

```text
cargo test -p zircon_editor --lib --locked --no-run --jobs 1 --message-format short
```

在 `zircon_runtime` library 编译阶段失败：

- `camera_loop.rs`、`wgpu_render_framework.rs`、`probe_buffer/resources.rs` 无法从 `crate::core::framework::render` 导入 `derive_planar_reflection_camera` / `PlanarReflectionUpdateState`（E0432）。两者仍由 `core/framework/render/advanced_lighting/mod.rs` 导出，但未穿过 `core/framework/render/mod.rs` 的当前父边界。
- `advanced_lighting/planar_filter/executor.rs` 无法从 `graphics::scene::scene_renderer::environment` 读取 `PLANAR_REFLECTION_TEXTURE_SIZE`（E0425）。常量仍由 `environment/probe_buffer/mod.rs` 导出，但未穿过 `environment/mod.rs` 的当前父边界。
- `wgpu_render_framework.rs:50` 的 E0282 是上述 `PlanarReflectionUpdateState` 缺失后的级联类型推断失败。

同日第二次受管重编译确认共享 owner 已修复 `core/framework/render/mod.rs` 的类型/函数导出，E0432 与级联 E0282 消失；当前唯一剩余错误仍是 `environment/mod.rs` 未导出 `PLANAR_REFLECTION_TEXTURE_SIZE` 的 E0425。交接保持 open，直到完整父边界和原始上行门禁均通过。

因此当前源码无法生成新的 `zircon_editor` 测试二进制，Layout 15 的 SearchField 共享注册表修复、组件化 subtree painter 回归和最终截图均不能进入验证。

## 最低共享层根因

Render 18 的平面反射实现 owner 仍存在，但其公开给 Runtime 内部消费者的两级模块导出合同发生漂移：advanced-lighting 类型/函数和 probe-buffer 常量停留在子模块，现有消费端继续依赖父模块稳定入口。最低修复层是恢复或硬切这些 Runtime 内部 owner/consumer 的唯一导出路径，并同步全部消费端；不应在 Editor 或单个调用点复制声明。

## 架构修复验收

- 平面反射相机函数、update-state 类型和纹理尺寸常量各自只有一个定义 owner，父模块导出与所有 Runtime 消费端一致。
- Render 18 平面反射 focused tests/check 通过，E0432/E0425/E0282 不再出现。
- 原始 `zircon_editor --lib --no-run` 命令越过 `zircon_runtime` 编译并生成当前测试二进制。
- Layout 15 重新运行组件注册表、SearchField 投影、组件化 workspace painter 和 Blend Space 截图门禁。

## 禁止临时方案

- 禁止在 Editor 侧复制类型/常量、feature-gate Render 18、添加兼容 alias 或跳过 Runtime 编译。
- 禁止在每个消费端写第二套局部常量或通过默认值掩盖缺失导出。
- 禁止使用 `unwrap/expect`、测试专用 stub 或弱化 Layout 15 截图断言来规避错误。

## 修复结果与回传

- 根因：Planar reflection function, update-state type, and texture-size constant remained in child owners while parent exports drifted.
- 架构修复：Render18 restored the unique core render and scene-renderer environment parent export paths without Editor aliases or copied constants.
- 验证：Managed Windows zircon_editor lib no-run build crossed zircon_runtime with prior E0432 E0425 E0282 absent, then stopped only on unrelated Editor14 PendingJob E0599.
- 回传：Returned fixed Render18 export boundary to Layout15; Layout15 can resume after the current Editor14 compile drift is repaired.
