---
related_code:
  - zircon_editor/src/core/document
  - zircon_editor/src/core/project
  - zircon_editor/src/core/hub_link/recent_writeback.rs
  - zircon_editor/src/ui/host/startup/recent_projects.rs
  - zircon_editor/src/ui/host/startup/resolve_session.rs
base_reports:
  - docs/plans/performance/01/2026-08-16-editor-document-scene-transaction-current-architecture-review.md
  - docs/plans/performance/01/2026-08-16-editor-project-generation-current-architecture-review.md
owner_plans:
  - docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md
  - docs/plans/zircon_editor/editor/03-command-transaction-and-undo.md
  - docs/plans/zircon_editor/editor/09-editor-asset-management.md
  - docs/plans/zircon_editor/editor/10-project-and-asset-reference-management.md
  - docs/plans/zircon_editor/editor/14-threading-and-job-scheduling.md
  - docs/plans/zircon_runtime/frameworks/01-runtime-crate-decomposition.md
  - docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md
  - docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
reference_sources:
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Public/Subsystems/AssetEditorSubsystem.h
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Private/Subsystems/AssetEditorSubsystem.cpp
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Private/FileHelpers.cpp
  - dev/UnrealEngine/Engine/Source/Editor/GameProjectGeneration/Private/GameProjectUtils.cpp
  - dev/UnrealEngine/Engine/Source/Editor/GameProjectGeneration/Private/SProjectBrowser.cpp
  - dev/Fyrox/editor/src/lib.rs
  - dev/Fyrox/editor/src/message.rs
  - dev/Fyrox/editor/src/scene/container.rs
  - dev/godot/editor/project_manager/project_list.cpp
doc_type: implementation-evidence
status: static_current_m0_complete_structural_cutover_required_dynamic_blocked
---

# Editor Document / Project currentness与scene move M0（2026-08-23）

## 当前指纹与结论

| 模块 | current Rust | 行 / bytes / tests | ordered path-NUL-raw-NUL SHA256 |
|---|---:|---:|---|
| `core/document/**` | 6/6 | 1,690 / 60,032 / 19 | `20e3207caebb53f49113935eea097561fd42581d0c52e2e46c7dcb8048ae7e4c` |
| `core/project/**` | 18/18 | 3,070 / 105,091 / 42 | `6c73c85ed04c33752617402944523f4287f6c1ca58d477291c639553e92171a1` |

本轮逐文件重读了上述24/24个Rust文件，并沿生产路径复核Hub最近项目读写、启动恢复、
`current_project_snapshot`调用者、scene open/create、catalog同步和authoring install。结论是：

- 8月16日报告中的Editor本地recent DTO、legacy migration和21文件规模已经失效。它们已迁到
  `zircon_runtime_interface::hub_protocol`与`core/hub_link`，Project当前为18文件；
- Hub注册表正常结果已硬限8项，`open_resolved`也把既有路径身份传给Runtime，二者应保留；
- 结构瓶颈仍未关闭：scene慢工作仍位于全局route gate中，项目创建/打开仍同步多阶段执行，
  active project仍通过深拷贝snapshot扩散，启动成功打开后仍再次读取并逐项目探测recent表；
- 本轮只落地一个不改变边界的M0：创建scene成功收尾由clone完整
  `ProjectSceneDocument`改为move，静态clone计数从1降为0。其他问题必须先完成generation、
  durability、线程亲和与短提交边界设计，不能用局部锁替换或私有线程掩盖。

没有current-source可运行编辑器，managed Cargo会话也不可执行。因此Rust测试、WPR/xperf、
allocator/RSS、package power和RenderDoc均未运行；本文不声明耗时、功耗、吞吐、最优性或与
其他引擎数值接近。两个模块继续属于动态未验收范围。

## 24/24逐文件结论

### Document 6/6

