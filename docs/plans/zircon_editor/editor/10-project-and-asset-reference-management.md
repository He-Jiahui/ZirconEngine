---
related_code:
  - zircon_runtime_interface/src/project
  - zircon_runtime_interface/src/project/template_pack
  - zircon_runtime_interface/src/serialization
  - zircon_runtime/src/asset/project/manifest.rs
  - zircon_runtime/src/asset/project/manifest
  - zircon_runtime/src/asset/project/paths.rs
  - zircon_runtime/src/asset/project/manager/open.rs
  - zircon_runtime/src/asset/project/manager/registry_access.rs
  - zircon_runtime/src/asset/project/manager/scan_and_import/sources.rs
  - zircon_runtime/src/asset/project/package_asset_registry.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/runtime.rs
  - zircon_editor/src/ui/host/project_access.rs
  - zircon_editor/src/ui/host/editor_error.rs
  - zircon_editor/src/core/project
  - zircon_editor/src/ui/host/startup
  - zircon_editor/src/ui/retained_host/app/helpers/animation_assets.rs
  - zircon_hub/src/projects/validation.rs
  - zircon_hub/src/projects/create_project.rs
  - zircon_hub/src/projects/recent_project.rs
  - zircon_hub/src/projects/editor_recent_sync.rs
  - templates/projects/renderable-empty
reference_sources:
  - dev/godot/core/io/resource_uid.h
  - dev/godot/editor/editor_file_system.h
  - dev/UnrealEngine/Engine/Source/Runtime/AssetRegistry/Public/AssetRegistry/IAssetRegistry.h
  - dev/UnrealEngine/Engine/Source/Runtime/CoreUObject/Public/UObject/SoftObjectPath.h
  - dev/bevy/crates/bevy_asset/src/path.rs
  - dev/bevy/crates/bevy_asset/src/id.rs
plan_sources:
  - docs/plans/zircon_editor/editor/00-editor-architecture-overview.md
  - docs/plans/zircon_editor/editor/09-editor-asset-management.md
  - docs/plans/zircon_editor/editor/11-serialization-and-versioning.md
  - docs/plans/zircon_editor/editor/16-cli-args-and-hub-integration.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
implementation_files:
  - zircon_runtime_interface/src/project/manifest_summary/parse.rs
  - zircon_runtime_interface/src/project/manifest_summary/migration.rs
  - zircon_runtime_interface/src/project/rel_path/value.rs
  - zircon_runtime_interface/src/project/template_pack/render.rs
  - zircon_runtime_interface/src/project/template_pack/embedded.rs
  - zircon_runtime/src/asset/project/manifest/project_manifest.rs
  - zircon_runtime/src/asset/project/manifest/load.rs
  - zircon_runtime/src/asset/project/manifest/save.rs
  - zircon_runtime/src/asset/project/package_asset_registry.rs
  - zircon_runtime/src/asset/project/manager/open.rs
  - zircon_runtime/src/asset/project/manager/source_path_for_uri.rs
  - zircon_runtime/src/asset/project/manager/source_uri_for_path.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/runtime.rs
  - zircon_editor/src/ui/host/project_access.rs
  - zircon_editor/src/ui/host/editor_error.rs
  - zircon_editor/src/core/project/authority.rs
  - zircon_editor/src/core/project/filesystem.rs
  - zircon_editor/src/core/project/new_project_draft.rs
  - zircon_editor/src/core/project/recent_project_entry.rs
  - zircon_hub/src/projects/validation.rs
  - zircon_hub/src/projects/create_project.rs
  - zircon_hub/src/projects/recent_project.rs
  - zircon_hub/src/projects/editor_recent_sync.rs
tests:
  - zircon_runtime_interface/src/project/tests/manifest_summary.rs
  - zircon_runtime_interface/src/project/tests/template_pack.rs
  - zircon_runtime/src/asset/tests/project/manifest.rs
  - zircon_runtime/src/asset/tests/project/package_assets.rs
  - zircon_runtime/src/asset/tests/pipeline/manager/watcher.rs
  - zircon_editor/src/ui/retained_host/app/helpers/animation_assets/tests.rs
  - zircon_editor/src/ui/host/editor_error.rs
  - zircon_editor/src/core/project/tests/template_creation.rs
  - zircon_editor/src/core/project/tests/recent_projects.rs
  - zircon_editor/src/core/project/tests/boundary.rs
  - zircon_hub/src/projects/validation.rs
  - tests/fixtures/serialization/project-manifest
doc_type: implementation-plan
status: in_progress
---

# 10 文件工程与资产引用管理

