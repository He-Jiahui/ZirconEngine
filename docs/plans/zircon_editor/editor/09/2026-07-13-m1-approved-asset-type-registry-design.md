---
status: approved
owner_plan: docs/plans/zircon_editor/editor/09-editor-asset-management.md
recorded_at: 2026-07-13
related_code:
  - zircon_runtime_interface/src/resource/marker.rs
  - zircon_editor/src/core/asset/type_registry
  - zircon_editor/src/core/editor_extension.rs
  - zircon_editor/src/core/editor_authoring_extension.rs
  - zircon_editor/src/ui/host/editor_asset_manager
  - zircon_editor/src/ui/layouts/views/asset_browser
  - zircon_plugins/editor_support/src/lib.rs
reference_sources:
  - dev/UnrealEngine/Engine/Source/Editor/AssetDefinition/Public/AssetDefinition.h
  - dev/UnrealEngine/Engine/Source/Editor/AssetDefinition/Public/AssetDefinitionRegistry.h
  - dev/Fyrox/editor/src/asset/preview
---

# Editor 09 M1 单一 AssetTypeRegistry 硬切设计

## 批准结论

用户于 2026-07-13 明确批准更新后的方案 A。本设计替代原计划“21 marker + 裸字符串表”的过时假设，
并作为 Editor09 M1 实施权威；不引入旧 API 兼容层、双注册表或 editor 平行 sidecar。

## 类型身份

`AssetTypeId` 是开放 string-newtype，不是 `ResourceKind` alias。合法值采用 lowercase dotted segments：

```text
segment := [a-z][a-z0-9_]*
asset_type_id := segment ('.' segment)*
```

例如 `model`、`material.graph`、`support.asset`、`tilemap_2d.tilemap` 合法；空值、首尾点、连续点、
大写、空白、路径分隔符与控制字符均返回 typed error。26 个 `ResourceKind` 通过唯一函数映射到
canonical `AssetTypeId`，现有 SDK 示例 `Model` 同切片硬切为 `model`，不保留 alias。

## 最终 definition 与 contribution

`AssetTypeRegistry` 是唯一运行期事实源。内建与插件不各持一套最终表：

- `AssetTypeDefinition` 是 materialized 结果，包含 identity、可选 runtime kind、presentation、toolkit、
  creation templates、context commands、thumbnail provider、import schema 与 source/write policy。
- `AssetTypeContribution` 是插件/cdylib 可序列化输入，可以提供自有 custom type base，或为已有内建
  type 补 toolkit/template/command/schema。
- registry merge 要求每个 scalar field 只有一个 owner；重复 toolkit、冲突 runtime kind、冲突
  presentation、重复 template id 或 schema owner 返回 typed error。列表按 canonical id 排序并去重。
- final definition 缺 display/icon/token/thumbnail 等必要 base 字段时 materialization 失败，不允许 UI
  临时猜测 fallback。

`ThumbnailProviderDescriptor` 与 toolkit 都使用序列化 descriptor / `EditorOperationPath`；动态插件边界
不传 Rust trait object、UI object 或 runtime world 引用。

## 硬切范围

M1 同切片删除：

- `EditorExtensionRegistry.asset_editors`、`asset_creation_templates`；
- `register_asset_editor`、`register_asset_creation_template`、`asset_editors()`、
  `asset_creation_templates()`；
- 独立 `AssetEditorDescriptor` 与带重复 `asset_kind` 的 creation-template 表形态；
- importer/graph/timeline descriptor 的裸 `asset_kind/output_kind: String`；
- `EditorAssetMetaDocument`、`editor_meta_path_for_source` 与 `*.editor.meta.toml` 读取路径；
- Asset Browser labels/icons/filter 与 preview palette/provider 的散落 kind match。

所有 `zircon_editor` 与 first-party plugin/editor 调用方在同一切片迁移到新 typed API。不得通过 deprecated、
feature flag、re-export、wrapper、serde alias 或双写维持旧形态。

## 消费与来源权限

- Browser snapshot 携带 registry 投影后的 type id、label、badge、icon 和 source write policy；四布局不再
  直接决定类型视觉。
- Preview owner 按 definition 的 provider descriptor 执行：texture source image、registered operation/
  preview scene 或明确 placeholder token；palette 只在 builtin definition owner 定义。
- double-click/open、create 与 context command 从同一 definition 取得 operation，不查询旧扩展表。
- `res://` 投影为 project writable；package/builtin/library 与 transient memory 投影为 read-only/non-
  authoring。mutation command 的 when 与 dispatch 两层都验证 write policy。

## TDD 与验收

RED 合同先覆盖：

1. `AssetTypeId` 合法/非法矩阵和 serde roundtrip；
2. 26 个 `ResourceKind` canonical mapping 完整且唯一；
3. builtin + plugin contribution materialization；
4. duplicate scalar/template/schema owner typed rejection；
5. 旧两张表/API 和平行 sidecar 静态零命中；
6. 四布局 type presentation、preview provider、open/create/context dispatch 均来自 registry；
7. project writable 与 engine/editor/derived source 拒绝矩阵。

M1 只有在 scoped rustfmt、结构/retired API guard、focused tests 与计划要求的 Windows
`cargo test -p zircon_editor --lib --locked` 证据齐备后才能关闭。

## 产出记录与时间

| 里程碑 | 切片 | 状态 | 完成日期 | 完成项目与证据 |
|---|---|---|---|---|
| M1 | 更新版方案 A 设计裁决 | `APPROVED-实现进行中` | 2026-07-13 | 用户明确批准；设计固定开放 typed `AssetTypeId`、唯一 materialized registry、serializable contributions、全仓旧 API/sidecar 硬切、统一 browser/preview/command 消费与 typed source write policy。 |
