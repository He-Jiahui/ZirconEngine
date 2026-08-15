---
related_code:
  - zircon_app/src/plugins
  - zircon_app/src/entry/engine_entry.rs
  - zircon_app/src/entry/builtin_modules.rs
  - zircon_app/src/entry/first_party_runtime_plugins.rs
  - zircon_app/src/entry/first_party_editor_plugins.rs
  - zircon_app/src/entry/entry_config.rs
  - zircon_runtime/src/builtin/runtime_modules/assembly
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_plugins/01-plugin-architecture-core.md
  - docs/plans/zircon_plugins/10-editor-integration.md
  - docs/plans/zircon_plugins/11-plugin-call-bridge.md
reference_sources:
  - dev/UnrealEngine/Engine/Source/Runtime/Projects/Private/PluginManager.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Launch/Private/LaunchEngineLoop.cpp
  - dev/bevy/crates/bevy_app/src/plugin_group.rs
tests:
  - current-source rustfmt 5/5 passed
  - managed Windows Cargo pending
  - WPR/xperf startup matrix pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# App插件组current-source性能审查（2026-08-14）

## 范围与快照

`zircon_app/src/plugins/**`当前源码 **5/5** 个Rust文件、**865** 行、**770** 个非空行、**12** 条`#[test]`已逐文件完整阅读；5/5通过`rustfmt +1.94.1 --edition 2021 --check`，审查时该目录及下列App入口锚点均无工作树改动。

| 文件 | 行/非空行 | SHA-256前缀 | 结论 |
|---|---:|---|---|
| `builder.rs` | 344/309 | `42D2A4098A87` | 启动期HashMap+order Vec、descriptor冻结与最终拓扑排序 |
| `groups.rs` | 53/43 | `3DBA2D907392` | 四种profile薄配置 |
| `groups/resolution.rs` | 72/65 | `9DB96A37ABA7` | 每次默认组重新执行runtime module assembly |
| `mod.rs` | 8/6 | `A0B0E6AF7634` | 导出边界 |
| `tests.rs` | 388/347 | `286D8C296115` | descriptor单次生成、disabled、嵌套与排序语义 |

交叉调用图已完整复读`engine_entry.rs`、`builtin_modules.rs`、`first_party_{runtime,editor}_plugins.rs`、`entry_config.rs`，并下钻Runtime assembly的`assembly.rs`、`feature_reports.rs`、`registration_reports.rs`、`target_modules.rs`。因此以下重复次数是当前源码调用图的确定下界，不是对局部文件的猜测。

## 当前调用图与量化下界

```text
BuiltinEngineEntry::for_*                                  entry generation
  -> builtin_modules_for_config*                          assembly A + sort A
     -> feature path: RuntimePluginCatalog build A
     -> App lifecycle state: RuntimePluginCatalog build B
  -> plugin_group_for_config(selection.modules)
     -> {Default|Dev|Headless|Minimal}Plugins::build
        -> resolve_builtin_plugin_group                   assembly B + sort B
     -> set/add selection modules over the baseline
     -> try_finish                                        descriptor once + sort C
```

在普通entry构造中，核心module assembly下界为 **2次**、activation order排序下界为 **3次**；带feature registrations的路径还至少完整建立 **2次** `RuntimePluginCatalog`。默认组产生的多数module `Arc`随后被selection同名覆盖并丢弃。此计数不包含Editor native discovery/host apply的既有重复，后者继续由`PERF-MVP-427`统一处理。

设启用module为`M`、依赖边为`D`、plugin registrations为`P`、feature registrations为`F`：当前入口至少重复遍历module/依赖全图，feature路径还重复物化`P+F` catalog及derived reports。实际wall、alloc bytes和功耗尚无current-source动态数据，不能据静态下界虚报收益。

## 性能判断

### P1：跨层重复编译插件计划，归并PERF-MVP-427

根因不是`PluginGroupBuilder`容器，而是App同时保留“Runtime module selection”和“profile plugin group resolution”两条权威装配路径；feature路径又在Runtime assembly与App lifecycle projection分别重建catalog。模块数小时绝对成本可能不高，但100/1000插件、Editor冷启动、动态provider与复杂feature dependency会按完整集合放大，且让诊断、可用性、extension与lifecycle存在代际漂移风险。