本计划落地 00 §6 的「工程身份」权威 `ProjectAuthority` 与引用图。

- fixed 已修复：[source-cubemap-source-texel-test-api-drift](10/fixed-2026-07-12-source-cubemap-source-texel-test-api-drift.md)
- fixed 已修复：[hub-message-legacy-test-drift](10/fixed-2026-08-27-hub-message-legacy-test-drift.md)
- fixed 已修复：[editor-libtest-link-disk-space](10/fixed-2026-07-16-editor-libtest-link-disk-space.md)
- 失败交接（`open / 待 Editor 16 M2 统一命令框架接线`）：[`16/failure-2026-07-11-migrate-assets-commandlet-registry.md`](16/failure-2026-07-11-migrate-assets-commandlet-registry.md)
- fixed 已修复：[editor10-runtime-layout-method-name](../../zircon_runtime/shader/06/fixed-2026-07-11-editor10-runtime-layout-method-name.md)

## 参照证据（dev/）

**godot 双轨 id**（`resource_uid.h:71-79`）：`create_id()/id_to_text(ID)/get_id_path(ID)`——uid 是一层间接：序列化存 uid，加载查表得路径，挪动只改表。`FileInfo.uid` 入编辑器索引。

**UE 离线索引六签名**（`IAssetRegistry.h:313-599`）：`GetAssetsByClass / GetAssets(FARFilter) / GetDependencies(id|package) / GetReferencers(id|package)`——**不加载资产**即可按类查/过滤查/正反依赖查；cook、删除检查、修复引用全靠它。

**UE 软引用**（`SoftObjectPath.h:48-148`）：`{ FTopLevelAssetPath, SubPathString }` 两段式——**子对象寻址是引用类型一等公民**。

**bevy 双轨**（`id.rs:29-44`、`path.rs:57-61`）：`AssetId::{Index, Uuid}` 运行时/稳定双态；`AssetPath { source, path, label }` 源+路径+子资产标签三段。

## 现状与证据（zircon，2026-07-05 实读；v2 两大论断修正）

### 修正一：工程 manifest schema 已存在且相当丰富（「无共同 schema」作废）

`ProjectManifest`（`asset/project/manifest.rs:14-33` 全文实读）：

```rust
pub struct ProjectManifest {
    pub name: String,
    pub format_version: u32,                      // PROJECT_FORMAT_VERSION = 1，serde default
    pub default_scene: AssetUri,
    pub asset_manifest: Option<String>,
    pub library_version: u32,                     // serde(alias = "schema_version")
    pub plugins: ProjectPluginManifest,           // 12 的工程插件启用表已在
    pub scripts: ProjectScriptManifest,           // 13 的脚本清单已在
    pub export_profiles: Vec<ExportProfile>,      // 15 的导出档已在
}
// load/save（:49-55）；文件=<root>/zircon-project.toml（paths.rs:21）
// 入口=ProjectAssetManager::open(root)（manager/open.rs:12）
```

**三方消费现状**：runtime 全量消费（open/scan）；编辑器 CLI `--project` 传 root 后由 runtime 解析（间接消费）；hub 有 `CheckProjectManifest` 消息（`hub_message/project.rs:31`）与 `.zircon` 封面目录约定（`projects/cover.rs:44`）但**只探存在性不读内容**——hub 是独立 workspace 不链接 `zircon_runtime`，无法复用解析器。真实缺口=**hub 可消费的 manifest 摘要 DTO**（interface 层）+ 编辑器 CreateProject 模板语义 + manifest 缺失字段（engine_version_req/asset_roots/settings 指针）。

### 修正二：uuid 已在铸造（「AssetUuid 两处孤岛」作废）

`project/manager/` 实测：`load_or_create_meta.rs`（**扫描时逐资产铸 uuid 写 sidecar**）、`scan_and_import.rs`、`asset_lookup.rs`（uuid 查询）；`AssetMetaEntry { uuid: AssetUuid, url: AssetUri, asset_kind }`（09 已核）；`PackageAssetRegistry { register_root / register_manifest_roots / root_for_package }`（package→根路径映射）。

**真实缺口收窄为**：uuid 铸造与查询在，但 (a) **序列化引用仍以路径为主**——`scene/world/project_io/references.rs` 的 `reference.uuid` 是少数派，无 `AssetRef` 统一引用类型；(b) 无依赖图（dependencies/referencers 查询全缺）；(c) 无 redirector，rename/move 断引用；(d) 删除无引用检查。

## 目标

