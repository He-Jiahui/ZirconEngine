---
related_code:
  - zircon_plugins/plugin_sdk/src/declaration.rs
  - zircon_plugins/plugin_sdk/src/declaration/macros.rs
  - zircon_plugins/plugin_sdk/src/declaration/tests.rs
  - zircon_plugins/plugin_sdk/src/runtime.rs
  - zircon_plugins/plugin_sdk/src/runtime_exports.rs
  - zircon_plugins/ai/runtime/src/plugin.rs
  - zircon_plugins/physics/runtime/src/plugin.rs
  - zircon_plugins/navigation/runtime/src/plugin.rs
  - zircon_plugins/ai/editor/src/plugin.rs
  - zircon_plugins/physics/editor/src/plugin.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/performance/02-unreal-aligned-engine-system-hard-cutover.md
  - docs/plans/performance/01/2026-08-23-first-party-runtime-plugin-catalog-projection-current-architecture-review.md
  - docs/plans/performance/01/2026-08-23-plugin-sdk-editor-schema-and-contribution-canonicalization-current-architecture-review.md
  - docs/plans/zircon_plugins/01-plugin-architecture-core.md
  - docs/plans/zircon_plugins/10-editor-integration.md
  - docs/plans/zircon_plugins/12-plugin-dx-and-structure-framework.md
reference_sources:
  - dev/UnrealEngine/Engine/Source/Runtime/Projects/Private/PluginManager.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Private/Modules/ModuleManager.cpp
tests:
  - current plugin SDK declaration/runtime projection slice 5 of 21 Rust files and 5 tests reviewed
  - 28 production runtime export macro sites and 6 editor runtime-manifest mirror sites inventoried
  - AI Physics and Navigation representative provider construction reviewed
  - focused rustfmt passed; current-source Cargo, counters, WPR and power pending
doc_type: implementation-evidence
status: source_reviewed_structural_plan_dynamic_blocked
---

# Plugin SDK静态declaration与runtime projection复审（2026-08-23）

## 范围与当前性

已逐行复读`declaration.rs`、`declaration/{macros,tests}.rs`、`runtime.rs`、`runtime_exports.rs`当前**5/5**文件，
**1,472物理行、55,687 B、5 tests**；SHA-256依次为`fc63e27f...6a5f`、`4250c564...93e5`、
`f1ab813d...ea2`、`e28c4d4d...62e1`、`1db89545...5153`。目录clean，focused rustfmt通过。连同上个
editor slice，SDK累计复审**8/21**；其余13文件仍pending。

## 当前源码判定

### 正确基线：runtime-independent schema与native ABI已是编译期数据

`PluginDeclaration`为`Clone+Copy`，只持`&'static str`和静态slice；target/platform/maturity/packaging/capability role均为
small enum。`declare_plugin!`生成一个const declaration，native plugin ID、entry C string、requested capability text和
runtime/editor registration manifest均由`concat!`形成静态`&[u8]`。这里没有启动期TOML拼接、serde、I/O、锁或线程，也不应
改成运行期builder/cache。

该schema仍会在`runtime_declaration()`投影时把target/platform/packaging转为owned Vec并构建descriptor；这是最终owned
runtime descriptor需要的工作，但同provider generation只能做一次。canonical runtime ID和capability-role长度检查目前在每次
projection执行，终态可在generated schema validation阶段一次完成。

### P0：runtime metadata façade会构造完整plugin与manager

`RuntimePluginDeclaration::descriptor()`clone整个builder再build，`package_manifest()`再经descriptor构建。更严重的是
`runtime_plugin_exports!`生成的`package_manifest()`、`runtime_selection()`和`plugin_registration()`全部先调用
`runtime_plugin()`。当前共有**28个生产macro使用点**。

代表性真实调用证据：

- AI `runtime_plugin()`创建`Arc<DefaultAiManager>`；`runtime_plugin_descriptor()`还会再创建manager并把它接入module
  descriptor。只调用`package_manifest()`也走完整plugin/manager构造后丢弃instance。
- Physics同样创建`Arc<DefaultPhysicsManager>`并把manager写入module descriptor；metadata query并非data-only。
- Navigation当前plugin本身只有descriptor，但manifest仍重建component/option/event/native module/distribution Vec/String。

editor侧有4个macro `mirrors_runtime_manifest:`站点和2个method调用站点（Navigation/Neural）；构造editor plugin declaration
时会调用runtime crate `package_manifest()`，因此可能为“只镜像manifest”创建runtime manager。first-party catalog重复
projection会按次数放大该成本，属于PERF-MVP-629确认的schema/instance owner错误，不是给builder加capacity可以解决的问题。

### 正确结构

`PluginDeclaration`升级为唯一`PluginProviderSchema`输入；generated schema validation和owned manifest/descriptor projection
<=1/provider/process。`runtime_plugin_exports!`的manifest/selection直接从schema generation借用或clone最终owned schema，
不得调用runtime plugin constructor。只有项目selection被admit后，才按runtime session创建manager/module factory/plugin
instance并生成registration instance。

不能用`OnceLock<RuntimePlugin>`缓存完整plugin：manager/resource/bridge/lifecycle可能属于session或project，跨session共享会
引入状态污染、锁竞争和卸载泄漏。也不能把module descriptor里的manager Arc继续当成metadata；schema只保存factory/slot，
instance generation保存manager。

## Unreal源码依据

`PluginManager.cpp:2034-2080`只在`PluginsToConfigure`非空时完成discover/enable/process/mount，之后清空待配置集合；
`2884-2978`按loading phase消费enabled plugin。`ModuleManager.cpp:992-1061`对已存在且loaded module直接返回，miss才创建并
初始化module。可转移原则是provider schema/configuration稳定复用，module/plugin instance按phase与session创建；读取plugin
descriptor或manifest不能隐式创建runtime subsystem manager。

Zircon的native const registration bytes比UE plugin descriptor多一个stable ABI投影，应保留该优势。进程级schema可共享，
runtime/editor manager和consumer不可共享；reload时先quiesce instance，再以新schema/instance generation原子发布。

## 量化验收

矩阵为providers 0/1/28/100、metadata manifest/selection/status queries 1/1k、sessions/projects 1/2/100、manager-bearing
providers 0/1/10、reload/unload 0/1/100。记录schema validation/projection、plugin/manager/module descriptor/manifest/report
builds、Vec/String/descriptor clone+alloc bytes、Arc RMW、locks、startup p50/p95、RSS和energy。

验收要求schema validation/projection<=1/provider/process；metadata query plugin/manager/module factory/registration build=0；
selected provider instance/manager<=1/provider/session；stable query schema build=0；跨session manager共享=0，unload后manager/
resource/bridge/module factory=0。AI/Physics metadata-only 1k queries的manager constructor必须严格为0。

本轮不改生产代码：静态declaration/native projection已经正确，修复metadata façade必须同步拆开descriptor schema和manager
instance，单改宏会丢失现有plugin-specific manifest override。current-source Cargo、constructor/allocator counter、F0/F4
WPR/RSS/power仍pending；继续留`pending.md`，不迁入`review.md`、不提交milestone、不发送完成企微。该非渲染切片不要求
RenderDoc。
