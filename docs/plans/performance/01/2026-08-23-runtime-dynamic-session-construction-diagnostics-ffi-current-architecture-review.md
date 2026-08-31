---
related_code:
  - zircon_runtime/src/dynamic_api/session/construction.rs
  - zircon_runtime/src/dynamic_api/session/diagnostics.rs
  - zircon_runtime/src/dynamic_api/session/error.rs
  - zircon_runtime/src/dynamic_api/session/ffi.rs
  - zircon_runtime/src/dynamic_api/session/registry/session_store.rs
  - zircon_runtime/src/runtime_diagnostics
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/performance/02-unreal-aligned-engine-system-hard-cutover.md
  - docs/plans/performance/01/2026-08-23-runtime-diagnostics-domain-generation-availability-current-review.md
  - docs/plans/performance/01/2026-08-23-runtime-dynamic-session-plugin-event-linked-host-current-architecture-review.md
  - docs/plans/performance/01/2026-08-23-runtime-dynamic-session-project-world-sync-reload-current-architecture-review.md
  - docs/plans/zircon_runtime/runtime/06-plugin-surface-and-lifecycle.md
  - docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md
  - docs/plans/zircon_runtime/runtime/10-dynamic-api-and-interface-convergence.md
  - docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
  - docs/plans/zircon_runtime/render/17-performance-and-profiling.md
reference_sources:
  - dev/UnrealEngine/Engine/Source/Runtime/Projects/Private/PluginManager.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/RenderCore/Private/RenderingThread.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Private/Stats/StatsData.cpp
tests:
  - current dynamic session construction/diagnostics/error/ffi 4 of 4 Rust files and 5 inline tests reviewed
  - supporting session store, runtime diagnostics collector and action/output call chains reviewed
  - M0 static performance contract 2 of 2 passed after RED
  - focused rustfmt 1.94.1 plus scoped diff check passed
  - current-source Cargo, startup/action scale, WPR, allocator, power and RenderDoc product traces pending
doc_type: implementation-evidence
status: m0_static_complete_dynamic_blocked
---

# Runtime dynamic session构建、诊断与FFI调度边界复审（2026-08-23）

## 范围与当前性

已逐行复读`dynamic_api/session/{construction,diagnostics,error,ffi}.rs`当前**4/4**个Rust文件。
实施前合计**1,599行、59,679 B、5 tests**；M0后为**1,598行、59,628 B、5 tests**，按
`path|lines|file-hash`生成的manifest SHA256为
`cff8f89c23297ad18d92b58503d14bd808aec0d1002ed576a0384cc3717692e2`。同时沿调用链复核session
store、runtime diagnostics collector、render/World action和owned output finalize/commit。`error.rs`与
`ffi.rs`已有其他Session的格式改动，本轮只读并保留；M0只落在原先干净的`construction.rs`。

## 当前源码判定

### Session create仍把完整启动管线压在FFI调用线程

`create_session`同步进入`RuntimeDynamicSession::new/build`。一次调用依次完成linked plugin/module
candidate、`CoreRuntime`构造、module register/activate、input manager解析、render bridge、project asset
open、navigation load、startup scripts、scene load、plugin world plan、reload queue、全部runtime UI root load、
World扫描首个Cube/orbit target及operation handler安装，最后才插入session registry。源码已有phase profile与
diagnostic log，但没有可取消startup ticket、progress/deadline或明确的main/render/worker affinity。

这不是“把build丢进任意线程”就能解决的问题。plugin/project owner尚存在平行catalog和聚合
`ProjectManager`快照；若先并行化，会把重复build、I/O与生命周期竞态扩散到更多线程。结构顺序必须是：
先由`PERF-MVP-629/638`发布immutable plugin/project generation candidate，再由Runtime11调度有界阶段，
最后只在必要线程做短activation commit。

### 单一session mutex覆盖JSON、World、render和长诊断工作

`with_session_result_finalized`在action闭包期间持session mutex，仅把ABI allocation finalize移到锁外；
committed变体随后重锁完成commit/rollback。当前FFI路径因此把以下不同性质工作串在同一临界区：

- `tick_frame`、`present_viewport`与`capture_frame`的simulation/render/readback；
- `query_world`的request decode、World投影和response JSON encode；
- accessibility snapshot后的整树JSON encode；
- profile control的snapshot与response encode；
- plugin event/world watch的request decode，以及各自page选择/编码。

per-session有序语义是正确约束，但“有序”不等于所有decode、projection、serialization与foreign/render wait都
必须持同一mutex并同步阻塞编辑器UI调用者。该结构与`PERF-MVP-597`的active Play主线程FFI/JSON阻塞是同一
根因。目标不是开放并发修改World，而是一个per-session ordered bounded ticket lane：request decode在锁外，
锁内只seal generation/handles和提交状态，重工作按显式affinity执行，完成结果按session+generation校验后短
commit；每session in-flight受硬界，stop/reload/shutdown可取消，过时代结果丢弃。

### 完整诊断查询在session锁内重复物化宽数据

`runtime_diagnostics_response`在session mutex内调用`collect_runtime_diagnostics`：重新resolve managers，取得
owned `query_stats()`，把render stats展开为约541条series写入store，再snapshot完整history并clone profiling
snapshot；dynamic API随后又把全部series/history映射为interface DTO并编码JSON。该路径是显式查询而非每帧
周期任务，旧“每秒深clone history”结论已失效；但一次显式大查询仍能长时间占用session锁并阻塞tick、input和
present。

