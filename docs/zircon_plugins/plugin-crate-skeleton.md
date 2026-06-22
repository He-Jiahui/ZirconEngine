---
related_code:
  - zircon_plugins/plugin_sdk_examples/plugin.toml
  - zircon_plugins/plugin_sdk_examples/editor/Cargo.toml
  - zircon_plugins/plugin_sdk_examples/editor/src/lib.rs
  - zircon_plugins/plugin_sdk_examples/editor/src/plugin.rs
  - zircon_plugins/plugin_sdk_examples/editor/src/capability.rs
  - zircon_plugins/plugin_sdk_examples/editor/src/extensions.rs
  - zircon_plugins/plugin_sdk_examples/editor/src/extension_ids.rs
  - zircon_plugins/plugin_sdk_examples/editor/src/tests.rs
  - plugin_structure_audits/skeleton.py
  - audit_plugin_structure.py
  - zircon_plugins/first_party_runtime_catalog/src/lib.rs
  - zircon_plugins/plugin_sdk/src/registration.rs
  - zircon_plugins/plugin_sdk/src/test.rs
  - zircon_plugins/animation/runtime/Cargo.toml
  - zircon_plugins/animation/runtime/src/lib.rs
  - zircon_plugins/animation/runtime/src/runtime_system.rs
  - zircon_plugins/asset_importers/data/runtime/src/lib.rs
  - zircon_plugins/asset_importers/data/runtime/src/plugin.rs
  - zircon_plugins/asset_importers/model/runtime/src/lib.rs
  - zircon_plugins/asset_importers/model/runtime/src/plugin.rs
  - zircon_plugins/asset_importers/shader/runtime/src/lib.rs
  - zircon_plugins/asset_importers/shader/runtime/src/plugin.rs
  - zircon_plugins/gltf_importer/runtime/src/lib.rs
  - zircon_plugins/gltf_importer/runtime/src/plugin.rs
  - zircon_plugins/obj_importer/runtime/src/lib.rs
  - zircon_plugins/obj_importer/runtime/src/plugin.rs
  - zircon_plugins/texture_importer/runtime/src/lib.rs
  - zircon_plugins/texture_importer/runtime/src/plugin.rs
  - zircon_plugins/audio_importer/runtime/src/lib.rs
  - zircon_plugins/audio_importer/runtime/src/plugin.rs
  - zircon_plugins/opus_importer/runtime/src/lib.rs
  - zircon_plugins/opus_importer/runtime/src/plugin.rs
  - zircon_plugins/shader_wgsl_importer/runtime/src/lib.rs
  - zircon_plugins/shader_wgsl_importer/runtime/src/plugin.rs
  - zircon_plugins/ui_document_importer/runtime/src/lib.rs
  - zircon_plugins/ui_document_importer/runtime/src/plugin.rs
  - plugin_structure_audits/registration.py
  - zircon_runtime/src/builtin/runtime_modules/ids/plugin_id.rs
  - zircon_runtime/src/builtin/runtime_modules/plugin_modules/loader.rs
implementation_files:
  - zircon_plugins/plugin_sdk_examples/editor/src/lib.rs
  - zircon_plugins/plugin_sdk_examples/editor/src/plugin.rs
  - zircon_plugins/plugin_sdk_examples/editor/src/capability.rs
  - zircon_plugins/plugin_sdk_examples/editor/src/extensions.rs
  - zircon_plugins/plugin_sdk_examples/editor/src/extension_ids.rs
  - zircon_plugins/plugin_sdk_examples/editor/src/tests.rs
  - plugin_structure_audits/skeleton.py
  - audit_plugin_structure.py
  - zircon_plugins/first_party_runtime_catalog/src/lib.rs
  - zircon_plugins/plugin_sdk/src/registration.rs
  - zircon_plugins/plugin_sdk/src/test.rs
  - zircon_plugins/animation/runtime/Cargo.toml
  - zircon_plugins/animation/runtime/src/lib.rs
  - zircon_plugins/animation/runtime/src/runtime_system.rs
  - zircon_plugins/asset_importers/data/runtime/src/lib.rs
  - zircon_plugins/asset_importers/data/runtime/src/plugin.rs
  - zircon_plugins/asset_importers/model/runtime/src/lib.rs
  - zircon_plugins/asset_importers/model/runtime/src/plugin.rs
  - zircon_plugins/asset_importers/shader/runtime/src/lib.rs
  - zircon_plugins/asset_importers/shader/runtime/src/plugin.rs
  - zircon_plugins/gltf_importer/runtime/src/lib.rs
  - zircon_plugins/gltf_importer/runtime/src/plugin.rs
  - zircon_plugins/obj_importer/runtime/src/lib.rs
  - zircon_plugins/obj_importer/runtime/src/plugin.rs
  - zircon_plugins/texture_importer/runtime/src/lib.rs
  - zircon_plugins/texture_importer/runtime/src/plugin.rs
  - zircon_plugins/audio_importer/runtime/src/lib.rs
  - zircon_plugins/audio_importer/runtime/src/plugin.rs
  - zircon_plugins/opus_importer/runtime/src/lib.rs
  - zircon_plugins/opus_importer/runtime/src/plugin.rs
  - zircon_plugins/shader_wgsl_importer/runtime/src/lib.rs
  - zircon_plugins/shader_wgsl_importer/runtime/src/plugin.rs
  - zircon_plugins/ui_document_importer/runtime/src/lib.rs
  - zircon_plugins/ui_document_importer/runtime/src/plugin.rs
  - plugin_structure_audits/registration.py
  - zircon_runtime/src/builtin/runtime_modules/ids/plugin_id.rs
  - zircon_runtime/src/builtin/runtime_modules/plugin_modules/loader.rs
