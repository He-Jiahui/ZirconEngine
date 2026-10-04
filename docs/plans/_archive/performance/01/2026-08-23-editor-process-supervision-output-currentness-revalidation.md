---
related_code:
  - zircon_editor/src/core/process.rs
  - zircon_editor/src/ui/host/export_cargo_process.rs
  - zircon_editor/src/ui/host/export_process_support
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/execution.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/execution
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/output_tail.rs
base_reports:
  - docs/plans/performance/01/2026-08-15-editor-process-supervision-output-current-review.md
  - docs/plans/performance/01/2026-08-15-editor-process-supervision-output-protected-plan-routing.md
owner_plans:
  - docs/plans/zircon_editor/editor/04-pie-and-simulation.md
  - docs/plans/zircon_editor/editor/14-threading-and-job-scheduling.md
  - docs/plans/zircon_editor/editor/15-build-export-and-publishing.md
  - docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
reference_sources:
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Private/Windows/WindowsPlatformProcess.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Private/Misc/MonitoredProcess.cpp
tests:
  - tools.tests.test_editor04_process_play_backend_contract
  - tools.tests.test_editor15_export_generation_inventory_contract
  - tools.tests.test_zircon_export_compile_host_output_gate_test_owner_boundaries
doc_type: currentness-revalidation
status: static_current_revalidated_partial_output_m0_present_structural_and_dynamic_pending
---

# Editor进程监管与输出currentness重验（2026-08-23）

## 当前冻结

| scope | Rust文件 | physical lines | bytes | tests | ordered path + NUL + raw bytes + NUL SHA256 |
|---|---:|---:|---:|---:|---|
| process/export output slice | 10/10 | 3,033 | 103,251 | 19 | `21a707cd7a05a3822aefb31a7247f377ae014e524abc94bfd832162221baefe7` |

10/10文件已按当前worktree完整复读；相对8月15日的2,927行/16 tests，8月22日生产变更已关闭部分
输出风险。当前4个foreign dirty文件只包含rustfmt import/assertion排版，未改变算法，本轮不覆盖。

## 已关闭的旧诊断

- Cargo不再把完整stdout/stderr持续累积为Vec后再复制完整String；每流保留256 KiB byte tail，并记录
  total/discarded bytes。
- `final_output_drain`全量聚合已删除。live与terminal都通过每流最多64 KiB的capture chunk循环，
  fast-exit remainder不再一次物化为完整输出。
- wizard继续把完整输出流式写入声明的artifact、增量BLAKE3和byte count，只保留512行tail；单行decoder
  上限16 KiB，tail前端移除已是`VecDeque` O(1)。

因此主计划`PERF-MVP-080`的“完整Vec/String使RSS线性”和`PERF-MVP-091`的“无界log”现状已过期，
必须改成回归门，不能重复实现第二套tail。以上是源码容量边界，尚无1 GiB allocator/RSS动态数据。

## 剩余结构性P0

### process/tree/terminal owner仍分裂

Play使用suspended Windows child加Job Object；export在Windows spawn时没有Job，取消时同步启动
`taskkill /PID /T /F`再fallback direct kill。`ProcessTreeLease::terminate(self)`和Windows
`JobObject::terminate(self)`消费唯一tree owner；失败后不能在同一typed session中retry。`succeeded`只表示
terminate调用被接受，不证明reap、descendant exit、pipe EOF和artifact cleanup全部完成。

`ExportProcessChildGuard::Drop`还会在任意drop线程同步terminate并可能wait。目标必须是：

`ProcessRequestGeneration -> PlatformSpawnTicket -> ProcessSessionGeneration ->
BoundedOutputDelta + CanonicalOutputArtifact -> TerminationReceipt -> ReapReceipt ->
PipeCloseReceipt -> ArtifactCleanupReceipt`

Runtime11唯一`ProcessSupervisor`保留所有native handles到各receipt结束；terminate failure保持retryable
`Terminating/Failed` session，Drop只提交idempotent nonblocking cleanup。

### Windows spawn仍扫描全系统线程

Play先用`std::process::Command`创建suspended child，再调用`CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD, 0)`
遍历全系统线程寻找owner PID，OpenThread后ResumeThread。spawn work随无关系统thread数增长，且无法严格证明
找到的就是CreateProcess返回的primary thread。

Unreal `WindowsPlatformProcess.cpp:850-882`的platform spawn直接取得`PROCESS_INFORMATION`中的process和
primary-thread handles，再由platform owner决定关闭时机；`:1353-1368`同样用刚创建的process handle做Job
assignment。Zircon应据此在自己的Windows platform spawn中直接保留`hProcess/hThread`、attach Job后resume
该thread。这是对Unreal ownership模式的适配，不是声称其通用CreateProc本身使用suspended resume。

