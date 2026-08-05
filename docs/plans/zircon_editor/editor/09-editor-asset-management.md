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
status: in_progress
---

# 09 编辑器资产管理

本计划落地 00 §6 的「资产元数据」权威 `EditorAssetIndex`（引用图归 10）。

```zircon-workflow
{
  "schema": 1,
  "workflow_id": "zircon-editor-asset-management",
  "goal": "完成编辑器资产类型注册、runtime registry 投影、导入工作流、脏态保存与缩略图管线，并保持来源权威单一。",
  "milestones": [
    {"id": "M1", "title": "类型注册表与浏览器接线", "depends_on": []},
    {"id": "M2", "title": "Runtime registry 投影与导入工作流", "depends_on": ["M1"]},
    {"id": "M3", "title": "脏态保存与缩略图", "depends_on": ["M2"]}
  ]
}
```

<!-- Workflow topology is maintained independently from milestone output records. Existing M1 slices predate coordinator workflow evidence and are adopted without promoting the parent plan beyond in_progress. -->

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

### 2026-07-13 执行前重核修订

2026-07-05 的现状证据已被后续 Plan10 与插件扩展实现推进，执行 M1 前以本节为当前事实：

- `ResourceKind` 当前为 26 个闭集 runtime marker，`zircon_runtime::asset::AssetKind` 只是其类型别名；
  但 editor plugin SDK 已存在 `animation.graph`、`material.graph`、`support.asset`、
  `terrain.heightfield`、`tilemap_2d.tilemap` 等开放 authoring type key。Editor09 不能把最终类型注册表
  键硬编码为 `ResourceKind`，也不能继续使用未校验的裸 `String`。
- Plan10 已把 `.zmeta` 硬切到 v7；`AssetMetaDocument` 当前已持有 `import_settings`、
  `source_digest` 与 tags，runtime 也已有 folder-backed `asset/registry/AssetRegistryIndex`。原 M2
  “新增 import_settings/source_digest 与离线索引”的描述已失效，执行时必须消费现 owner，不得新建
  editor 平行索引或第二 digest/schema。
- `ui/host/editor_asset_manager/` 已拥有 catalog、reference graph、preview cache/scheduler 与 project sync，
  但这些实现位于 UI host owner，且又读取 `*.editor.meta.toml` 的 `EditorAssetMetaDocument`。该平行
  sidecar 当前只提供 `editor_adapter`，与目标单一类型注册表冲突，后续硬切不得把它保留为 fallback。
- 类型显示名、badge、icon 与 preview palette 仍分别硬编码在 Asset Browser labels、thumbnail nodes 与
  preview refresh 中；`asset_editors` / `asset_creation_templates` 两张扩展表只有零散查询或测试消费，
  尚未形成浏览器、打开操作、创建菜单和预览生成的统一闭环。
- 首次仓库级迁移枚举已确认旧注册 API 横跨 `zircon_editor` 与多个 first-party plugin/editor crate；
  M1 必须同切片迁移所有调用方，不得留下旧 API、alias、兼容 wrapper 或双注册表。

当前重核与待批准的硬切裁决归档在
[`09/2026-07-13-m1-current-state-and-hard-cutover-audit.md`](09/2026-07-13-m1-current-state-and-hard-cutover-audit.md)。

### 缺口（收窄后）

尚无消费 Runtime registry/watch 的 FileInfo 级编辑器状态投影；导入无进度/结果 UI 契约；26 个
runtime kind 与开放 plugin authoring type 尚未通过单一 definition 提供「类型→图标/操作/编辑器」消费；
无统一脏态/保存框架；缩略图仍由 UI host 的平行缓存/硬编码 provider 维持。`import_settings` 与
`source_digest` 已由 `.zmeta` v7 持有，不再列为 Editor09 待新增字段。

## 目标

