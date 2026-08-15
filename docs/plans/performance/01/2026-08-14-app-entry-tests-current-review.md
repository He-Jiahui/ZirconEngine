---
related_code:
  - zircon_app/src/entry/tests
  - zircon_app/src/entry/runtime_entry_app
  - zircon_app/src/entry/runtime_library
  - zircon_runtime/src/core/runtime/config_store.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/runtime/02-core-spine-and-root-surface.md
  - docs/plans/zircon_runtime/runtime/03-schedule-and-frame-loop-alignment.md
  - docs/plans/zircon_runtime/runtime/10-dynamic-api-and-interface-convergence.md
  - docs/plans/zircon_runtime/runtime/12-input-stack-and-action-mapping.md
reference_sources:
  - dev/UnrealEngine/Engine/Source/Runtime/Launch/Private/LaunchEngineLoop.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Projects/Private/PluginManager.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/ApplicationCore/Private/GenericPlatform/AsyncInputConsumer.cpp
tests:
  - current-source hash stability 44/44 passed
  - direct rustfmt 44/44 passed
  - 78 tests inspected: 42 behavior and 36 source-shape
  - managed Windows zircon_app build failed after 324.2 s with 6 current-source zircon_runtime errors; tests not run
  - product WPR matrix pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# App entry tests current-source性能审查（2026-08-14）

## 范围与分类

`zircon_app/src/entry/tests/**`当前 **44/44** 个Rust文件、**4,767** 行、**4,488** 个非空行、
**78** 条`#[test]`已逐文件完整阅读；直接`rustfmt +1.94.1 --edition 2021 --check
--config skip_children=true`为44/44通过，复核前后SHA-256前缀44/44不变。9个文件有其它Session修改，
本轮只读、不覆盖。

78条测试分为两类：

- **42条行为测试**：`profile_bootstrap.rs` 28条、`builtin_engine_entry.rs` 12条、export bootstrap 2条；
  它们真实构造Entry/CoreRuntime、manager、plugin lifecycle、config、viewport或文件fixture。
- **36条源码形状测试**：读取或拼接生产`.rs`/Cargo文本，断言函数名、token顺序、调用次数或目录存在；
  device/input/source/surface/window lifecycle guards几乎全部属于此类。

行为测试是语义回归证据，但没有current-source Cargo执行结果，也没有wall/CPU/alloc/lock/energy指标；
源码形状测试连语义执行都未发生。两类都不能冒充产品性能验收。

## P1：源码测试把冗余config双写锁成合同

42条行为测试只验证bootstrap后的PlatformConfig、RenderProfileBundle、WindowDescriptor和manager状态，
没有任何一条要求同值写两次。唯一要求`self.store_entry_config(&runtime)?`恰好出现2次的是
`entry_config_storage.rs`的`include_str!`测试。

生产`ConfigStore::store`每次都会`serde_json::to_value`，随后获取Mutex并覆盖`Arc<Value>`；无相等性或
generation短路。故第二阶段至少重复3次typed serialization和3次锁内replace，Editor另有sandbox与可选
subsystem value覆盖。测试当前保护的是实现形状，不是“consumer观察同一startup generation”的合同。

处置归Runtime02/App entry：增加config generation、write count、serialized bytes和mutex wait测试计数；
注册激活期合法/非法写者，验证失败回滚和最终consumer值。若无合法写者，则bootstrap每key只写1次并删除
双写源码断言；若允许变更，只提交changed generation，禁止无条件全量reconcile。managed行为测试通过前
不直接删除。

## P0：事件、帧、surface与teardown性能门仍是false-green

source guards能确认`about_to_wait -> pump_frame_loop`、native present先于fallback、input/window event
委托顺序和teardown token存在，但无法证明：

- reactive idle 30/60秒没有隐藏tick、wake或`Poll`；
- 125/500/1,000 Hz输入下ABI/registry/session/manager锁按batch而非event增长；
- host request/input storm有entries/bytes/age上限，且main-thread p95受控；
- native帧capture/readback/fallback counter为0，resize不重复rebind或重建；
- event-loop、callback、session和process-log失败路径真正drain并输出artifact。

