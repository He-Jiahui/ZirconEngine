---
related_code:
  - zircon_runtime/src/asset/project/manifest.rs
  - zircon_runtime/src/asset/project/paths.rs
  - zircon_runtime/src/asset/project/meta.rs
  - zircon_runtime/src/asset/project/manager/open.rs
  - zircon_runtime/src/asset/project/manager/load_or_create_meta.rs
  - zircon_runtime/src/asset/project/manager/scan_and_import.rs
  - zircon_runtime/src/asset/project/package_asset_registry.rs
  - zircon_runtime/src/scene/world/project_io/references.rs
  - zircon_editor/src/core/gui_startup_request.rs
  - zircon_hub/src/settings/hub_config.rs
  - zircon_hub/src/state/hub_message/project.rs
  - zircon_runtime_interface/src/resource/resource_handle.rs
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
status: planned
---

# 10 文件工程与资产引用管理

本计划落地 00 §6 的「工程身份」权威 `ProjectAuthority` 与引用图。

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
