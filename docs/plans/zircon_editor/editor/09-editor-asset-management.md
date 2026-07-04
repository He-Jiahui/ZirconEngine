---
related_code:
  - zircon_editor/src/ui/layouts/views/asset_browser.rs
  - zircon_editor/src/core/editor_extension.rs
  - zircon_editor/src/ui/asset_editor
  - zircon_runtime/src/asset/mod.rs
  - zircon_runtime/src/asset/pipeline/worker_pool.rs
  - zircon_runtime/src/asset/importer/ingest
  - zircon_runtime/src/asset/watch/mod.rs
  - zircon_runtime/src/asset/watch/is_meta_sidecar.rs
  - zircon_runtime/src/asset/project/meta.rs
  - zircon_runtime_interface/src/resource/resource_handle.rs
reference_sources:
  - dev/UnrealEngine/Engine/Source/Runtime/CoreUObject/Public/AssetRegistry/AssetData.h
  - dev/godot/editor/editor_file_system.h
  - dev/bevy/crates/bevy_asset/src/meta.rs
  - dev/Fyrox/editor/src/asset
plan_sources:
  - docs/plans/zircon_editor/editor/00-editor-architecture-overview.md
  - docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md
  - docs/plans/zircon_editor/editor/10-project-and-asset-reference-management.md
  - docs/plans/zircon_editor/editor/14-threading-and-job-scheduling.md
  - docs/plans/zircon_editor/editor/03-command-transaction-and-undo.md
status: planned
---

# 09 编辑器资产管理

本计划落地 00 §6 的「资产元数据」权威 `EditorAssetIndex`（引用图归 10）。

## 参照证据（dev/）

**godot 导入伴生模型**（`editor_file_system.h:46-116`）：`FileInfo { file, type, uid, import_modified_time, import_md5, import_dest_paths: Vector<String>, import_valid }`——**导入状态是索引一等字段**，浏览器直接从索引渲染「损坏/过期/正常」徽标，不触资产本体。

**bevy 导入参数伴生**（`bevy_asset/src/meta.rs:27-37`）：`AssetMeta { meta_format_version, processed_info: Option<ProcessedInfo>, asset: AssetAction }`——sidecar 携带 loader settings 与处理指纹；**导入参数 schema 由 loader 声明、sidecar 持久化**。

**UE 类型注册**（`AssetData.h:157-219`）：`FAssetData { PackageName, PackagePath, AssetName, AssetClassPath, TagsAndValues }`——类型与标签驱动浏览器行为（图标/双击/右键查 AssetDefinition）。

**Fyrox**（`editor/src/asset/`）：预览生成独立模块、右键按资源类型分派的轻量样板。

## 现状与证据（zircon，2026-07-05 实读）

### runtime 资产域结构完整，且 sidecar 已存在（v2「meta 缺位」作废）

`asset/` 一级目录：`artifact / assets / facade / formats / importer / load / management.rs / module.rs / pack / pipeline / project / runtime_asset_path.rs / virtual_geometry_cook / watch`。

**watch 模块 18 文件**（本次实测）：notify 系监视器（`recommended_watcher/map_notify_event`）+ 事件折叠（`fold_events`）+ `AssetChange/AssetChangeKind/AssetWatchEvent` 类型 + `shutdown_on_drop/spawn/watch_loop` 生命周期——文件监视是成品，编辑器索引只需订阅。

**meta sidecar 已在**：`is_meta_sidecar` 认 `.zmeta` 与 `.meta.toml` 双后缀（`watch/is_meta_sidecar.rs:3-7`）；`asset/project/meta.rs` 有 `AssetMetaDocument`（load/save，:109-116）、`AssetMetaEntry::new(uuid: AssetUuid, url: AssetUri, asset_kind: AssetKind)`（:74-85）、`AssetSourceUnit/PreviewState/AssetMetaError`——**uuid/url/kind 三元组的 sidecar 通道现成**（`AssetUuid` 即 10 计划「孤岛」之一的本体，收编而非新建）。缺口收窄为：sidecar 无**导入参数**字段（bevy settings 对应物）与**源指纹**字段（godot import_md5 对应物）。