这些缺口已经分别归Runtime03/10/12、Render17和App F0报告；本测试目录必须消费同一counter/阶段ID，
不能继续新增token列表来“证明”热点被解决。

## P5：测试自身存在重复源码拼接，但不抢占产品优化

`runtime_entry_input_guards/sources.rs`把converter、drag/drop、host request、IME、keyboard、pointer、window、
gamepad等完整源码反复`.join("\n")`；surface/window/source helpers也各自重新拼接。成本约为
O(被拼接源码总字节 x 调用测试数)，只发生在测试进程。`entry_tree.rs`手工枚举约120个路径却不是当前
159个entry文件的完整集合，新leaf仍可漏过；它也不能代替current-source哈希账本。

非验证类代码优先，因此本轮不为这些测试微优化。后续把目录owner交给统一convention/manifest检查，
行为留在owner模块单测；若仍需源码分析，单次构建共享fixture或使用Rust解析器，不在每条测试拼接整棵源码。
`std::env::temp_dir()` fixture必须由managed validator把`TEMP/TMP`定向D/E/F盘；本任务不运行会落C盘的
裸测试命令。

## Unreal源码依据

- Unreal `LaunchEngineLoop.cpp:1474-1552`直接发布process/idle CPU、game/render/RHI/GPU time、input
  latency、free memory和target frame budget；`1734-1742`又在最早PreInit建立boot profiling。它以真实
  counter和阶段数据验收，不以函数名存在替代运行行为。
- `PluginManager.cpp:2034-2084`对一个pending generation完成一次plugin configure后清空集合，后续
  loading phase消费既有状态；Zircon测试应断言plan build/write generation次数，而不是断言某段源码出现。
- `AsyncInputConsumer.cpp:50-121`由consumer真实drain queue并按输入语义合并绝对axis；Zircon的input
  验收必须提交实际burst、queue、coalesce和latency数据，token顺序不能证明算法规模。

## 迁移与验收计划

| Owner计划 | 必须替换的false-green | 验收证据 |
|---|---|---|
| Runtime02 + App entry | config双写源码次数 | per-key generation/write/bytes/lock counter，激活写者与最终consumer行为 |
| Runtime03 | event-loop/frame-loop token顺序 | 30/60秒idle与continuous/reactive/headless实际pump/wake/control-flow counter、CPU/energy |
| Runtime10/12 | input/host-request源码拼接 | 1k/10k/100k burst、batch/pages、锁次数、queue age、顺序barrier、p50/p95/p99 |
| Runtime10 + Render17 | surface/teardown token顺序 | native/fallback/readback/rebind counter，RenderDoc，failure injection和owner drain artifact |
| App test owner | 手工entry tree与重复include/join | convention/manifest完整集合检查；owner模块行为测试；测试fixture全在D/E/F盘 |

动态验收先运行current-source Windows managed `zircon_app` focused test batch，再以同一source fingerprint
运行cold/warm startup、idle、input storm、native/forced fallback和teardown WPR/xperf矩阵；记录CPU、I/O、
alloc/RSS、lock、CSwitch/ReadyThread、GPU与energy。RenderDoc只用于surface-ready后的GPU边界。

本轮managed validator的dry run只生成D盘target命令，未执行Cargo。首次实际提交因自建E盘空temp目录
未登记而在预检拒绝；该空目录随后由coordinator删除，artifact audit恢复为0。`TEMP/TMP`改指向工作区
现有E盘`.codex/tmp`后，后续build-only `zircon_app`矩阵在D盘managed target上运行324.2秒并以exit 101
失败：212条warning、6个`zircon_runtime` current-source编译错误，分别落在并发修改中的query-state cache、
resource management generation、neutral graph buffer view与UI event routing。它们均为本Session范围外的
foreign dirty源码，本轮不覆盖；因此44条entry tests一条也未执行，WPR产品矩阵也未开始。managed作业已
释放，本Session未在C盘写入产物。

本轮没有源码修改：9个测试文件foreign dirty，且managed build失败、产品profile尚未恢复。44个文件继续留在
`pending.md`，不进入`review.md`，不形成提交或企微发布里程碑。
