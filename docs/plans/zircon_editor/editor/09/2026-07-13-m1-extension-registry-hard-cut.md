---
status: completed
owner_plan: docs/plans/zircon_editor/editor/09-editor-asset-management.md
recorded_at: 2026-07-13
related_code:
  - zircon_editor/src/core/asset/type_registry/
  - zircon_editor/src/core/editor_extension.rs
  - zircon_editor/src/core/editor_authoring_extension.rs
  - zircon_editor/src/core/editor_plugin.rs
  - zircon_editor/src/core/editor_plugin_catalog_gen.rs
  - zircon_editor/src/core/editor_plugin_sdk/
  - zircon_editor/src/ui/host/editor_extension_registration.rs
  - zircon_editor/src/ui/host/editor_event_runtime_access.rs
  - zircon_plugins/editor_support/src/lib.rs
tests:
  - zircon_editor/src/tests/editor_asset_type_registry/extension_registry.rs
  - zircon_editor/src/tests/editor_asset_type_registry/materialization.rs
  - zircon_editor/src/tests/editor_asset_type_registry/typed_authoring_descriptors.rs
---

# Editor09 M1.2 扩展注册表硬切产出

## 产出记录与时间

| 里程碑 | 切片 | 状态 | 完成日期 | 完成项目与证据 |
|---|---|---|---|---|
| M1 | 1.2 `EditorExtensionRegistry` 与 first-party plugin typed hard cut | `COMPLETED` | 2026-07-13 | 删除 `AssetEditorDescriptor`、`asset_editors`、独立 `asset_creation_templates` 表及旧 register/read API；creation template 收入 `AssetTypeContribution`；importer/graph/palette/timeline 统一使用 `AssetTypeId`；host/catalog 按 plugin owner materialize 单一 registry；first-party editor plugins 同切片迁移。Windows focused suite 16/16 通过；13 个 first-party editor crates 的 locked `cargo check` 矩阵通过；retired API 生产源码静态扫描零命中。 |

### 实现摘要

- `EditorExtensionRegistry` 只保留 `asset_type_contributions` 一个资产类型扩展入口，同一插件对同一
  `AssetTypeId` 重复注册会返回 typed duplicate error。
- `AssetCreationTemplateDescriptor` 迁入 folder-backed `core/asset/type_registry/`，模板由最终
  `AssetTypeDefinition` 持有；跨插件 template id 冲突会报告首个与第二个 owner。
- `AssetImporterDescriptor.output_type`、`GraphEditorDescriptor.asset_type`、
  `GraphNodePaletteDescriptor.asset_type` 与 `TimelineEditorDescriptor.asset_type` 均为严格 serde
  `AssetTypeId`，无 string alias、deprecated wrapper 或双字段。
- `EditorExtensionRegistration` 保留插件 package id，host 注册及 capability-filtered runtime 查询均从
  built-ins + enabled contributions materialize；SDK facade 不再 re-export 旧 descriptor。
- 迁移覆盖 material、animation graph、navigation、network、particles、physics、prefab、terrain、
  tilemap、timeline、desktop export 与 SDK 示例等 first-party editor crates；新增直接
  `zircon_runtime_interface` manifest 依赖，避免经 runtime crate 借道使用 `ResourceKind`。

### 验证证据

- scoped `rustfmt --edition 2021`：通过。
- `cargo test -p zircon_editor --lib editor_asset_type_registry --locked --no-run --jobs 1`：通过；
  coordinator job `0acdca5d9d484144afec00c1d260d6cf`。
- `.codex/tmp/zircon_editor-editor09-m1-2-20260713.exe editor_asset_type_registry
  --test-threads=1 --nocapture`：16 passed，0 failed，3129 filtered。
- `cargo check --manifest-path zircon_plugins/Cargo.toml --locked --jobs 1` 对 13 个受影响
  first-party editor crates：通过；日志
  `.codex/tmp/editor09-m1-2-first-party-plugin-check-20260713-r3.log`。
- `AssetEditorDescriptor`、旧两张表、旧 register API、`with_output_kind` 与 authoring descriptor
  `asset_kind()` 生产源码扫描：零命中。

### 边界与后续

M1.2 不宣称整个 M1 完成。Browser 四布局、preview provider/palette、open/create/context registry
projection、`*.editor.meta.toml` 退役属于 M1.3；A/B source authority 和双层只读拒绝属于 M1.4。
