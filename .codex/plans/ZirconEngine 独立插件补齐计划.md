# ZirconEngine 独立插件补齐计划

## Summary
- 采用“Unity Package/Renderer Feature + Unreal Plugin/Module + Bevy PluginGroup + Fyrox EditorPlugin 生命周期”的混合模型：`plugin.toml` 管包，runtime/editor/native/VM 分轨注册，所有行为最终收敛到同一套 descriptor、capability、operation、serialized DTO、host handle。
- 当前基线已具备插件骨架：`zircon_plugins` 独立 workspace、runtime/editor plugin catalog、editor capability gate、NativeDynamic ABI v2 雏形、VM plugin mock/unavailable 路径。主要缺口是插件清单与实际 workspace 不一致、physics/animation 仍内建、importer/editor contribution 深度不足、NativeDynamic 生产行为 ABI 仍需补齐。
- 参考依据：本地 `dev/UnrealEngine` 的 `.uplugin`/`IModuleInterface`/`FPluginDescriptor`，`dev/bevy` 的 `Plugin`/`PluginGroup`/`DefaultPlugins`，`dev/Fyrox` 的 `EditorPlugin` 生命周期，`dev/Graphics` 的 Unity URP `package.json`/`ScriptableRendererFeature`；Unity 官方资料见 Package manifest、Plugin Inspector、native plug-in interface、URP renderer feature 文档。

## Key Changes
- **先修正插件基线事实**
  - 以 [zircon_plugins/Cargo.toml](E:/Git/ZirconEngine/zircon_plugins/Cargo.toml:1)、[runtime_modules.rs](E:/Git/ZirconEngine/zircon_runtime/src/builtin/runtime_modules.rs:1)、[editor_plugin.rs](E:/Git/ZirconEngine/zircon_editor/src/core/editor_plugin.rs:1) 为准，更新 README/计划中“physics/animation 已插件化”的过期描述。
  - 保持 `zircon_plugins` 独立 workspace；root workspace 继续只保留 `zircon_app`、`zircon_runtime`、`zircon_editor`、`zircon_runtime_interface`。

- **必要插件**
  - `physics`、`animation`：从 `zircon_runtime` 内建域迁回真实 runtime/editor 插件包，补 `plugin.toml`、workspace 成员、runtime module descriptor、editor drawer/tool/view/menu/operation。
  - `runtime_ui`：把当前 `plugin-ui` feature 后的 UI runtime 行为外移为 `zircon_plugins/ui/runtime`，runtime 本体只留 UI contract、surface bridge、template DTO。
  - `asset_importers`：拆成 `gltf_importer`、`obj_importer`、`texture_importer`、`audio_importer`、`shader_wgsl_importer`、`ui_document_importer`，asset manager 只保留资源身份、缓存、依赖图。
  - `editor_build_export_desktop`：把 Windows/Linux/macOS export 面板、兼容性诊断、SourceTemplate/LibraryEmbed/NativeDynamic 报告做成 editor 插件；最终计划权威仍在 host。
  - `plugin_sdk_examples`：保留一个窗口插件、一个 importer+inspector 插件作为 SDK contract fixture。

- **已存在但要深化**
  - `sound`、`texture`、`net`、`navigation`、`particles`：从“最小行为”推进到真实 manager/component/importer/drawer/operation；禁用后对应 UI 和 runtime capability 必须消失。
  - `virtual_geometry`、`hybrid_gi`：继续把 heavy state、prepare/feedback/readback/renderer resources 收进插件，runtime 只留中性 render DTO、RHI/render graph、host-owned GPU handles。
  - `runtime_diagnostics`、`ui_asset_authoring`、`native_window_hosting`：继续 editor-only，但要补真实模板、菜单 payload、operation schema 和 capability-gated lifecycle。

- **可选插件池**
  - Rendering：`post_process`、`decals`、`ssao`、`reflection_probes`、`baked_lighting`、`ray_tracing_policy`、`shader_graph`、`vfx_graph`。
  - Authoring：`terrain`、`tilemap_2d`、`material_editor`、`prefab_tools`、`timeline_sequence`、`animation_graph`。
  - Platform/Services：`xr_openxr`、`webgpu_wasm_export`、`mobile_platform_adapters`、`localization`、`profiler`、`high_level_multiplayer`、`package_registry_adapter`。

## Public Interfaces
- Extend `PluginPackageManifest` with explicit SDK/API version, package category, supported targets/platforms, dependency ids, editor/runtime/native/VM module declarations, capability list, asset/content roots, and default packaging.
- Extend `RuntimeExtensionRegistry` with importer/codec/template-compiler registrations plus stable render feature command/feedback surfaces.
- Extend `EditorExtensionRegistry` with versioned operation payload schemas, importer/editor descriptors, inspector/component drawers, viewport tools, build/export panels, and lifecycle events.
- Complete Native ABI v2 before production native behavior: versioned host function table, opaque handles, serialized command/event payloads, plugin-owned allocation/free, panic-to-diagnostic boundary, unload, state save/restore, capability negotiation. No Rust trait objects, `Arc`, borrowed refs, Slint objects, `wgpu` objects, world data, or editor state crosses ABI.

## Test Plan
- Manifest/catalog consistency: every builtin catalog entry has matching `plugin.toml` and workspace member; stale physics/animation mismatch is caught.
- Target loading: `core-min` starts without optional plugins; client/server/editor only accept target-compatible linked reports; required missing plugins error, optional missing plugins warn.
- Editor gates: enabling/disabling plugins changes views, menus, operations, drawers, importers, templates, and capability snapshot immediately.
- Importer registry: duplicate extensions, override priority, invalid outputs, missing dependencies, and reimport invalidation are covered.
- Native/VM safety: ABI mismatch, missing symbol, denied capability, panic callback, malformed owned buffer, state save/restore, unload, unavailable VM backend, and hot reload handle stability are tested.
- Render plugins: disabled VG/HGI/postprocess/decals do not initialize heavy state; linked descriptors alone enable advanced passes; runtime never depends on `zircon_plugins`.

## Assumptions
- 不复制 Unity C# Reference Source 代码，只复刻架构思路；UnityCsReference 是 reference-only 源码。
- NativeDynamic 在 ABI v2 完成前只作为 discovery/diagnostics/export fixture；真实 gameplay/user-code 长期优先走 VM plugin。
- Runtime 本体保留 core lifecycle、plugin loader、asset identity、scene ECS authority、base render/RHI、input/platform baseline；editor 本体保留 shell、Plugin Manager、capability gate、project/session、最小 scene authoring。
- 硬切迁移，不保留旧路径 re-export、shim、compat bridge。

Sources: [Unity Package manifest](https://docs.unity.cn/Manual/upm-manifestPkg.html), [Unity Plugin Inspector](https://docs.unity.cn/Manual/PluginInspector.html), [Unity native plug-in interface](https://docs.unity.cn/6000.0/Documentation/Manual/native-plugin-interface.html), [Unity URP Renderer Feature API](https://docs.unity.cn/6000.0/Documentation/Manual/urp/renderer-features/scriptable-renderer-features/scriptable-renderer-feature-reference.html), [Unity Graphics source](https://github.com/Unity-Technologies/Graphics), [UnityCsReference](https://github.com/Unity-Technologies/UnityCsReference).
