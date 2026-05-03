---
related_code:
  - zircon_editor/src/core/editor_authoring_extension.rs
  - zircon_editor/src/core/editor_extension.rs
  - zircon_editor/src/core/editor_operation.rs
  - zircon_editor/src/core/editor_plugin.rs
  - zircon_plugins/editor_support/src/lib.rs
  - zircon_runtime/src/asset/assets/authoring.rs
  - zircon_runtime/src/asset/assets/imported.rs
  - zircon_runtime/src/asset/assets/scene.rs
  - zircon_runtime/src/asset/artifact/store.rs
  - zircon_runtime/src/builtin/runtime_modules.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog.rs
  - zircon_runtime_interface/src/resource/marker.rs
  - zircon_plugins/terrain/plugin.toml
  - zircon_plugins/tilemap_2d/plugin.toml
  - zircon_plugins/material_editor/plugin.toml
  - zircon_plugins/prefab_tools/plugin.toml
  - zircon_plugins/timeline_sequence/plugin.toml
  - zircon_plugins/animation_graph/plugin.toml
implementation_files:
  - zircon_editor/src/core/editor_authoring_extension.rs
  - zircon_editor/src/core/editor_extension.rs
  - zircon_editor/src/core/editor_operation.rs
  - zircon_editor/src/core/editor_plugin.rs
  - zircon_plugins/editor_support/src/lib.rs
  - zircon_plugins/terrain/runtime/src/lib.rs
  - zircon_plugins/terrain/editor/src/lib.rs
  - zircon_plugins/terrain/README.md
  - zircon_plugins/tilemap_2d/runtime/src/lib.rs
  - zircon_plugins/tilemap_2d/editor/src/lib.rs
  - zircon_plugins/tilemap_2d/README.md
  - zircon_plugins/material_editor/editor/src/lib.rs
  - zircon_plugins/material_editor/README.md
  - zircon_plugins/prefab_tools/runtime/src/lib.rs
  - zircon_plugins/prefab_tools/editor/src/lib.rs
  - zircon_plugins/prefab_tools/README.md
  - zircon_plugins/timeline_sequence/editor/src/lib.rs
  - zircon_plugins/timeline_sequence/README.md
  - zircon_plugins/animation_graph/editor/src/lib.rs
  - zircon_plugins/animation_graph/README.md
  - zircon_runtime/src/asset/assets/authoring.rs
  - zircon_runtime/src/asset/assets/imported.rs
  - zircon_runtime/src/asset/assets/scene.rs
  - zircon_runtime_interface/src/resource/marker.rs
tests:
  - cargo check -p zircon_runtime_interface --lib --locked --message-format short --color never
  - cargo check -p zircon_runtime --lib --locked --message-format short --color never
  - cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_terrain_runtime -p zircon_plugin_tilemap_2d_runtime -p zircon_plugin_prefab_tools_runtime --locked --message-format short --color never
  - cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_editor_support -p zircon_plugin_terrain_editor -p zircon_plugin_tilemap_2d_editor -p zircon_plugin_material_editor_editor -p zircon_plugin_prefab_tools_editor -p zircon_plugin_timeline_sequence_editor -p zircon_plugin_animation_graph_editor --locked --message-format short --color never
  - cargo test -p zircon_runtime --lib authoring --locked --message-format short --color never
  - cargo test -p zircon_runtime --lib authoring_plugin_manifests_match_catalog_and_workspace_shape --locked --jobs 1 --message-format short --color never
  - cargo test -p zircon_runtime --lib source_template_links_runtime_backed_authoring_and_excludes_editor_only_authoring --locked --jobs 1 --message-format short --color never
  - cargo test -p zircon_editor --lib authoring --locked --message-format short --color never
  - cargo test -p zircon_editor --lib authoring_registry_rejects_invalid_operation_payload_schema_ids --locked --jobs 1 --target-dir E:\cargo-targets\zircon-authoring-payload-schema --message-format short --color never
  - cargo test -p zircon_editor --lib builtin_editor_catalog_declares_authoring_plugin_capabilities --locked --jobs 1 --message-format short --color never
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_editor_support authoring_batch_registers_menu_items_payload_schemas_and_all_descriptor_families --locked --jobs 1 --message-format short --color never
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_material_editor_editor -p zircon_plugin_timeline_sequence_editor -p zircon_plugin_animation_graph_editor -p zircon_plugin_prefab_tools_editor -p zircon_plugin_terrain_editor -p zircon_plugin_tilemap_2d_editor --locked --jobs 1 --target-dir E:\cargo-targets\zircon-authoring-plugin-behavior --message-format short --color never
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_prefab_tools_runtime -p zircon_plugin_terrain_runtime -p zircon_plugin_tilemap_2d_runtime --locked --target-dir E:\cargo-targets\zircon-plugins-runtime-interface-boundary --jobs 1 -- --format terse
  - cargo test --manifest-path zircon_plugins/Cargo.toml --workspace --locked --target-dir E:\cargo-targets\zircon-plugins-runtime-interface-boundary --jobs 1 -- --format terse
  - cargo build --workspace --locked --verbose
  - cargo test --workspace --locked --verbose
  - cargo test --workspace --locked --target-dir E:\cargo-targets\zircon-runtime-interface-boundary --jobs 1 -- --format terse
  - cargo build --manifest-path zircon_plugins/Cargo.toml --workspace --locked --verbose
