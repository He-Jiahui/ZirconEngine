---
related_code:
  - zircon_app/src/entry/entry_runner/editor.rs
  - zircon_app/src/entry/entry_runner/editor
  - zircon_editor/src/ui/host/editor_operation_dispatch.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/runtime/10-dynamic-api-and-interface-convergence.md
  - docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md
  - docs/plans/zircon_editor/editor/08-tool-orchestration-and-commands.md
  - docs/plans/zircon_editor/editor/12-plugin-management.md
  - docs/plans/zircon_editor/editor/14-threading-and-job-scheduling.md
  - docs/plans/zircon_plugins/01-plugin-architecture-core.md
reference_sources:
  - dev/UnrealEngine/Engine/Source/Runtime/Launch/Private/LaunchEngineLoop.cpp
tests:
  - current-source hash stability 10/10 passed
  - direct rustfmt 10/10 passed
  - managed Windows Cargo and cold/warm WPR matrix pending
  - failure-path and teardown profile export matrix pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# App Editor entry current-source性能审查（2026-08-14）

## 范围与快照

`zircon_app/src/entry/entry_runner/editor.rs`及`editor/**`合计 **10/10** 个Rust文件、
**3,386** 行、**3,119** 个非空行、**81** 条`#[test]`已逐文件完整阅读；直接
`rustfmt +1.94.1 --edition 2021 --check --config skip_children=true`为10/10通过，复核前后
SHA-256前缀10/10不变。

| 文件 | SHA-256前缀 | 状态 |
|---|---|---|
| `editor.rs` | `56A104B995FB` | foreign dirty |
| `editor/{composition,project_automation,startup_diagnostics}.rs` | `C106F9B31857`、`8EA95C9D932F`、`CAC169E7EC0D` | 前两项foreign dirty |
| `editor/tests/{cli_operation,gui_startup,help}.rs` | `0DAFC343CF52`、`3BD5822A0C58`、`29F4E675F63D` | 后两项foreign dirty/untracked |
| `editor/tests/{host_config,mod,runtime_loading}.rs` | `9DBFCCE5C9B0`、`3BC9BFA796D7`、`0F6A196A734D` | 前两项foreign dirty |

6/10文件正由其它Session修改或新增，本轮只读，不覆盖其实现。以下是当前哈希快照的静态调用图，
不是产品性能验收。

## 当前启动图与已有正确边界

GUI路径先解析参数、准备/打开项目并投影Runtime/Editor插件，然后才启动profile capture；其后串行执行
App core bootstrap、EditorManager解析、native editor插件能力排序/激活、Runtime DLL加载、projectless
RuntimeSession和gateway创建，最后进入retained host。项目本身只由Editor owner打开一次，gateway session
保持projectless；这是正确的单项目authority边界，不能为了局部加速重新引入第二次project open。

`prepare_editor_startup_from_prepared_project`当前记录的结构计数为project open 1、project manifest clone 1、
runtime projection 1、editor projection 1。它避免重复打开项目，但仍把同一manifest分别投影为EntryConfig、
runtime registrations、editor registrations，随后又单独生成native editor registration/capability集合。
`EditorApplicationComposition::from_prepared_startup`还会clone整组runtime registrations，一份交给bootstrap、
一份留给linked runtime session。该问题与App plugin审查中的`PERF-MVP-427`是同一根因：应由一个
generation-owned compiled plugin plan供module、runtime、editor和host消费，而不是在入口层增加私有cache。

## P0：性能观测合同存在结构性缺口

当前capture在项目create/open、manifest clone和runtime/editor registration projection之后才开始；
`bootstrap`、manager解析、native插件激活、DLL加载和RuntimeSession创建中的任一`?`早退都会绕过
`stop_and_export_capture_from_env`。正常路径又在`drop(runtime_session)`和teardown failure收集之前停止capture。
因此成功报告不含关闭成本，失败报告可能完全不存在，GUI项目准备成本也始终不可见。

自动化、Commandlet和CLI operation在进入GUI capture分支前直接返回，也没有同口径的过程级采样。
这不是单纯“少一个marker”：当前数据无法回答冷启动时间究竟消耗在项目I/O、插件投影、DLL/session、
首帧还是退出drain，任何结构性并行化结论都会缺乏因果证据。

修正目标是process-lifetime capture owner：在确定help/运行模式后、该模式第一次项目或运行时I/O之前启动，
以RAII/finally覆盖成功和所有错误出口；只有host、gateway、session及其teardown完成后才停止导出。
阶段ID至少包含`args/project-prepare/plugin-plan/core/dll/session/gateway/host-first-present/host-run/
session-drain/export`，并携带project/plugin/scene规模。help仍应早退且不启动引擎采样。

