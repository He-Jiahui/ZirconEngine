---
related_code:
  - zircon_plugins/Cargo.toml
  - zircon_plugins/plugin_sdk/src/lib.rs
  - zircon_plugins/first_party_runtime_catalog/src/lib.rs
  - zircon_plugins/first_party_editor_catalog/src/catalog.rs
  - zircon_app/Cargo.toml
implementation_files:
  - zircon_plugins
  - zircon_plugins/plugin_sdk/src
plan_sources:
  - user: 2026-09-09 构建 ZirconEngine 详细 Wiki 文档集合
  - docs/plans/zircon_plugins/01
tests:
  - zircon_plugins/plugin_sdk/src
  - zircon_plugins/first_party_runtime_catalog/src
  - zircon_plugins/first_party_editor_catalog/src
doc_type: inventory-reference
---

# ZirconEngine 插件清单

本页是 `zircon_plugins` 工作区的机器可读/人类可读索引，基于仓库中的 `plugin.toml`、`Cargo.toml` 与 Rust 入口源文件盘点。它描述当前源码形态，不代表所有能力都已达到生产成熟度。

## 阅读约定

- `runtime`：运行时 crate，向 `zircon_runtime` 注册模块、资源、系统或导入器。
- `editor`：编辑器 crate，向 `zircon_editor` 注册窗口、工具、资产作者或检查器。
- `dist`：分发/原生 ABI crate，通常通过 `native_dist_runtime_plugin_v3!` 或 `native_dist_editor_plugin_v3!` 导出动态库。
- `native`：原生 ABI 或第三方后端 fixture；不等同于普通 runtime crate。
- `feature`：包内可选能力，路径位于 `<package>/features/<id>`。
- 入口惯例：runtime/editor 通常公开 `capability.rs` 中的 `declare_plugin!`、`plugin.rs` 中的 descriptor/registration 函数；dist 公开 `native_dist_*_plugin_v3!`。具体 API 仍以对应 Rust 源码为准。
- “缺失描述”表示静态 manifest 没有 `description`，或该目录只有工作区聚合/支持 crate、没有独立 `plugin.toml`。

## 静态插件包

