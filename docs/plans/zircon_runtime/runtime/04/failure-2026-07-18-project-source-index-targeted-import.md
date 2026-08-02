---
handoff_kind: failure
status: open
created_at: 2026-07-18
summary_slug: project-source-index-targeted-import
origin_plan: docs/plans/zircon_editor/editor/10-project-and-asset-reference-management.md
fixing_plan: docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md
origin_child_dir: docs/plans/zircon_editor/editor/10
fixing_child_dir: docs/plans/zircon_runtime/runtime/04
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/project_asset_manager.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/runtime.rs
  - zircon_runtime/src/asset/pipeline/manager/service_contracts/asset_manager_contract.rs
  - zircon_runtime/src/scene/module/level_manager_contract.rs
  - zircon_runtime/src/scene/module/level_manager_project_io.rs
tests:
  - active generation locator lookup survives source deletion without filesystem stat
  - package locator becomes typed-missing after full reimport removes its source
  - failed generation preparation preserves the published project/source index
  - targeted import preserves unrelated generation records when an unrelated source disappears
---

# Runtime04：project generation source index 与 targeted import 缺失

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/10-project-and-asset-reference-management.md`
- 来源执行者：`editor10-project-reference-regression-20260717`
- 来源执行切片：ProjectAuthority / AssetRef 当前源码回归收束独立复审
- 修复责任计划：`docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md`
- 交接原因：manager-owned source-path index 已由 Editor10 与 Runtime asset manager 边界闭合，但真正的 single-source transactional import 涉及 artifact、registry、dependency graph 与 compound-source topology，最低共享 owner 属于 Runtime04 asset pipeline。

## 失败现象与复现证据

- Editor 热 locator 已停止 deep clone 完整 `ProjectManager`，但 Runtime `ProjectManager::source_path_for_uri` 仍对每个 project root 执行 `Path::exists()`；1/100/1000 locator 正常投影仍产生 N×roots 文件 stat。
- `AssetManager::import_asset(uri)` 接收目标 locator 后仍调用 `ProjectManager::scan_and_import()`，枚举并读取完整 source inventory；工程文档或 layout preset 显式 save 后并非 targeted import。

## 最低共享层根因

- 最低共享层缺口分成两层：Runtime `ProjectAssetManager` 没有随活动 generation 持有 `locator -> physical source path` 内存投影；import pipeline 也没有只替换目标 source 的 ResourceRegistry/AssetRegistry candidate commit。

## 架构修复验收

- 一次 full scan/sync 由 Runtime `ProjectAssetManager` 构建并整体发布 manager-owned source-path index；活动 generation 的 locator 查询只读该索引，不触发 `exists`/`canonicalize`/目录遍历。
- targeted import 从 generation source entry 读取一个目标；新 `res://` source 明确落在 primary root，不枚举其他 source tree。
- target commit 仅替换同 source path 的 root/subasset records，保留 unrelated ResourceRegistry 与 AssetRegistry entries；失败不发布半更新内存 generation。
- full scan、watch rename/remove 与多 root duplicate URI 检测继续保持 typed error；失败的下一代 scan 不污染当前 source index。
- 行为回归必须证明：full scan 后删除磁盘 source，旧 generation locator 仍返回已索引 path；修改 A 且删除 B 后 targeted import A 成功，B 的旧 generation record 不被隐式删除。

## 禁止临时方案

- 禁止在 Editor 缓存第二份 locator/path truth。
- 禁止把全量 `scan_and_import` 改名为 targeted import。
- 禁止为躲避文件 stat 固定拼接第一个 root，从而吞掉多 root ambiguity。

## 独立复审未关闭项

- targeted commit 必须在任何可失败步骤后恢复 sidecar/artifact/AssetRegistry/ProjectManager/ResourceManager；不能先写磁盘再因 registry persist 或 resource sync 失败留下跨代分叉。
- 新增目标、删除/换 UUID/subasset 时必须精确刷新受影响 referencer 的 dependency UUID、反向边、Runtime `dependency_ids` 与 unresolved diagnostics，不能只重算目标 owner。
- duplicate GUID owner/remint/diagnostic 预检必须发生在 artifact 写入前；不得用 `insert_checked` 失败代替既有增量规范化。
- existing compound source 的成员 add/remove 必须重建 descriptor，或 typed 拒绝并要求 full generation scan；不得复用上一代固化的 `included_paths` 静默漏成员。
- 上述不变量未闭合前，`AssetManager::import_asset(uri)` 保持当前 full import 语义；不保留本轮未通过复审的 partial targeted production path。