| 文件 | 当前性能结论 |
|---|---|
| `lifecycle.rs` | 1,024项上限和锁外facts发布应保留；`scene_route_gate`覆盖完整route，ID占用检查反向扫描两个map values，淘汰扫描并clone key。100K churn测试证明这是现实规模路径。 |
| `lifecycle/retention_snapshot.rs` | 显式诊断会在state mutex内遍历两个map并统计路径bytes；当前无每帧调用，应保持按需probe。 |
| `lifecycle/tests.rs` | 14项覆盖稳定ID、100K churn、collision、session/ticket隔离；缺少精确probe/eviction次数、分配、锁等待/持有和并发预算。 |
| `mod.rs` | 仅模块与re-export，无独立热点。 |
| `scene_route.rs` | file load/save、catalog import/reconcile、authoring install和rollback仍在route gate内；activation仍保留完整`ProjectSceneDocument`。 |
| `scene_route_tests.rs` | 5项覆盖already-active、stale session、create冲突和补偿；缺少延迟I/O、同key single-flight、项目切换、clone bytes、锁持有和durable restart门。 |

### Project 18/18

| 文件 | 当前性能结论 |
|---|---|
| `authority.rs` | create仍逐entry建目录/写文件、load+save manifest、publish后再`ProjectManager::open_resolved`；open已复用解析身份，是应保留的进展。 |
| `created_project.rs` | move交付prepared manager正确，但wrapper和内部manager仍可深clone。 |
| `error.rs` | typed I/O/rollback/scene错误，无独立循环；必须保留精确故障归属。 |
| `filesystem.rs` | canonical/link/reparse防护正确；每次scene操作仍重建ancestor路径vector并逐层获取no-follow句柄，应收敛为project-generation root capability，不能删安全检查。 |
| `mod.rs` | 窄facade；recent持久化owner已移出本模块。 |
| `new_project_draft.rs` | 小型输入验证；accepted probe仍缺可在commit时轻量复验的identity/mutation ticket。 |
| `new_project_template.rs` | enum到pack ID映射，无热路径。 |
| `opened_project.rs` | prepared manager move边界正确；目标仍应是immutable generation handle而非cloneable aggregate。 |
| `project_probe.rs` | 持有root/summary，但缺物理identity、mutation stamp、root lease和prepared generation，probe成果无法晋升。 |
| `recent_project_entry.rs` | 小型UI投影；成本来自每次snapshot重新probe磁盘而非DTO自身。 |
| `recent_project_validation.rs` | 小型结果enum，无独立热点。 |
| `scene_document.rs` | 同步path guard/load/save/stage/publish仍在调用线程；本轮`finish`已从完整document clone改为move，其他owner边界不动。 |
| `tests/boundary.rs` | 1项检查core无UI/退役模板依赖；不覆盖复杂度。 |
| `tests/directory_transaction.rs` | 7项覆盖target竞争、commit/restore/rollback失败和staging ownership；迁移到共享durable owner时必须原样保留语义。 |
| `tests/mod.rs` | fixture固定在测试二进制物理输出根，避免C盘临时目录；无产品热点。 |
| `tests/root_resolution.rs` | 6项覆盖junction/symlink/SUBST/manifest alias；后续应增加root capability复用与stale/replaced ticket，不得削弱现有检查。 |
| `tests/scene_document.rs` | 5项覆盖open/create/rollback/identity；缺完整Scene clone bytes、handle open次数和generation fault矩阵。 |
| `tests/template_creation.rs` | 15项覆盖模板内容、registry rebuild、路径、设置和transaction rollback；缺unique-parent、shared bytes、parse/encode次数、主线程等待与大规模RSS计数。 |

## 当前结构瓶颈

### P0：scene正确性由全局慢锁串行化

`DocumentLifecycleAuthority::with_scene_route`在`lifecycle.rs:376`持有唯一gate；
`scene_route.rs:67-189`在闭包中执行文件加载/创建、catalog导入与补偿、world安装和document激活。
这保证旧picker不能跨project session提交，但也让磁盘、解析、catalog和失败清理成为同一串行临界区。
正确目标是`prepare outside owners -> generation check -> short move commit -> facts after unlock`，
不是把同一闭包搬到线程池后再同步等待。

### P0：Project仍是可深拷贝的主动authority

