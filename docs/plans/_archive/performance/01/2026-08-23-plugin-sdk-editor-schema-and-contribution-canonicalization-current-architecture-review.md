---
related_code:
  - zircon_plugins/plugin_sdk/src/lib.rs
  - zircon_plugins/plugin_sdk/src/editor.rs
  - zircon_plugins/plugin_sdk/src/editor_contribution.rs
  - zircon_runtime_interface/src/editor_contribution.rs
  - zircon_editor/src/core/runtime_event_consumer/registration.rs
  - zircon_editor/src/core/plugin/registration.rs
  - zircon_plugins/first_party_editor_catalog/src/catalog.rs
  - tools/tests/test_editor12_sdk_contribution_builder_contract.py
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/performance/02-unreal-aligned-engine-system-hard-cutover.md
  - docs/plans/performance/01/2026-08-23-first-party-editor-plugin-catalog-instance-current-architecture-review.md
  - docs/plans/zircon_plugins/01-plugin-architecture-core.md
  - docs/plans/zircon_plugins/10-editor-integration.md
  - docs/plans/zircon_plugins/12-plugin-dx-and-structure-framework.md
  - docs/plans/zircon_editor/editor/12-plugin-management.md
reference_sources:
  - dev/UnrealEngine/Engine/Source/Runtime/Projects/Private/PluginManager.cpp
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Public/EditorModeRegistry.h
tests:
  - current plugin SDK editor slice 3 of 21 Rust files and 4 inline tests reviewed
  - shared serialized editor contribution owner and its 2 tests reviewed
  - Python performance contract 2 of 2 passed after a focused RED to GREEN cycle
  - focused rustfmt and diff check passed
  - current-source Rust tests, allocator benchmark, WPR and power pending
doc_type: implementation-evidence
status: m0_implemented_source_reviewed_dynamic_blocked
---

# Plugin SDK editor schema与贡献canonicalization复审（2026-08-23）

## 范围与当前性

已逐行复读`zircon_plugins/plugin_sdk/src/{lib.rs,editor.rs,editor_contribution.rs}`当前**3/21**个SDK Rust文件，合计
**613物理行、21,627 B、4 tests**；同时复读直接ABI owner
`zircon_runtime_interface/src/editor_contribution.rs`，总计**4文件、877物理行、30,282 B、6 tests**。SDK三文件SHA-256
分别为`e142c419...392f`、`af1d76bd...2f6f`、`a2374daa...2bf6`；ABI owner实施后为
`67e7a06a...489`。SDK其余18个Rust文件仍未计入本切片完成；已有dirty的
`manifest/importer_runtime.rs`、`native.rs`、`registration.rs`未触碰。

## 当前源码判定

### P0结构问题：immutable metadata与mutable session state混在同一declaration

`EditorPluginDeclaration`同时拥有descriptor/base manifest等immutable schema，以及
`EditorRuntimeEventConsumerRegistry`。后者的registration持`Arc<Mutex<State>>`和begin/apply/end闭包；registry clone只复制
BTreeMap与Arc，不深构造state，但每次`editor_plugin()`/macro `Default`会重新执行consumer factory并创建新的mutable state。

首方SDK插件普遍提供`editor_plugin_declaration/descriptor/package_manifest/editor_capabilities/plugin_registration`多个公开
façade。典型descriptor路径为“构造plugin和declaration -> clone declaration -> clone descriptor”，manifest/capability查询也
可能先创建并立即丢弃consumer state。`registration_report`又clone base manifest、执行完整extension registration、consumer
discovery/manifest比较和report构造。当前first-party catalog只选择Navigation/Neural并调用`plugin_registration`，所以不能把
所有façade定义次数冒充启动调用次数；但catalog重复projection会真实地重复上述instance/schema构造，归PERF-MVP-629。

