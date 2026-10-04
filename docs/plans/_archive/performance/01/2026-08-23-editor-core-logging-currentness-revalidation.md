---
related_code:
  - zircon_editor/src/core/logging
  - zircon_editor/src/core/context/builder.rs
  - zircon_editor/src/core/script_build/diagnostics_sink.rs
  - zircon_editor/src/ui/host/editor_activity_log.rs
  - zircon_editor/src/ui/host/editor_event_runtime_access
base_reports:
  - docs/plans/performance/01/2026-08-16-editor-core-logging-current-architecture-review.md
  - docs/plans/performance/01/2026-08-16-editor-core-logging-protected-plan-routing.md
owner_plans:
  - docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md
  - docs/plans/zircon_editor/editor/13-script-compilation-management.md
  - docs/plans/zircon_editor/editor/14-threading-and-job-scheduling.md
  - docs/plans/zircon_editor/editor/17-editor-services-and-recovery.md
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
reference_sources:
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Private/Misc/OutputDeviceFile.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Private/Misc/OutputDeviceRedirector.cpp
  - dev/UnrealEngine/Engine/Source/Developer/MessageLog/Private/Presentation/MessageLogListingViewModel.cpp
  - dev/UnrealEngine/Engine/Source/Developer/MessageLog/Private/UserInterface/SMessageLogListing.cpp
tests:
  - tools.tests.test_editor17_activity_log_projection_contract
  - tools.tests.test_editor17_play_log_routing_contract
doc_type: currentness-revalidation
status: static_current_revalidated_dynamic_blocked_structural_cutover_required
---

# Editor core logging currentness重验（2026-08-23）

## 当前冻结与结论

| scope | Rust文件 | physical lines | bytes | tests | ordered path + NUL + raw bytes + NUL SHA256 |
|---|---:|---:|---:|---:|---|
| `zircon_editor/src/core/logging/**` | 13/13 | 1,889 | 59,980 | 16 | `405b501147771b555850f1885dc8bcdd036f8922da72d57b13eb2c8bf7af2aca` |

13/13文件和16个测试已完整复读。相对2026-08-16报告，生产逻辑未变；最近源码漂移仅为3处
测试中trait-object `Arc`构造从显式`Arc::clone`改为`clone`。因此旧报告的核心P0仍成立，且
`PERF-MVP-644`仍只存在于路由报告，尚未进入Performance01主计划。

当前模块已有可保留的基础：内存store同时按条目数与估算字节限制；event queue也有条目/字节
上限和resync discontinuity；sequence在clear后不复用；source、jump和Play identity为强类型；
event callback在event-dispatch mutex之外调用，允许受控reentry。这些正确性合同不能在优化中退化。

## 当前结构性瓶颈

### P0：生产线程在全局锁内同步执行每条日志文件I/O

`EditorLogService::emit`持有唯一`emission` mutex，先写memory store，再调用rolling sink。后者对
每条记录执行`create_dir_all`、整行escape/format、`metadata`、`OpenOptions::open`、`write_all`和
`flush`。文件或存储设备停顿会直接串行阻塞所有runtime、editor、plugin、compiler/import和Play
producer；现有测试还把“sequence分配和文件append必须由同一锁串行”固化为合同。

锁释放后，抢到dispatcher的producer仍在自己的线程同步drain event sink。queue虽有界，却没有把
sink延迟从producer隔离。script diagnostics逐row调用`emit`，所以1M诊断即使memory最终只保留
2,048条，仍可能完成1M次格式化、锁竞争、文件flush和fanout尝试。

### P0：稳定UI读取重复全量扫描、clone、格式化和解析

`activity_log_console_output`每次先对store做完整filter scan并clone全部匹配`LogRecord`，再构造
第二层`ActivityLogView`，随后把每行格式化为`Vec<String>`并join完整console text，同时构造levels
和jump sequences。它被editor snapshot、status和reflection等宽路径调用，而不是只在日志generation、
filter或visible window变化时调用。

production context还为每条日志构造JSON并发布`EditorTopic::log()`，但精确调用点搜索只找到测试
subscriber；实际UI直接读取log service。这形成一个无人消费的逐条投影和另一条全量pull authority。
`record(sequence)`又是线性扫描。逻辑字节预算不覆盖`Arc`、queue、format buffer和UI字符串的RSS。

## Unreal源码依据与适配边界

