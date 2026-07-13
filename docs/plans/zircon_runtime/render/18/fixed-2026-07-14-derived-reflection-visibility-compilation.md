---
handoff_kind: fixed
status: fixed
created_at: 2026-07-14
resolved_at: 2026-07-14
summary_slug: derived-reflection-visibility-compilation
origin_plan: docs/plans/zircon_runtime/render/18-advanced-lighting-features.md
fixing_plan: docs/plans/zircon_plugins/08-zr-vm.md
origin_child_dir: docs/plans/zircon_runtime/render/18
fixing_child_dir: docs/plans/zircon_plugins/08
related_code:
  - zircon_runtime/src/scene/reflect/builtin_reflection/component_support.rs
  - zircon_runtime/src/scene/reflect/builtin_reflection/registration.rs
  - zircon_runtime/src/scene/components/scene/reflection/local_transform.rs
  - zircon_runtime/src/scene/components/scene/reflection/mesh_renderer.rs
  - zircon_runtime/src/scene/components/scene/reflection/rigid_body.rs
tests:
  - cargo test -p zircon_plugin_hybrid_gi_runtime hybrid_gi_resolve_accepts_external_or_transient_scene_velocity
---


# Plugins 08: derived reflection visibility compilation

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/render/18-advanced-lighting-features.md`
- 来源执行切片：HGI-M4 真实 Editor 产品重建与组合图资源契约回归
- 修复责任计划：`docs/plans/zircon_plugins/08-zr-vm.md`
- 交接原因：失败来自 Plugins08 M1 正在建立的 derive reflection owner、helper visibility 与注册边界，Render18 只是第一个重新编译到该共享层的消费者。

## 失败现象与复现证据

HybridGI 精确测试在进入 HGI crate 前编译 `zircon_runtime` 失败。2026-07-14 04:39 的受管 Windows 重跑报告 31 个 Runtime errors，其中 Reflection 最低错误包括：

- `builtin_reflection::registration::register` 只有 child-parent 可见性，不能由 `builtin_reflection` 再向 `reflect` re-export；
- `component_support::get` 同时借用 `World` 与 `type_path`，返回引用未显式绑定到 `World` lifetime；
- derive 在 `scene.rs` 父模块生成字段访问代码，但 `local_transform`、`mesh_renderer`、`rigid_body` helper 只对各自直接父模块可见，产生 24 个 E0603。

复现日志：`.codex/tmp/plan18_hgi_scene_velocity_green_r2.stderr.log`。同一编译快照另有 Text05 与 Runtime04 错误，它们不属于本交接。

## 最低共享层根因

M1 将手写 fixed reflection 硬切到 derive + 分目录 helper 后，helper 使用了单层 `pub(super)`，但实际调用或 re-export 位于两层外的稳定 owner。可见性必须精确提升到最窄的拥有模块，不能改成公共 API。`component_support::get` 还缺少与 `World` 绑定的命名 lifetime。

## 架构修复验收

- Reflection helper 只对 `scene` 或 `scene::reflect` 所需 owner 可见，crate 外保持不可见。
- `component_support::get` 返回引用显式绑定到 `World`，不绑定到错误消息的 `type_path`。
- 原 HybridGI 精确测试重新编译时不再出现上述 Reflection E0106/E0364/E0603；Text05/Runtime04 独立失败不得计入本交接。

## 禁止临时方案

- 不恢复 `scene/reflect/fixed/**`，不增加兼容 re-export、公共 facade 或 test-only bypass。
- 不把 helper 扩为 `pub`，不削弱 derive schema 或 Render18 验收条件。

## 修复结果与回传

- 根因：Derived reflection helpers used one-level pub(super) visibility after a two-level module split, and component_support::get lacked a World-bound output lifetime.
- 架构修复：Field helpers now use the narrow scene owner visibility, registration uses the narrow scene::reflect owner visibility, and get returns a reference tied only to the World lifetime.
- 验证：The same HybridGI exact-test compile reduced Runtime errors from 31 to 3 with zero remaining reflection E0106/E0364/E0603 diagnostics.
- 回传：Reflection compilation boundary is fixed and returned; Render18 can resume after the independent AssetUri and ShaderAssetKind errors are resolved.
