---
related_code:
  - zircon_app/src/entry
  - zircon_app/src/entry/cli
  - zircon_runtime/src/core/runtime/config_store.rs
  - zircon_app/src/entry/tests/entry_config_storage.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/runtime/02-core-spine-and-root-surface.md
  - docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md
  - docs/plans/zircon_editor/editor/12-plugin-management.md
  - docs/plans/zircon_plugins/01-plugin-architecture-core.md
reference_sources:
  - dev/UnrealEngine/Engine/Source/Runtime/Projects/Private/PluginManager.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Launch/Private/LaunchEngineLoop.cpp
tests:
  - current-source hash stability 11/11 passed
  - direct rustfmt 11/11 passed
  - inline tests 14 inspected, managed Windows Cargo pending
  - plugin/config generation and export-root WPR matrix pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# App entry根与CLI current-source性能审查（2026-08-14）

## 范围与证据边界

`zircon_app/src/entry`根 **9/9** 与`cli/**` **2/2**，合计 **11/11** 个Rust文件、
**2,040** 行、**1,845** 个非空行、**14** 条内联`#[test]`已逐文件完整阅读。直接
`rustfmt +1.94.1 --edition 2021 --check --config skip_children=true`为11/11通过，复核前后
SHA-256前缀11/11不变；仅`export_bootstrap.rs`有其它Session修改，本轮不覆盖。

当前源码已修复2026-07-23报告中的`DescriptorBackedEngineModule`动态文本泄漏：`module_name`和
`module_description`直接借用descriptor-owned `String`，1/100/1,000 cardinality测试比较底层指针，
生产路径没有`Box::leak`。旧问题不得继续作为current瓶颈或优化依据。

## 当前入口算法

```text
EntryConfig/project manifest
  -> first-party runtime/editor registration projection
  -> builtin module selection + availability/feature report
  -> RuntimePluginCatalog + extension/lifecycle state
  -> default profile PluginGroup merge + resolved descriptor snapshot
  -> module-selection report and CoreRuntime bootstrap
  -> config store -> register -> activate -> config store
```

CLI先把参数收集为一个Vec，再用clone iterator识别Commandlet；Commandlet在GUI解析和host创建前返回。
参数量小且分流正确，不是F0主瓶颈。render profile overlay只枚举3个固定feature，每个对manifest做线性
查重，复杂度为O(3P)=O(P)，没有理由在这里引入索引或并行。

## P0/P4：插件selection没有单generation owner

带runtime registrations的module选择先由`EntryConfig::project_plugin_manifest`克隆P个selection，随后
`project_manifest_for_plugin_selection`再次克隆effective manifest；feature路径又从R个registration和F个
feature registration克隆构造`RuntimePluginCatalog`，分别计算feature dependency、extension report和
lifecycle state。module descriptor随后进入PluginGroup snapshot，diagnostic report再复制需要持有的字段。

单次startup中的这些复制不全是无效：lifecycle/report必须拥有跨阶段数据，CoreRuntime也必须取得owned
descriptor。结构问题是同一selection/catalog/extension判定在App、Runtime builtin、Editor和host多次生成，
没有generation identity证明复用或失效。它与已有`PERF-MVP-427`是同一根因，禁止再建App私有cache。

目标是一个immutable compiled plugin plan：输入为project manifest generation、target、render profile与
compiled feature set；输出同时提供availability、ordered modules、runtime/editor registrations、capabilities、
extension/lifecycle views。0/1/100/1,000插件下每generation plan build<=1，selection visit、registration clone
bytes、descriptor clone bytes和activation顺序可计数；manifest或feature generation不变时所有消费者共享同一
artifact，变更时原子发布新generation并保留last-good。

## P1：bootstrap无条件重复配置序列化与覆盖

`BuiltinEngineEntry::bootstrap`在module register/activate前后各调用一次`store_entry_config`。每次至少对
PlatformConfig、RenderProfileBundle、WindowDescriptor执行3次`serde_json::to_value`，并分别获取
`ConfigStore`的`Mutex<HashMap<String, Arc<Value>>>`覆盖值；Editor还重复写sandbox和可选subsystem value。
`ConfigStore::store`没有相等性短路或generation判断，所以第二次调用不是零成本读取。