1. **`AssetTypeRegistry`**：以严格校验、可 serde 的开放 `AssetTypeId` string-newtype 为 key；26 个 `ResourceKind` 映射为内建 canonical definitions，插件经可序列化 `AssetTypeContribution` 扩展。最终 `AssetTypeDefinition` 统一持有 runtime-kind 映射、display/badge/icon/design-token、toolkit operation、context commands、creation templates、thumbnail provider descriptor 与 import settings schema ref；`asset_editors/asset_creation_templates` 两表和裸 `asset_kind: String` API 全仓硬删除，浏览器/预览/打开/创建分派只消费 materialized registry。
2. **`EditorAssetIndex`**（godot FileInfo 直译）：行 `{ path, type_marker, uuid（收编 AssetMetaEntry.uuid）, source_modified, source_digest, import_products, import_valid, dirty }`；由 `asset/watch` 事件流驱动增量维护（`fold_events` 折叠语义沿用）；浏览器只读索引不触盘。
3. **导入工作流**：导入经 14 job（`JobCategory::Import`，互斥组=同 path）派发 worker_pool（`request` + `completion_receiver` 既有通道）；任务级进度经 `progress_sink` 扩项（runtime asset owner 会签，被否降级三态）；直接消费 `.zmeta` v7 既有 `AssetMetaDocument.import_settings`、`source_digest` 与 tags，不新增 editor sidecar 或第二 schema/digest；重导入命令（08）=digest 失配或手动。
4. **脏态与保存框架**：`DirtyRegistry` 以 03 `saved_top` 为文档级事实源 + 外部效应位；save/save_all/关闭询问统一（06 `DocumentToolkit::save` 的调度者）；保存前引用检查（10 提供查询）。
5. **缩略图管线**：`ThumbnailProvider` 注册（纹理直采样内建；模型/材质经 07 `PreviewScene`）；缓存 `.zircon/cache/thumbnails/<digest>.png`，digest 失配失效；`JobCategory::Thumbnail` 低优先；`PreviewState`（meta.rs 既有枚举）收编为缓存状态字段。
6. **A/B 源隔离**：工程资产源（可写）与引擎/编辑器资产源（只读，staged 合并树）；只读源写操作被命令 when 门控拒绝。

## 非目标

- GUID 体系/引用图/重定向（10，`AssetUuid/AssetUri/PackageAssetRegistry` 的收编在 10 统筹）；cook/打包（15）；具体导入器实现（runtime 各计划）；版本控制集成。

## 架构设计

### 模块布局

```
zircon_editor/src/core/asset/
  mod.rs                 # 精选 façade，零行为
  type_registry/
    mod.rs               # folder-backed owner 接线
    asset_type_id.rs     # 开放 typed key 与 serde 校验
    definition.rs        # 最终 materialized definition
    contribution.rs      # plugin/cdylib 可序列化贡献
    registry.rs          # merge、冲突拒绝、查询与版本
    builtin.rs           # 26 个 ResourceKind canonical definitions
    error.rs             # typed registration/materialization error
  index.rs              # EditorAssetIndex + watch 事件泵
  import_flow.rs        # 导入编排（job 派发/事件回流/sidecar 读写经 AssetMetaDocument）
  dirty.rs              # DirtyRegistry + save/save_all 调度
  thumbnails.rs         # provider 注册/缓存/失效
```

runtime 侧改动最小化：Plan10 已拥有 `.zmeta` v7 的 `import_settings/source_digest/tags` 与 `AssetRegistryIndex`，Editor09 只消费，不再新增字段或第二索引。后续 runtime 扩项仅剩 worker_pool 任务级进度合同，且必须由 asset owner 会签；不得增加 `.zeditor`/`.editor.meta.toml` fallback。

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

