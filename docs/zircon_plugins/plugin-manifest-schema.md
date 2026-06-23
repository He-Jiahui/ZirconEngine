# 插件 Manifest Schema（`plugin.toml`）权威定义

> 本文是 ZirconEngine 插件 `plugin.toml` 的**唯一 schema 权威**，由 [Plugins 12](../plans/zircon_plugins/12-plugin-dx-and-structure-framework.md) 落地、[引擎结构规范 §6.2](../plans/engine-code-structure-convention.md) 引用。所有插件（含 `asset_importers/*`、native-dynamic）必须提供符合本 schema 的 `plugin.toml`，位于插件 crate 根。
>
> 状态：in_progress（schema 定稿；2026-06-22 已落 D-S7/D3 静态生成标记与 native 单源嵌入、补齐 `asset_importers/*` 与 `opus_importer` 缺失 manifest，并让 `plugins_12_manifest_schema_uniform` 与 `tools/audit_plugin_structure.py --json` 覆盖 37 份 manifest 的必选字段与 `supported_platforms`；2026-06-23 已让首个 `[distribution]` 段进入 Plugins 13 M1 guard，`distribution_section_violations = 0`，并在 Plugins 13 M4/T2 接入 runtime typed manifest 与 native loader compatibility gate；完整 runtime schema 校验器、全 feature descriptor parity 与 capability 四源一致性 guard 仍由 Plugins 12 后续切片收口）

## 1. 设计原则

- **必选段固定形状**：30 行与 105 行插件共享同一骨架，差异只在可选段是否出现，不在必选段结构。
- **单一来源**：`capabilities` 与 crate `capability.rs` 的 `pub const` 双向一致（`capability_source_mismatches = 0`）。
- **四源一致性**：`plugin.toml` ↔ `capability.rs` ↔ runtime descriptor ↔ workspace member 由 guard 交叉核对。
- **静态 manifest 生成标记**：非 native `plugin.toml` 必须以 `# @generated from Rust descriptor package_manifest(); do not edit by hand.` 开头，表明静态插件 manifest 是 descriptor 派生产物，人工编辑需回写 descriptor/source owner；当前 guard 覆盖 36 个非 native manifest（含嵌套 `asset_importers/*` 与 `opus_importer`）。

## 2. 必选段

```toml
id = "<plugin>"                       # 插件标识，与 capability 前缀一致（runtime.plugin.<plugin>）
version = "0.1.0"
sdk_api_version = "0.1.0"             # 插件 SDK ABI 版本
display_name = "..."
category = "runtime|asset_importer|editor|..."
description = "..."
supported_targets = ["client_runtime", "editor_host", ...]
supported_platforms = ["windows", "linux", "macos"]
capabilities = ["runtime.plugin.<plugin>", ...]
maturity = "stable|beta|experimental"

[[modules]]                           # 每个 crate 一条
name = "<plugin>.runtime"             # 形如 <plugin>.{runtime|editor}
kind = "runtime|editor"
crate_name = "zircon_plugin_<plugin>_runtime"
target_modes = [...]
capabilities = [...]
system_anchors = [...]                # 与实际注册的 runtime system 源核对
```

## 3. 可选段（按需出现，形状固定）

| 段 | 用途 | 关键字段 |
|---|---|---|
| `[[capability_statuses]]` | 能力成熟度 | `capability`、`status`(`partial`/`stable`) |
| `[[asset_importers]]` | 导入器声明 | `id`、`source_extensions`、`output_kind`、`importer_version`、`required_capabilities` |
| `[[optional_features]]` | 可选功能 | `id` |
| `[[dependencies]]` | 插件间依赖 | `id` |
| `[[options]]` | 运行期选项 | `key`、默认值 |
| `[[event_catalogs]]` | 事件目录 | namespace |
| `[distribution]` | 独立构建/分发（见下） | `forms`、`abi_version`、`engine_compat`、`dist_crate`、`descriptor_symbol`、`runtime_entry`、`assets` |

### 3.1 `[distribution]` 段（插件独立构建）

由 [Plugins 13](../plans/zircon_plugins/13-standalone-plugin-build.md) 落地，规范权威 [`plugin-standalone-build.md`](plugin-standalone-build.md)。声明插件的产物形态与可分发动态包契约：

```toml
[distribution]
forms = ["embed", "dist"]                 # embed=rlib 静态链接 / dist=cdylib 可分发
default_packaging = ["library_embed", "native_dynamic"]
abi_version = 3                            # 与 ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V3 钉
engine_compat = ">=0.1, <0.2"             # 引擎兼容区间，loader 加载期校验
dist_crate = "zircon_plugin_<plugin>_runtime"
descriptor_symbol = "zircon_native_plugin_descriptor_v3"
runtime_entry = "zircon_plugin_<plugin>_runtime_entry_v3"
editor_entry  = "zircon_plugin_<plugin>_editor_entry_v3"   # 可选
assets = ["assets/**"]                     # 随包资产（进 per-plugin zrpack 子包）
```

- 非 native 插件的 `[distribution]` 由 descriptor `package_manifest()` 投影（`@generated`），不手写漂移。
- runtime 侧 `PluginPackageManifest` 已 typed 承载 `[distribution]`，native loader 会在库路径探测前校验 `forms`、`abi_version` 与 `engine_compat`；不匹配时输出结构化诊断并跳过插件。
- `dist` 形态产物的依赖闭包禁含 `zircon_runtime`（依赖边界 guard，见规范 §8）。

## 4. native-dynamic 插件规则

两 `[[modules]]`（runtime + editor）必须以 `kind` 区分，且 `crate_name` 不得同名却不区分；若 runtime/editor 共用一个 cdylib，须显式以 `kind` 标注用途，避免 reviewer 无法判断哪个 module 导出 editor API。

native-dynamic 仍保留手写根 `plugin.toml`，但 native cdylib 代码不得维护第二份内嵌 TOML；ABI descriptor 应通过 `include_str!` 嵌入同一根 manifest，并在 C ABI 指针处追加 `\0`。

## 5. 校验器与 guard（Plugins 12 M1）

- 校验器：`zircon_runtime/src/plugin/package_manifest/*` 扩 schema 校验。
- 一致性 guard：`plugins_12_manifest_schema_uniform`、`plugins_12_static_plugin_manifest_is_generated`、`plugins_12_capability_single_source`、`plugins_13_dist_dependency_boundary_clean`。当前已落的 `plugins_12_static_plugin_manifest_is_generated` 覆盖生成头、静态 runtime descriptor 子集 parity、native 单源嵌入以及多行 TOML 数组解析；`plugins_12_manifest_schema_uniform` 与 `tools/audit_plugin_structure.py --json` 已覆盖 36 个 generated 非 native manifest + 1 个 native 手写 manifest 的必选字段和 `supported_platforms`；Plugins 13 M1 首个 `[distribution]` guard 覆盖 `native_dynamic_fixture`，报告 `distribution_section_violations = 0`、`dist_dependency_boundary_violations = 0`。后续仍需收口 runtime schema 校验器、全 feature descriptor parity 与 capability 四源一致性。
- 审计字段：`missing_plugin_toml`、`manifest_schema_violations`、`capability_source_mismatches`、`distribution_section_violations`、`dist_dependency_boundary_violations`（见 `tools/plugin_structure_audits/`）。