现有`entry_config_storage.rs`却用`include_str!`硬断言恰好调用两次，并把它描述为“激活前后fail closed”。
当前生产搜索没有发现Platform/Window key的其它写者，Render key除entry外只在dynamic session构造写入；
但这还不足以直接删除第二阶段，因为44个`entry/tests/**`及模块激活行为尚未完成current-source复审。

验收顺序：先用行为测试注册一个激活期尝试改写这些key的module，明确合同是禁止写、允许owner变更还是
需要reconcile；再增加config generation/write/serialized-bytes/mutex-wait counter。若activation无合法写者，
删除第二次调用并把源码形状测试换成“startup generation exactly once + consumers观察一致值”；若确需
reconcile，则只对changed generation写入，不能无条件覆盖全部5类值。

## P2/F5：export-root发现存在重复物理探测候选

当前foreign-dirty实现分别展开executable parent和working directory的所有ancestors，对每项执行
`ProjectPaths::resolve_existing`、manifest path resolve与`exists`。两条路径共享祖先时会重复canonicalize/probe；
成本为O(exe depth + cwd depth)，只在显式export bootstrap发生，不是默认Editor/Runtime稳定帧热点。
先测本地盘/网络盘、深度4/16/64、alias/no-alias的resolve/stat次数与wall；只有冷I/O显著时，才按resolved
physical identity去重候选。测试fixture必须由managed validator把TEMP/TMP定向D/E/F盘，不得落C盘。

## Unreal源码依据

- `PluginManager.cpp:2034-2084`只在`PluginsToConfigure`非空时构造一次discovery context，完成compile-time、
  command-line、target、program、dependency、mark/process/mount后清空待配置集合；后续相同调用直接复用已配置
  状态。Zircon应同样让一个generation artifact成为后续module/editor/host阶段的事实源。
- `PluginManager.cpp:2884-2910`在各loading phase先确保configured，再遍历enabled plugins加载该阶段module；
  它保留阶段顺序，而不是让每个phase重新解析project selection。`3150-3171`查询也消费已配置状态。
- `LaunchEngineLoop.cpp:2525,2601-2681,3469-3495`分别计时project load、task/thread pools、core/preinit
  modules与后半启动阶段；参考的是可观测的阶段owner，不支持把整个入口算法并行化或用微优化掩盖重复generation。

## 跨计划交付与动态门

| Owner计划 | 必须解决的合同 | Performance验收 |
|---|---|---|
| Runtime02 | Core config generation与module activation读写规则；typed store counter | startup key write/serialization次数、bytes、mutex wait；稳定generation每key写<=1 |
| Editor01/12 + Plugins01 | `PERF-MVP-427` compiled plugin plan由单owner发布 | 0/1/100/1K插件下build<=1/generation；App/Runtime/Editor module/capability结果一致 |
| App entry/tests | 删除“两次调用”源码形状false-green，改为真实module activation与consumer行为 | 合法/非法activation写者、失败回滚、最终generation和错误均可断言 |
| Export owner | physical candidate去重仅在profile证明I/O瓶颈后实施 | local/network、alias、depth矩阵的resolve/stat和cold/warm p50/p95 |

WPR/xperf使用E/D/F盘current-source managed build，记录CPU sample、File/Disk I/O、alloc/RSS、mutex wait、
CSwitch/ReadyThread与energy；插件与config counter和WPR阶段对齐。RenderDoc不适用于这组CPU/plugin/config
根因，只在后续surface-ready渲染验收使用。

本轮没有源码修改：`export_bootstrap.rs`为foreign dirty，config双写又被尚未行为验收的跨文件测试合同锁定；
在完整复审相关`entry/tests/**`、取得current-source counter/WPR并证明语义后再修改。该组继续留在
`pending.md`，不进入`review.md`，不形成提交或企微发布里程碑。