| package id | manifest 路径 | 角色 | 功能/description | maturity | 可选 feature | Rust 入口线索 |
|---|---|---|---|---|---|---|
| `ai` | `zircon_plugins/ai/plugin.toml` | runtime + editor + dist | 行为树、黑板、感知快照与 agent tick 的 AI 合约 | experimental | 无 | runtime `declare_plugin!`; editor 感知 overlay |
| `animation` | `zircon_plugins/animation/plugin.toml` | runtime + editor + dist | 动画调度、片段播放、状态机、IK、GPU skinning 与编辑器作者工具 | beta | 无 | runtime `declare_plugin!`; editor plugin descriptor |
| `animation_graph` | `zircon_plugins/animation_graph/plugin.toml` | editor + dist | 动画图和状态机图编辑、校验与编译 | experimental | 无 | editor `validate_animation_graph_asset`, `compile_animation_graph` |
| `asset_importer.audio` | `zircon_plugins/asset_importers/audio/plugin.toml` | runtime + dist | 旧版音频导入器家族聚合包 | experimental | 无 | `AudioAssetImporterRuntimePlugin`, importer descriptor |
| `asset_importer.data` | `zircon_plugins/asset_importers/data/plugin.toml` | runtime + dist | 结构化数据资产导入器家族聚合包 | experimental | 无 | `DataAssetImporterRuntimePlugin`, runtime descriptor |
| `asset_importer.model` | `zircon_plugins/asset_importers/model/plugin.toml` | runtime + dist | 旧版模型/网格导入器家族聚合包 | experimental | 无 | `ModelAssetImporterRuntimePlugin`, `import_mesh_model` |
| `asset_importer.shader` | `zircon_plugins/asset_importers/shader/plugin.toml` | runtime + dist | 旧版 shader 导入器家族聚合包 | experimental | 无 | `ShaderAssetImporterRuntimePlugin`, `import_shader` |
| `asset_importer.texture` | `zircon_plugins/asset_importers/texture/plugin.toml` | runtime + dist | 旧版纹理导入器家族聚合包 | experimental | 无 | `TextureAssetImporterRuntimePlugin` |
| `audio_importer` | `zircon_plugins/audio_importer/plugin.toml` | runtime + dist | 音频资源导入 | stable | 无 | `AudioImporterRuntimePlugin`, `runtime_plugin_descriptor` |
| `editor_build_export_desktop` | `zircon_plugins/editor_build_export_desktop/plugin.toml` | editor + dist | 桌面导出面板、SourceTemplate/LibraryEmbed/NativeDynamic 报告与打包操作 | experimental | 无 | `ExportWizardRegion`, `ExportWizardAction` |
| `editor_contribution_fixture` | `zircon_plugins/editor_contribution_fixture/plugin.toml` | native + dist | 版本化序列化 editor contribution ABI 的原生 fixture | experimental | 无 | `declare_plugin!`, `native_dist_editor_plugin_v3!` |
| `gltf_importer` | `zircon_plugins/gltf_importer/plugin.toml` | runtime + dist | glTF/GLB 模型导入 | stable | 无 | `GltfImporterRuntimePlugin`, `import_gltf` |
| `hybrid_gi` | `zircon_plugins/hybrid_gi/plugin.toml` | runtime + editor + dist | 混合全局光照运行时与编辑器能力 | experimental | 无 | `HybridGiRuntimePlugin`, `HybridGiEditorPlugin` |
| `material_editor` | `zircon_plugins/material_editor/plugin.toml` | editor + dist | 材质图校验、编译与材质资产作者工具 | experimental | 无 | `validate_material_graph`, `compile_material_graph` |
| `native_dynamic_fixture` | `zircon_plugins/native_dynamic_fixture/plugin.toml` | native + dist | NativeDynamic ABI v3 加载 fixture，并覆盖 ABI v2 fallback | experimental | 无 | `native_dist_plugin_v3!` |
| `native_window_hosting` | `zircon_plugins/native_window_hosting/plugin.toml` | editor + dist | 原生浮动窗口 surface 的编辑器集成；实际 Workbench/Prefab 窗口仍由 Editor core 持有 | experimental | 无 | `NativeWindowHostingEditorPlugin` |
| `navigation` | `zircon_plugins/navigation/plugin.toml` | runtime + editor + native + dist | navmesh 烘焙、世界扫描和路径查询 | beta | 无 | runtime navigation plugin; native `zircon_plugin_navigation_recast` |
| `net` | `zircon_plugins/net/plugin.toml` | runtime + editor + dist | 客户端/服务器网络、连接、传输、复制和网络事件目录 | beta | `net.http`, `net.websocket`, `net.rpc`, `net.replication`, `net.reliable_udp`, `net.content_download` | runtime plugin + feature crates；editor diagnostics |
| `neural` | `zircon_plugins/neural/plugin.toml` | runtime + editor + dist | 神经模型资产、CPU inference 与后处理集成 | experimental | `neural.post_process` | runtime neural ops/interpreter；editor plugin |
| `obj_importer` | `zircon_plugins/obj_importer/plugin.toml` | runtime + dist | Wavefront OBJ 模型导入 | stable | 无 | `ObjImporterRuntimePlugin`, importer descriptor |
| `opus_importer` | `zircon_plugins/opus_importer/plugin.toml` | runtime + dist | 由 native-dynamic decoder 支持的 Opus 音频导入 | experimental | 无 | runtime importer + native decoder boundary |
| `particles` | `zircon_plugins/particles/plugin.toml` | runtime + editor + dist | 粒子模拟、渲染、事件与 VFX 作者工具 | experimental | `particles.physics`, `particles.animation_control`, `particles.gpu_simulation` | runtime simulation/render/service；editor authoring |
| `physics` | `zircon_plugins/physics/plugin.toml` | runtime + editor + dist | 物理世界、查询、约束、碰撞后端与物理作者工具 | experimental | 无 | runtime manager/module/system；editor plugin |
| `plugin_sdk_examples` | `zircon_plugins/plugin_sdk_examples/plugin.toml` | editor + dist | SDK 示例窗口、模型导入器、检查器、组件 drawer 与资产模板 | experimental | 无 | editor sample registrations |
| `prefab_tools` | `zircon_plugins/prefab_tools/plugin.toml` | runtime + editor + dist | prefab 资产、实例化与作者工具 | beta | 无 | runtime prefab plugin; editor authoring |
| `rendering` | `zircon_plugins/rendering/plugin.toml` | runtime + editor + dist | 渲染 feature owner，统一管理后处理、SSAO、阴影、雾、decals、probes、光照、shader/VFX graph | stable | `rendering.post_process`, `rendering.ssao`, `rendering.contact_shadow`, `rendering.volumetric_fog`, `rendering.oit`, `rendering.light_cookies`, `rendering.irradiance_volumes`, `rendering.planar_reflections`, `rendering.subsurface_scattering`, `rendering.decals`, `rendering.reflection_probes`, `rendering.baked_lighting`, `rendering.ray_tracing_policy`, `rendering.shader_graph`, `rendering.vfx_graph` | umbrella runtime metadata；feature `render_feature_descriptor`/executor；editor feature registration |
| `runtime_diagnostics` | `zircon_plugins/runtime_diagnostics/plugin.toml` | editor + dist | 查看嵌入式 runtime 状态的编辑器诊断视图 | experimental | 无 | editor diagnostics plugin |
| `shader_wgsl_importer` | `zircon_plugins/shader_wgsl_importer/plugin.toml` | runtime + dist | WGSL shader 导入 | stable | 无 | runtime importer plugin |
| `solari` | `zircon_plugins/solari/plugin.toml` | runtime + dist | 实时 ray-traced lighting provider contract | experimental | 无 | runtime Solari descriptor；realtime pass 当前为 partial |
| `sound` | `zircon_plugins/sound/plugin.toml` | runtime + editor + dist | runtime 音频、sound authoring 与动态事件目录 | beta | `sound.timeline_animation_track`, `sound.ray_traced_convolution_reverb` | runtime sound plugin；editor authoring |
| `terrain` | `zircon_plugins/terrain/plugin.toml` | runtime + editor + dist | 高度场 terrain 资产与编辑器作者工具 | beta | 无 | runtime terrain descriptors；editor authoring |
| `texture_importer` | `zircon_plugins/texture_importer/plugin.toml` | runtime + dist | 纹理与图像资源导入 | stable | 无 | runtime texture importer |
| `texture` | `zircon_plugins/texture/plugin.toml` | runtime + editor + dist | 纹理处理 runtime 与编辑器工具 | stable | 无 | runtime texture manager/module；editor plugin |
| `tilemap_2d` | `zircon_plugins/tilemap_2d/plugin.toml` | runtime + editor + dist | 2D tilemap 资产、组件和作者工具 | beta | 无 | runtime tilemap plugin；editor authoring |
| `timeline_sequence` | `zircon_plugins/timeline_sequence/plugin.toml` | editor + dist | 动画 sequence/timeline 编辑器作者工具 | experimental | 无 | editor timeline plugin |
| `ui_asset_authoring` | `zircon_plugins/ui_asset_authoring/plugin.toml` | editor + dist | retained UI asset document 作者工具 | experimental | 无 | editor UI asset authoring plugin |
| `ui_document_importer` | `zircon_plugins/ui_document_importer/plugin.toml` | runtime + dist | UI 文档资源导入 | stable | 无 | runtime UI document importer |
| `virtual_geometry` | `zircon_plugins/virtual_geometry/plugin.toml` | runtime + editor + dist | 虚拟化几何、cluster/page residency、GPU indirect draw 与自定义 geometry permutation | experimental | 无 | runtime virtual geometry renderer/prepare；editor plugin |
| `zr_vm_language` | `zircon_plugins/zr_vm_language/plugin.toml` | runtime + dist | ZrVM language backend、编译调用点与运行时执行支持 | experimental | 无 | runtime backend/call-site API |