plan_sources:
  - .codex/plans/ZirconEngine 独立插件补齐计划.md
  - user request 2026-05-02 ZirconEngine Authoring 插件补齐实施计划
  - dev/UnrealEngine/Engine/Documentation/Source/Shared/LandscapeEditor
  - dev/UnrealEngine/Engine/Plugins/2D/Paper2D/Source
---

# Authoring Plugin Extension Contracts

## Scope

本轮 Authoring 插件补齐把六个工具型插件接入统一插件体系：

- runtime-backed：`terrain`、`tilemap_2d`、`prefab_tools`。它们拥有 runtime crate、editor crate、`plugin.toml`、runtime catalog ID 和 runtime asset/component DTO。
- editor-only：`material_editor`、`timeline_sequence`、`animation_graph`。它们只注册 editor catalog、editor package manifest 和 authoring surface，不进入 runtime export 链路。

Unreal 参照用于拆分生命周期和编辑器表面：Landscape 负责 heightfield/layer/import/tool mode 结构，Paper2D 负责 tileset/tilemap/Tiled importer 结构，MaterialEditor/AnimGraph/Sequencer 负责 graph、palette、timeline 和 operation 的编辑器描述符形状，LevelInstance/BlueprintGraph 负责 prefab instance、override 和 create/open/apply/revert 操作的边界。Zircon 的实现仍以现有 `plugin.toml`、catalog、capability gate、runtime `ResourceKind` 和 editor extension registry 为准，不复制 Unreal 源码。

## Editor Registry Contract

`zircon_editor::core::editor_authoring_extension` 是通用 Authoring 描述符层，当前包含：

- `AssetCreationTemplateDescriptor`
- `ViewportToolModeDescriptor`
- `GraphEditorDescriptor`
- `GraphNodePaletteDescriptor`
- `GraphNodeDescriptor`
- `GraphPinDescriptor`
- `TimelineEditorDescriptor`
- `TimelineTrackDescriptor`

这些 descriptor 都带有 capability gate 字段。`EditorExtensionRegistry` 为每一类 descriptor 提供独立 map 和 `register_*` 方法，并执行重复 ID、空 ID、graph palette 空节点和重复 node ID 校验。`EditorPluginCatalog::editor_extensions()` 聚合插件扩展时统一合并 view、drawer、template、operation、menu item、importer、asset editor、component drawer、tool mode、graph/timeline descriptor；workbench 侧应继续消费通用 descriptor，不为单个 Authoring 插件写特殊分支。

`zircon_plugins/editor_support` 的 `EditorAuthoringContributionBatch` 是插件包使用的批量注册入口。每个 editor crate 先注册一个基础 authoring surface，再通过 batch 注册 operation、menu item、importer、asset editor、component drawer、template、tool mode、graph editor、palette 或 timeline track。batch 测试固定了 menu item、payload schema 和所有 authoring descriptor family 会在一次注册中进入同一个 registry。

`EditorOperationDescriptor` 现在有可选 `payload_schema_id`，用于把命令 ID、菜单路径和操作参数 DTO 版本连起来。六个 Authoring editor crate 的用户可触发 operation 都同时声明：

- `menu_path`，例如 `Plugins/Terrain/Import Heightfield` 或 `Plugins/Material Editor/Compile Graph`。
- `payload_schema_id`，例如 `terrain.import_heightfield.v1` 或 `material_editor.compile_graph.v1`。
- `required_capabilities`，沿用 `editor.extension.<plugin_id>_authoring`。

这样 UI、远程调用、菜单投影和后续 schema 文档可以通过通用 operation descriptor 发现操作，不需要为单个插件写硬编码分支。

`payload_schema_id` 使用点号分段的稳定 ID 规则，和 operation path 共享 segment 字符集：ASCII 字母、数字、`_`、`-`，且至少三段。`EditorOperationRegistry::register` 会拒绝空段、空白字符和分段过少的 schema ID，避免菜单、远程调用和 payload DTO 文档之间出现不可寻址的 operation 参数契约。

## Runtime Data Contract

新增 runtime authoring asset DTO 位于 `zircon_runtime/src/asset/assets/authoring.rs`：