### 输出仍双写且轮询占worker

`create_output_capture`仍在host `std::env::temp_dir()`创建两份完整relay文件；child先写relay，monitor每次
读取64 KiB，wizard再写canonical artifact。因此完整bytes仍经历relay write + relay read + artifact write，
且fixture/product默认root在本机Windows可能是C盘。Cargo只有bounded tail，没有canonical full artifact。

Cargo/wizard分别每100/25 ms `thread::sleep`；长进程持续占用一个general job worker并以10/40 Hz唤醒。
`join_output_with_poll`还把两次文件读取和poll嵌入job join。多个Play/export/compiler进程会与scene/import
工作争用worker，且能耗随process count/wakeup增长。

wizard每行仍同步调用UI output callback；下游若没有count+bytes+time backpressure，1M-line child仍可制造
1M次UI delta。core compile adapter把最多512×16 KiB行join为stdout/stderr bytes并写report，已有限但仍应只
传locator/digest/count/tail。

## Unreal源码依据与适配边界

- `WindowsPlatformProcess.cpp:850-882`集中CreateProcess、pipe handles与`PROCESS_INFORMATION` lifetime；
  `:1353-1368`把创建的process handle用于Job assignment。支持唯一platform spawn owner和direct handles。
- `MonitoredProcess.cpp:183-233`由同一monitor owner按terminate/read-final/close-pipe/return-code顺序完成；
  支持显式terminal phases，不支持Zircon现在各caller自行拼接tree/pipe/artifact状态。
- `MonitoredProcess.cpp:237-260`本身仍用sleep loop和per-process runnable。Zircon不能复制这个调度算法，
  必须用Runtime11 shared process/I/O lane或Windows wait/readiness completion，避免每进程占worker/thread。

## 依赖有序优化计划

1. Runtime11定义`ProcessSpec`、session/generation和完整terminal receipt schema；Windows直接create/Job
   attach/primary-thread resume，Unix统一group/session owner。
2. Editor14提供shared wait/readiness与bounded output delta lane；process可运行数小时而不占general CPU
   worker，不按每session建立private thread。
3. Editor15让child直接写唯一canonical stdout/stderr artifact；同一stream stage维护digest/tail/UI delta，
   删除temporary relay、重复write/read和per-line无界fanout。
4. Editor04把Play迁到同一session ticket；stop/unload/cancel按generation并保留retryable cleanup state。
5. hard-cut `taskkill` steady path、Toolhelp primary-thread discovery、sleep polling、blocking Drop及双process owner。

## 量化验收

| matrix | 必须记录 | acceptance |
|---|---|---|
| children `1/16/100`，runtime `1s/1h`，unrelated Windows threads `1/1K/10K` | spawn/thread visits、handles、workers/threads、wakeups/context switches、CPU/power | unrelated thread visits=0；每session private worker/thread=0；tree owner=1；idle wakeups近0 |
| output `0/64KiB/1GiB`，lines `1/1M`，fast/slow exit | relay/artifact read/write bytes、chunk peak、tail/RSS、UI deltas/backlog、digest | full writes=1；relay bytes=0；working set与tail预算常数；UI delta count/bytes/time有界；artifact/digest完整 |
| terminate success/fail/retry，descendant/pipes slow close，Drop/shutdown | terminate/reap/pipe/artifact receipts、retry、blocking Drop wall、leaked handles/processes | receipt顺序确定；failure owner不丢失且可retry；blocking Drop=0；stale completion=0；leak=0 |
| F4 Play/export至少31次cold/warm可比run | WPR CPU/waits/file I/O/context switch、allocator/RSS、package power | 同机分布稳定；UI/main worker无process wait/I/O；CPU/RSS/power优于before且功能/输出一致 |

RenderDoc不适用于process/output CPU与I/O验收。

## 本轮静态门

- 3个Python模块18/18 tests通过；scoped `git diff --check`通过，仅有既有LF/CRLF提示。
- 10文件逐个rustfmt为5/10通过；5个RED均是现有import/assertion排版，其中4个为foreign dirty，
  本轮不改动。不能声称全slice format通过。
- 未运行19个Rust tests：多个fixture调用`std::env::temp_dir()`，可能写C盘，且managed validator已归档。
- 无current-source可执行文件，未运行WPR、allocator/RSS或功耗。切片继续pending，无commit/企微。