- `OutputDeviceFile.cpp:476-532`惰性创建并保留archive及`FAsyncWriter`，不是每条记录重新打开文件。
  `:559-595`把格式化结果送入async writer；正常路径不逐行flush，逐行flush只属于显式
  `FORCELOGFLUSH`。`Flush()`在`:419-429`是命名的durability边界。
- `OutputDeviceRedirector.cpp:500-529`把buffer drain交给独立primary thread，并明确区分async与
  waiting flush；`:532-545`把drain作为可分析的CPU profiler scope。这支持“producer admission、
  writer drain、terminal flush”三段所有权。
- `MessageLogListingViewModel.cpp:31-68`由model change/filter change驱动filtered handles更新；
  `:392-414`维护handle数组并广播变化。`SMessageLogListing.cpp:38-53`把handles交给`SListView`，
  `:294-300`按需生成row，而不是在每次外层snapshot中拼接完整文本。

Zircon不应照搬Unreal全局对象、无限buffer或MessageLog的全量filter refresh。应保留现有typed identity、
有限字节预算和deterministic discontinuity，同时采用持久async writer、显式flush和可见窗口所有权。

## PERF-MVP-644结构目标

目标链固定为：

`LogIngressRange -> LogStoreGeneration -> DiagnosticsWriterBatch ->
LogPersistenceReceipt -> FilteredLogWindowGeneration -> RetainedConsoleRowDelta`

1. producer只执行有界memory admission，按range分配sequence并返回admitted/dropped/discontinuity receipt；
2. store暴露immutable generation、cursor/range和O(1) retained-sequence index；disk或sink延迟不得延长
   store critical section；
3. Runtime11/Editor14拥有唯一recursion-safe diagnostics I/O lane，保留一个active segment，按条目、
   字节、年龄和deadline批量format/write；只在byte/age/fatal/shutdown边界flush；
4. Editor13、import/process/plugin等burst producer使用bounded batch ingress，禁止N rows获得N次锁并
   发出N次flush；
5. Editor17发布typed log generation/range invalidation；EditorUI08按filter generation和visible加
   overscan窗口生成row，稳定宽snapshot只引用cached generation；
6. typed consumer接通后hard-cut无人消费的逐条JSON bus projection，不能保留双authority；
7. fatal/shutdown只在一个显式有界terminal phase等待，并记录producer wait、queue age/RSS、batch、
   write/flush、UI visits/clones/formatted bytes和visible rows。

## 量化验收

| matrix | 必须记录 | acceptance |
|---|---|---|
| producers `1/16/64`，rows `1/1K/1M`，payload `64B/8KiB`，disk/sink stall `0/10ms/1s` | admission wait、lock hold、queue rows/bytes/age、dropped ranges、RSS | producer disk/flush/sink work为0；进入overflow policy前wait与stall无关；order和discontinuity确定；资源有界 |
| segment `1KiB/32MiB`，day/rollover，normal/fatal/shutdown | create-dir、metadata/open、write batch、flush、terminal latency | active open不超过1；metadata/open接近segment数；normal不逐row flush；failure receipt和terminal flush有界 |
| retained `1/2,048/100K`，changed `0/1/1%`，filters `1/6`，visible `20/100`，60 Hz | rows visited/cloned/formatted、joined bytes、invalidations、sequence probes | stable scan/clone/format为0；changed work接近affected加visible rows；sequence lookup O(1) |
| F0/F4 diagnostics burst，至少31次可比cold/warm run | WPR CPU stacks、CSwitch/mutex wait、file I/O/flush、allocator/RSS、package power | current-source结果跨run稳定；无producer I/O热点；与同payload/rate/storage/power-plan的本地Unreal经验值同量级 |

RenderDoc不用于证明本模块CPU/I/O改进；只有console row rendering发生结构变化时，才用于验证像素、
draw/resource和overdraw parity。

## 本轮静态门与阻塞

- 两个Python契约4/4通过；`rustfmt --edition 2021 --check`为13/13通过。
- 未修改production：局部去掉flush、移动mutex或删除bus会破坏durability/order/resync，必须按上述
  owner顺序完成hard cutover。
- 未运行Rust/Cargo测试：managed validator session已归档；现有Rust测试使用`std::env::temp_dir()`，
  在本机可能写入C盘，因此本轮没有执行。
- 没有current-source可执行文件，WPR、allocator、RSS、功耗和RenderDoc均未运行；模块保持pending，
  不得写入`review.md`，不触发里程碑commit或企微通知。
