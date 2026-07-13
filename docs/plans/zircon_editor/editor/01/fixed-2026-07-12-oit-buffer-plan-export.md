---
handoff_kind: fixed
status: fixed
created_at: 2026-07-12
summary_slug: oit-buffer-plan-export
origin_plan: docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md
fixing_plan: docs/plans/zircon_runtime/render/18-advanced-lighting-features.md
origin_child_dir: docs/plans/zircon_editor/editor/01
fixing_child_dir: docs/plans/zircon_runtime/render/18
related_code:
  - zircon_runtime/src/core/framework/render/advanced_lighting/mod.rs
  - zircon_runtime/src/core/framework/render/advanced_lighting/oit.rs
  - zircon_runtime/src/core/framework/render/mod.rs
  - zircon_runtime/src/graphics/feature/render_feature_descriptor/construct.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/resource_descriptors.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/advanced_lighting/oit_buffers/mod.rs
tests:
  - cargo test -p zircon_editor --lib --locked tests::host::manager::minimal_host_contract::optional_features::editor_manager_plugin_status_lists_owner_optional_feature_dependencies --jobs 1 -- --exact --test-threads=1
resolved_at: 2026-07-12
---


# Render 18：OIT buffer plan 导出边界阻断 Editor M1 向上门禁

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md`
- 来源执行切片：Editor M1 三份已修复 handoff 的当前源码向上精确复验
- 修复责任计划：`docs/plans/zircon_runtime/render/18-advanced-lighting-features.md`
- 交接原因：最低共享故障位于 Render 18 AF-M4 新增的 OIT 框架契约导出边界，不属于 Editor M1 或插件 provider 调用点。

## 失败现象与复现证据

当前源码从干净独立 Windows target 构建 Editor provider 精确门禁，在 `zircon_runtime` library 编译阶段稳定失败，测试未开始：

```text
error[E0432]: unresolved import `crate::core::framework::render::OitBufferPlan`
  --> zircon_runtime/src/graphics/pipeline/render_pipeline_asset/resource_descriptors.rs:2:24
```

编译器同时确认 `advanced_lighting::oit::OitBufferPlan` 已存在，但 `advanced_lighting/mod.rs` 仅在该 crate-private owner 内 `pub(crate) use`，而公开框架 facade `core/framework/render/mod.rs` 只导出了 `OitSettings`。`resource_descriptors.rs` 已通过 facade 请求该类型，因此当前所有 Editor test target 都在进入原 handoff 断言前被阻断。复现命令运行 1,883.7 秒后以 exit 101 结束。

## 最低共享层根因

Render 18 AF-M4 已让 graph resource descriptor 消费 `OitBufferPlan::for_view(...)`，但没有把同一框架契约沿既有 `core::framework::render` curated facade 导出。生产消费者与契约 owner 之间因此形成不完整模块边界；该错误不是 Editor provider、字体或 ZUI 修复回退。

## 架构修复验收

- 由 Render 18 owner 决定并落实唯一稳定边界：若 `OitBufferPlan` 是跨 graphics 子系统框架契约，则从 `core::framework::render` facade 明确导出；若仅是 advanced-lighting 内部计算，则把 resource descriptor 计算收回其 owner，不允许 graphics 旁路进入私有子模块。
- 运行 `cargo check -p zircon_runtime --lib --locked`，确认 E0432 消失且不新增兼容 re-export 链。
- 重新运行原 Editor provider 精确门禁，并继续 native provider、HUD 字体、ZUI governance 向上回归。

## 禁止临时方案

- 禁止在 Editor 或 provider 测试调用点增加条件编译、测试专用 stub 或跳过 Runtime graphics。
- 禁止把 `advanced_lighting` 子模块整体公开、增加平行 alias/兼容 facade，或复制 OIT size 计算到 `resource_descriptors.rs`。
- 禁止削弱原 handoff 的精确测试和 Editor M1 验收标准。

## 修复结果与回传

- 根因：OitBufferPlan existed in advanced-lighting owner but was missing from the curated core::framework::render facade consumed by graph resource sizing.
- 架构修复：Exported the cross-graphics OIT contract through the existing curated facade without opening the advanced_lighting module or adding compatibility aliases; also repaired same-change graph mutation compile defects and corrected the OIT draw WGSL `include_str!` path to the existing graphics shader owner.
- 验证：zircon_runtime lib locked/offline check passed. The follow-up `debug=0` Editor exact build no longer reports E0432, graph-mutation errors, unstable `str_as_str`, or a missing OIT WGSL include; current-source execution is externally blocked before assertions by four ColliderShape E0004 errors in physics-owned modules, so the Editor exact is not counted green.
- 回传：Render 18 export root cause fixed and lower layer validated; origin should rerun the exact Editor gate after the physics collider exhaustiveness errors are closed.