## P1：只读operation可能过度初始化

`run_editor_operation`对invoke/list/history一律bootstrap完整core，创建1280x720 default level、EditorState与
controller，加载Runtime DLL，创建session/gateway并attach。下游`ListOperations`只锁command registry、过滤
descriptor并构造JSON；`QueryOperationHistory`只读取最多128条transaction记录。静态图说明存在分阶段启动
机会，但不能直接删除runtime初始化：可用操作集合、能力门和history owner可能依赖完整manager/plugin plan。

先对0/1/100/1,000 operations比较full与staged候选的DLL I/O、scene construction、alloc、wall/CPU和输出；
只有enabled descriptors、required capabilities、history generation、JSON和错误完全一致时，list/history才走
“plugin plan + manager registry/transaction store”最小服务集，invoke继续按operation声明的依赖启动。

## P2/F5：自动化报告为显式O(scene nodes)输出

project automation只创建一次composition并复用retained host bindings，这是正确边界；完成后会把完整
`scene_nodes: Vec<NodeRecord>`移入snapshot再JSON序列化，时间和输出内存至少为O(scene nodes + output bytes)。
这是显式headless证据路径，不是GUI稳定帧热点。先测1/1K/100K nodes的snapshot/build/serialize/write p50/p95、
峰值RSS和artifact bytes；若成为实际瓶颈，再增加summary/page/detail或流式artifact合同，默认完整证据不得静默
截断。81条测试中有 **14** 条通过`include_str!`断言生产源码形状；它们不能替代启动失败导出、teardown、
单项目open、插件generation和operation输出一致性的行为测试。

## Unreal源码依据

- `LaunchEngineLoop.cpp:1734-1760`在PreInit最早阶段初始化boot profiling，并用scope-exit发布阶段完成；
  `2525`项目加载、`2601/2612`TaskGraph/线程池、`3191-3223`RHI/PSO/shader cache和`3469-3495`
  PreInit后半段分别计时。这支持“从首次真实工作开始、按owner分阶段观测”，而不是项目打开后才采样。
- `LaunchEngineLoop.cpp:2238-2258`先明确Commandlet运行模式及render/audio许可，再进入其模式所需服务；
  Zircon也应让operation/automation声明服务集，不能靠无依据删初始化或让所有模式默认等价于完整GUI。
- `LaunchEngineLoop.cpp:4880-5108`用scope-exit保证Init完成通知，并拆分engine create/init/start；
  `5120-5357`把Exit、asset compiling、streaming/input/render、module unload与TaskGraph shutdown纳入有序退出，
  `7069-7177`的AppPreExit同样有boot timing。这直接反证Zircon在session drop前停止采样的现状。

参考引擎提供阶段、模式、owner和有序退出原则，不提供Zircon的毫秒阈值；阈值必须来自同机、同构建、
同项目规模的current-source数据。

## 跨计划交付与验收

| Owner计划 | 必须解决的合同 | Performance验收 |
|---|---|---|
| Runtime10 | versioned session startup/teardown阶段与失败可观测；DLL unload前完成drain | 每个注入失败点仍有E/F盘artifact；capture覆盖session drop，callback/worker归零 |
| Editor01/14 | GUI、automation、operation按服务依赖分阶段启动，纯I/O/parse可交给已有有界worker | 主线程blocked、CSwitch/ReadyThread、queue depth/age、worker overlap；不新建入口私有pool |
| Editor08 | list/history的最小服务集与invoke依赖声明 | full/staged输出、能力门、history generation和错误逐字节一致；冷/warm p50/p95 |
| Editor12 + Plugins01 | `PERF-MVP-427`单generation compiled plugin plan | 0/1/100/1K插件：plan build<=1/generation，registration visits/clone bytes/activation wall可归因 |
| App/Performance01 | process-lifetime capture owner和统一阶段ID | GUI create/open/welcome、automation、invoke/list/history成功/失败都导出；start-to-first-present及teardown可分解 |

动态矩阵使用E/D/F盘managed build与artifact：cold/warm各5次，项目插件0/1/100/1K，scene nodes
1/1K/100K；WPR/xperf记录CPU sample、File/Disk I/O、CSwitch/ReadyThread、working set与energy，内建/Tracy
counter对齐阶段ID。RenderDoc只用于GUI surface-ready后的首帧/稳定帧GPU边界，不解释项目I/O或CLI CPU成本。

本轮没有源码修改：foreign-dirty ownership和缺失的current-source动态证据都不允许此时进行结构切换。
managed Cargo、WPR/xperf、failure/teardown export和operation staged parity未完成，本组继续留在
`pending.md`，不进入`review.md`，也不形成可提交/企微发布的验收里程碑。