1. **manifest 扩展 + 三方统一**（**不新建 ZirconProjectManifest，扩既有 `ProjectManifest`**）：
   - 新字段：`engine_version_req: Option<String>`（semver，hub 安装匹配）、`asset_roots: Vec<RelPath>`（默认 `["assets"]`，与 `PackageAssetRegistry` 多根对接）、`settings: Option<RelPath>`（17 工程设置指针）——`format_version` 升 2，11 迁移链一步。
   - `zircon_runtime_interface/src/project/` 新增 `ProjectManifestSummary` DTO（name/engine_version_req/default_scene 文本/最近打开所需摘要）+ 独立轻量解析（hub 消费；runtime 侧保证 `ProjectManifest` serialize 与 Summary 解析同源测试）。
   - 编辑器 `CreateProject(NewProjectDraft)` 模板语义定稿=模板目录复制 + manifest 改写；`.zircon/` 派生物目录约定（cache/registry/autosave/play/thumbnails，全部可再生，VCS 忽略）。
2. **`AssetGuid` 引用落地**（既有 `AssetUuid` 直接沿用为类型本体，改名不动语义）：序列化引用统一 `AssetRef { guid: AssetUuid, path_hint: RelPath, sub: Option<String> }`（UE SubPathString/bevy label 子资产位）；解析顺序 guid 表 → path_hint 回退（回写待修复）→ 悬挂报告。
3. **`AssetRegistryIndex`**：条目 `{ uuid, path, type_marker, tags, dependencies: Vec<AssetUuid>, source_digest }`；依赖由 importer 声明提取器产出；查询面直译 UE 六签名；持久化 `.zircon/registry/`（损坏全量重建）；watch 增量维护（09 同泵）；`asset_lookup/PackageAssetRegistry` 既有能力收编为其查询底座。
4. **redirector 与 rename/move**：编辑器 rename/move → registry 记 `Redirect` 并更新 uuid 表；guid 引用天然免疫，redirector 只服务 path_hint 与外部文本引用；fix-up 批处理命令（08）重写后清 redirector；删除前 `referencers` 非空→阻断对话。
5. **悬挂治理**：加载期悬挂→事件 + 诊断面板数据源（17）；保存前校验钩子供 09。

## 非目标

- VCS 集成；pak 内 guid 解析表容器（15 引用本计划类型）；`ResourceHandle` 运行时形态不改（guid 是持久层，加载后折算 ResourceId）。

## 架构设计

### 归属

```
zircon_runtime_interface/src/project/     # ProjectManifestSummary + AssetRef DTO（hub/ABI 消费）
zircon_runtime/src/asset/project/         # manifest 扩字段 + 既有 manager 保持 owner
zircon_runtime/src/asset/registry/        # 新：AssetRegistryIndex + 依赖提取器 + uuid 表
                                          # （asset_lookup/PackageAssetRegistry 迁入或被包装，执行时按耦合度裁决记状态节）
zircon_editor/src/core/project/           # ProjectAuthority：打开/创建/模板/最近记录回写（16 协同）
zircon_editor/src/core/asset/refactor.rs  # rename/move/fix-up/删除检查（命令+事务）
zircon_hub/src/projects/                  # RecentProject 改存 Summary 摘要
```

registry 住 runtime 的理由：cook（15 headless）与世界载入都要 guid 解析，编辑器经 gateway 查询；日后独立工具链需要时按 frameworks 计划升 crate。

### 引用解析数据流

```
AssetRef{guid, path_hint, sub}
  → registry uuid 表命中 → 加载
  → 未命中 → path_hint 直查（跟随 redirector）→ 命中回写 uuid 表 + 记「待修复」
  → 仍未命中 → DanglingRef 事件（面板/保存检查消费）
```

### 迁移策略（既有资产已多有 uuid，迁移量比 v2 预估小）

`--run migrate-assets` commandlet（16）：(a) 无 sidecar 资产补铸（`load_or_create_meta` 既有逻辑复用——扫描即铸造，多数工程已覆盖）；(b) 场景/资产内**路径引用改写为 `AssetRef`**（主工作量；11 迁移链一步，`references.rs` 既有 `reference.uuid` 消费点收编）；(c) 幂等（已为 AssetRef 者跳过）；(d) `.meta.toml` 后缀退役并写 `.zmeta`（09 会签项）。

### 深度测试

夹具工程（三资产互引+一场景）：rename→引用不断；delete 被引用→阻断列 referencer；registry 删库重建==增量态；hub Summary 解析 == runtime 全量解析的投影（同源断言）。

## 里程碑

### M1 manifest 扩展与三方统一