- 切片 1.1：folder-backed `core/asset/type_registry/` + typed `AssetTypeId` + 26 个 `ResourceKind` 内建定义；先以 RED 合同锁 key 校验、built-in completeness、contribution merge、duplicate field owner 与 serde roundtrip。
- 切片 1.2：`EditorExtensionRegistry` 硬切为 `AssetTypeContribution` 单入口；`AssetEditorDescriptor` / 独立 `AssetCreationTemplateDescriptor` 表、旧 register/read API 与 importer/graph/timeline 裸 `asset_kind: String` 全仓迁移并删除，不留 shim。
- 切片 1.3：浏览器四布局、preview palette/provider、打开/创建/上下文命令改为 registry projection；删除散落 kind match 与 `EditorAssetMetaDocument` / `*.editor.meta.toml`。
- 切片 1.4：A/B source authority 与只读 typed when/dispatch guard；`res://` 工程源可写，package/builtin/library/derived/transient 不可作为工程资产写入。
- 测试阶段：`cargo test -p zircon_editor --lib --locked`（四布局既有测试须过 + 分派矩阵 + 只读拒绝）。更新 `docs/zircon_editor/core/asset.md`。

### M2 Runtime registry 投影与导入工作流

- 切片 2.1：`index.rs` 作为 runtime `AssetRegistryIndex` + watch event 的 editor 状态投影，不复制 UUID/path/dependency graph owner；digest 与 tags 直接读取 `.zmeta` v7 权威字段。
- 切片 2.2：`import_flow.rs`：14 job 派发、worker_pool request/completion、任务级进度回流；19 导入器 settings schema 补齐清单记状态节（首批 texture/gltf/material），重导入只比较既有 `source_digest` 与当前 blake3 owner。
- 测试阶段：`cargo test -p zircon_runtime --lib --locked`（watch/worker_pool/registry 不回归 + progress contract）+ editor 导入时序单测（假 importer 成功/失败/取消）+ Runtime registry 投影与 watch 累积一致性；不再测试已由 Plan10 拥有的旧字段迁移。

### M3 脏态保存与缩略图

- 切片 3.1：`dirty.rs`（saved_top 投影 + 外部效应位）；save_all/关闭询问；引用检查接 10（未落地跳过记债）。
- 切片 3.2：`thumbnails.rs`：纹理直采样 + digest 缓存 + `PreviewState` 收编；模型/材质经 PreviewScene（依赖 07 M2，未落地占位图标）。
- 测试阶段：脏态生命周期矩阵（编辑→脏/保存→净/撤到 saved_top→净）；缓存命中/失效；save_all 与热重载幂等（保存触发重导入不得再置脏）。

## 风险与开放问题

- runtime `progress_sink` 扩项需 asset owner 会签；若合同需要调整，必须改当前 worker/task owner，禁止以 editor 平行 sidecar、test-only worker 或第二任务状态真源兜底。
- `.zmeta` 与 `.meta.toml` 双后缀并存的收敛：与 zui 后缀收敛计划同口径（zui-suffix-convergence 在案），本计划新写一律 `.zmeta`，退役 `.meta.toml` 记 10 的 migrate-assets commandlet 任务。
- digest 直接消费 Plan10 当前 blake3 `source_digest` owner，不再保留 blake3/xxhash 二选一；缩略图缓存 LRU 上限入 17 设置。

## 产出记录与时间

请将产出记录放置在子计划中，此处仅展示当前现状的概述。