- `TerrainAsset`、`TerrainLayerAsset`、`TerrainLayerStackAsset`
- `TileSetAsset`、`TileMapAsset`、`TileMapLayerAsset`、`TileMapProjectionAsset`
- `PrefabAsset`、`PrefabInstanceAsset`、`PrefabPropertyOverrideAsset`
- `MaterialGraphAsset`、`MaterialGraphNodeAsset`、`MaterialGraphNodeKindAsset`、`MaterialGraphLinkAsset`、`MaterialGraphParameterAsset`

这些类型进入 `ImportedAsset`、direct reference 收集、artifact store、project asset manager loading 和 resource payload store。`SceneEntityAsset` 也显式增加 `terrain`、`tilemap`、`prefab_instance` 三个可选 runtime DTO 字段，确保 prefab/terrain/tilemap 能参与场景引用收集，同时保持旧 TOML 通过 `serde(default)` 兼容解析。

新增 `ResourceKind`/marker：

- `MaterialGraph`
- `Terrain`
- `TerrainLayerStack`
- `TileSet`
- `TileMap`
- `Prefab`

runtime-backed 插件的能力名固定为：

- `runtime.plugin.terrain`
- `runtime.plugin.tilemap_2d`
- `runtime.plugin.prefab_tools`

editor 能力名固定为：

- `editor.extension.terrain_authoring`
- `editor.extension.tilemap_2d_authoring`
- `editor.extension.material_editor_authoring`
- `editor.extension.prefab_tools_authoring`
- `editor.extension.timeline_sequence_authoring`
- `editor.extension.animation_graph_authoring`

## Plugin Package Shape

`terrain`、`tilemap_2d`、`prefab_tools` 使用 runtime + editor 双 crate：

- runtime crate 贡献 `RuntimePluginDescriptor`、component type 和 importer descriptor。
- editor crate 贡献 authoring view、drawer、template、operation、importer、asset editor、component drawer 和插件专属工具描述符。
- runtime builtin catalog 中三者的 `category` 固定为 `authoring`，并要求 `plugin.toml`、workspace member、runtime module、editor module 和 capability 名完全一致。
- 每个插件目录有本地 `README.md`，记录 package id、runtime/editor 能力、runtime export 边界、authoring surface 和测试关注点。

`material_editor`、`timeline_sequence`、`animation_graph` 使用 editor-only crate：

- `material_editor` 注册 material graph editor、material asset editor、compile/preview/validate/create operation 和 v1 node palette：`output`、`texture_sample`、`scalar_parameter`、`vector_parameter`、`add`、`multiply`。
- `timeline_sequence` 基于 `animation.sequence` 注册 timeline editor 和 v1 track type：`transform`、`component_property`、`event_marker`，并依赖 animation 插件能力。
- `animation_graph` 基于 `animation.graph`/`animation.state_machine` 注册 graph editor、validate/compile/open operation、animation player component drawer 和 v1 palette：`clip`、`blend`、`output`、`state`、`transition`、`condition`。
- 这三个 editor-only 包不能出现在 runtime builtin catalog，也不能拥有 `*/runtime` workspace member；export plan 测试固定它们不会进入 runtime registration、linked runtime crate 或 generated Cargo manifest。
- 每个 editor-only 插件目录同样有本地 `README.md`，明确该包 excluded from runtime export。

## V1 Authoring Helpers

插件包现在把可测试的 v1 行为放在各自 editor crate 内，保持 workbench 只消费通用 descriptor：

- `terrain` 提供 heightfield 导入请求校验、import output kind 解析和 `TerrainImportPlan`，覆盖 `raw`、`r16`、`png` 默认扩展，并区分 heightfield 与 layer stack 输出。
- `tilemap_2d` 提供 tilemap 编辑器校验、projection 支持判定、layer/tile 统计和 `TilemapPaintRequest` 网格写入 helper。
- `material_editor` 提供 material graph 校验、最小 `MaterialAsset` compile 和 operation-style `MaterialGraphCompileReport`，v1 支持 output、texture sample、scalar/vector parameter、add、multiply。
- `prefab_tools` 提供 prefab instance source/override 校验、override precedence 合并、apply/revert overrides 和 break instance 的 editor authoring 状态。
- `timeline_sequence` 提供 timeline keyframe 范围/排序校验、track path deterministic sort、event marker payload 校验和 keyframe move helper。
- `animation_graph` 提供 animation graph/state machine 校验、最小 compile output source 解析和 state machine compile report。

## Validation Notes

基线已验证（以下是 menu/payload schema 和新增 manifest/export 测试加入前的 Authoring 基线证据）：