Runtime `ProjectManager`仍派生`Clone`，`AssetManagerContract::current_project_snapshot`返回完整
clone。当前Editor资产编辑、watcher、layout、scene与project access仍有十余个生产调用点；稳定查询
会复制paths、manifest、registries、catalog/import/artifact/shader/task state。应由Runtime04发布一个
immutable project/asset generation store，调用者只取handle、typed query或delta；迁移完成时硬删除
`ProjectManager: Clone`与`current_project_snapshot`，不留兼容facade。

### P0：启动recent路径重复读取、重复磁盘探测

`resolve_session.rs:19`先调用`recent_projects_snapshot`，该函数在
`recent_projects.rs:12-20`加载共享JSON并对每项执行`validate_recent_project`。成功打开后
`resolve_session.rs:50`再次执行完整snapshot。虽然协议限制最终registry为8项，但
`recent_writeback.rs:94-106`先`fs::read`整个文件、serde分配，再做typed validate；path/summary也无字段
bytes上限。目标不是猜一个文件常量，而是定义版本化ingress budget、一次validated recent generation
和一次project identity/preparation ticket；稳定UI投影不得触发filesystem probe。

### P0：项目创建是调用线程上的重复阶段

`authority.rs:35-119`同步创建staging、逐entry创建父目录与写入、再次load/save manifest、rename
发布、打开Runtime manager；故障时还会同步递归清理/回滚。应把typed template artifact、unique parent
plan、bounded bytes、write/fsync/replace/recovery交给Runtime11与Frameworks01的唯一durable transaction，
Editor只提交intent并消费generation receipt。

### P1：Document identity有界但不是O(1)

`lifecycle.rs:399-429`为淘汰扫描map并clone key，`:525-531`为每个候选ID扫描两个map values。
在两个map各最多1,024项时，100K distinct identity churn的静态上界形状约为2.048亿次value比较，
尚未计collision stepping。应使用typed canonical key、direct occupied/reverse ID index和显式bounded
retention order，使known lookup不分配、占用检查与淘汰为均摊O(1)。

### P1：完整scene仍跨提交边界保留

本轮去掉create finish的一次完整clone，但`SceneDocumentRouteActivation`仍持有完整
`ProjectSceneDocument`且派生`Clone`；installer还从`&Scene`构造authoring state。最终receipt只应携带
document/project/authoring generation、稳定asset identity、URI和messages，不应携带source scene。

## 已落地M0：PreparedSceneCreation成功收尾move

`PreparedSceneCreation.document`改为`Option<ProjectSceneDocument>`；发布/回滚阶段通过borrow访问，
`finish(mut self)`用`take()`转移唯一owner。Drop仅在document仍存在且published时删除source，并继续清理
staging，因此成功、显式rollback和abandoned creation的补偿语义不变。

静态量化：create成功路径在`finish`处完整`ProjectSceneDocument`/`Scene` clone **1 -> 0**；没有把它
外推为总clone=0、耗时下降或RSS下降。TDD source contract先在旧实现上RED，再在move实现上GREEN。

## 参考引擎证据与适配边界

- Unreal `AssetEditorSubsystem.h:443-446`维护asset->editor和editor->asset两个索引；
  `AssetEditorSubsystem.cpp:285,399-447`用`MultiFind/Add/Remove`直接维护两向关系。这支持Zircon的
  direct key/reverse-ID index，不支持复制UE object pointer身份。
- Unreal `FileHelpers.cpp:3250-3376`以`UE_SCOPED_ENGINE_ACTIVITY`和LoadMap start/end事件显式标记
  地图加载阶段。这证明长事务需要阶段与telemetry；其同步实现不是Zircon继续锁内I/O的依据。
- Unreal `GameProjectUtils.cpp:843,1742-1842,2099-2115`有命名create阶段、待复制/待重写集合和
  exact created-file cleanup。这支持prepared plan与精确rollback ownership；其全量同步copy不是最优算法。
- Unreal `SProjectBrowser.cpp:809-821`为find建立CPU scope并用`TSet`去重候选，支持保留的项目发现
  generation；不能据此证明重复open validation合理。