当前状态：M1 执行前现状与硬切迁移面审计已完成；原计划中 21 marker、缺少
`import_settings/source_digest`、缺少离线 registry 的证据已被当前源码推翻。新的单一
`AssetTypeRegistry` 需要同时容纳 26 个内建 runtime kind 与开放插件 authoring type key，并删除
旧两张扩展表、平行 editor sidecar 和浏览器/预览硬编码分派。更新版方案 A 已获用户批准，
`20260713-editor09-m1-asset-type-registry` 会话已进入 TDD 实现；M1.1 typed id/materialized registry
核心已完成 Windows 编译与 focused 验证；M1.2 扩展注册表/first-party plugin typed hard cut 已完成，
13 个受影响 first-party editor crates 的 locked check 通过；M1.3 Browser/Assets Activity/preview/
open/create/context registry projection、editor sidecar 与 suffix/adapter hard cut 已完成，当前 focused
suite 22/22、open 2/2、Browser 41/41、Activity 9/9、workspace 4/4、reference drag 9/9、UI Asset
Authoring plugin locked 2/2。M1.4 source authority、source write policy、command target metadata、
`AssetWritable` when 与实际 dispatch 拒绝实现已落地；Windows no-run 门自然退出 0，完整 registry
24/24 与 descriptor when 1/1 通过，M1.4 功能切片已完成。ProjectAuthority Manager 夹具已统一硬切，
当前源码 binary 的 Manager suite 由原 66/83 收敛并自然取得 83/83；Frameworks02 observer 导入、
Frameworks05 `RuntimeProfileId` consumer、Runtime02 weak caller lifetime 均已按责任计划回传 fixed。
M1 全量测试阶段仍在执行，因此 M1 与本计划继续保持 `in_progress`，不提前写 completed。Runtime13
`HostRegistry` consumer E0308/E0599 已修复回传；第二轮完整 Windows job
`e81ed19d256f40c28ddb2437e9a18460` 成功编译并进入 3157 项串行执行，在第 1755 项前观察到 130 个
跨功能失败，随后连续第二轮停在 Editor15 单 worker export subprocess 双流捕获测试。现场超过 10 分钟
无日志、两个 capture 文件为 0 字节后人工终止并以 `-1` 释放；这不是自然 summary。同一 current
binary 的该 exact 后续 1/1、19.65s 自然通过，Editor15 独立功能归属已 fixed 校正，缺失完整 summary
继续由 Runtime11/Editor14 full-harness 资源生命周期记录接管。新增业务失败已写入 Editor07、Editor03、
Render01，既有 UI/Project/Plugin 聚类已在原 failure 追加当前证据；Editor09 不跨 owner 修复，也不以
部分失败数关闭 M1。完整门进一步暴露 M1.3 generic toolkit payload 仍把 locator 当物理路径的
Editor09 自有缺口；2026-07-14 已进入 typed `AssetToolkitOpenRoute`/ProjectManager source resolution
硬切。RED 与实现已落地，Cargo GREEN 因共享 target 被其他 Editor 测试占用仍待取得，因此该修正与
M1 均保持 `in_progress`。

2026-07-18：asset type registry 的 clone-on-augment 性能失败源码修复已完整落地。existing definition 改为 borrowed validate 后原位 commit，失败不复制/不改变 definition 与 generation；templates/commands 改 binary ordered insertion，plugin catalog 以 registration generation + `OnceLock<Arc<_>>` 复用同 generation materialization。Workbench shell 再以唯一 extension generation + 排序 capability snapshot 复用 `Arc<AssetTypeRegistry>`，definition/open/create/context/Activity/Browser 投影不再按查询重放 contributions。1,000 次逆序增量、失败原子性、Arc identity 与 host cache hit 测试已落地，静态门通过；source-bound Cargo/review 仍受 Coordinator01 immutable full-input barrier 阻塞，所以 failure 与 M1 testing stage 保持 open。

2026-07-23：asset catalog/preview 性能 Failure 的 generation、worker、change-stream 与 host drain budget 源码阶段已推进。旧的查询期全量 DTO 构建已硬切为 generation-owned Arc rows、UUID/locator/folder indexes 与 cached details；retained host/workspace 不再把 manager generation 深复制回 owned catalog。preview 改走统一 EditorJobSystem `Thumbnail`/`Background` lane，按 source hash 隔离 artifact，并以全局唯一 admission token 覆盖 success、stale、cancel/panic 与槽位补位；cooperative cancel 在耗时阶段和最终 commit point 前重复检查，终止只完成匹配 token、不隐式重新标脏，永久 meta load 失败则 generation-safe 发布 terminal Error，避免 shutdown/cancel/error 无退避重提。原unbounded channel已硬切为512-key bounded/coalesced shared-Arc mailbox；全局publish sequence保证并发订阅一致，同key更新移到队尾且拒绝revision回退，overflow回退最新CatalogChanged。retained host三流分别限制256项/600µs并记录pending/queue-age，startup使用len cutoff而非无限drain。当前静态合同9/9，generation/preview与change-stream两轮独立最终复审均为0/0/0；10k catalog单行更新保留9,999个row allocation与双索引的Rust回归已写入、尚待Cargo执行。ProjectManifest+PackageAssetRegistry+ResourceRecord typed delta与latest-started source sync线性化已落地，但它不是完整catalog input：复审0/1/0指出source mtime、完整meta投影、artifact direct references仍由Runtime04 authority持有，故错误unchanged早退已安全撤回，并新建`runtime/04/failure-2026-07-23-project-catalog-input-generation.md`；原`.zmeta preview_state`字段级CAS Failure继续open。由于current-source Cargo仍受Coordinator01 validation-copy failure阻塞，本阶段只记`in_progress/dependency_blocked_validation_blocked`，不提前写accepted/completed；Runtime04两项返回后再完成unchanged零工作与10k动态门。