plan_sources:
  - docs/plans/zircon_plugins/12-plugin-dx-and-structure-framework.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
tests:
  - python audit_plugin_structure.py --json: sample_conformance_status=sample-clean, sample_expected_count=1, migration_debt_count=35 on 2026-06-22
  - python -m py_compile audit_plugin_structure.py plugin_structure_audits/__init__.py plugin_structure_audits/manifest_schema.py plugin_structure_audits/skeleton.py: passed 2026-06-22
  - rustfmt --edition 2021 --config skip_children=true --check zircon_plugins/plugin_sdk_examples/editor/src/*.rs zircon_plugins/first_party_runtime_catalog/src/lib.rs: passed 2026-06-22
  - cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_sdk_examples_editor -p zircon_first_party_runtime_catalog --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-skeleton-m2-0622 --message-format short --color never: passed 2026-06-22 with existing warning noise
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_sdk_examples_editor --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-skeleton-m2-0622 --message-format short --color never -- --test-threads=1: timed out after 1200s on 2026-06-22, not counted as passing
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_first_party_runtime_catalog plugins_12_crate_skeleton_conformance --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-skeleton-m2-0622 --message-format short --color never -- --test-threads=1 --nocapture: timed out after 900s on 2026-06-22, not counted as passing
  - cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_sdk --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-test-runtime-m2-0622 --message-format short --color never: passed 2026-06-22 with existing zircon_runtime warnings
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_sdk --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-test-runtime-m2-0622 --message-format short --color never test_runtime_builder -- --test-threads=1 --nocapture: 2 passed, 0 failed on 2026-06-22
  - cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_sdk -p zircon_plugin_animation_runtime --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-registration-m3-0622 --message-format short --color never: passed 2026-06-22
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_sdk --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-registration-m3-0622 --message-format short --color never runtime_registration_builder -- --test-threads=1 --nocapture: 1 passed, 0 failed on 2026-06-22
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_animation_runtime --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-registration-m3-0622 --message-format short --color never animation_registration_contributes_runtime_module -- --test-threads=1 --nocapture: 1 passed, 0 failed on 2026-06-22
  - python audit_plugin_structure.py --json: registration_conformance.m3_t1_gate_status=family-single-entry-clean, asset_importer_family_free_function_registration_sites=0 on 2026-06-23
  - python -m py_compile audit_plugin_structure.py plugin_structure_audits/__init__.py plugin_structure_audits/manifest_schema.py plugin_structure_audits/skeleton.py plugin_structure_audits/registration.py: passed 2026-06-23
  - cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_asset_importer_data_runtime -p zircon_plugin_asset_importer_model_runtime -p zircon_plugin_asset_importer_shader_runtime --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-importer-m3-0622 --message-format short --color never: passed 2026-06-23 with existing zircon_runtime warnings
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_asset_importer_model_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-importer-m3-0622 --message-format short --color never registration_contributes_stl_ply_and_dxf_importers -- --test-threads=1 --nocapture: blocked 2026-06-22 by unrelated zircon_runtime MaterialCaptureSeed / MaterialRuntime::capture_seed lib-test drift
  - rustfmt --edition 2021 --check split importer lib/plugin files plus zircon_runtime builtin plugin id/loader: passed 2026-06-23
  - python audit_plugin_structure.py --json: registration_conformance.m3_split_importer_gate_status=split-importer-single-entry-clean, split_importer_free_function_registration_sites=0, split_importer_registration_owner_files=0, m3_importer_gate_status=importer-single-entry-clean on 2026-06-23
  - cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_gltf_importer_runtime -p zircon_plugin_obj_importer_runtime -p zircon_plugin_texture_importer_runtime -p zircon_plugin_audio_importer_runtime -p zircon_plugin_opus_importer_runtime -p zircon_plugin_shader_wgsl_importer_runtime -p zircon_plugin_ui_document_importer_runtime --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-split-importer-m3-0623 --message-format short --color never: passed 2026-06-23 with existing zircon_runtime warnings
doc_type: module-detail
status: in_progress
---

# 插件 Crate 骨架（Plugin Crate Skeleton）

> 唯一插件 crate 目录骨架，由 [Plugins 12](../plans/zircon_plugins/12-plugin-dx-and-structure-framework.md) 落地、[引擎结构规范 §6.1](../plans/engine-code-structure-convention.md) 定义。新插件 day 1 即用此骨架；存量插件 touch-it-conform-it 迁入。
>
> 状态：in_progress（骨架定稿；Plugins 12 M2/T1 `plugin_sdk` builder baseline、M2/T2 首个样例/符合度 guard、M2/T3 native SDK helper、M2/T4 editor authoring macro/workspace dependency inheritance、M2/T5 test runtime fixture、M3/T2 runtime registration builder + animation 代表迁移、M3/T1 `asset_importers/*` family 与 split importers 自由注册函数清零已落地；capability 单源 / 存量插件迁移债仍 pending）

## 1. 目录骨架

```
<plugin>/
  plugin.toml          # 包级 manifest（统一 schema，强制，见 plugin-manifest-schema.md）
  runtime/
    Cargo.toml
    src/
      lib.rs             # 薄：pub use 公共 API + 导出 Plugin struct + 常量
      plugin.rs          # 唯一注册 owner：impl RuntimePlugin + descriptor()
      capability.rs      # capability id pub const —— 单一来源
      contract/          # 该插件 ABI-safe DTO（纯消费 interface 则省略）
      backend/           # 实际算法 / importer / 协议实现 owner（按结构规范 §1 拆叶子）
      systems/           # 注册进调度图的 ECS 系统
      tests/             # folder-backed
  editor/                # 镜像同骨架（能力对称）
    Cargo.toml
    src/{lib.rs, plugin.rs, capability.rs, ...}
```

## 2. 各文件职责

| 文件 | 职责 | 约束 |
|---|---|---|
| 根 `plugin.toml` | 包级 manifest，列出所有 runtime/editor/native/vm modules | 与 `package_manifest()` / SDK builder 投影一致 |
| module `Cargo.toml` | 单个 runtime/editor/native/vm crate 声明 | 优先使用 workspace 统一依赖（M2/T4 后强制） |
| `lib.rs` | 薄 façade：导出公共 API + Plugin struct + 常量 | 零行为（规范 R1.1） |
| `plugin.rs` | **唯一注册入口** `impl RuntimePlugin::register` + `descriptor()` | 自由函数注册收编于此（规范 §6.3） |
| `capability.rs` | capability id `pub const` | 单一来源，与 `plugin.toml` 一致（§6.4） |
| `contract/` | 该插件 ABI-safe DTO | 纯消费 interface 时省略 |
| `backend/` | 算法 / importer / 协议实现 | 按 owner 叶子拆，软 800 / 硬 1000 行 |
| `systems/` | ECS 系统 | 与 `plugin.toml` `system_anchors` 核对 |

## 3. 导入器类插件

`backend/` 即 importer 实现；`plugin.rs` 的 `register` 同时 `register_module` + 注册 importer descriptor。M3/T1 已把 `asset_importers/{data,model,shader}` 和 root-level split importers 收编到 `RuntimePlugin` trait 入口并删除/避免 `registration.rs` 自由函数分离写法；`asset_importers/audio` 与 `asset_importers/texture` 仍是 declaration-only 迁移债，但不再是公开注册自由函数 owner。

## 4. `plugin_sdk` builder（祝福路径）

`zircon_plugins/plugin_sdk/` 提供 builder API，使新插件以一文件声明 manifest module、capability、target modes 与 runtime/editor descriptor 投影，降低样板。当前 M2/T1 baseline 已提供：

- `PluginManifestBuilder`：填充 `sdk_api_version`、默认平台与默认 packaging，并产出 runtime-owned `PluginPackageManifest`。
- `PluginModuleBuilder`：标准化 runtime/editor/native/vm module 声明；editor module 默认 `EditorHost`。
- `RuntimePluginDeclaration`：从同一声明投影 `RuntimePluginDescriptor` 与 `PluginPackageManifest`。

M2/T3 native ABI helper 已提供 `plugin_sdk::native` feature、ABI v3 类型、SDK-owned byte buffers、entry export macros，并让 `native_dynamic_fixture` 改为使用 SDK native helper。M2/T4 editor authoring macro 已提供 `plugin_sdk::editor::EditorPluginDeclaration` 与 `authoring_plugin!`，首个 editor 样例用宏生成主 plugin 样板，`zircon_plugins/Cargo.toml` 提供 `[workspace.dependencies]`，样例 editor crate 和 native fixture 已改用 workspace dependency inheritance。M2/T5 test runtime fixture 已提供 `plugin_sdk::test::TestRuntime::builder()`，把跨插件测试常见的 foundation/asset/scene 基础模块、runtime extension merge、world extension install、插件 module 激活和固定步长 tick helper 收进 SDK。

M3/T2 runtime registration builder 已提供 `plugin_sdk::registration::RuntimePluginRegistrationBuilder` 与 `RuntimePluginModuleRegistration`。runtime 插件在 `impl RuntimePlugin::register(...)` 内先声明 module，再通过 module handle 声明 runtime scene system、set、order 和 before/after constraint，SDK 内部负责 owner token 顺序。`zircon_plugins/animation/runtime` 已作为代表插件迁到该路径。

M3/T1 importer registration slices 已新增并扩展 `plugin_structure_audits::registration`，`audit_plugin_structure.py --json` 当前报告 `registration_conformance.m3_t1_gate_status = family-single-entry-clean`、`registration_conformance.m3_split_importer_gate_status = split-importer-single-entry-clean`、`m3_importer_gate_status = importer-single-entry-clean`，且 importer free-function registration sites 为 0。`asset_importers/{data,model,shader}/runtime/src/plugin.rs` 和 root-level split importer `runtime/src/plugin.rs` 现在拥有 trait-backed plugin entry；因为 `RuntimePluginId` 仍是 core 封闭 enum，本轮仅补 data/model/shader 和 opus importer 临时枚举接线，D6 string-newtype 仍是 M5 工作。

## 5. 首个骨架样例

`zircon_plugins/plugin_sdk_examples/editor` 是 M2/T2 的首个 skeleton-conformance 样例：

- `src/lib.rs` 只保留模块声明和精选 `pub use`，不承载扩展注册行为。
- `src/capability.rs` 是 editor capability 常量单源。
- `src/plugin.rs` 拥有 `EditorPlugin` 实现、`authoring_plugin!` 主插件声明、`package_manifest()` 投影和 registration report 构造。
- `src/extensions.rs` 拥有 window、asset importer、asset inspector、UI template、component drawer 等 editor extension 注册。
- `src/extension_ids.rs` 拥有 view/importer/template/component id 常量。
- `src/tests.rs` 验证插件注册贡献和 manifest metadata。

## 6. 符合度 guard（Plugins 12 M2）

- `audit_plugin_structure.py --json` 输出 `skeleton_conformance` 与 `plugin_skeleton_gate`。
- M2/T2 样例门禁字段：`sample_conformance_status = sample-clean`、`sample_expected_count = 1`、`sample_violation_count = 0`。
- M2/T4 样例 workspace dependency 门禁字段：`sample_workspace_dependency_status = sample-workspace-deps-clean`、`sample_workspace_dependency_violation_count = 0`。
- `plugins_12_crate_skeleton_conformance` 消费同一 JSON，锁定首个样例不回退。
- `registration_conformance.m3_t1_gate_status = family-single-entry-clean` 锁定 `asset_importers/*` 家族不再出现公开 `pub fn register(...)` 自由函数或 `runtime/src/registration.rs` owner。
- `registration_conformance.m3_split_importer_gate_status = split-importer-single-entry-clean` 和 `m3_importer_gate_status = importer-single-entry-clean` 锁定 split importers 与 aggregate importer 口径不再出现公开注册自由函数或 `runtime/src/registration.rs` owner。
- 存量插件仍按 `migration_debt_roots` 记录为迁移债；当前目标不是一次性硬切所有插件，而是在 M5 touch-it-conform-it 中递减到 0。
- `native_dynamic_fixture` 作为 native-only ABI fixture 继续豁免 runtime/editor 骨架规则；M2/T3 已收编其 ABI 样板到 `plugin_sdk::native`，但它仍不是 runtime/editor 双 crate 骨架样例。