## Feature crate 索引

每个 feature 通常成对提供 `runtime/Cargo.toml` 与 `editor/Cargo.toml`；若表中只有一侧，则该能力只在该侧实现。所有 feature dist 导出由所属包的 dist crate 负责，feature 本身没有独立 `plugin.toml`。

| owner | feature | crate 路径 | 主要入口/功能线索 |
|---|---|---|---|
| `net` | `http` | `net/features/http/runtime` | HTTP runtime transport |
| `net` | `websocket` | `net/features/websocket/runtime` | WebSocket runtime transport |
| `net` | `rpc` | `net/features/rpc/runtime` | RPC runtime |
| `net` | `replication` | `net/features/replication/runtime` | state replication |
| `net` | `reliable_udp` | `net/features/reliable_udp/runtime` | reliable UDP transport |
| `net` | `content_download` | `net/features/content_download/runtime` | content download |
| `neural` | `post_process` | `neural/features/post_process/runtime` | neural post-process render feature |
| `particles` | `physics` | `particles/features/physics` | particle/physics interop |
| `particles` | `animation_control` | `particles/features/animation_control` | particle animation interop |
| `particles` | `gpu_simulation` | `particles/features/gpu_simulation` | GPU particle simulation |
| `rendering` | 15 render features | `rendering/features/<feature>/{runtime,editor}` | descriptor, executor, component or graph registration；feature 名见上表 |
| `sound` | `timeline_animation_track` | `sound/features/timeline_animation_track/{runtime,editor,dist}` | timeline animation audio track |
| `sound` | `ray_traced_convolution_reverb` | `sound/features/ray_traced_convolution_reverb/{runtime,editor,dist}` | ray-traced convolution reverb |