## Source-path index 独立复审整改

- project open 以 generation write gate 串行化。source-path index、全部 artifact payload 与新 watcher 都先在候选态完成可失败准备；随后在 project write lock 内执行无失败的 ResourceManager/index/project/watcher owner 替换，旧 watcher 先标记 retired，并在 gate 释放后停止，避免旧工程事件进入新 generation。
- `import_asset`、`reimport_all` 与 watcher refresh 不再直接修改活动 `ProjectManager`：它们 clone candidate、完成 full scan 与全部 artifact/source-index 预载后才提交；任何 prepare error 都丢弃 candidate，保留活动 project/index/ResourceManager。
- watcher 在安装前处于 Pending，期间事件与错误进入内存队列；generation 提交后才 Active 并重放队列。准备失败时 Pending watcher 可直接停止，不会等待 generation gate 形成 join deadlock，也不丢失 scan→activation 窗口事件。
- `res://` 与 `package://` 的当前 generation lookup 都以索引为权威；package miss 不再调用 destination-style root join，而是返回 typed `MissingProjectAssetUri`。labelled subasset 继续复用相同 `(scheme, path)` source entry。

## Scene LevelManager consumer补充（2026-07-22）

Runtime scene逐文件性能审查确认`LevelManagerContract::{load_level_asset,save_level_asset}`仍绕过manager-owned active generation：每个调用按`project_root: &str`重新`ProjectManager::open`并同步`scan_and_import`完整工程，save单scene也先全量扫描。`save_world/save_level`还深clone完整World并在caller线程同步serialize/create-dir/write。

Runtime04完成targeted source transaction时必须把Scene LevelManager迁移为prepared project-generation consumer，并为save发布immutable scene artifact ticket到bounded I/O lane；不得只修`AssetManager::import_asset`而保留这个第二条full-scan入口，也不得在Scene facade新增第二份ProjectManager cache。验收纳入PERF-MVP-453：warm prepared generation open/scan=0、单scene save不全量import、主线程I/O=0，内容/引用/错误/rollback/atomic publish等价。

### World project I/O producer补充（2026-07-22）

`World::save_project_to_path`内部第二次World clone、normalize entity-id Vec与builtin locator重复parse已按PERF-MVP-462止损；Level snapshot仍先clone World，`to_scene_asset`/legacy document仍逐entity构造宽投影并同步pretty serialize完整String、`fs::write`。本failure的immutable scene artifact ticket必须把projection/serialization/atomic replace全部移到Runtime11 bounded I/O lane，按world/content generation single-flight/merge并在shutdown flush；不能把删除一次clone当成PERF-MVP-453关闭条件。

### Project/registry性能审查补充（2026-07-22）

Runtime asset逐文件审查确认`scan_and_import`内部不只由外部caller触发全量工作：source collection分别递归扫描regular files与`.zmeta`/compound，reference preflight、duplicate GUID prepare、`inspect_project`、逐source import和最终registry rebuild继续多次加载sidecar，单次调用可达约5轮目录/meta inventory。stable source仍同步读取完整bytes、计算digest/mtime并复读artifact；watch单path的`apply_watch_changes`仍`scan_project_metas`全项目、clone registry、全量refresh dependency edges并persist完整registry。

PERF-MVP-494/495已局部删除`.zmeta`第二次TOML syntax parse、dependency UUID O(D²)去重和refresh edge-list URI Vec深拷贝，但不能关闭本failure。Targeted transaction必须一次发布source/meta/artifact/dependency generation，并用content/file watcher generation让warm unchanged read/hash/parse=0；changed path只准备source与reverse closure。`AssetRegistryIndex`还须按PERF-MVP-497同步维护AssetId→uuid、reverse dependency adjacency及source `(scheme,path)`→entry slots，消除referencer、AssetId反解与每change source removal的全表扫描；这些索引与registry candidate必须同代原子提交，不能由Editor另建truth。

## 产出记录与时间

