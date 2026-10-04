---
related_code:
  - zircon_plugins/first_party_editor_catalog/src
  - zircon_app/src/entry/first_party_editor_plugins.rs
  - zircon_app/src/entry/entry_runner/editor.rs
  - zircon_editor/src/core/plugin/registration.rs
  - zircon_plugins/navigation/editor/src/plugin.rs
  - zircon_plugins/neural/editor/src/plugin.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/performance/02-unreal-aligned-engine-system-hard-cutover.md
  - docs/plans/performance/01/2026-08-23-first-party-runtime-plugin-catalog-projection-current-architecture-review.md
  - docs/plans/zircon_editor/editor/12-plugin-management.md
  - docs/plans/zircon_plugins/01-plugin-architecture-core.md
  - docs/plans/zircon_plugins/10-editor-integration.md
reference_sources:
  - dev/UnrealEngine/Engine/Source/Runtime/Projects/Private/PluginManager.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Private/Modules/ModuleManager.cpp
tests:
  - current first-party editor catalog 3 of 3 Rust files and 6 tests reviewed
  - App projection, Editor registration and Navigation/Neural provider construction reviewed
  - plugin structure audit passed and focused rustfmt 1.94.1 passed
  - current-source Cargo, provider benchmark, startup WPR, allocator and power pending
doc_type: implementation-evidence
status: source_reviewed_structural_plan_dynamic_blocked
---

# First-party editor plugin catalog schema/instance复审（2026-08-23）

## 范围与当前性

已逐行复读`zircon_plugins/first_party_editor_catalog/src/**`当前**3/3**个Rust文件、**222行、7,832 B、
6 tests**，path+NUL+raw bytes+NUL manifest为
`4e2106c27f928c42ca3236ae1862e7caf356c1e6812f49d5cc187ce837d06426`；目录当前clean。本轮同时复核App
project/config调用、`EditorPluginRegistrationReport::from_plugin`及Navigation/Neural真实provider。配对的plugin
structure audit已在runtime catalog切片通过，focused rustfmt通过。

## 当前源码判定

### P0：每次projection重建schema，也重建有状态editor instance

catalog对EditorHost manifest逐selection parse、HashSet去重，再调用Navigation或Neural `plugin_registration()`。
`EditorPluginRegistrationReport::from_plugin`每次新建candidate/final extension registry、跨panic boundary执行extension
registration和runtime consumer discovery、比较consumer manifest、构造package manifest、复制capabilities并创建多组
lifecycle/diagnostic Vec。

Navigation更关键：`NavigationEditorPlugin::default`每次创建新的`Arc<Mutex<NavigationPieMirror>>`，用它生成runtime event
consumer，再注册viewport/operation等extension。重复projection不仅重复immutable descriptor/schema工作，也重复创建真正
的mutable session state。Neural同样重新构造声明、extension与authoring contribution。App editor startup随后把owned
reports交给capability、plugin manager和UI projection，形成PERF-MVP-629已确认的平行catalog链。

不能把完整`EditorPluginRegistrationReport`简单放进process `OnceLock`：它会让多个Editor session共享Navigation mirror、
extension object与未来lifecycle state，导致跨项目事件污染、卸载不完整和锁竞争。正确边界是两层：

1. process级immutable `EditorProviderSchemaGeneration`只拥有validated descriptor、manifest、capability、extension/consumer
   factories与dense slot；每provider构建不超过1次；
2. editor session级`EditorPluginInstanceGeneration`按选中slot创建mutable mirror/extension/lifecycle，实例化不超过
   1/provider/session，卸载按generation retire。

project/target selection只返回ordered slot handles；同session重复status/menu/capability投影借用同一instance generation，
不再次调用provider constructor或registration。

### P0：现有“性能测试”反向放大宽构建且预算缺少产品意义

App `first_party_editor_plugins.rs`的feature测试用**21 samples × 1,024 iterations = 21,504**次完整provider projection，
再以**250,000 us/1,024 calls**作为p95阈值。它在普通单测进程内反复创建Navigation mirror/registry/report，测到的是
测试机器、debug/release和allocator混合结果；没有记录constructor、registration、clone bytes、RSS或锁，也没有验证
真实startup只物化一次。该门可能让测试本身重度，却仍放过每startup重复schema构建。

Plugins10/Editor12应把它迁为受管benchmark：cold schema build、session instance build、stable lookup分别测量，并以
counter先验约束build count。小型行为测试只验证target/disable/duplicate/order/consumer/capability，不在单测中做
21,504次wall-clock循环。

### P1：线性分派与源码形状门和runtime catalog同源

当前compiled provider仅2个，O(S*P)并非单独热点；target非EditorHost有正确O(1)空返回。外部manifest可能重复，canonical
HashSet去重不能删除。终态generated dense slot能把lookup变为O(1)，但必须保留unknown/open ID。当前catalog test强制
HashSet/Vec/for-loop源码文本，Runtime D6测试又强制typed if/no match，应改为行为与build/alloc counter，避免实现拼写
阻塞schema/instance分层。

## Unreal源码依据

`PluginManager.cpp:2034-2080`把discover/enable/process/mount封装为一次configuration generation，随后清空
`PluginsToConfigure`；`2884-2978`在明确loading phase上消费已配置plugin集合。`ModuleManager.cpp:992-1061`对已存在且
loaded module直接返回，只有miss才进入创建和初始化。可转移原则是stable schema/catalog lookup不重复构造，而module/
editor object instance仍按lifecycle创建和退出。

Zircon比UE此处多出typed Rust extension registry和per-session mirror，应保留这项差异；schema可共享，mutable mirror不可
process-global。卸载时先停止consumer admission、quiesce callback，再retire instance generation；不得靠clone缓存或泄漏
Arc规避生命周期。

## 量化验收

矩阵为providers 0/1/2/100、manifest rows 0/1/100/1k、sessions/projects 1/2/100、duplicates 0/1/100%、runtime
consumer/events 0/1/1k、reload/unload、callback stall 0/1/16ms/10s。记录schema/instance/registration/extension/consumer
builds、mirror instances、descriptor/manifest/registry clone+alloc bytes、locks、startup p50/p95、RSS和energy。

验收要求schema build<=1/provider/process、instance build<=1/provider/session、stable projection build=0；Navigation mirror
owner=1/session且跨session不共享，unload后consumer/extension/mirror=0；target/disable/duplicate/order/diagnostics行为等价。
普通单测wall-clock循环=0，benchmark输出counter与build profile。

本轮未发现能在不冻结schema/instance生命周期前安全落地的局部生产修改，因此没有改Rust。current-source Cargo、受管
benchmark、F4 startup WPR/allocator/power仍pending；该catalog切片不使用RenderDoc。继续留`pending.md`，不迁入
`review.md`、不提交milestone、不发送完成企微。
