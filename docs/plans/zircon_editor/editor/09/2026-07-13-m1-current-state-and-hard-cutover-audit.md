---
status: completed
owner_plan: docs/plans/zircon_editor/editor/09-editor-asset-management.md
recorded_at: 2026-07-13
related_code:
  - zircon_runtime_interface/src/resource/marker.rs
  - zircon_runtime/src/asset/project/meta.rs
  - zircon_runtime/src/asset/registry
  - zircon_editor/src/core/editor_extension.rs
  - zircon_editor/src/core/editor_authoring_extension.rs
  - zircon_editor/src/ui/host/editor_asset_manager
  - zircon_editor/src/ui/layouts/views/asset_browser.rs
  - zircon_editor/src/ui/layouts/views/asset_browser
  - zircon_plugins/editor_support/src/lib.rs
reference_sources:
  - dev/UnrealEngine/Engine/Source/Editor/AssetDefinition/Public/AssetDefinition.h
  - dev/UnrealEngine/Engine/Source/Editor/AssetDefinition/Public/AssetDefinitionRegistry.h
  - dev/godot/editor/editor_file_system.h
  - dev/Fyrox/editor/src/asset/preview
---

# Editor 09 M1 当前状态与硬切迁移面审计

## 审计边界

本记录在 Editor09 M1 写代码前重核计划证据、当前 `zircon_editor` / `zircon_runtime` / plugin SDK
资产类型表面与参考引擎 owner。它只记录已证实事实和待批准架构裁决，不把设计审计声明为 M1 实现完成。

## 当前源码证据

### runtime 资产元数据与索引已由 Plan10 推进

- `zircon_runtime_interface::resource::ResourceKind` 当前有 26 个 variant；26 个 marker 都通过
  `ResourceMarker::KIND` 映射到该闭集。
- `zircon_runtime::asset::AssetKind` 是 `ResourceKind` 的类型别名，不是可扩展 editor type key。
- `.zmeta` 当前 format version 为 v7，`AssetMetaDocument` 已包含 `import_settings: toml::Table`、
  `source_digest: String` 与严格 tags；旧 `source_hash` 已由 typed error 拒绝。
- runtime 已有 `asset/registry/AssetRegistryIndex`，包括 load/rebuild、incremental、query、persistence、
  dependency edges 与 scan-safety 测试。Editor09 后续索引只能投影/订阅该 owner，不能重建第二份
  editor path/UUID/dependency registry。

因此原计划 M2 中“新增 import_settings/source_digest”和“新建离线 registry”的任务必须在设计批准后
改写为消费当前 v7/registry owner；不得恢复旧格式迁移、双 digest 或 editor 私有 registry。

### editor 已有一套位置错误且分派分裂的资产实现

`zircon_editor/src/ui/host/editor_asset_manager/` 当前已经包含：

- `DefaultEditorAssetManager`、catalog snapshot 与 details records；
- project sync、folder projection、reference analysis 与独立 `ReferenceGraph`；
- preview cache/scheduler、按 kind 的 placeholder palette 与 preview artifact 生成；
- `EditorAssetMetaDocument`，从 `<source>.editor.meta.toml` 读取 `editor_adapter`。

该现物证明 Editor09 不是从零创建 manager，而是要执行 owner 收束：资产领域行为不能继续堆在 UI host，
reference graph 不得复制 Runtime/Plan10 registry，`editor_adapter` 也不得靠平行 sidecar 维持第二事实源。

Asset Browser 当前又在多个 owner 重复 `ResourceKind` match：

- `asset_browser/labels.rs`：badge、compact/summary/full display label；
- `asset_browser/thumbnail_nodes.rs`：icon name；
- `editor_asset_manager/manager/preview_refresh/preview_palette.rs`：raw RGBA palette；
- `generate_preview_artifact.rs`：texture 与 placeholder provider 分派；
- `asset_browser.rs`：固定 kind filter chips。

M1 目标必须是把这些投影改为单一 registry 数据消费，而不是再加一层 helper 包装现有 match。

### 插件 authoring type key 是开放集合

仓库级构造调用提取确认当前 literal editor type key 至少包括：

```text
animation.graph
animation.sequence
animation.state_machine
material
material.graph
model（现有 SDK 示例仍写作不规范的 `Model`）
prefab.asset
support.asset
terrain.heightfield
tilemap_2d.tilemap
tilemap_2d.tileset
```