该问题继续归`PERF-MVP-324/418`：render/physics/animation owner发布generation-owned sealed diagnostics，
API提供domain mask、summary/detail和if-newer；session锁只读取稳定Arc/receipt，DTO projection与JSON encode在
锁外完成。不得在dynamic session再建立一份私有stats cache。

### Error枚举不是稳态热点，output所有权仍需ABI收口

`error.rs`的path/String格式化只发生在错误路径，本轮未发现值得脱离真实trace优化的稳态算法。更重要的是
`PERF-MVP-574`：App在检查status后才接管owned output，ABI尚未冻结error-after-output的exactly-once free；
session destroy失败还可能留下registration/proxy。应以RAII output guard、明确错误合同和有界teardown终态
解决，不能通过压缩错误文本掩盖所有权问题。

## Unreal源码依据与统一结构

Unreal `PluginManager.cpp:2034-2085`仅在pending plugins非空时构造一次discovery/configure context，处理后
清空pending集合；`2884-2988`按显式loading phase推进并维护单调completed phase。可转移原则是“一份candidate
generation + 显式阶段 + 一次activation”，不是复制UE的全局manager或阶段常量。

Unreal `RenderingThread.cpp:421-505`把thread mode、affinity、task graph attach与ownership transition明确化，
切换前以fence/flush封住边界；`679-720`把flush作为带trace的显式阻塞边界。它支持Zircon把render/main affinity
声明在统一调度器和ticket上，而不是让任意FFI caller在session mutex内隐式承担全部render工作。flush不是日常
JSON/diagnostic工作的模板，也不能被理解为“主线程等待合理”。

Unreal `StatsData.cpp:627-646`由专门owner限制history frames；`895-993`在stats thread聚合/condense frame
history。可转移原则是stats publication/history有单一所有者、consumer读取已封存代际；Zircon不复制UE线程
拓扑和数值预算，最终预算必须由本引擎WPR/allocator数据决定。

统一目标数据流为：

1. Runtime06/04发布immutable plugin/project generation candidate，construction只消费stable handles。
2. Runtime11用共享有界task graph执行startup phases；每阶段有affinity、cancel、deadline、progress和generation。
3. Runtime10建立每session唯一ordered ticket lane；World/render保持串行owner，但decode、encode和只读projection
   不持session lifecycle mutex。
4. Runtime07/Render17发布domain generation与summary/detail snapshots，history owner唯一且总bytes/age硬有界。
5. App/Runtime10冻结owned output与teardown合同，所有成功/失败路径exactly-once release。

## 本轮M0

`construction.rs`原先把`linked_extensions.registry.modules()`整组`to_vec()`，随后注册时又逐模块clone。
`ModuleDescriptor`包含owned identity/dependency/config字段，因此P个linked modules在session startup发生**2P**次
descriptor深clone。中间Vec没有跨phase lifetime需求。

本轮让construction直接遍历registry borrowed slice，仅在`register_module`所有权边界clone一次。静态深clone
从**2P降为P**，中间`Vec<ModuleDescriptor>`分配从**1降为0**；模块顺序、错误映射、activation和registry
所有权不变。未进一步改变`register_module`合同，因为那属于唯一compiled plugin generation的结构迁移。

`tools/tests/test_runtime_session_linked_module_registration_m0_performance_contract.py`先得到**0/2 RED**，实施后
**2/2 GREEN**；测试27行、872 B、SHA256
`7d33d6825100fdc9317e3b1e560c09a0747f70d81097c112384848263c66df44`。focused
`rustfmt +1.94.1 --edition 2021 --check`与scoped diff check通过。current-source Cargo不可执行，现有5条
Rust tests没有运行；2P到P是源码所有权计数，不冒充wall time、RSS或功耗结果。

## 动态验收矩阵

| owner | matrix | 必须采集与验收 |
|---|---|---|
| construction | plugins/modules/assets/scripts/UI roots 0/1/100/1K/10K；cold/warm/1% change；success/failure/cancel | phase wall、candidate/generation builds、descriptor/project clone bytes、I/O、main/worker occupancy、commit lock、startup p50/p95/RSS/energy；stable build=0、每accepted generation build/publish<=1、失败publish=0 |
| session action lane | actions tick/input/query/capture/present/diagnostics；provider 0/1/16 ms/10 s；threads 1/16；queue 0/1/N | UI foreign/JSON wall、in-flight/queued count+bytes+age、session lock wait/hold、cancel/stale/order、context switches；UI重工作wall=0、per-session in-flight<=1、队列硬有界、无loss/dup/reorder |
| diagnostics | domains hidden/render/physics/animation/full；history 1/64/1K；same/changed generation；poll 0/1/60/240 Hz | manager resolve、stats build/query、series writes、history/DTO/JSON clone bytes、lock hold、Arc hits、p95/RSS；hidden=0、same generation build<=1、多consumer deep clone=0、JSON不持session锁 |
| output/teardown | success/error/invalid JSON/free failure；output 0/1 KiB/64 MiB；destroy failures 1/1K/100K | allocations/frees/leaked bytes、registry/proxy/quarantine count+bytes+age、callback after detach；free=owned outputs、double free=0、leak=0，或终态隔离有硬界且可观测 |

同一硬件、电源计划、foreground、frame cap与fixture至少运行三次并报告median/range及profiler overhead。
WPR/ETW负责CPU、thread/wake/lock/context switch/I/O/power，allocator负责clone/RSS。RenderDoc只在F2/F4验证
capture/present改造后的像素、draw/pass/upload/present parity，不作为CPU或功耗结论。current-source binary尚不
可得，本切片继续留在`pending.md`，不提交milestone、不发送完成企微。