目标结构是每个project/entry generation只发布一个不可变`CompiledPluginPlan`（名称由owner计划最终确定），共同持有manifest generation、catalog、availability、feature dependency/extension report、ordered modules/descriptors和staged lifecycle plan。App、Runtime、Editor只借用或共享同一代际产物；不得再由App重建默认profile后覆盖，也不得从同一registration集合建立第二catalog。

### 不立项：局部builder微优化

`HashMap<String, PluginEntry> + Vec<String>`只在启动配置阶段使用；`try_finish`仅为enabled module生成一次descriptor。`add_group`确会先完成内组排序、外组再排序，但当前生产搜索无调用，仅测试使用。`module_keys`分配也只出现在测试/显式diagnostics。把字符串键改成`TypeId`会破坏动态模块/ABI/config依赖的稳定名称身份，且没有热路径证据；当前不改。

## 参考引擎依据

- Unreal `PluginManager.cpp:2884-2986`为每个显式loading phase进入一个带`TRACE_CPUPROFILER_EVENT_SCOPE`的`LoadModulesForEnabledPlugins`，先`ConfigureEnabledPlugins`，再按phase加载并维护单调`LastCompletedLoadingPhase`；`2773-2976`保留单plugin load/unload入口。`LaunchEngineLoop.cpp:3587-4996`由启动阶段驱动这些phase，`5318-5319`统一进入shutdown unload。`PluginManager.cpp:3708-3772`先反向模拟、再按phase反向卸载。应借鉴的是“一份enabled plugin状态 + 显式阶段 + 可观测生命周期”，不是复制其API。
- Bevy `plugin_group.rs:282-585`同样以map+order保存启动配置，`add_before/after`允许线性position，`finish`一次消费到App。这证明小规模builder不是帧热路优化对象；Zircon额外的问题是builder外存在第二套Runtime/App权威计划。

## 跨计划交付

| Owner计划 | 必须解决的合同 | Performance验收 |
|---|---|---|
| `zircon_plugins/01` | catalog/feature/extension/ordered module由单一generation owner发布 | 同generation catalog与derived projection各不超过1次 |
| `zircon_plugins/11` | load/activate/ready/quiesce/unload使用同一staged lifecycle plan，失败反向回滚 | phase顺序、失败点、反向卸载与诊断等价 |
| `zircon_plugins/10` + Editor startup owner | Editor native/runtime/editor registrations只投影一次并共享给host | discovery/load/entry/contribution各不超过1次/project generation |
| App/Runtime entry owner | 删除“selection后再建默认组并覆盖”的第二装配路径 | module assembly、activation sort、enabled descriptor generation各不超过1次/entry generation |

这些约束补充既有`PERF-MVP-427`，不另建重复根因编号，也不在本报告越权修改其它owner源码。

## 动态验收矩阵

1. 在owner边界增加仅用于profile/evidence的generation counter：manifest parse、registration projection、catalog build、feature/extension projection、module assembly、descriptor generation、activation sort、phase load/unload及clone bytes。
2. Windows受管current-source二进制执行Editor/Runtime/Headless/Minimal × 0/1/100/1000 plugins × cold/warm；WPR/xperf记录F0各phase wall/p50/p95、CPU sample、alloc、file I/O、线程wait/ready与进程能耗。
3. 优化后每entry/project generation满足：catalog≤1、module assembly≤1、activation sort≤1、每enabled module descriptor≤1、manifest/registration deep-clone bytes=0；稳定warm generation上述build均为0。
4. 对拍module order、required/optional/target过滤、availability/diagnostics、feature dependency、extension install、失败回滚、reload/unload与Editor贡献物。Cargo聚焦测试及F0/F2/F4 smoke全部通过。
5. RenderDoc只在surface-ready/first-presented之后验收渲染提交与资源；本F0 CPU组合链不用GPU capture替代WPR。current-source受管Cargo与WPR目前仍被外部unmanaged artifact治理阻断，故本模块保持`pending.md`，不进入`review.md`。

## 本轮决策

静态审查完成，确认一项结构性P1重复并归并`PERF-MVP-427`；没有证据支持局部builder代码修改。待单一代际插件计划实现并完成上述动态矩阵后，才可宣称瓶颈消失、量化收益、提交里程碑并发送企微。