## 工作区支持 package（无独立 plugin.toml）

| package | 路径 | 角色与 Rust API |
|---|---|---|
| `zircon_plugin_sdk` | `zircon_plugins/plugin_sdk` | 插件声明、manifest、capability、ABI 宏与 descriptor 合约；核心宏包括 `declare_plugin!`、`native_dist_plugin_v3!`、`native_dist_runtime_plugin_v3!`、`native_dist_editor_plugin_v3!` |
| `zircon_plugin_editor_support` | `zircon_plugins/editor_support` | 编辑器 authoring contribution 支持；`EditorAuthoringSurface`、`EditorAuthoringExtensions`、`register_authoring_extensions` |
| `zircon_first_party_runtime_catalog` | `zircon_plugins/first_party_runtime_catalog` | 按 manifest 投影 runtime plugin registrations；`first_party_runtime_plugin_registrations_for_manifest` |
| `zircon_first_party_editor_catalog` | `zircon_plugins/first_party_editor_catalog` | 按 manifest 投影 editor plugin registrations；`first_party_editor_plugin_registrations_for_manifest` |

## API 与导出约定

运行时集成通常从 `runtime/src/lib.rs`、`module.rs`、`plugin.rs`、`capability.rs` 开始阅读：`capability.rs` 的 `declare_plugin!` 是包身份、目标、能力和成熟度的声明来源；`plugin.rs` 的 descriptor/registration 是行为入口；`module.rs`/`manager.rs`/`service.rs` 承载实际系统。编辑器集成从 `editor/src/lib.rs`、`plugin.rs`、`capability.rs`、`authoring.rs` 阅读，重点函数是 `editor_plugin_descriptor()`、`editor_plugin()`、`package_manifest()` 及 authoring registration。资源导入器通常公开 `import_<format>(context: &AssetImportContext)`、`runtime_plugin_descriptor()` 和 capability 列表。分发 crate 通过 SDK ABI 宏导出入口，不能以 dist crate 的本地常量替代声明式 manifest。

推荐调用流程：先由 catalog/manifest 选择 package 和 target，再启用 capability/optional feature，随后加载 runtime 或 editor registration；应用层不要直接把 `plugin.toml` 当作身份 authority，应使用 Rust descriptor 投影。静态 manifest 是生成快照，`cargo zircon plugin check` 用于检查快照漂移、workspace member、catalog 与 ABI 元数据一致性（本次盘点未运行 Cargo）。

## 盘点边界与缺失项

本页没有为以下内容虚构 description：`zircon_plugins/.zircon-cache` 是缓存目录；各包的 `dist`/feature crate 没有独立 manifest identity；`editor_support` 和两个 first-party catalog 是工作区支持 package。若未来新增 `plugin.toml`、optional feature 或入口命名，应同步更新本页；当前静态包 manifest 均存在 `description`，因此没有静态包缺失 description。

## Cargo package matrix

以下逐行覆盖 zircon_plugins 根 Cargo.toml 之外发现的每个 Cargo package；路径字段是相对于仓库根目录的 manifest 路径。

