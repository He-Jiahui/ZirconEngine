---
related_code:
  - zircon_plugins/plugin_sdk/src/manifest
  - zircon_runtime/src/plugin/package_manifest/constructors.rs
  - zircon_runtime/src/plugin/package_manifest/plugin_feature_bundle_manifest.rs
  - zircon_runtime/src/plugin/runtime_plugin/descriptor/builder/runtime_plugin_descriptor_builder.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/performance/02-unreal-aligned-engine-system-hard-cutover.md
  - docs/plans/performance/01/2026-08-23-plugin-sdk-static-declaration-runtime-projection-current-architecture-review.md
  - docs/plans/zircon_plugins/01-plugin-architecture-core.md
  - docs/plans/zircon_plugins/12-plugin-dx-and-structure-framework.md
  - docs/plans/zircon_plugins/13-standalone-plugin-build.md
reference_sources:
  - dev/UnrealEngine/Engine/Source/Runtime/Projects/Private/PluginManager.cpp
tests:
  - current plugin SDK manifest slice 7 of 7 Rust files and 7 tests reviewed
  - current importer move implementation and legacy parity benchmark source reviewed
  - focused rustfmt and diff check passed
  - current-source Cargo, parity test, managed allocator benchmark, WPR and power pending
doc_type: implementation-evidence
status: source_reviewed_foreign_m0_present_dynamic_blocked
---

# Plugin SDK manifest builder move projection复审（2026-08-23）

## 范围与当前性

已逐行复读`zircon_plugins/plugin_sdk/src/manifest/**`当前**7/7**文件、**986物理行、34,678 B、7 tests**；path hash
逐文件记录为`defaults 313d6b4a...4644`、`feature 6a528809...465e`、`importer 6a63d2ee...b398`、
`mod 1b29e03e...bc79`、`package f4294055...8d35`、`module 1e464ed6...c9a1`、`tests fd3312d0...dfb0`。
focused rustfmt和diff check通过。连同前两片，SDK累计复审**15/21**。

`manifest/importer_runtime.rs`在本轮开始前已dirty：现有他人改动包含move-backed manifest assembly、legacy test helper、
等价行为测试和ignored release benchmark。本轮保留并按当前源码复审，没有覆盖、回退或把它误归为本轮实现。

## 当前源码判定

### 局部builder通过：by-value append/collect，没有隐藏全图重建

`PluginManifestBuilder`、`PluginModuleBuilder`、`PluginFeatureBundleBuilder`均包装owned manifest并以by-value链式move写入。
底层`PluginPackageManifest`/feature manifest方法只做push/extend/collect，不在每个字段追加时sort、全量validate或发布generation。
module bulk setter用一次collect替换Vec，feature capability module仅为module和feature各保存一份必要的capability String。

`default_supported_platforms/default_export_packaging`返回小型定长数组；importer的NativeDynamic contains只扫通常2-3个packaging
enum，不是规模热点。本目录没有I/O、锁、线程、callback或逐帧入口。真正的重复成本来自上层每次metadata/report query重建
整份manifest，继续归PERF-MVP-629，而不是在这里引入并行或HashSet。

### 当前foreign M0：importer builder字段clone静态模型37 -> 1

旧`build_package_manifest`借用self分别调用`dist_module_manifest()`与`distribution_manifest()`，对32 capability样本会clone
32个capability String及5个builder String字段。当前dirty实现析构self，把capabilities/importers/dist module/engine compat/
entry全部move，只为dist module和distribution同时需要的`dist_crate_name`保留1次clone。descriptor manifest构建和硬编码
forms/symbol等两侧共同成本不计入该差值。

静态模型因此是**builder-field clones/build 37 -> 1，capability clones 32 -> 0**，语义等价测试比较完整
`PluginPackageManifest`。这是可信的源码计数，不是allocator实测：String clone通常分配，但容器容量、SSO不存在、allocator行为和
descriptor内部clone未由当前测试计数。

现有ignored benchmark使用21组交替顺序、每组256 builds、C=32 capabilities/I=8 importers，要求p95至少20%提升。其设计
排除了builder构造时间且交替顺序优于单向测量，但仍是测试进程wall-clock机器阈值；打印的37/1是常量，不是allocator
instrumentation，也没有受管benchmark receipt、CPU frequency/affinity/build profile。未执行前不得声称20%已经取得。

## Unreal源码依据

`PluginManager.cpp:2034-2080`以一次configure generation处理enabled plugins并清空`PluginsToConfigure`；manifest/descriptor
数据应在该generation构建一次。Zircon builder的by-value局部形态符合“一次构建、随后消费”；当前上层反复调用provider
constructor/manifest façade才违背该生命周期。

native distribution与importer metadata必须保持owned/versioned DTO，不能为了减少String clone跨DLL传borrowed Rust地址。
正确边界是process schema generation拥有最终manifest，project/session只持slot/range；reload构建完整candidate后原子发布。

## 量化验收

importer M0矩阵为capabilities C=0/1/32/100/1k、importers I=0/1/8/100/1k、String长度0/16/1k、descriptor modules
0/1/100。受管release benchmark记录builder-field clone、String clone/bytes、alloc count/bytes、peak RSS、p50/p95；要求
capability clones=0、builder-field clones=1、manifest parity=100%、失败/duplicate/schema行为等价。wall阈值只能作为观测值，
主要门是结构counter与allocator receipt。

架构验收仍要求manifest/schema build<=1/provider/process，stable metadata query build=0；28-provider F0/F4 startup/reload记录
manifest builds、alloc/RSS/wall/energy。当前Cargo、等价Rust test、ignored benchmark均未执行，本报告不声明foreign M0动态
通过。

本目录局部没有新增生产修改。SDK累计15/21，current-source Cargo、managed benchmark、F0/F4 WPR/RSS/power未完成，继续留
`pending.md`，不迁入`review.md`、不提交milestone、不发送完成企微；非渲染切片不要求RenderDoc。