| 里程碑 | 切片 | 状态 | 完成日期 | 完成项目与证据 |
|---|---|---|---|---|
| Runtime04 / Editor10 M1 | manager-owned source-path index | `实现完成-独立复审0C/0I/0M-受管行为门待完成` | 2026-07-18 | full scan、artifact payload 与 watcher 都先进入候选准备；成功后才在 project write lock 内整体发布 Runtime manager source-path index、ResourceManager、ProjectManager 与 watcher owner。活动 `res://`/`package://` locator 以 `(scheme, path)` 读内存映射，subasset label 复用同一物理源；失败准备保留上一代。删除 source 后旧 generation 仍返回 indexed path、prepare failure 不污染活动索引、package remove+reimport 后 typed missing，以及 Pending→Draining 顺序、retire 二次核验、失败 scan watcher shutdown 的回归已加入；scoped rustfmt/diff check 与独立终审 `0/0/0` 通过，Cargo 尚未执行。 |
| Runtime04 / Editor10 M1 | transactional targeted import | `实现候选-二次独立复审和受管验证待终态` | 2026-08-01 | single-source targeted import 现以隔离 candidate 准备 artifact/meta/registry 清单、依赖反向 closure 与 catalog input generation；generation/epoch CAS 通过后才提交文件并发布 ResourceManager/ProjectManager。duplicate GUID preflight 与 compound topology typed rejection 均在 artifact 事务前；单源 watch Added/Modified/Removed 复用延后提交，复杂 batch 保持完整协调。静态检查通过，尚无受管 Cargo terminal。 |

## 修复结果与回传

Open state：manager-owned source-path index 的首轮 `0C/4I/1M` 已整改并通过独立终审 `0C/0I/0M`，待受管 Windows exact tests；transactional targeted import 的四项不变量已有实现候选，正在进行二次独立复审并等待可创建的受管 focused gate。failure 在这些终态证据存在前保持 `open`。

### 2026-08-01 前向修复候选：deferred targeted generation publish

- `ProjectManager::prepare_targeted_generation` 现在只构造候选 `ResourceRegistry`、`AssetRegistryIndex`、shader reverse-dependency closure、catalog input generation 与 artifact/meta/registry 原子写入清单；准备期间不写入这三类 generation-owned 文件。
- `AssetManager::import_asset` 先完成候选资源同步准备，再在 project generation 与 preparation epoch 比对通过后提交文件清单，随后无失败地提交 ResourceManager 和 active `ProjectManager`。CAS 拒绝或任何准备错误会丢弃候选，不会留下 source artifact、sidecar 或 asset-registry 的跨代前进。
- watcher 的单个、完整 Added/Modified/Removed source event 使用相同的延后文件提交；Removed 仅准备 registry 写入清单。标记 `requires_reconciliation`、rename、拆分 move、多 source batch 仍走既有完整协调路径，未把部分路径伪装成完整原子 batch。
- 新增 `targeted_generation_prepare_defers_disk_publication_until_commit`、`targeted_watch_removal_prepare_defers_registry_publication_until_commit` 与 facade epoch-supersession 回归：真实准备 target v2 或删除候选后，artifact、`.zmeta` 与 `asset-registry.json` 字节仍保持旧 generation；`import_asset` 的 epoch CAS 被 importer 推进后返回 superseded，活动 source hash 也保持旧值。既有 fault-injection rollback、compound membership typed rejection 与 reverse shader dependency tests 保持覆盖。
- 已执行 scoped `rustfmt +1.94.1 --edition 2024` 与 `git diff --check`（仅工作树行尾提示）；二次独立复审已确认 `0 Critical / 0 Important / 0 Minor`。受管 Windows focused Cargo 尚未生成终态 receipt，本 failure 保持 `open`，不声明 GREEN、fixed 或 accepted。
- 本候选的 coordinator `cargo acquire test --dry-run` 在创建 ticket 前被共享环境的“unregistered D/E/F artifacts”前置检查拒绝；没有 Cargo 进程、测试计数或 RED/GREEN 结论。该环境维护项不改变本 failure 的 owner、也不把本会话置为 waiting/blocked。

### Retained-host model/default-scene consumers补充（2026-07-30）

F4模型按钮在UI callback先`import_asset(model)`，再由Editor第二次`gltf::import`生成skeleton与A个clips并逐URI调用`import_asset`，最后默认材质再调用一次；因当前backend每次clone project、full `scan_and_import`、prepare/commit全部resources，单次操作至少A+3次全项目transaction。asset tick的default-scene变化还重新`ProjectManager::open + scan_and_import`，绕过活动generation。Runtime04 targeted transaction必须同时覆盖compound model outputs与prepared default-scene artifact：一次source parse、一次affected closure prepare、一次原子publish；Runtime11执行stage/derive/I/O，Editor09只持一个ticket。验收增加animations 0/1/100/1K、model 4KiB/256MiB/1GiB：每按钮Runtime transaction/scan/parse≤1、default-scene reload open/scan=0、UI I/O=0，并保持derived stable ID、dependency、duplicate GUID、rollback与last-good。证据：`docs/plans/performance/01/2026-07-30-editor-retained-host-assets-current-review.md`；no pass claimed。
