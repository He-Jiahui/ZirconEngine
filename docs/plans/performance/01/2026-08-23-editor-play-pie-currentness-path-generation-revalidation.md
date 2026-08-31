---
related_code:
  - zircon_editor/src/core/play
  - zircon_editor/src/ui/host/editor_event_execution/menu_action.rs
  - zircon_editor/src/ui/host/editor_host_event_controller.rs
  - zircon_editor/src/ui/workbench/project/project_root_path.rs
  - zircon_runtime/src/asset/project/paths.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/performance/02-unreal-aligned-engine-system-hard-cutover.md
  - docs/plans/performance/01/2026-08-15-editor-play-pie-lifecycle-current-architecture-review.md
  - docs/plans/zircon_editor/editor/04-pie-and-simulation.md
  - docs/plans/zircon_editor/editor/14-threading-and-job-scheduling.md
  - docs/plans/zircon_runtime/runtime/03-schedule-and-frame-loop-alignment.md
  - docs/plans/zircon_runtime/runtime/10-dynamic-api-and-interface-convergence.md
  - docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
  - docs/plans/zircon_plugins/01-plugin-architecture-core.md
reference_sources:
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Private/PlayLevel.cpp
tests:
  - current play tree 39 of 39 Rust files and 56 inline tests reconciled
  - 16-file delta from the 2026-08-15 frozen baseline reviewed against current source
  - focused rustfmt for 39 of 39 files and scoped diff check passed
  - Play Python contracts 15 of 15 passed after one missing Rust coverage contract was RED
  - two added play-log Rust behavior tests remain unexecuted because current-source managed Cargo is unavailable
  - WPR xperf allocator power and RenderDoc pending
doc_type: implementation-evidence
status: currentness_revalidated_structural_plan_dynamic_blocked
---

# Editor Play/PIE currentness与路径generation复验（2026-08-23）

## 范围与当前性

2026-08-15报告冻结`core/play/**` 37/37文件、5,128行。当前目录为**39/39 Rust文件、5,261行、173,314 B、
56 tests**，path+line+file-SHA manifest为`c152d36903c2bb5fd679445ec14e502655991c03904522edffab4614e2003ae3`。

本轮以旧37文件逐文件基线为底，复读其后**16个变化/新增文件**的current diff与所有变化生产函数；增量集合当前3,548行、
119,308 B、53 tests。新增文件是`process_backend/error.rs`与`snapshot/error.rs`，其余变化集中在controller、native activation、
process backend、snapshot与测试。Play生产文件已有他人dirty，本轮没有覆盖或回退；只在外层host controller补回两个日志归属
Rust测试。

39/39 focused rustfmt和Play scoped diff check通过。4组Python合同共15项，首次执行只有“attached/unattached backend日志归属必须有
Rust回归测试”RED；补测试后15/15 GREEN。新增Rust测试未由Cargo执行，不冒充行为动态通过。

## 增量源码判定

### 正确性改进成立，但不改变主线程结构

- `on_build_finished`在activation/backend start失败时从Building恢复Edit并发布Building -> Edit，修复command consumer卡在
  Building；stop/poll后plugin deactivation失败仍可能在process已终止后报告Playing，旧P0.2未修。
- native activation从`Arc<NativePluginLiveHost>`硬切到`NativePluginHostHandle`，统一live-host identity；discovery/load/enter/exit
  仍在controller transition与activation transition锁内同步执行，旧P0.1/P0.4未修。
- persisted scene改为validated `RelPath`，snapshot/dynamic-scene/install错误改为typed error，process参数使用project-relative
  scene并以项目根为working directory。跨边界不再接受absolute scene path，这是应保留的安全/一致性改进。
- output delta只有capture error Debug与rustfmt，1,024 lines、4 MiB queue、64 KiB line、64 lines/256 KiB/2 ms drain等当前
  bounds不变；每session两个reader thread和无deadline join仍open。