**worker_pool API**（`pipeline/worker_pool.rs` 实读）：`AssetWorkerPool::new(options) / request(AssetRequest) -> Result / completion_receiver() -> ChannelReceiver<CpuAssetPayload>` + `AssetWorkerPoolDiagnostics/FrameSampler/record_diagnostics`——提交/完成通道齐备；**无任务级进度回调**（诊断是池级采样），v2 的 `progress_sink` 扩项判断成立。

其余已核：19 导入器（`importer/ingest/`）；`AssetReloadFrameApplyReport{applied,failed,stale,pending_count}` 按帧产出不进 bus（02 接）；`ResourceHandle<TMarker>` 21 marker；编辑器侧 `asset_importers/asset_editors/asset_creation_templates` 三表无消费闭环；浏览器四布局（compact/summary/table/thumbnail）成熟；`zircon_editor/assets` 与 runtime assets staged 合并（CLAUDE.md）。

### 缺口（收窄后）

无 FileInfo 级编辑器索引；导入无进度/结果 UI 契约；sidecar 缺导入参数与源指纹字段；21 marker 无「类型→图标/操作/编辑器」映射消费；无统一脏态/保存框架；无缩略图管线。

## 目标

1. **`AssetTypeRegistry`**：21 marker 为初始全集，`AssetTypeDefinition { marker_key, display_name, color, icon, open_toolkit: Option<ToolkitFactoryRef>(06), context_commands: Vec<CommandId>(08), thumbnail: ThumbnailProvider, import_settings_schema: Option<SchemaRef> }`；`asset_editors/asset_creation_templates` 两表并入其字段；浏览器分派全走注册表。
2. **`EditorAssetIndex`**（godot FileInfo 直译）：行 `{ path, type_marker, uuid（收编 AssetMetaEntry.uuid）, source_modified, source_digest, import_products, import_valid, dirty }`；由 `asset/watch` 事件流驱动增量维护（`fold_events` 折叠语义沿用）；浏览器只读索引不触盘。
3. **导入工作流**：导入经 14 job（`JobCategory::Import`，互斥组=同 path）派发 worker_pool（`request` + `completion_receiver` 既有通道）；任务级进度经 `progress_sink` 扩项（runtime asset owner 会签，被否降级三态）；**sidecar 扩展而非新建**——`AssetMetaEntry` 增 `import_settings: Option<serde_json::Value>`（schema 由 importer 声明）与 `source_digest: Option<String>` 两字段（11 迁移链配版本升级）；重导入命令（08）=digest 失配或手动。
4. **脏态与保存框架**：`DirtyRegistry` 以 03 `saved_top` 为文档级事实源 + 外部效应位；save/save_all/关闭询问统一（06 `DocumentToolkit::save` 的调度者）；保存前引用检查（10 提供查询）。
5. **缩略图管线**：`ThumbnailProvider` 注册（纹理直采样内建；模型/材质经 07 `PreviewScene`）；缓存 `.zircon/cache/thumbnails/<digest>.png`，digest 失配失效；`JobCategory::Thumbnail` 低优先；`PreviewState`（meta.rs 既有枚举）收编为缓存状态字段。
6. **A/B 源隔离**：工程资产源（可写）与引擎/编辑器资产源（只读，staged 合并树）；只读源写操作被命令 when 门控拒绝。

## 非目标

- GUID 体系/引用图/重定向（10，`AssetUuid/AssetUri/PackageAssetRegistry` 的收编在 10 统筹）；cook/打包（15）；具体导入器实现（runtime 各计划）；版本控制集成。

## 架构设计

### 模块布局

```
zircon_editor/src/core/asset/
  mod.rs
  type_registry.rs      # AssetTypeRegistry（06 store asset 族消费者）
  index.rs              # EditorAssetIndex + watch 事件泵
  import_flow.rs        # 导入编排（job 派发/事件回流/sidecar 读写经 AssetMetaDocument）
  dirty.rs              # DirtyRegistry + save/save_all 调度
  thumbnails.rs         # provider 注册/缓存/失效
```