2026-07-23：UI asset watcher reverse-generation/worker 硬切已完成源码阶段。open-session direct/import reverse index、missing/invalid edge recovery、跨文档共享 physical parse generation、`EditorJobSystem` background index worker、project/dependency/baseline/fingerprint commit gate、dependency→session 原子锁 epoch、per-asset 6 次有界退避与 refresh typed diagnostics 已落地；同项目 save 保留 pending work，project-root cutover 才 cancel/reset。旧扁平 `imports.rs`/`refresh.rs` 不再存在，模块与测试锚点已迁往 `imports/*`、`refresh/pipeline/*`。静态合同 11/11 与精确 rustfmt 通过；三轮独立复审从 0/6/2、0/5/2 收敛为 0/0/0。受管 Rust gate 仍待，Coordinator01 validation-copy failure/外部 Cargo 队列未解除，所以本切片保持 `in_progress/validation_blocked`，对应 watcher Failure 保持 open，1k/10k storm/p95、fixed return 和 managed commit 不提前宣称。

2026-07-30：Performance01按current source复读`core/asset/**`31/31（5,889行、43 tests）。旧dirty全量snapshot、import无single-flight/无entry-byte-age预算和registry无batch三项描述已失效：当前已有4,096-generation dirty journal+Editor03 cursor、exact `(uuid, uri, digest)` shared flight+三预算、按asset type单次finalize的`apply_contributions`。本计划仍须收口四个生产接入门：554的changed/reset effect-map锁内深clone与8次重试预算；555的caller-thread admission Condvar等待、distinct-UUID mutex-group全表扫描/时间桶线性删除和observer结果深clone；556的dormant `EditorAssetIndex`全量collect/sort、全量replacement及unknown watch path无界驻留；562的host materialize重建26 builtins并逐条clone/apply、尚未消费现成batch。联动Editor03/14、Runtime04与Plugins12按PERF-MVP-554/555/556/562验收；current-source managed Cargo、allocator/lock/RSS counter及F1/F4产品trace未完成，故M1/M2/M3与相关failure保持open，不写accepted/completed。

2026-07-31：Performance01回查native close发现M3.1的统一save/save-all/关闭询问尚未成为产品owner：retained host仍clone pending dirty rows并直接逐view调用UI asset/animation同步save；UI asset链在native回调执行serialize、`fs::write`、disk-baseline clone、import、workspace refresh、hydrate与sync，中途失败保留旧prompt并在retry重复先前成功项。Editor09按PERF-MVP-602发布typed `SaveDirtyViewsRequest/Result`与dirty-generation token，先验证全部view kind/path/writability/reference policy，再只提交一次canonical save batch；结果逐view标记success/failure，retry仅重提失败或新generation项，全部成功且generation匹配才允许EditorUI08 close commit。实际I/O/import预算由Editor14执行，不在retained host、toolkit或每domain editor建立平行save-all循环。

## 子计划与失败交接索引