- 切片 1.1：manifest 三新字段 + `format_version=2` 迁移步；`ProjectManifestSummary` DTO + 同源测试；`asset_roots` 接 `PackageAssetRegistry`。
- 切片 1.2：编辑器 `ProjectAuthority`（打开/创建含模板复制）；hub `RecentProject` 改存 Summary；`.zircon/` 目录约定文档化。
- 测试阶段：`cargo test -p zircon_runtime_interface --locked`（Summary 往返/坏文件形状）+ `cargo test -p zircon_runtime --lib --locked`（manifest v1→v2 迁移 + 既有 open/scan 不回归）+ hub 解析单测；夹具工程三方等价断言。更新 `docs/engine-architecture/project-manifest.md`（新建）。

### M2 AssetRef 与 registry

- 切片 2.1：`AssetRef` DTO；铸造点确认（`load_or_create_meta` 既有 + 创建模板）；sidecar `source_digest` 字段（09 协同）。
- 切片 2.2：`asset/registry/`：uuid 表（asset_lookup 收编）+ 依赖提取器（首批 scene/material/model 手写）+ watch 增量 + 全量重建 + 六签名查询。
- 切片 2.3：`migrate-assets` commandlet + 序列化引用切 `AssetRef`（11 迁移步）。
- 测试阶段：`cargo test -p zircon_runtime --lib --locked`（registry 增量一致性/重建等价/解析回退三级）+ 迁移幂等（跑两遍产物一致）+ 旧样本场景 加载→迁移→保存→再加载 等价。

### M3 refactor 闭环

- 切片 3.1：rename/move（03 事务化）+ redirector 记录/跟随/清理 + fix-up 命令。
- 切片 3.2：删除阻断对话数据源；悬挂事件与面板数据源；09 保存前校验接通。
- 测试阶段：rename→加载→fix-up→清 redirector 全链；强制删除→悬挂事件；证据记状态节。

## 风险与开放问题

- 文本资产手工复制导致 guid 撞车：registry 扫描期检测重复，后见者重铸+警告（godot 同款），测试覆盖。
- 依赖提取器的反射化通用版待 runtime/13 反射能力，首批三类手写先行。
- `library_version`（带 `schema_version` alias）与新 `format_version=2` 的语义区分：`format_version`=manifest 结构版本、`library_version`=资产库内容版本——迁移步文档里显式写明，避免双版本混淆。
- `engine_version_req` 不满足时 hub 阻止还是警告：hub 侧裁决，本计划只落字段。

## 产出记录与时间

> 请将产出记录放置在子计划中，此处仅展示当前现状的概述

- fixed 已修复：[runtime-module-structure-cfg-fence](10/fixed-2026-07-11-runtime-module-structure-cfg-fence.md)
- 当前失败交接（`open / 待修复`）：[`10/failure-2026-07-12-project-asset-reference-full-gate-regressions.md`](10/failure-2026-07-12-project-asset-reference-full-gate-regressions.md)
- fixed 已修复：[hybrid-gi-project-fixture-api-drift](../../zircon_runtime/render/18/fixed-2026-07-13-hybrid-gi-project-fixture-api-drift.md)

> M2.1 全包 exit 1 的最低 owner 已定位：7553 项现有 Runtime binary 中，`animation::` 24/24、`builtin::` 14/15；唯一失败是 Frameworks03 的 runtime module 结构守卫未识别复合 `cfg(all(test, feature = "graphics"))` fence。Plan10 聚焦 9/9 仍通过，但完整 Runtime 门须待上述 failure 修复后复跑。