runtime 侧改动最小化：watch 事件流与 worker_pool 均经 01 gateway 暴露；扩项仅两个——worker_pool `progress_sink` 参数、`AssetMetaEntry` 两新字段（均 owner 会签件）。

### 数据流（导入一例）

```
watch AssetChange(新文件/变更, fold_events 折叠后)
  → index 行 digest 失配 → import_flow 产 ImportJob
  → 14 job(Import, 互斥组=path) → worker_pool.request(AssetRequest)
  → progress_sink → job 事件 → bus → 通知中心(17)/浏览器徽标
  → completion_receiver 收 CpuAssetPayload → index 行更新(products/valid/digest)
  → sidecar 回写（AssetMetaDocument::save）
  → AssetReloadFrameApplyReport（既有）→ 02 WorldFact → 场景侧刷新
```

### 徽标状态机（索引行投影）

`Normal / Stale(digest 失配) / Importing(job 活跃) / Broken(import_valid=false) / ReadOnly(源 B)`——浏览器四布局共用同一投影函数。

### 深度测试

夹具资产类型（假 marker + 假 importer + 假 provider）注册后：图标/双击 toolkit/右键命令/导入产 sidecar/徽标随失败变红——`core/asset/` 五文件零改动。

## 里程碑

### M1 类型注册表与浏览器接线

- 切片 1.1：`type_registry.rs` + 21 marker 内建定义（图标/颜色走设计 token）；两表并入删除（06 store 协同）；浏览器分派改查注册表删硬编码。
- 切片 1.2：A/B 双源与只读 when 门控。
- 测试阶段：`cargo test -p zircon_editor --lib --locked`（四布局既有测试须过 + 分派矩阵 + 只读拒绝）。更新 `docs/zircon_editor/core/asset.md`。

### M2 索引与导入工作流

- 切片 2.1：`index.rs` + watch 泵（经 gateway 订阅 `AssetWatchEvent`）；digest 计算与徽标状态机。
- 切片 2.2：`import_flow.rs`：job 派发/进度回流/sidecar 扩展字段（`AssetMetaEntry` 增两字段 + 11 版本迁移；19 导入器 settings schema 补齐清单记状态节，首批 texture/gltf/material）；重导入命令。
- 测试阶段：`cargo test -p zircon_runtime --lib --locked`（watch/worker_pool 不回归 + progress_sink 新参数 + AssetMetaDocument 新字段往返与旧文件迁移）+ editor 导入时序单测（假 importer 成功/失败/取消）+ 索引增量一致性（全量扫描==watch 事件累积）。

### M3 脏态保存与缩略图

- 切片 3.1：`dirty.rs`（saved_top 投影 + 外部效应位）；save_all/关闭询问；引用检查接 10（未落地跳过记债）。
- 切片 3.2：`thumbnails.rs`：纹理直采样 + digest 缓存 + `PreviewState` 收编；模型/材质经 PreviewScene（依赖 07 M2，未落地占位图标）。
- 测试阶段：脏态生命周期矩阵（编辑→脏/保存→净/撤到 saved_top→净）；缓存命中/失效；save_all 与热重载幂等（保存触发重导入不得再置脏）。

## 风险与开放问题

- 两个 runtime 扩项（progress_sink、AssetMetaEntry 字段）均需 asset owner 会签；progress 被否降级任务三态，meta 字段被否则编辑器侧以平行 `.zeditor` sidecar 兜底（**下策**，产生双 sidecar，尽力避免）。
- `.zmeta` 与 `.meta.toml` 双后缀并存的收敛：与 zui 后缀收敛计划同口径（zui-suffix-convergence 在案），本计划新写一律 `.zmeta`，退役 `.meta.toml` 记 10 的 migrate-assets commandlet 任务。
- digest 算法与 10 `source_digest` 选型统一（blake3/xxhash 二选一，避免双 digest）；缩略图缓存 LRU 上限入 17 设置。