- runtime interface `ResourceKind`/marker 编译通过。
- runtime authoring DTO、`ImportedAsset`、artifact loading 和 runtime builtin catalog 编译通过。
- 三个 runtime-backed 插件 crate 编译通过。
- 六个 editor crate 与 `editor_support` 编译通过。
- `zircon_runtime --lib authoring` 中 4 个 Authoring asset 测试通过，覆盖 terrain roundtrip/reference、tilemap projection/layer size、material graph output/reference、prefab scene reference。
- `zircon_editor --lib authoring` 中 51 个 authoring 相关测试通过，覆盖 descriptor 注册、capability gate、重复 graph node ID、既有 UI authoring 回归。
- root `cargo build --workspace --locked --verbose` 通过，验证当前 Authoring 改动没有破坏根工作区编译。
- 六个 Authoring editor package 的 v1 helper 测试通过：`animation_graph` 4、`material_editor` 5、`prefab_tools` 3、`terrain` 2、`tilemap_2d` 3、`timeline_sequence` 4，共 21 个 package-level 测试。命令使用独立 target dir `E:\cargo-targets\zircon-authoring-plugin-behavior` 和 `--jobs 1`，仅保留既有 runtime/editor warning。

本轮新增覆盖：

- `EditorAuthoringContributionBatch.menu_items` 批量注册测试，覆盖 menu item、operation payload schema、importer、asset editor、component drawer、asset creation template、viewport tool mode、graph editor、palette、timeline editor 和 track type。
- 六个 Authoring editor crate 的 operation/menu/payload schema 可观测测试。
- `EditorOperationDescriptor.payload_schema_id` getter 与 TOML roundtrip 测试。
- `EditorOperationRegistry` 对非法 payload schema ID 的拒绝测试，覆盖带空格的 schema ID 不能注册。
- `EditorPluginDescriptor::builtin_catalog()` 的六个 Authoring editor capability 测试。
- runtime package manifest 测试，固定 runtime-backed 三包与 runtime catalog/workspace/plugin.toml 一致，并固定 editor-only 三包不进入 runtime catalog。
- export plan 测试，固定 `terrain`、`tilemap_2d`、`prefab_tools` 会链接 runtime crate，而 `material_editor`、`timeline_sequence`、`animation_graph` 不进入 runtime registration 或 generated Cargo manifest。
- 六个插件目录补齐本地 README，固定 runtime-backed/editor-only 边界和各包 v1 authoring surface。
- 2026-05-03 runtime-interface 缺口收束中，`terrain`、`tilemap_2d`、`prefab_tools` 的 runtime `runtime_package_manifest()` helper 改为从 `RuntimePluginDescriptor::package_manifest()` 直接投影基础 manifest，再追加 component/importer 贡献。这样 `RuntimePlugin::package_manifest(&runtime_plugin())` 不会在 override 内回到 helper 自身并造成递归栈溢出。

本轮新增后的验证状态：

- 针对 `prefab_tools`、`terrain`、`tilemap_2d` runtime manifest helper 的 `cargo fmt --manifest-path zircon_plugins/Cargo.toml -p ...` 已通过。
- `cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_prefab_tools_runtime -p zircon_plugin_terrain_runtime -p zircon_plugin_tilemap_2d_runtime --locked --target-dir E:\cargo-targets\zircon-plugins-runtime-interface-boundary --jobs 1 -- --format terse` 已通过：三个包各 1 个 runtime manifest/registration 测试通过，doc-tests 为空。
- `cargo test --manifest-path zircon_plugins/Cargo.toml --workspace --locked --target-dir E:\cargo-targets\zircon-plugins-runtime-interface-boundary --jobs 1 -- --format terse` 已通过；此前记录的 plugin workspace full validation 阻塞已经不再复现。
- root `cargo test --workspace --locked --target-dir E:\cargo-targets\zircon-runtime-interface-boundary --jobs 1 -- --format terse` 也已通过；此前 virtual-geometry debug snapshot 行为阻塞已经由 runtime-side execution snapshot/stat 收束修复。
- `cargo test -p zircon_editor --lib authoring_registry_rejects_invalid_operation_payload_schema_ids --locked --jobs 1 --target-dir E:\cargo-targets\zircon-authoring-payload-schema --message-format short --color never` 在多会话 Cargo 队列中两次超时，分别约 604 秒和 904 秒；两次都没有产生通过/失败输出，第二次遗留的 matching Cargo/rustc 进程已按该 target dir 清理。静态兜底检查确认新增 `EditorOperationRegistryError` variant 没有现有 exhaustive match 漏点，且当前声明的 payload schema ID 都符合三段点号规则。
- 仍未重新运行 CI 形态的 `cargo build --workspace --locked --verbose` / `cargo test --workspace --locked --verbose`。本次接受证据使用的是 locked workspace test 的 terse 形态和独立 target dir。