因此最终 key 不能直接采用闭集 `ResourceKind`；推荐建立严格校验、可 serde、开放的
`AssetTypeId` string-newtype，并为 26 个 runtime kind 提供唯一 canonical 映射。现有 `Model`
大小写漂移必须硬切为 canonical id，不能靠 alias 兼容。

旧 `register_asset_editor` / `register_asset_creation_template` 与读取 API 横跨 `zircon_editor`、
animation/material/navigation/physics/terrain/tilemap/prefab/net/particles/timeline/plugin SDK 示例等 first-party
editor crates。批准后必须在一个硬切切片中迁移全仓调用并删除旧表面。

## 参考引擎裁决输入

- UE `UAssetDefinition` 把 type display name、class、color、categories、open support/open action、
  rename/duplicate/import 能力与 thumbnail/icon 查询放在同一 definition；
  `UAssetDefinitionRegistry` 按 asset class 提供唯一最终 definition，并以注册版本通知变更。
- Fyrox 将 preview generator 做成按 type UUID 注册的集合，缓存只消费 generator；这支持 Zircon 将
  thumbnail provider 归入类型定义，但 Zircon 的 plugin/cdylib 边界要求 provider 使用可序列化 operation/
  descriptor，而不是跨边界 Rust trait object。
- Godot `EditorFileSystem` 的 FileInfo 继续支持“索引行拥有 import 状态，浏览器只读投影”的 Editor09
  方向；当前 Zircon 应消费 Runtime registry + editor 状态投影，不再复制磁盘扫描 owner。

## 已批准的硬切裁决

推荐的更新版方案 A：

1. `core/asset/` 拥有唯一 materialized `AssetTypeRegistry`；key 为开放、严格校验的 `AssetTypeId`。
2. 26 个 `ResourceKind` 映射为内建 canonical definitions；插件只提交可序列化
   `AssetTypeContribution`，最终 registry 负责合并和冲突拒绝。
3. 每个最终 definition 统一持有 display/badge/icon/design-token、runtime kind 映射、toolkit operation、
   creation templates、context commands、thumbnail provider descriptor 与 import schema ref。
4. `AssetImporterDescriptor`、graph/timeline authoring descriptor 中的裸 asset-kind string 同步改用
   `AssetTypeId`，避免旁路重新形成 stringly-typed owner。
5. 删除 `asset_editors`、`asset_creation_templates` 两张表和全部旧注册/读取 API；不留 wrapper、alias、
   re-export 或双写。
6. 删除 `EditorAssetMetaDocument` / `*.editor.meta.toml` 路径；adapter/toolkit 从最终 type definition 推导。
7. Asset Browser 四布局、preview generator、打开与创建命令消费同一 registry projection；旧 kind match
   只允许存在于内建 definition 构造 owner，不散落 UI/preview。
8. authoring 写权限由 resource scheme/source authority 投影：`res://` 可写；package/builtin/library 与
   transient memory 不可作为工程资产写入，mutation command 使用 typed when/dispatch guard 拒绝。

用户已于 2026-07-13 明确批准该裁决；实施权威与硬切验收口径见
`09/2026-07-13-m1-approved-asset-type-registry-design.md`。

## 产出记录与时间

| 里程碑 | 切片 | 状态 | 完成日期 | 完成项目与证据 |
|---|---|---|---|---|
| M1 | 执行前现状与迁移面审计 | `审计完成-设计已批准-实现进行中` | 2026-07-13 | 重核 26 个 runtime kind；确认 zmeta v7 已有 import settings/source digest/tags、Runtime AssetRegistryIndex 已存在；枚举开放 plugin authoring type key 与全仓旧注册 API；确认 UI host 重复 catalog/reference/preview 与 `.editor.meta.toml` 平行 sidecar；UE/Fyrox/Godot owner 对照支持单一 materialized registry + serializable contributions。 |
| M1 | 更新版方案 A 用户裁决 | `APPROVED-进入TDD实现` | 2026-07-13 | 用户明确回复“批准”；批准范围为开放 typed `AssetTypeId`、唯一 materialized `AssetTypeRegistry`、可序列化 contribution、全仓旧表/API 硬切、平行 editor sidecar 删除及浏览器/预览/命令统一消费。批准后的规范设计见 `09/2026-07-13-m1-approved-asset-type-registry-design.md`。 |