- M1 当前状态审计：[2026-07-13-m1-current-state-and-hard-cutover-audit.md](09/2026-07-13-m1-current-state-and-hard-cutover-audit.md)
- M1 批准设计：[2026-07-13-m1-approved-asset-type-registry-design.md](09/2026-07-13-m1-approved-asset-type-registry-design.md)
- M1.1 核心产出：[2026-07-13-m1-asset-type-registry-core.md](09/2026-07-13-m1-asset-type-registry-core.md)
- M1.2 扩展注册表硬切产出：[2026-07-13-m1-extension-registry-hard-cut.md](09/2026-07-13-m1-extension-registry-hard-cut.md)
- M1.3 Browser/Preview/操作投影产出：[2026-07-13-m1-browser-preview-registry-projection.md](09/2026-07-13-m1-browser-preview-registry-projection.md)
- M1.3 typed toolkit route 修正当前产出：[2026-07-14-m1-asset-toolkit-route-hard-cut.md](09/2026-07-14-m1-asset-toolkit-route-hard-cut.md)
- fixed 已修复：[editor-operation-path-deserialize-validation-bypass](09/fixed-2026-07-15-editor-operation-path-deserialize-validation-bypass.md)
- M1.4 source authority/write guard 当前产出：[2026-07-13-m1-source-authority-write-guards.md](09/2026-07-13-m1-source-authority-write-guards.md)
- registry delta/generation 性能切片：[2026-07-18-asset-type-registry-delta-generation.md](09/2026-07-18-asset-type-registry-delta-generation.md)
- open 源码完整（registry/plugin/host generation cache 已完成；受管规模证据、review 与 fixed return 待完成）：[asset-type-registry-clone-on-augment](09/failure-2026-07-17-asset-type-registry-clone-on-augment.md)
- open 性能交接（dirty batch generation）：[dirty-registry-snapshot-retry-clone-budget](09/failure-2026-07-22-dirty-registry-snapshot-retry-clone-budget.md)
- open 性能交接（import single-flight/backpressure）：[asset-import-duplicate-admission-backpressure](09/failure-2026-07-22-asset-import-duplicate-admission-backpressure.md)
- open 性能交接（UI asset watcher reverse generation/worker；源码完成、动态验收待）：[ui-asset-watcher-unbounded-refresh](09/failure-2026-07-17-ui-asset-watcher-unbounded-refresh.md)
- M1 testing stage 当前产出：[2026-07-13-m1-full-windows-acceptance.md](09/2026-07-13-m1-full-windows-acceptance.md)
- fixed 已修复：[project-authority-test-fixture-cutover](../../zircon_runtime/runtime/02/fixed-2026-07-13-project-authority-test-fixture-cutover.md)
- fixed 已修复：[font-decoration-display-size-argument](09/fixed-2026-07-13-font-decoration-display-size-argument.md)
- fixed 已修复：[native-plugin-runtime-target-mode-test-path](09/fixed-2026-07-13-native-plugin-runtime-target-mode-test-path.md)
- fixed 已修复：[render-framework-pipeline-registration-test-double-migration](09/fixed-2026-07-13-render-framework-pipeline-registration-test-double-migration.md)
- fixed 已修复：[runtime-module-lifecycle-observer-import-cutover](09/fixed-2026-07-13-runtime-module-lifecycle-observer-import-cutover.md)
- fixed 已修复：[editor-manager-weak-runtime-caller-lifetime](09/fixed-2026-07-13-editor-manager-weak-runtime-caller-lifetime.md)
- fixed 已修复：[runtime-profile-id-consumer-cutover](09/fixed-2026-07-13-runtime-profile-id-consumer-cutover.md)
- fixed 已修复：[host-registry-generational-handle-consumer-cutover](09/fixed-2026-07-13-host-registry-generational-handle-consumer-cutover.md)
- fixed 已修复：[plugin-structure-audit-report-fixture-drift](09/fixed-2026-07-15-plugin-structure-audit-report-fixture-drift.md)
- fixed 已修复（Plugins01 extension registry finalize/read boundary guard）：[extension-registry-finalize-coverage-guard-drift](09/fixed-2026-07-15-extension-registry-finalize-coverage-guard-drift.md)
- fixed 已修复：[export-cargo-single-worker-windows-output-hang](09/fixed-2026-07-13-export-cargo-single-worker-windows-output-hang.md)
- open 待修复（Editor07 动画资产打开测试夹具未迁移索引权威）：[animation-asset-open-index-fixture-cutover](07/failure-2026-07-13-animation-asset-open-index-fixture-cutover.md)
- fixed 已修复：[editing-operation-owner-structure-guard-drift](09/fixed-2026-07-14-editing-operation-owner-structure-guard-drift.md)
- fixed 已修复：[editor-viewport-resolve-job-guard-drift](09/fixed-2026-07-14-editor-viewport-resolve-job-guard-drift.md)
- fixed 已修复（Plugins05 Navigation 产出记录已收束到编号子目录）：[navigation-plan-output-record-archive-limit](09/fixed-2026-07-15-navigation-plan-output-record-archive-limit.md)
- fixed 已修复：[plan-output-record-archive-limit](09/fixed-2026-07-14-plan-output-record-archive-limit.md)
- fixed 已修复：[plan-output-archive-notice](09/fixed-2026-08-05-plan-output-archive-notice.md)
- open 待修复（EditorUI10 与 index output notice）：[editor-ui-plan-output-notices](../editor_ui/10/failure-2026-07-13-editor-ui-plan-output-notices.md)
- open 待修复（EditorUI11 archive notice）：[plan-output-archive-notice](../editor_ui/11/failure-2026-07-13-plan-output-archive-notice.md)
- 2026-07-18 shader hot-reload性能交接：shader package、template registry与IDE preview对同一WGSL分别extract includes与strip，至少双全文扫描且strip分配临时行Vec。Editor09联动Runtime04/Render08提供单遍parse artifact（stripped source/deps/line map/hash input）并按module generation共享；稳定source scan=0、changed source scan=1，依赖图只增量失效changed nodes；见PERF-MVP-358。
- 2026-07-18 material management性能交接：现有runtime getter会深clone全部material readiness details并重建全量record set/indices，query再sort后分页；当前尚未确认pane产品consumer。Editor09联动Render08/17只接generation-owned compact rows+indices，UI缓存generation+query+page，详情按selected id懒取；stable 60Hz poll build/clone/sort=0，changed visits近changed+page；见PERF-MVP-360。
- 2026-07-18 shader IDE env补充：preview matrix URI index已从shader×variant降为每批一次；但每次生成仍加载全部ready shader、每stub递归全扫依赖并拼接全文/Naga parse，S×V preview全部assembly/validate，最后递归扫stale文件。Editor09按PERF-MVP-358消费Runtime04 indexed module DAG与generation preview artifacts，在bounded asset job中增量生成；稳定UI请求scan/parse/assembly/I/O=0，changed工作近affected closure。
- 2026-07-22 runtime asset management投影交接：Runtime helper当前为kind/list/overview/status/issue重复全registry scan+sort+deep clone，scene records/entities还重复加载同一scene。Editor09按PERF-MVP-500只消费Runtime04 generation-owned compact rows/indices/summary，缓存`generation + query + page`并仅为visible/selected行请求detail；stable 60Hz不得重建全量record sets，见Runtime04 `asset-management-generation-projection` open failure。
- 2026-07-22 importer capability投影交接：Editor09不得为每次source hover/inspect重走Vec registry select并clone完整descriptor/status message；按PERF-MVP-503消费Runtime04 importer generation的compact matcher/capability row，缓存`generation + source suffix`，详情只按selected importer取。插件reload只失效changed plugin slots。
- 2026-07-22 material asset投影补充：PERF-MVP-515已把底层material summary 9次records扫描降为1次；Editor09仍不得以轮询调用overview/readiness/descriptor来重建properties、slots和diagnostics。按PERF-MVP-516及既有360只消费Runtime04 effective-material generation的compact rows/summary，selected detail借用共享artifact；stable 60Hz material build/scan/clone=0。
- 2026-07-22 shader asset投影补充：PERF-MVP-517已把底层shader summary 14次records扫描降为1次；当前management record仍同时持有compact summary与完整imports/entries/defs/pipeline-layout report。Editor09按PERF-MVP-518/360只读compiled shader generation counters和分页row，完整diagnostics/layout仅selected shader懒取；stable 60Hz wide report build/clone=0。
- 2026-07-22 scene asset投影补充：PERF-MVP-519已把底层scene/entity summary和overview多遍扫描降为单遍，并删除entity list经full scene record中转的宽row clone。Editor09仍须按PERF-MVP-520/500只消费Runtime04 compact scene/entity generation、分页rows与selected detail；稳定列表不得调用`SceneAsset::overview/direct_references`或复制全scene entity rows。
- 2026-07-22 texture asset投影补充：PERF-MVP-521已删除render descriptor重复owned clone/normalize与Cube LUT三份defaults；Editor09不得在列表/hover轮询中反复调用upload readiness并重parse DDS/KTX/ASTC或复制format/payload。按PERF-MVP-522只读Runtime04 compact readiness/generation row，selected detail才取parsed layout；preview按523消费shared resident/chunk handle，stable 60Hz parse/clone/decode/upload=0。
- 2026-07-22 asset UI generation补充：PERF-MVP-219当前ModelRc metadata已有identity/scroll groups与partition visible range，但generation build仍分配String-key BTreeMap/分组排序，visible query还clone fixed rows并sort；改为template compiled typed slots与borrowed ordered ranges。PERF-MVP-560的asset creation menu复用同一asset-type/template generation action table，click不得重建全entries；责任failure在Editor06 `06/failure-2026-07-22-workbench-menu-control-generation.md`。
- 2026-07-22 asset-type bulk generation补充：PERF-MVP-562确认单entry binary insert在1,000个反向逐条contribution下仍累计O(N²)搬移并发布1,000 generations。Editor09联动Plugins12提供catalog-generation `apply_contributions_transaction`，按asset type聚合delta后一次validate/merge/sort/publish；更新现有[open failure](09/failure-2026-07-17-asset-type-registry-clone-on-augment.md)。
- 2026-07-30 Performance01 project-startup catalog交接：current `refresh_from_runtime_project -> sync_from_project`在F0 caller遍历完整registry、逐asset读meta、对ready asset读artifact并提取references，再全量重建locator/UUID map、catalog与preview scheduler；这是既有[`09/failure-2026-07-17-editor-asset-catalog-full-rebuild-and-preview-lock.md`](09/failure-2026-07-17-editor-asset-catalog-full-rebuild-and-preview-lock.md)的产品启动证据。Editor09按PERF-MVP-499消费Runtime04同一candidate generation，首帧只commit MVP/visible rows，details/reference/preview走Runtime11有界single-flight；warm unchanged reads/builds=0，禁止在UI caller再建第二catalog。证据见`../../performance/01/2026-07-30-editor-ui-host-startup-project-current-review.md`。
- 2026-07-31 Performance01 asset-drag交接：current pointer route已有item/row index，但press按UUID重扫10K catalog/reference rows，并在任何left-down立即clone UUID/display/extension、双locator及kind/summary/status。Editor09按PERF-MVP-109在同一asset generation发布immutable `AssetDragSource` slot/handle，供EditorUI01的typed candidate在真实drag Begin O(1)解引用；click-only不得物化payload，ABI/serde边界才拥有最终Strings，禁止新增drag私有UUID map。证据见`../../performance/01/2026-07-31-editor-retained-asset-drag-payload-current-review.md`。
- 2026-07-31 retained backend-refresh补证：`refresh_backend_state_for_events`的`selected_asset_uuid`只重复设置Catalog/Reference分支已无条件为true的details flag，caller却为此构建完整`editor_snapshot()`。Editor09按PERF-MVP-104删除该无效参数/宽snapshot，refresh flag只由event generation决定；先补源码合同证明Catalog/Reference、Preview/Admission、Resource/default-scene语义不变。证据见`../../performance/01/2026-07-31-editor-retained-tick-projection-adapters-current-review.md`。
