---
related_code:
  - zircon_editor/src/ui/host/startup
  - zircon_editor/src/ui/host/editor_host_startup.rs
  - zircon_editor/src/ui/host/editor_manager_project.rs
  - zircon_editor/src/ui/host/project_access.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md
  - docs/plans/zircon_editor/editor/09-editor-asset-management.md
  - docs/plans/zircon_editor/editor/10-project-and-asset-reference-management.md
  - docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md
  - docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
tests:
  - inline tests: 5
  - rustfmt check: blocked by project_access.rs import ordering
  - current-source managed Windows Cargo pending
  - F0 cold/warm project-startup trace pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Editor UI host startup/project当前源码复核（2026-07-30）

## 范围

以下MVP project startup/activation路径 **10/10** 个Rust文件、**1,079** 行、**5** 条`#[test]`已逐文件阅读；path+raw-content SHA-256为`d3a2548adf20f69b73eb5cae62209f04f5052eae6c891868c73c98346e1e7553`。5个tracked文件与`editor_host_startup.rs`外部未提交内容只读纳入，本轮未修改Rust。

| 模块 | 文件 | 行 | 测试 | 当前边界 |
|---|---:|---:|---:|---|
| `ui/host/startup/**` | 7/7 | 370 | 1 | session/recent/welcome；同步recent validation仍在 |
| `editor_host_startup.rs` | 1/1 | 93 | 0 | project activation与startup-state的同步串联点 |
| `editor_manager_project.rs` | 1/1 | 260 | 2 | document事件正确；native discovery/load第二次执行 |
| `project_access.rs` | 1/1 | 356 | 2 | runtime/editor asset、watcher、document与save同步链 |

## 发现

- **PERF-MVP-499 / P0首帧主线程长链**：`EditorHostStartupSession::open_with_prepared_project`在`ui.run`和首帧前同步调用runtime `open_prepared_project`。其`project_generation_write` guard覆盖importer clone、watcher prepare、全量`scan_and_import`、resource prepare/commit、broadcast与watcher drain；随后Editor host又同步执行`refresh_from_runtime_project`、UI asset watcher启动和document load。
- Editor09的`sync_from_project`会遍历完整registry；每个asset读取`.meta`，ready asset再读取artifact并提取references，之后重建UUID/locator maps、catalog与preview scheduler。document load还同步读取workspace、project settings和默认scene。prepared manager消除了重复manager open，但没有消除全量I/O/residency、caller-thread工作或长generation锁。
- **PERF-MVP-075/100 / recent重复投影与I/O**：显式open成功后，`remember_opened_project`先load/decode session并异步请求持久化，紧接着`recent_projects_snapshot`再次load/decode并逐项canonical+manifest validation；刚打开的project也被再读一次manifest。自动restore虽复用last row的validation结果，但随后`open_project`仍重新canonical/parse。正常列表最多8项；legacy raw JSON在cap前迁移的无界问题继续归100。
- Config `set_value`只更新内存并交给25ms debounce persistence worker，当前不是同步磁盘写；不得按旧假设把它改成新的线程池。剩余成本是重复Value clone/decode/encode、recent filesystem validation和shutdown durability。
- **PERF-MVP-427 / native双加载**：entry native selection与`EditorManager::apply_project_plugin_manifest`分别调用一次`NativePluginLoader.load_discovered_editor`，同project generation重复目录发现、manifest/DLL load、entry与贡献物化。
- project save仍在caller同步执行scene/workspace/settings持久化，随后reimport default scene、全量editor asset refresh并重启watcher；失败只降为diagnostic，并未移出UI线程。继续归075与Runtime04/11，不另建根因。
- welcome view注册/实例恢复只持短registry锁，lock在layout command前释放；document lifecycle消息数量固定，topic clone不是独立热点。

## 参考与目标

- Bevy `dev/bevy/crates/bevy_asset/src/server/mod.rs:324-325,573-603`对已加载path复用handle，并把load放入`IoTaskPool`；开始潜在阻塞task前释放asset-info锁。Zircon应复用Runtime11统一预算与single-flight ticket，而不是照搬私有pool。
- Unreal `dev/UnrealEngine/Engine/Source/Runtime/Launch/Private/LaunchEngineLoop.cpp:6951-6956`把pre-init plugin load纳入独立boot timing scope；Zircon F0也应分别记录project scan、catalog、plugin、document和state build阶段。

Runtime04/11在锁外准备project metadata/resource candidate与MVP working set，generation guard只做短CAS publish；Editor09消费同一candidate generation，首帧只发布必要catalog rows，meta/reference/preview detail按visible/selected需求异步single-flight。Editor10把bounded recent validation变成generation ticket；Editor01/12复用唯一native load report与prepared startup artifact。所有阶段共用统一cancel/supersede/deadline/shutdown合同，不建立editor、asset或plugin私有无界队列。

## 动态验收

按assets `1/1K/100K`、artifact `4KiB/256MiB`、recent `0/1/8/1K/legacy-1M`、plugins `0/1/100/1K`、asset roots `1/8`运行cold/warm/unchanged/1% change与失败回滚，记录：UI caller filesystem/decode/plugin wall，generation/project/editor锁wait+hold，scan/import/meta/artifact/scene/settings/workspace reads，resident bytes，catalog visits/builds，watcher starts，session decode/encode，canonical/manifest reads，native discovery/load/entry，queue entry/bytes/age，F0 wall/p95与RSS。

验收要求：长I/O/decode/plugin load不在UI caller或generation/editor锁内；generation锁持有接近常数commit；warm unchanged read/build=0；startup resident bytes接近MVP working set；recent cap在逐项I/O前执行且stable generation I/O=0；native load/entry与project parse/scan各不超过1/generation；失败保留last-good并保持rollback、watcher、document、diagnostic与shutdown durability语义。managed Cargo、规模counter与F0产品trace完成前保留在`pending.md`，不进入`review.md`。