| Cargo package | Cargo.toml | 角色 |
|---|---|---|
| zircon_plugin_ai_dist | zircon_plugins/ai/dist/Cargo.toml | dist |
| zircon_plugin_ai_editor | zircon_plugins/ai/editor/Cargo.toml | editor |
| zircon_plugin_ai_runtime | zircon_plugins/ai/runtime/Cargo.toml | runtime |
| zircon_plugin_animation_graph_dist | zircon_plugins/animation_graph/dist/Cargo.toml | dist |
| zircon_plugin_animation_graph_editor | zircon_plugins/animation_graph/editor/Cargo.toml | editor |
| zircon_plugin_animation_dist | zircon_plugins/animation/dist/Cargo.toml | dist |
| zircon_plugin_animation_editor | zircon_plugins/animation/editor/Cargo.toml | editor |
| zircon_plugin_animation_runtime | zircon_plugins/animation/runtime/Cargo.toml | runtime |
| zircon_plugin_asset_importer_audio_dist | zircon_plugins/asset_importers/audio/dist/Cargo.toml | dist |
| zircon_plugin_asset_importer_audio_runtime | zircon_plugins/asset_importers/audio/runtime/Cargo.toml | runtime |
| zircon_plugin_asset_importer_data_dist | zircon_plugins/asset_importers/data/dist/Cargo.toml | dist |
| zircon_plugin_asset_importer_data_runtime | zircon_plugins/asset_importers/data/runtime/Cargo.toml | runtime |
| zircon_plugin_asset_importer_model_dist | zircon_plugins/asset_importers/model/dist/Cargo.toml | dist |
| zircon_plugin_asset_importer_model_runtime | zircon_plugins/asset_importers/model/runtime/Cargo.toml | runtime |
| zircon_plugin_asset_importer_shader_dist | zircon_plugins/asset_importers/shader/dist/Cargo.toml | dist |
| zircon_plugin_asset_importer_shader_runtime | zircon_plugins/asset_importers/shader/runtime/Cargo.toml | runtime |
| zircon_plugin_asset_importer_texture_dist | zircon_plugins/asset_importers/texture/dist/Cargo.toml | dist |
| zircon_plugin_asset_importer_texture_runtime | zircon_plugins/asset_importers/texture/runtime/Cargo.toml | runtime |
| zircon_plugin_audio_importer_dist | zircon_plugins/audio_importer/dist/Cargo.toml | dist |
| zircon_plugin_audio_importer_runtime | zircon_plugins/audio_importer/runtime/Cargo.toml | runtime |
| zircon_plugin_editor_build_export_desktop_dist | zircon_plugins/editor_build_export_desktop/dist/Cargo.toml | dist |
| zircon_plugin_editor_build_export_desktop_editor | zircon_plugins/editor_build_export_desktop/editor/Cargo.toml | editor |
| zircon_plugin_editor_contribution_fixture_native | zircon_plugins/editor_contribution_fixture/native/Cargo.toml | native |
| zircon_plugin_editor_support | zircon_plugins/editor_support/Cargo.toml | support |
| zircon_first_party_editor_catalog | zircon_plugins/first_party_editor_catalog/Cargo.toml | support |
| zircon_first_party_runtime_catalog | zircon_plugins/first_party_runtime_catalog/Cargo.toml | support |
| zircon_plugin_gltf_importer_dist | zircon_plugins/gltf_importer/dist/Cargo.toml | dist |
| zircon_plugin_gltf_importer_runtime | zircon_plugins/gltf_importer/runtime/Cargo.toml | runtime |
| zircon_plugin_hybrid_gi_dist | zircon_plugins/hybrid_gi/dist/Cargo.toml | dist |
| zircon_plugin_hybrid_gi_editor | zircon_plugins/hybrid_gi/editor/Cargo.toml | editor |
| zircon_plugin_hybrid_gi_runtime | zircon_plugins/hybrid_gi/runtime/Cargo.toml | runtime |
| zircon_plugin_material_editor_dist | zircon_plugins/material_editor/dist/Cargo.toml | dist |
| zircon_plugin_material_editor_editor | zircon_plugins/material_editor/editor/Cargo.toml | editor |
| zircon_plugin_native_dynamic_fixture_native | zircon_plugins/native_dynamic_fixture/native/Cargo.toml | native |
| zircon_plugin_native_window_hosting_dist | zircon_plugins/native_window_hosting/dist/Cargo.toml | dist |
| zircon_plugin_native_window_hosting_editor | zircon_plugins/native_window_hosting/editor/Cargo.toml | editor |
| zircon_plugin_navigation_dist | zircon_plugins/navigation/dist/Cargo.toml | dist |
| zircon_plugin_navigation_editor | zircon_plugins/navigation/editor/Cargo.toml | editor |
| zircon_plugin_navigation_recast | zircon_plugins/navigation/native/Cargo.toml | native |
| zircon_plugin_navigation_runtime | zircon_plugins/navigation/runtime/Cargo.toml | runtime |
| zircon_plugin_net_dist | zircon_plugins/net/dist/Cargo.toml | dist |
| zircon_plugin_net_editor | zircon_plugins/net/editor/Cargo.toml | editor |
| zircon_plugin_net_content_download_runtime | zircon_plugins/net/features/content_download/runtime/Cargo.toml | runtime |
| zircon_plugin_net_http_runtime | zircon_plugins/net/features/http/runtime/Cargo.toml | runtime |
| zircon_plugin_net_reliable_udp_runtime | zircon_plugins/net/features/reliable_udp/runtime/Cargo.toml | runtime |
| zircon_plugin_net_replication_runtime | zircon_plugins/net/features/replication/runtime/Cargo.toml | runtime |
| zircon_plugin_net_rpc_runtime | zircon_plugins/net/features/rpc/runtime/Cargo.toml | runtime |
| zircon_plugin_net_websocket_runtime | zircon_plugins/net/features/websocket/runtime/Cargo.toml | runtime |
| zircon_plugin_net_runtime | zircon_plugins/net/runtime/Cargo.toml | runtime |
| zircon_plugin_neural_dist | zircon_plugins/neural/dist/Cargo.toml | dist |
| zircon_plugin_neural_editor | zircon_plugins/neural/editor/Cargo.toml | editor |
| zircon_plugin_neural_post_process_runtime | zircon_plugins/neural/features/post_process/runtime/Cargo.toml | runtime |
| zircon_plugin_neural_runtime | zircon_plugins/neural/runtime/Cargo.toml | runtime |
| zircon_plugin_obj_importer_dist | zircon_plugins/obj_importer/dist/Cargo.toml | dist |
| zircon_plugin_obj_importer_runtime | zircon_plugins/obj_importer/runtime/Cargo.toml | runtime |
| zircon_plugin_opus_importer_dist | zircon_plugins/opus_importer/dist/Cargo.toml | dist |
| zircon_plugin_opus_importer_runtime | zircon_plugins/opus_importer/runtime/Cargo.toml | runtime |
| zircon_plugin_particles_dist | zircon_plugins/particles/dist/Cargo.toml | dist |
| zircon_plugin_particles_editor | zircon_plugins/particles/editor/Cargo.toml | editor |
| zircon_plugin_particles_runtime | zircon_plugins/particles/runtime/Cargo.toml | runtime |
| zircon_plugin_physics_dist | zircon_plugins/physics/dist/Cargo.toml | dist |
| zircon_plugin_physics_editor | zircon_plugins/physics/editor/Cargo.toml | editor |
| zircon_plugin_physics_runtime | zircon_plugins/physics/runtime/Cargo.toml | runtime |
| zircon_plugin_sdk_examples_dist | zircon_plugins/plugin_sdk_examples/dist/Cargo.toml | dist |
| zircon_plugin_sdk_examples_editor | zircon_plugins/plugin_sdk_examples/editor/Cargo.toml | editor |
| zircon_plugin_sdk | zircon_plugins/plugin_sdk/Cargo.toml | support |
| zircon_plugin_prefab_tools_dist | zircon_plugins/prefab_tools/dist/Cargo.toml | dist |
| zircon_plugin_prefab_tools_editor | zircon_plugins/prefab_tools/editor/Cargo.toml | editor |
| zircon_plugin_prefab_tools_runtime | zircon_plugins/prefab_tools/runtime/Cargo.toml | runtime |
| zircon_plugin_rendering_dist | zircon_plugins/rendering/dist/Cargo.toml | dist |
| zircon_plugin_rendering_editor | zircon_plugins/rendering/editor/Cargo.toml | editor |
| zircon_plugin_rendering_baked_lighting_editor | zircon_plugins/rendering/features/baked_lighting/editor/Cargo.toml | editor |
| zircon_plugin_rendering_baked_lighting_runtime | zircon_plugins/rendering/features/baked_lighting/runtime/Cargo.toml | runtime |
| zircon_plugin_rendering_contact_shadow_editor | zircon_plugins/rendering/features/contact_shadow/editor/Cargo.toml | editor |
| zircon_plugin_rendering_contact_shadow_runtime | zircon_plugins/rendering/features/contact_shadow/runtime/Cargo.toml | runtime |
| zircon_plugin_rendering_decals_editor | zircon_plugins/rendering/features/decals/editor/Cargo.toml | editor |
| zircon_plugin_rendering_decals_runtime | zircon_plugins/rendering/features/decals/runtime/Cargo.toml | runtime |
| zircon_plugin_rendering_irradiance_volumes_editor | zircon_plugins/rendering/features/irradiance_volumes/editor/Cargo.toml | editor |
| zircon_plugin_rendering_irradiance_volumes_runtime | zircon_plugins/rendering/features/irradiance_volumes/runtime/Cargo.toml | runtime |
| zircon_plugin_rendering_light_cookies_editor | zircon_plugins/rendering/features/light_cookies/editor/Cargo.toml | editor |
| zircon_plugin_rendering_light_cookies_runtime | zircon_plugins/rendering/features/light_cookies/runtime/Cargo.toml | runtime |
| zircon_plugin_rendering_oit_editor | zircon_plugins/rendering/features/oit/editor/Cargo.toml | editor |
| zircon_plugin_rendering_oit_runtime | zircon_plugins/rendering/features/oit/runtime/Cargo.toml | runtime |
| zircon_plugin_rendering_planar_reflections_editor | zircon_plugins/rendering/features/planar_reflections/editor/Cargo.toml | editor |
| zircon_plugin_rendering_planar_reflections_runtime | zircon_plugins/rendering/features/planar_reflections/runtime/Cargo.toml | runtime |
| zircon_plugin_rendering_post_process_editor | zircon_plugins/rendering/features/post_process/editor/Cargo.toml | editor |
| zircon_plugin_rendering_post_process_runtime | zircon_plugins/rendering/features/post_process/runtime/Cargo.toml | runtime |
| zircon_plugin_rendering_ray_tracing_policy_editor | zircon_plugins/rendering/features/ray_tracing_policy/editor/Cargo.toml | editor |
| zircon_plugin_rendering_ray_tracing_policy_runtime | zircon_plugins/rendering/features/ray_tracing_policy/runtime/Cargo.toml | runtime |
| zircon_plugin_rendering_reflection_probes_editor | zircon_plugins/rendering/features/reflection_probes/editor/Cargo.toml | editor |
| zircon_plugin_rendering_reflection_probes_runtime | zircon_plugins/rendering/features/reflection_probes/runtime/Cargo.toml | runtime |
| zircon_plugin_rendering_shader_graph_editor | zircon_plugins/rendering/features/shader_graph/editor/Cargo.toml | editor |
| zircon_plugin_rendering_shader_graph_runtime | zircon_plugins/rendering/features/shader_graph/runtime/Cargo.toml | runtime |
| zircon_plugin_rendering_ssao_editor | zircon_plugins/rendering/features/ssao/editor/Cargo.toml | editor |
| zircon_plugin_rendering_ssao_runtime | zircon_plugins/rendering/features/ssao/runtime/Cargo.toml | runtime |
| zircon_plugin_rendering_subsurface_scattering_editor | zircon_plugins/rendering/features/subsurface_scattering/editor/Cargo.toml | editor |
| zircon_plugin_rendering_subsurface_scattering_runtime | zircon_plugins/rendering/features/subsurface_scattering/runtime/Cargo.toml | runtime |
| zircon_plugin_rendering_vfx_graph_editor | zircon_plugins/rendering/features/vfx_graph/editor/Cargo.toml | editor |
| zircon_plugin_rendering_vfx_graph_runtime | zircon_plugins/rendering/features/vfx_graph/runtime/Cargo.toml | runtime |
| zircon_plugin_rendering_volumetric_fog_editor | zircon_plugins/rendering/features/volumetric_fog/editor/Cargo.toml | editor |
| zircon_plugin_rendering_volumetric_fog_runtime | zircon_plugins/rendering/features/volumetric_fog/runtime/Cargo.toml | runtime |
| zircon_plugin_rendering_runtime | zircon_plugins/rendering/runtime/Cargo.toml | runtime |
| zircon_plugin_runtime_diagnostics_dist | zircon_plugins/runtime_diagnostics/dist/Cargo.toml | dist |
| zircon_plugin_runtime_diagnostics_editor | zircon_plugins/runtime_diagnostics/editor/Cargo.toml | editor |
| zircon_plugin_shader_wgsl_importer_dist | zircon_plugins/shader_wgsl_importer/dist/Cargo.toml | dist |
| zircon_plugin_shader_wgsl_importer_runtime | zircon_plugins/shader_wgsl_importer/runtime/Cargo.toml | runtime |
| zircon_plugin_solari_dist | zircon_plugins/solari/dist/Cargo.toml | dist |
| zircon_plugin_solari_runtime | zircon_plugins/solari/runtime/Cargo.toml | runtime |
| zircon_plugin_sound_dist | zircon_plugins/sound/dist/Cargo.toml | dist |
| zircon_plugin_sound_editor | zircon_plugins/sound/editor/Cargo.toml | editor |
| zircon_plugin_sound_ray_traced_convolution_dist | zircon_plugins/sound/features/ray_traced_convolution_reverb/dist/Cargo.toml | dist |
| zircon_plugin_sound_ray_traced_convolution_editor | zircon_plugins/sound/features/ray_traced_convolution_reverb/editor/Cargo.toml | editor |
| zircon_plugin_sound_ray_traced_convolution_runtime | zircon_plugins/sound/features/ray_traced_convolution_reverb/runtime/Cargo.toml | runtime |
| zircon_plugin_sound_timeline_animation_dist | zircon_plugins/sound/features/timeline_animation_track/dist/Cargo.toml | dist |
| zircon_plugin_sound_timeline_animation_editor | zircon_plugins/sound/features/timeline_animation_track/editor/Cargo.toml | editor |
| zircon_plugin_sound_timeline_animation_runtime | zircon_plugins/sound/features/timeline_animation_track/runtime/Cargo.toml | runtime |
| zircon_plugin_sound_runtime | zircon_plugins/sound/runtime/Cargo.toml | runtime |
| zircon_plugin_terrain_dist | zircon_plugins/terrain/dist/Cargo.toml | dist |
| zircon_plugin_terrain_editor | zircon_plugins/terrain/editor/Cargo.toml | editor |
| zircon_plugin_terrain_runtime | zircon_plugins/terrain/runtime/Cargo.toml | runtime |
| zircon_plugin_texture_importer_dist | zircon_plugins/texture_importer/dist/Cargo.toml | dist |
| zircon_plugin_texture_importer_runtime | zircon_plugins/texture_importer/runtime/Cargo.toml | runtime |
| zircon_plugin_texture_dist | zircon_plugins/texture/dist/Cargo.toml | dist |
| zircon_plugin_texture_editor | zircon_plugins/texture/editor/Cargo.toml | editor |
| zircon_plugin_texture_runtime | zircon_plugins/texture/runtime/Cargo.toml | runtime |
| zircon_plugin_tilemap_2d_dist | zircon_plugins/tilemap_2d/dist/Cargo.toml | dist |
| zircon_plugin_tilemap_2d_editor | zircon_plugins/tilemap_2d/editor/Cargo.toml | editor |
| zircon_plugin_tilemap_2d_runtime | zircon_plugins/tilemap_2d/runtime/Cargo.toml | runtime |
| zircon_plugin_timeline_sequence_dist | zircon_plugins/timeline_sequence/dist/Cargo.toml | dist |
| zircon_plugin_timeline_sequence_editor | zircon_plugins/timeline_sequence/editor/Cargo.toml | editor |
| zircon_plugin_ui_asset_authoring_dist | zircon_plugins/ui_asset_authoring/dist/Cargo.toml | dist |
| zircon_plugin_ui_asset_authoring_editor | zircon_plugins/ui_asset_authoring/editor/Cargo.toml | editor |
| zircon_plugin_ui_document_importer_dist | zircon_plugins/ui_document_importer/dist/Cargo.toml | dist |
| zircon_plugin_ui_document_importer_runtime | zircon_plugins/ui_document_importer/runtime/Cargo.toml | runtime |
| zircon_plugin_virtual_geometry_dist | zircon_plugins/virtual_geometry/dist/Cargo.toml | dist |
| zircon_plugin_virtual_geometry_editor | zircon_plugins/virtual_geometry/editor/Cargo.toml | editor |
| zircon_plugin_virtual_geometry_runtime | zircon_plugins/virtual_geometry/runtime/Cargo.toml | runtime |
| zircon_plugin_zr_vm_language_dist | zircon_plugins/zr_vm_language/dist/Cargo.toml | dist |
| zircon_plugin_zr_vm_language_runtime | zircon_plugins/zr_vm_language/runtime/Cargo.toml | runtime |