### P0仍是完整同步事务，且项目根被filesystem解析两次

菜单Play仍在workbench shell锁域内执行`project_scene -> World clone -> DynamicScene::from_world -> pretty JSON ->
request_play`。controller持`transition_gate`调用native plugin activation和backend start；backend持`active` mutex执行snapshot
materialize、write/fsync/rename、argument projection、process spawn和pipe/thread建立。

current menu先调用`project_root_path()`；它用`ProjectPaths::resolve_existing_path`做absolute normalization与physical canonicalize。
同一个PathBuf进入`PlaySnapshotStore::materialize`后又调用`ProjectPaths::from_root -> resolve_path`，再次查找deepest existing
ancestor并`canonicalize_physical_path`，随后才构造9个project-owned PathBuf。静态计数是**filesystem project-root resolution/
Play request 2**，终态应为1。别名/escape安全要求不能通过“PathBuf看起来绝对”来跳过第二次检查。

正确hard-cut是project open generation拥有`ResolvedProjectPath + ProjectPaths`，`PlayStartRequestGeneration`只携带该immutable
identity/handle；plugin activation、snapshot store与process command借用同一generation。菜单不再重新解析，snapshot store也不
接受raw PathBuf。该改动必须和UI锁外Play preparation一起完成，避免只省一次canonicalize却保留World clone/fsync/spawn主瓶颈。

### 其他加载期重复

`ProcessPlayBackend::start`为了诊断先构造`Vec<OsString>`，再为每项转String并join；`PlayChild::spawn`又通过`configure()`投影
同一参数。规模固定且只在start发生，不是独立MVP热点；最终process ticket应拥有一次构建的structured launch plan，诊断从该plan
borrow/format。`for_current_install`在editor startup对install root做physical resolve一次，属于正确冷启动身份建立，不应缓存raw
current_exe文本或绕过junction解析。

## Unreal源码依据

`PlayLevel.cpp:1002-1035`把Play请求保存到next tick并提供cancel；`:1116-1128`每次attempt后清除queued request；
`:1138-1226`在一个session flow中处理旧session停止、save validation与目标分支，`:1419-1423`只排队stop intent。可转移原则是
UI操作发布一个明确request/session identity，重CPU/I/O/foreign stage按该identity执行并只提交匹配generation的receipt；不是照搬
Unreal仍可能存在的game-thread重工作，也不是以记忆中的UE毫秒数作为Zircon验收。

Zircon额外需要保留`RelPath`和physical `ResolvedProjectPath`双视图优势：process/ABI用稳定project-relative path，host-owned
generation保留operation/display identity；reload/cancel不重新从字符串猜项目身份。

## 量化验收

矩阵保持scene entities 1/1k/100k、artifact 1 KiB/64 MiB/1 GiB、plugins 0/1/49、start/stop/reload 1/100、path aliases
none/junction/SUBST/symlink。新增记录absolute/ancestor/canonicalize calls与wall、ProjectPaths builds、PathBuf/String alloc bytes；并继续
记录World clone/projection/JSON bytes、shell/controller/backend/plugin lock wait/hold、write/fsync/spawn、threads、RSS和energy。

验收要求project identity resolution<=1/project open generation、per Play request=0；UI shell/transition/backend lock内filesystem
resolution/World clone/serialization/I/O/plugin foreign callback/spawn=0；unchanged Play artifact build=0；stale completion commit=0；
process/plugin/pipe/snapshot ownership直到terminal receipt，inactive poll locks/syscalls=0。路径alias parity=100%，escape/absolute scene
仍拒绝。

当前没有current-source executable，故WPR/xperf/power/RenderDoc均不运行；RenderDoc只能在F4可启动后验证首个稳定Play frame的GPU
证据，不能证明上述CPU/锁/I/O问题。current Cargo与新增Rust测试也未执行。39/39静态currentness不等于动态验收，本轮不迁入
`review.md`、不提交milestone、不发送完成企微。