| 里程碑 | 切片 | 状态 | 完成日期 | 完成项目与证据（命令输出 / 文件 / 测试名） |
|---|---|---|---|---|
| M1 | 1.1 manifest v2、共享 Summary 与多资产根权威 | `实现完成-接口门通过-Runtime与Hub被外部计划阻断` | 2026-07-11 | `ProjectManifest` 只写 v2，v1 通过 interface `MigrationChain` 值域迁移且不保留 v1 DTO；新增 `engine_version_req`（`semver::VersionReq`）、`asset_roots: Vec<RelPath>`、`settings`，Hub/interface/runtime 共用 `tests/fixtures/serialization/project-manifest/{v1,v2,future,invalid}`。`ProjectPaths` 固定 `assets` 字段、`assets_root()` 与零参数 `ensure_layout()` 已硬删除，全仓调用清零；Runtime/Editor/App 的读取、扫描、监听与写入切到 manifest roots，读取要求唯一命中，新写入显式选择 primary root，第二根模型动画派生保持同根。canonical project/root containment 拒绝符号链接或目录联接逃逸并保留 typed source；Editor 路径 API 保留 `AssetImportError`/URI/Core/SceneProject 错误链。新增双根扫描/重复 URI/链接逃逸/第二根 watcher/第二根动画派生回归。规格复审 `SPEC APPROVED`，质量复审修复四项后 `QUALITY APPROVED`；scoped `rustfmt --check`、`git diff --check` 与 retired API 扫描通过。受管 `cargo build/test -p zircon_runtime_interface --locked` 最终通过（212/212 + doc-tests）；首轮 boundary gate 因新增 `semver` 未登记失败，已在当前计划修复并复跑全绿。Runtime production build已完成，但 lib-test 在 Render 11 `source_cubemap/tests.rs:278,282` 的 E0599×2 前置编译失败，已交接 `../../zircon_runtime/render/11/failure-2026-07-11-source-cubemap-source-texel-test-api-drift.md`；Hub build 通过，但 integration test 在 Hub 07 `project_management_contract.rs:201` 的退役 `HubMessage::legacy` E0599 前置失败，已交接 `../../zircon_hub/07/failure-2026-07-11-hub-message-legacy-test-drift.md`。两侧 Plan10 行为断言均未执行，故 M1 测试阶段保持未关闭；M1.2 已完成并通过规格/质量复审，等待统一测试；M2、M3 未开始。模块文档：`docs/engine-architecture/project-manifest.md`、`docs/zircon_runtime/asset/project-manifest.md`、`docs/zircon_runtime_interface/project.md`。 |
| M1 | 1.2 ProjectAuthority、共享模板包、Summary recent 与 `.zircon` 派生根硬切 | `实现完成-接口门通过-Hub07与Editor验证空间阻断` | 2026-07-11 | 新增 folder-backed `zircon_editor/src/core/project/`，打开、模板创建、typed staging/rollback、最近工程 Summary/validation/session 均由 `ProjectAuthority` 持有；旧 `ui/workbench/startup` 工程 DTO/持久化实现和 `EditorProjectDocument::create_renderable_template/ensure_runtime_assets` 已删除且无 re-export。`zircon_runtime_interface::project::template_pack` 将唯一版本控制目录 `templates/projects/renderable-empty/` 编译期封装给 Editor/Hub，manifest 通过 typed v2 API 改名，PBR 模板硬切 surface v2 + `zr_material_surface`；pack/source 完整性与 link/reparse guard 已写。Hub `RecentProject` 改存 `ProjectManifestSummary`，创建/导入/Editor sync/Hub config load 刷新 manifest 摘要，merge/roundtrip 测试源码已更新。`ProjectPaths` 派生物硬切 `.zircon/{cache,registry,autosave,play,thumbnails}`，asset artifacts 明确进入 `.zircon/cache/assets`，Runtime/Editor/App/reflection-probe 旧 `library_root()/runtime_cache_root()` 与物理 `library/` 生产调用清零，相关 test owner 从 `library_*` 改名为 `artifact_cache_*`；生产 shader IDE/variant cache 的 `.zircon-cache` 路径已硬切 `.zircon/cache`，`examples/vampire/library/**` 563 个可再生缓存已删除且由通用 VCS ignore 阻止回流。规格复审两轮整改后 `SPEC APPROVED`；质量复审修复共享工程名路径逃逸、typed commit/restore 双失败保留 backup、UI authority probe、Hub typed request error 后 `QUALITY APPROVED`，Editor/Hub 均有 unsafe-name 与事务故障注入测试源码。已执行 scoped rustfmt、template tracked-input/status 与 retired-path 静态扫描。受管 interface build/test 全绿；Hub 首轮修复本切片编译问题后 production build 通过，full test 仅剩 Hub 07 `HubMessage::legacy` E0599，已更新对应 Hub 07 failure；Editor production build 通过，lib-test 链接因 E 盘 27.19 GB 报 LNK1180，已交接 Runtime 01 `../../zircon_runtime/runtime/01/failure-2026-07-11-editor-libtest-link-disk-space.md` 管理受管验证容量与清理权限。两侧行为测试均未完整执行，M1 测试阶段保持未关闭。模块文档：`docs/zircon_editor/core/project.md`、`docs/zircon_hub/projects/project-authority.md`，并同步 manifest/interface/runtime 文档。 |
| M2 | 2.1 `AssetRef` DTO 与 `.zmeta` v7 `source_digest` 硬切 | `实现完成-规格质量复审通过-接口与Runtime聚焦门通过-全包门待归属` | 2026-07-11 | `zircon_runtime_interface/src/project/asset_ref/` 新增 folder-backed `AssetRef { guid, path_hint, sub }`，私有不变量字段配 typed constructor/accessor/custom serde；拒绝空、`#`、控制字符 subpath 与 traversal `RelPath`，且不与 `resource::AssetReference` 建 alias、re-export、自动转换或 URI fallback。`AssetMetaDocument` 只写/读 v7 `source_digest`，`from_toml_str` 以 `RetiredSourceHashField`、`UnsupportedOldFormatVersion`、`UnsupportedFutureFormatVersion` 和 deserialize typed error 严格拒绝旧字段/旧版本/未来版本/未知字段；无 serde alias 或 v6 自动迁移。Runtime/Editor/prewarm 侧 sidecar 调用、Rust 内嵌 sidecar 字符串、55 个 tracked `.zmeta` 与共享模板 sidecar 已机械切到 v7/current key，ResourceRecord/render/cache 的独立 `source_hash` 语义保持不变。测试源码：`project/tests/asset_ref.rs`、`asset/tests/project/zmeta/schema_v7.rs` 与 `asset/project/meta_io.rs` 内故障注入。质量首轮整改新增 exact JSON keys/unknown-key rejection/bincode roundtrip；format_version Missing/NonInteger/Negative/OutOfRange typed 分类且 old/future 优先于 current-v7 retired-field/serde shape；`AssetMetaEntry` nested unknown rejection；sidecar save 改为同目录唯一 staging 的 write_all/flush/sync_all；Windows 用单次 `ReplaceFileW(replacement, backup)`，Unix 用 hard-link/copy backup 后 rename-overwrite，整个 commit 期间 target 始终可见，注入失败仍可读原内容且清理事务文件；commit 后 backup cleanup 为 best-effort；Windows ReplaceFileW 失败若生成 backup 则保留并在同 kind error/source chain 中报告原 OS code 与 backup path，Unix backup sync 失败注入锁定 staging+backup 清理。AssetRef bincode 以固定 79-byte golden 同时锁当前编码与 historical bytes 解码。`bincode` 仅为 interface reviewed dev-dependency，不扩大 production dependency。规格复审整改 Runtime15 旧 typed-error guard 后 `SPEC APPROVED`；质量复审两轮整改原子 sidecar 写入、严格 version 分类/nested unknown、AssetRef exact human/binary golden、Windows ReplaceFileW 失败 backup 保留与 Unix sync-failure 清理后 `QUALITY APPROVED`。已执行 scoped rustfmt、diff/retired-key 静态扫描；首轮受管 interface gate 因 `Cargo.lock` 未登记测试专用 bincode 依赖被 `--locked` 拒绝，已在当前切片修复，复跑 `cargo build/test -p zircon_runtime_interface --locked` 全绿（含 unit/doc-tests）。受管 Runtime 全包门成功生成 1.18 GB lib-test 二进制但最终 exit 1；从同一受管产物精确复验本切片 `asset::project::` 3/3 与 `asset::tests::project::zmeta::schema_v7::` 6/6 全绿，证明 v7 sidecar、typed version 分类和原子写入行为已执行。全包失败尚未取得所属测试名，不能据此关闭整个 Runtime 回归阶段，也不能反向判定 M2.1 行为失败；后续须继续按最低 owner 归档。模块文档同步 `docs/zircon_runtime_interface/project.md`、`docs/zircon_runtime/asset/zmeta-shader-material.md`、`docs/zircon_runtime/asset/importer.md`。 |
| M2 | 2.2 离线 AssetRegistryIndex、依赖图与增量重建 | `实现完成-规格质量复审通过-生产lib类型门通过-行为门待Frameworks03` | 2026-07-11 | 新建 folder-backed `zircon_runtime/src/asset/registry/`，条目持有 uuid/path/type_marker/tags/dependencies/source_digest；旧 `project/manager/asset_lookup.rs`、三张 UUID/path map 与旧调用硬删除且无 wrapper/re-export/fallback。六签名为 type/filter、UUID/path dependencies 与 UUID/path referencers，全部离线查 sidecar 索引；scene/material/model 手写提取器接 importer。多 asset roots 全量重建与 watch Added/Modified/Removed/Renamed 候选态增量提交覆盖 root+subasset/反向边；duplicate GUID 初建按稳定路径首见，增量按提交前 owner 保留原件并只重铸副本、回写 sidecar 与 typed diagnostic。`.zircon/registry/asset-registry.json` 损坏重建，持久化复用 v7 sidecar 同目录唯一 staging、flush/sync、Windows ReplaceFileW/Unix rename-overwrite 事务及 write/sync/replace fault injection；提交失败内存/磁盘/ResourceRegistry 不分叉。扫描拒绝 symlink/junction/reparse、canonical root 逃逸与循环。v7 current sidecar 新增 root/subasset tags 权威，private raw DTO + custom Deserialize 在构造集合前严格拒绝重复、空、首尾空白与控制字符，直接 serde 入口不可绕过；场景引用删除路径 ID fallback，未命中返回 typed `DanglingAssetReference` 并贯穿 project IO。测试源码覆盖六签名、三类依赖、增量==全建、多根、损坏、duplicate owner、Removed/Renamed、事务回滚、tags 与链接逃逸。三轮规格整改后 `SPEC APPROVED`；质量整改原子事务、候选提交、owner事件语义、tags不可绕过、typed source/E2 命名与测试 owner 后 `QUALITY APPROVED`。scoped rustfmt、`git diff --check`、旧 API 扫描通过；受管 target-server package check 中 library 本体成功产出，随后 shader IDE/prewarm bins 因 Frameworks03 未按 profile 门控的 graphics/dynamic-api import exit 101，已追加对应 failure，故新增行为测试尚未执行且 M2 测试阶段不关闭。模块文档：`docs/zircon_runtime/asset/registry.md`。 |
| M2 | 2.3 持久引用硬切与 `migrate-assets` Runtime 迁移能力 | `实现完成-规格质量复审通过-静态门通过-CLI与行为门待下层修复` | 2026-07-12 | 新增 `zircon_runtime::asset::migration` dry-run/apply 业务 API、严格扫描、v6/`.meta.toml`→v7 `.zmeta`、缺 sidecar 事务化铸造、scene/model/material 生产 reader/writer 与唯一旧 `{ uuid, url }` 值迁移器；持久引用统一为 discriminated Project/Builtin，GUID 权威、仅 GUID 缺失时允许 `path_hint` 修复，完整 stale/resolved `AssetRef` 修复通过 importer outcome 回传。跨文件事务在 staging 前持久 intent journal；磁盘日志视为不可信输入，校验 owner/白名单/角色名/状态/digest/link-reparse 后只清理保留产物并由同次 Apply 前向重跑，不以磁盘 backup 覆盖或删除 live target；进程内失败仍走内存回滚。补齐 target replace、retired delete、journal sync、partial rollback、恶意日志、链接逃逸和无-sidecar 铸造崩溃窗口；最终修复 current v7 与 retired v6 并存时的重复 UUID 登记，同时保留退休删除事务，二次 Apply 验证 changed=0 与产物字节幂等。事务、文档与测试 owner 均已拆分至低于 800 行；错误保留 typed source chain，旧本地 CLI/第二命令注册表与兼容入口均未保留。规格复审 `SPEC APPROVED`，质量终审在前向收敛整改后 `QUALITY APPROVED`；全量相关 Rust scoped `rustfmt --check` 与 scoped `git diff --check` 通过。受共享 Cargo/Frameworks03 门禁影响，本切片新增 Runtime 行为测试尚未执行；合法 `--run migrate-assets` 统一命令注册投影、Headless runner 与退出码 0/1/2/3 已作为 open failure 交接 Editor16：`16/failure-2026-07-11-migrate-assets-commandlet-registry.md`，因此 M2 测试阶段不关闭。模块文档：`docs/zircon_runtime/asset/migration.md`、`docs/zircon_runtime_interface/project/retired_asset_ref_migration.md`。 |
| M1/M2 | 当前 Editor 全量门 ProjectAuthority / AssetRef 回归 | `未通过-已归档到本功能计划` | 2026-07-12 | Editor03/08 统一受管 job `520d85713df249afae31661a7697ad07` 完成 test binary 编译后，至少 10 个 project/reference 用例失败，涵盖工程边界、创建/打开、损坏 workspace 回退、preset/文档往返、模板 scaffold 与场景/动画/物理引用追踪。全量随后发生 Editor14 资源停滞，未生成 panic summary；精确复现与修复责任见 [`failure-2026-07-12-project-asset-reference-full-gate-regressions.md`](10/failure-2026-07-12-project-asset-reference-full-gate-regressions.md)。不得在 Editor03/08 恢复旧工程 DTO 或路径 fallback。 |
| M1/M2 | ProjectAuthority generation-bound 当前源码回归收束 | `实现完成-独立复审整改已落地-复审与受管行为门待执行` | 2026-07-18 | `OpenedProject` 持有唯一 prepared `ProjectManager` generation，Runtime 在同一实例执行唯一 source scan，Editor asset/document/watcher/UI external effects 复用快照；locator/preset 正常投影改走 Runtime manager-owned path/URI 查询，不再深 clone 完整 generation。旧 reopen/root helper 硬删除。welcome 创建/打开校验迁入 Background/Index job，snapshot 只读缓存，并加入迟到 generation、取消、job/submit failure 行为回归；preset typed source chain 保留。详见 [`10/2026-07-17-project-authority-reference-regression-closeout.md`](10/2026-07-17-project-authority-reference-regression-closeout.md)。两个 failure 在当前源码 exact Cargo 与独立复审完成前保持 open。 |