正确硬切是宏生成process级borrowed/static `EditorPluginSchema`，只含package/module/capability/root/extension/consumer factories；
metadata查询借用该schema，不创建consumer。选中provider后才按editor session创建
`EditorPluginInstanceGeneration`和mutable consumer/mirror，稳定投影复用同一generation。不能缓存完整
`EditorPluginDeclaration`，否则AI/Navigation等session state会跨项目共享并增加锁竞争。

### M0已实施：canonical sort后不再创建第二棵树

旧`SerializedContributionBatch::new`先对C条贡献执行稳定排序，再把已排序键逐条插入`BTreeSet`判重：canonicalization总计
两段`O(C log C)`工作，排序与set都需要`O(C)`辅助内存。成功批次的key必须唯一，稳定排序对相等键没有输出语义；排序后
重复键必相邻。

本轮改为`sort_unstable_by`后在schema验证同一遍扫描中比较`previous_key`。新边界是sort `O(C log C)` + validate/dedup
`O(C)`；dedup树比较从`O(C log C)`降为C-1次相邻比较，canonicalization辅助heap allocation从`O(C)`降为0（不含调用者
已经构造的DTO Vec/String及错误路径String）。成功输出顺序、schema验证、duplicate kind/id、serde反序列化复用入口均保持。
Rust行为测试把两个duplicate drawer用另一类输入隔开，约束“排序后相邻拒绝”。

RED阶段还发现既有Python合同仍断言native feature不含已经落地的`declaration`依赖；该断言已同步当前Cargo feature边界。
复跑结果2/2通过，focused rustfmt与diff check通过。新增/修改Rust测试因受管Cargo会话不可执行，不能声明Rust tests通过。

### 局部非热点

本切片没有文件I/O、线程创建、sleep、主线程等待或逐帧callback。`with_capabilities`/runtime manifest merge使用Vec
`push_unique`，最坏`O(K^2)`，但K是插件声明期的小型capability/root集合；在真实K与比较计数证明前不换容器。DTO schema
字段必须跨ABI/serde拥有String，不能只因看到`.to_string()`改成进程地址或Rust trait object。

## Unreal源码依据

`PluginManager.cpp:2034-2080`把discover/enable/process/mount封装在一次configure generation，完成后清空
`PluginsToConfigure`；`2884-2978`只在明确loading phase消费已配置集合。`EditorModeRegistry.h:35-56,81-85,108-136`
把mode metadata/factory注册放在module startup/shutdown，并在`CreateMode()`才创建实例。可转移原则是schema/configure一次，
mutable editor object按session/lifecycle实例化；重复catalog投影不得再次执行声明与factory。

Zircon额外需要stable serialized contribution ABI，canonical order必须保留。排序后相邻判重是该约束下的直接算法收敛，
并不依赖容器偏好；native边界继续传versioned DTO/bytes，不跨DLL共享Rust object或进程地址。

## 量化验收

M0静态规模为C=0/1/10/100/1k/100k，duplicate位于首/中/尾且输入相邻/分离，schema invalid首/中/尾。受管benchmark记录
sort/validation/key comparisons、heap alloc count/bytes、wall p50/p95；要求dedup compare<=max(C-1,0)、dedup heap
alloc/bytes=0、输出canonical且错误等价。当前只有算法上界与2/2 Python合同，不报产品wall提升。

架构矩阵为providers 0/1/2/21/100、metadata queries 1/1k、sessions 1/2/100、consumer states 0/1/100及reload/unload。
记录schema/declaration/plugin/consumer/extension/report builds、manifest/descriptor/registry clone bytes、Arc RMW、locks、startup
p50/p95、RSS和energy。要求schema<=1/provider/process，metadata query instance/consumer/report build=0，instance/report
<=1/provider/session，跨session consumer state共享=0，unload后state/closure/extension=0。

本切片完成一个可证明的M0算法修正，但完整SDK仍为3/21，current-source Cargo、Rust tests、allocator benchmark、F4 WPR/RSS/
power未完成。因此继续留`pending.md`，不迁入`review.md`、不提交milestone、不发送完成企微；该非渲染切片不要求
RenderDoc。
