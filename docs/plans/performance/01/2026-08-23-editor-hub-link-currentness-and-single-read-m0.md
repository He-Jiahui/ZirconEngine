---
related_code:
  - zircon_editor/src/core/hub_link
  - zircon_editor/src/ui/host/editor_manager_project_session.rs
  - zircon_editor/src/ui/host/startup/recent_projects.rs
  - zircon_editor/src/ui/retained_host/app.rs
base_reports:
  - docs/plans/performance/01/2026-08-16-editor-hub-link-current-architecture-review.md
owner_plans:
  - docs/plans/zircon_editor/editor/10-project-and-asset-reference-management.md
  - docs/plans/zircon_editor/editor/14-threading-and-job-scheduling.md
  - docs/plans/zircon_editor/editor/16-cli-args-and-hub-integration.md
reference_sources:
  - dev/UnrealEngine/Engine/Source/Runtime/Projects/Public/ProjectEditorRecords.h
  - dev/UnrealEngine/Engine/Source/Runtime/Projects/Private/ProjectEditorRecords.cpp
  - dev/UnrealEngine/Engine/Source/Editor/GameProjectGeneration/Private/SProjectBrowser.cpp
tests:
  - tools.tests.test_editor16_hub_recent_projects_single_read_m0_performance_contract
doc_type: implementation-evidence
status: static_current_revalidated_single_read_m0_applied_dynamic_pending_structural_cutover_required
---

# Editor Hub link currentness与single-read M0（2026-08-23）

## 当前冻结

| state | Rust文件 | physical lines | bytes | tests | ordered path + NUL + raw bytes + NUL SHA256 |
|---|---:|---:|---:|---:|---|
| M0前 | 6/6 | 785 | 27,613 | 4 | `7d6a993bd82c5bcd98d1b68728a84a9a4520a94f682d9e560415bf72e5bba564` |
| M0后 | 6/6 | 790 | 27,772 | 4 | `7211487a547e6586ed5b6db02f14f335975f697770f37eedb1919031dae47cc4` |

6/6文件及项目激活、Welcome recent actions、retained-host watcher/handshake调用链已完整复读。
相对2026-08-16旧冻结（721行、3 tests），当前增加了focus watcher startup consume回归等内容；
事件驱动架构结论不变。模块仍为`static_complete / dynamic_pending`，不得进入`review.md`。

## 逐文件结论

| file | current result |
|---|---|
| `error.rs` | typed failure DTO；owned path/instance只在错误路径产生，无稳定热路。 |
| `focus_signal.rs` | atomic publish与rename claim按请求触发，不做帧轮询；单mailbox有意合并重复focus。无界保留malformed/mismatched claimed files仍需count/age清理策略。 |
| `focus_watch.rs` | 一个non-recursive OS watcher，按精确mailbox path过滤；注册后立即consume一次关闭startup race。正确方向是测duplicate callback和attention coalescing，不是改回轮询。 |
| `handshake.rs` | ready/failed为一次性小JSON atomic write，ready发生在watcher建立和host startup gate之后。 |
| `mod.rs` | thin protocol facade；项目liveness仍唯一归`SessionGuard`。 |
| `recent_writeback.rs` | 8行硬界使collection算法不是根因；跨进程无限等待、锁内完整read/validate/pretty encode/atomic write及与项目提交耦合才是P0。 |

## 当前结构瓶颈

`complete_project_open`当前同步执行：配置诊断、应用plugin manifest、记录recent project、开始document
session，随后外层才提交`SessionGuard`。因此`WaitForSingleObject(INFINITE)`、磁盘延迟或可恢复的历史
写失败，都会回滚一个本来可用的项目。Welcome snapshot/remove/update同样同步执行文件与路径探测。

自动恢复还会重复读取与探测：registry snapshot逐行`validate_recent_project`，选择项目打开后又写回，
后续startup/session和Welcome presentation再次请求snapshot。最多8行只限制数量，不能限制missing、
remote或冷存储路径的主线程延迟。

正确authority chain保持为：

`ProjectAuthority/SessionGuard commit -> RecentProjectIntent -> bounded ordered projection lane ->
HubRecentProjectsGeneration -> affected-row health delta`。

Recent history是可恢复projection，不得决定项目是否打开。跨进程read-merge-write仍需序列化，但必须有
有限deadline、per-path latest coalescing、entries/bytes/age界和terminal receipt；UI消费内存generation，
不在自己的写回后重新读取同一文件。

## Single-read M0

`load_recent_projects_at`原先先调用`Path::exists()`，再调用`fs::read()`。本轮改为单次read并将
`io::ErrorKind::NotFound`映射为空registry，其他I/O错误保持原语义。

- registry存在：文件系统调用形状由`1 metadata + 1 read`降为`1 read`，每次load和锁内mutation少
  一次元数据往返；
- registry缺失：仍为一次文件系统调用；
- 删除了exists/read之间的TOCTOU窗口；JSON decode、validate和error contract不变。

这只是可证明的M0调用计数差值，不声明磁盘时延、CPU或功耗改善；无限锁等待和主线程transaction
boundary仍未关闭。

## Unreal源码依据与计划

- `ProjectEditorRecords.h:40-52`把`QueueUpdate`定义为worker上的system-lock/load/mutate/save入口。
- `ProjectEditorRecords.cpp:96-123`通过TaskGraph串行链接每次update，不阻塞caller；`126-135`只在显式
  teardown等待。其一分钟critical-section lease只证明需要有限合同，不是Zircon目标值。
- `SProjectBrowser.cpp:809-894`给discovery加CPU scope，取得recent snapshot后复用它构建/排序模型。

依赖顺序：Editor10先把项目commit与history persistence解耦；Editor14提供唯一ordered bounded lane；
Editor16迁移record/remove intent、有限lease/retry和同一typed generation；最后做两进程故障注入及F0/F1
产品测量。不得新增Hub专用线程池、第二项目锁或第二recent authority。

## 量化验收

| gate | matrix | acceptance |
|---|---|---|
| read/startup | rows 0/1/8；local warm/cold、missing、delayed path | read/decode <=1/file generation；选中项目manifest验证不重复；其余health probe离开UI线程 |
| write contention | writers 1/2/16；hold 0/10/100/1000ms；abandoned owner | project commit主线程lock/I/O wait=0；有限deadline；intent/result entries、bytes、age有界且顺序确定 |
| focus/handshake | signals 1/100/10K；duplicate events；malformed claims | frame polling=0；每个coalesced signal至多一次attention；bad claim count/age有界；ready在watcher后 |
| F0/F1 product | 至少31组matched cold/warm runs | WPR/xperf记录CPU/wait/file-I/O/context switch/RSS/package power；RenderDoc不用于此CPU/process slice |

## 本轮静态门

- M0契约先RED 1/1（命中`registry_path.exists()`），实现后GREEN 1/1。
- `rustfmt --edition 2021 --check`：6/6通过；scoped `git diff --check`通过，仅现存LF/CRLF提示。
- 未运行Rust/Cargo、两进程contention、WPR、allocator或功耗；managed validator session已归档，且没有
  current-source可执行文件。无里程碑commit或企微通知。