## 性能审阅交接

- 2026-07-22 migrate-assets性能交接：Editor10的`migrate-assets`产品命令消费Runtime04 PERF-MVP-511/512唯一inventory/transaction ticket，并展示scanned/changed/committed、bytes、queue age与cancel进度；不得在Editor重扫目录或复制完整报告/文档。1/1k/100k文件dry-run/apply保持UI线程filesystem/parse/hash/write=0，取消只发生在安全阶段，现有CLI exit/report与崩溃恢复语义不变。
- 2026-07-22 project document性能交接：PERF-MVP-525已把material/model/scene load parse 2→1、save post-parse 1→0并删除TOML String中间态；Editor10按527只消费Runtime04 sealed typed generation与Runtime11 serialize/write ticket，UI线程不得物化完整Value/authoring/pretty树。stable save=0，changed serialize≤1/generation，取消与atomic publish仍服从本计划事务合同。
- 2026-07-30 Editor project当前源性能复核：`core/project/**` 18/18确认`OpenedProject/CreatedProject`已携带prepared `ProjectManager`，Runtime activation与Editor document/locator复用同一generation，PERF-MVP-075旧的三次manager reopen结论已过时。剩余产品重复是Welcome后台probe后点击再probe+open、startup validate last后再open，以及save重新canonical resolve并同步serialize/import/catalog/watcher；Editor10按075以canonical identity+manifest fingerprint ticket复用已验generation，Runtime04/11负责promotion/dirty I/O，不能恢复第二manager或弱化link/reparse检查。
- 2026-07-30 session/template性能交接：raw legacy recent migration会在typed decode与8项裁剪前按外部数组长度分配并逐项probe，Editor10按PERF-MVP-100先做bytes/schema/entries hard cap与dedup，再允许background validation。PERF-MVP-568仍确认template clone全部embedded bytes、逐entry mkdir/write、Editor忽略rendered summary并post-write load+save；消费Runtime04唯一manifest artifact和Runtime11 transaction ticket，非manifest bytes borrowed/shared、unique parent只建一次，保留现有staging/backup/rollback测试合同。
- 2026-07-30 host startup实链补充：显式project open后`remember_opened_project` load/decode并请求后台持久化，紧接`recent_projects_snapshot`再次load/decode且对最多8项逐个canonical+manifest validation；刚打开project也被再读manifest。restore路径validate last后仍open/parse，save仍同步serialize/reimport/catalog/watcher。Editor10继续按PERF-MVP-075/100发布pre-cap、generation-keyed recent validation/open promotion ticket；Config persistence已有worker，不得另建recent私有线程池。证据见`../../performance/01/2026-07-30-editor-ui-host-startup-project-current-review.md`。
- 2026-07-30 Welcome project probe补充：当前源码已有50ms trailing debounce、250ms max latency、一个pending+一个active generation、same-draft复用和I/O阶段取消，1K/1M burst测试源码存在，PERF-MVP-559由“待实现”改为“实现存在、动态门待验”。Editor10联动Editor14只补受管测试、queue entry/bytes/age、filesystem-call与F0 trace，并把成功后台ticket接给075点击promotion；不得新增第二debounce/cache。原failure在动态证据完成前保持open：[welcome-project-probe-admission-storm](10/failure-2026-07-22-welcome-project-probe-admission-storm.md)。
- 2026-07-22 Assets Activity虚拟化性能交接：`zircon_editor/src/tests/ui/assets_activity`现有5行合同显式要求视口以下行仍进入scroll-source geometry，只锁定scroll extent而未证明viewport materialization。Editor10联动EditorUI06按PERF-MVP-109/177/219把数据总量与visible+overscan投影分离；补1/1k/10k assets滚动测试，要求offscreen presentation/render rows=0、stable generation rebuild=0、每帧materialized rows≤visible+overscan且scroll p95受预算，同时保留总scroll extent、键盘导航、选择与drop target语义。