- Fyrox `editor/src/lib.rs:2062-2116,2605-2631`在`loading_scenes`中做同path admission，释放锁后
  在engine task pool加载，再用`AddScene`消息安装。这支持keyed single-flight和锁外prepare；Zircon仍需
  更强的project generation与durable commit，不照搬私有任务模型。
- Godot `project_list.cpp:712-744,793-870,1066-1075`在可取消scan线程发现项目，完成后发布，单独加载
  project data。这佐证后台discovery+retained projection；Zircon使用共享Runtime11 scheduler而非新增私有线程。

## Hard-cut顺序与owner计划

1. Editor10定义bounded recent ingress和`ProjectIdentityTicket`；一次validated recent generation服务
   startup与Welcome，成功open后只应用compact recent delta。
2. Runtime04/Frameworks01发布唯一immutable project/asset generation和durable outcome；删除深clone
   snapshot与Editor重复open/reconcile authority。
3. Runtime11/Editor14提供按project generation+scene identity keyed的bounded prepare ticket，覆盖
   source/decoded/result bytes、age、deadline、cancel和single-flight。
4. Editor01/03只执行短main-affinity move commit；锁内filesystem、parser、catalog/plugin callback、
   wait和rollback均为0，facts在锁外发布。
5. Editor09消费Runtime04 exact catalog delta；scene失败不允许`reimport_all`或完整catalog rebuild。
6. Document registry切换到typed key、direct reverse index与O(1) retention order，并把activation结果
   硬切到compact generation receipt。
7. 删除旧route gate、full-scene result、summary-only probe、validate-open-validate启动链、
   `current_project_snapshot`和每scene project-root完整guard chain；不留shim或dual authority。

## 验收矩阵

| 门 | 输入 | 必须采集 | 接受条件 |
|---|---|---|---|
| recent/startup | file `1KiB/1MiB/64MiB`，rows `0/8/1K/1M` | cap前read/alloc、rows probed、canonical/stat/read/parse、UI wait、RSS | oversize在逐row I/O前终止；accepted project一次identity/manifest prepare；stable projection I/O=0 |
| scene prepare/commit | entities `1/1K/100K`，delay `0/250ms/2s`，同key callers `1/16` | admission、single-flight、queue age、source/decoded/result bytes、owner lock wait/hold | request/commit锁不含慢阶段；same-key prepare=1；stale apply=0；完整Scene clone bytes=0 |
| document registry | identities `1/100/10K/100K`，ops `1/1M`，threads `1/16` | key alloc/hash、ID probes、eviction visits/clones、lock p95、RSS | known lookup alloc=0；occupancy与eviction均摊O(1)；稳定ID/顺序不变 |
| project create | entries `1/1K/100K`，bytes `1KiB/1GiB`，fault each phase | unique parents、mkdir/write/fsync、parse/encode、copied bytes、cleanup、UI wait/RSS | unchanged entry deep copy=0；one typed manifest generation；one durable owner；UI wait/I/O=0 |
| active project | rows `1/1K/100K`，consumers `1/16/128` | aggregate clone count/bytes、handle/query、lock wait/hold | `current_project_snapshot`=0；generation acquire O(1)；work proportional to requested keys/delta |
| product | cold/warm startup/open/create/F4 switch，至少31个可比样本 | WPR CPU/File I/O/CSwitch/waits/locks/RSS/power，typed phase IDs | 报告p50/p95/p99/CI/effect size；无current binary不得填数值；首帧仅用RenderDoc做draw/pixel/resource parity |

## 静态验证记录

- 24/24 current Rust文件已逐文件重读；Document 6/6、Project 18/18清单与raw fingerprint重算完成。
- `tools/tests/test_editor_document_scene_finish_move_contract.py`：1/1 GREEN；旧实现已先验证RED。
- `rustfmt --edition 2021 --check zircon_editor/src/core/project/scene_document.rs`：GREEN。
- scoped `git diff --check`：GREEN，仅显示checkout行尾转换warning。
- Rust/Cargo未运行；因此M0仍是static integrated，不是功能或性能里程碑验收。
- WPR/xperf、allocator/RSS、power与RenderDoc未启动：无current-source可执行文件；RenderDoc也不能验收
  本文CPU/filesystem/lock问题。

