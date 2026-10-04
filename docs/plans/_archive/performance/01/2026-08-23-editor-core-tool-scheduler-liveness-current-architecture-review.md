---
related_code:
  - zircon_editor/src/core/tools
  - zircon_editor/src/core/context/tool_scheduler.rs
  - zircon_editor/src/core/context/builder.rs
  - zircon_editor/src/core/context/editor_context.rs
  - zircon_editor/src/core/editor_message/message/tool.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/performance/02-unreal-aligned-engine-system-hard-cutover.md
  - docs/plans/performance/01/2026-07-30-editor-core-context-tools-current-review.md
  - docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md
  - docs/plans/zircon_editor/editor/08-tool-orchestration-and-commands.md
  - docs/plans/zircon_editor/editor/15-build-export-and-publishing.md
reference_sources:
  - dev/UnrealEngine/Engine/Source/Runtime/InteractiveToolsFramework/Private/InteractiveToolManager.cpp
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Private/EditorModeManager.cpp
tests:
  - current core tools 4 of 4 Rust files and 17 inline tests reviewed
  - scheduler service, context construction, message payload and product reachability reviewed
  - M0 static performance contract 3 of 3 passed after RED
  - focused rustfmt 1.94.1 and scoped diff check passed
  - 3 added Rust behavior tests and current-source Cargo remain unexecuted
doc_type: implementation-evidence
status: m0_static_complete_dynamic_blocked
---

# Editor core tool scheduler活性与接线复审（2026-08-23）

## 范围与当前性

已逐行复读`zircon_editor/src/core/tools/**`当前**4/4**个Rust文件。实施前为**1,336行、41,588 B、
14 tests**，其path+NUL+raw bytes+NUL manifest与2026-07-30报告一致：
`6224ccf477a2eb12b4ec6df06aa541dad4bca065f4a838d4b245c3424e340bae`。M0后为**1,416行、
44,479 B、17 tests**，manifest为
`bacfffefa65992e96cf110c5f0366eb35f5ffdbaf00134933a7cc6ab8cf9c78a`。

同时完整复读`core/context/tool_scheduler.rs`当前**144行、4,197 B**，SHA256为
`cf5e2c97ce44582012871a00664c9fd5f5852e6b7d8b9bc31bb3109fab62cc21`，并沿Builder、
EditorContext与`ToolMessage`核对可达性。当前工作区生产代码只有一次service构造和一个accessor声明，
`EditorContext::tools()`的跨owner acquire/release consumer为**0**；Editor05 scene mode与Editor15 export wizard
接线仍未完成。因此本轮修复是接线前的活性门，不能冒充当前产品wall time或功耗改善。

## 当前源码判定

### 已修复P0：set激活会让无关空闲资源永久滞留

单资源请求在任意set request等待时会进入自己的queue，即使目标资源空闲，以保持set全局FIFO。旧`release`、
`release_set`和`withdraw_set`在一次操作成功激活set head后，不再调用
`promote_waiting_single_resources`。如果该激活使set queue变空，其他互斥域上已经排队且仍空闲的单资源工具不会被
推进；没有后续状态变化时会无限等待。这不是微小排序成本，而是资源利用率从可用降为0和交互活性失败。

M0保留set FIFO：只要set queue仍非空，single仍不得越过；一旦set queue清空，三个固定资源统一尝试推进。三条新
Rust测试分别覆盖`withdraw_set`、单holder `release`和`release_set`触发set激活后，另一空闲resource的single waiter
也被激活。旧源码下三场景均失败；Rust tests因managed Cargo不可用尚未执行，故不把它们写成GREEN动态证据。

### 已修复P1：三值resource set不应构造树

`ExclusiveResource`只有三个枚举值。旧`ToolResourceSet::new`先为每个unique resource插入`BTreeSet`节点，再分配并
收集最终Vec：U个不同值的容器分配模型为**U+1，U<=3**。M0改为一次Vec收集、`sort_unstable + dedup`，容器分配
降为**1**，canonical order、去重、非空错误和serde合同不变。这是固定小集合的直接算法收敛，不改变scheduler
authority。

### 已修复P1：内建topic从每操作解析收敛为每service一次

旧`publish_events`每次调度API调用都`EditorTopic::parse("editor.tool")`，即使report无event也分配并验证String。
M0在`ToolSchedulerService`构造时解析一次并保存topic；操作级parse/String allocation从**1降为0**，service lifetime
parse保持**1**。per-event topic/event clone及全局message-bus锁仍属PERF-MVP-019，本轮不建立私有bus或破坏锁外发布。

### 仍开放：未接线的全局scheduler不是模式系统

当前服务在启动时构造，但没有生产consumer；scene viewport仍有自己的`HandleToolRegistry`，scene mode/export也未
获得完整resource lease。先对无调用的scheduler做更复杂索引、线程化或大规模缓存不会改善MVP。Editor08必须在
Editor05/15同一里程碑定义tool/mode generation、transaction、input-router owner、accept/cancel和unload release-all，
然后硬切真实consumer，不保留第二套host-local仲裁。

队列虽有64 cap、resource仅3个、ToolId最多128 B，但duplicate/withdraw为O(Q)，`release_all`对同tool set请求先clone
再逐项中删，最坏O(Q^2)，Q<=64。它是有界取消路径，不应无数据改成复杂index；接线后的1M operation与plugin unload
trace若超预算，再用单pass retain/rebuild。消息report保留完整events又在bus边界clone，归PERF-MVP-019统一处理。

## Unreal源码依据

`InteractiveToolManager.cpp:121-279`的`UInteractiveToolManager`由明确side owner查询builder、在切换时先按
accept/cancel策略停用active tool，再构造/Setup新tool、注册InputRouter并广播started；tool change还可纳入undo
transaction。可转移原则是mode/tool生命周期、输入路由和transaction属于同一个交互owner，而不是独立全局队列。

`EditorModeManager.cpp:936-1104`由`FEditorModeTools`维护active/pending/recycled modes，激活前移除不兼容mode；
`1499-1533`在tick边界退出pending mode、维持default mode并只tick active mode/context。Zircon有跨viewport/modal/
export的atomic resource set需求，可以保留有界FIFO作为差异，但必须由真实mode/wizard consumer驱动，并在帧边界统一
publish activation generation；不能只在EditorContext里挂一个未消费service。

## M0证据与动态验收

`tools/tests/test_editor_tool_scheduler_unblocked_queue_m0_performance_contract.py`的scheduler合同先为**0/2 RED**；加入
topic合同后旧service为**2/3**，最终为**3/3 GREEN**。脚本42行、1,384 B、SHA256为
`c824e3c7373273d817d25d25bfeb49eaa6f6d6c904a07b0d6f2b6fab332c17c9`。focused rustfmt与scoped
diff check通过。没有创建C盘项目产物。

接线后的矩阵为single/set queue 0/1/64、resources 1/3、tools 1/64、operations 1/1M、subscribers 0/1/100/1k、
plugin unload与export cancel、callback stall 0/1/16ms/10s、threads 1/16。记录queue comparisons/moves、promotion count、
idle-with-waiter violations、topic parses/clone bytes、scheduler/bus lock wait/hold、message entries/bytes/age、p50/p95、RSS
和energy。验收要求set FIFO不被single越过、set queue变空后idle-with-waiter=0、built-in topic parse=1/service、锁内
publish=0、排队export不启动process、unload release-all无残留。

该切片不涉及渲染，RenderDoc不适用。current-source Cargo、真实Editor05/15 consumer、WPR/allocator/power未通过前仍留
`pending.md`，不迁入`review.md`、不提交milestone、不发送完成企微。
