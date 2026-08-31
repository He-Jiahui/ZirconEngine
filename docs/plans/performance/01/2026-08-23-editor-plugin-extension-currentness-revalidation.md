---
related_code:
  - zircon_editor/src/core/plugin
  - zircon_editor/src/core/extension
base_reports:
  - docs/plans/performance/01/2026-08-16-editor-core-plugin-catalog-lifecycle-current-architecture-review.md
  - docs/plans/performance/01/2026-08-19-editor-core-extension-lifecycle-generation-architecture-review.md
owner_plans:
  - docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md
  - docs/plans/zircon_editor/editor/06-ui-extension-framework.md
  - docs/plans/zircon_editor/editor/12-plugin-management.md
  - docs/plans/zircon_plugins/01-plugin-architecture-core.md
  - docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
reference_sources:
  - dev/UnrealEngine/Engine/Source/Runtime/Projects/Public/Interfaces/IPluginManager.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Modules/ModuleManager.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Features/IModularFeatures.h
  - dev/UnrealEngine/Engine/Source/Developer/ToolMenus/Public/ToolMenus.h
  - dev/godot/editor/plugins/editor_plugin.h
doc_type: implementation-evidence
status: static_current_structural_cutover_required_dynamic_blocked
---

# Editor Plugin / Extension currentness复验（2026-08-23）

## 当前清单

| 模块 | current Rust | 行 / bytes / tests | path+raw SHA256 | currentness结论 |
|---|---:|---:|---|---|
| `core/plugin/**` | 35/35 | 6,323 / 227,022 / 51 | `fb0ba5dff8439113b93b88d3e553fc16eae9d782d4e4f6cf5e70bd5f20847f11` | 对8月16日全量逐文件审查完成diff复验；生产算法未变 |
| `core/extension/**` | 30/30 | 5,267 / 170,756 / 38 | `2425d286df8435193c0bf5aaebcb918a3359c08f7f0937b0dd2ec1515e2fda34` | 与8月19日全量逐文件审查指纹完全一致 |

本轮以两份base report的65/65逐文件全文审查为基线，核对其后所有相关提交及当前工作区。`extension/**`无需重读未变化文件；`plugin/**`只对发生漂移的文件和其owner调用链重新逐行核对。没有修改这两棵Rust源码。

## 漂移复验

`08094b9b9`是基线后唯一相关提交。`plugin/catalog_store.rs`只有内联测试import路径变化；`plugin/manager/tests.rs`调整测试snapshot保留与owner；`plugin/manager/tests/project_registration.rs`改为先由manager发布、再读取state snapshot。`extension/store/tests.rs`仅调整owned-string断言。生产publication、callback、catalog build、mounted contribution和retirement行为均未改变。

因此此前结构结论仍成立：

- routine document/play/asset/UI fact仍会进入bridge二次队列，在manager mutation gate内串行调用插件，再重建完整catalog、projection和active extension generation；成功且不改变结构的回调仍不是`O(1)`结构空操作。
- bridge队列、callback history和lifecycle report仍无统一count+bytes+age+deadline admission；回调延迟仍能阻塞编辑器owner和后续插件变更。
- project open仍把native registration publication与manifest apply分成两个manager transaction，候选registry/materializer仍有重复clone/replay。
- Plugin Manager的desired active owners与Workbench实际mounted callbacks仍是两个authority；`ContributionStore::revoke()`仍无生产调用者，disable/reload/unload前没有统一quiescence和owner-root census。
- extension registration仍跨shell锁执行外部准备、registry replay和多family可见性变更；DocumentToolkit registry仍可能在mutex内调用`descriptor()`或drop callback-owned对象。

这些是publication unit和lifetime ownership错误，不是替换一个容器或删除一个clone可以安全修复的问题。本轮不实施会固化错误契约的局部补丁。

## 参考引擎约束

- Unreal `IPluginManager.h`以refresh、loading phase和enabled plugin集合表达定义/加载边界；`ModuleManager.h`提供loaded/unloaded generation与pre-unload/shutdown顺序。Zircon应把definition generation、active-set generation和callback retirement分离。
- Unreal `IModularFeatures.h`要求register/unregister成对；`ToolMenus.h`以owner清除注册。采用其owner-scoped lifetime规则，但不照搬`ModularFeatures.cpp`锁内broadcast或ToolMenus全局扫描算法。
- Godot `editor_plugin.h`为dock、menu、import/export、gizmo和Inspector提供成对add/remove接口。Zircon目标同样必须先撤销所有callback roots，再允许native module unload。

参考源码只确定ownership、generation和复杂度方向，不能替代Zircon同机动态数据。

## 必须执行的结构性优化

1. 以`EditorPluginDefinitionGeneration`保存不可变descriptor/dependency/contribution handles；只有discover、project package change和reload可替换。
2. 以独立`EditorPluginRuntimeGeneration`保存active/faulted状态。成功瞬态callback不替换definition/catalog generation。
3. `CompiledEditorExtensionGeneration`仅由`{definition_generation, active_set_generation}`驱动，一次编译所有family index；稳定帧和普通生命周期事件build/publish为0。
4. bridge按count+bytes+deadline分页；短锁下取得稳定callback handles，锁外调度并以generation fence提交fault。callback-under-bridge-lock和callback-under-manager-lock都必须为0。
5. project activation先在锁外生成一个validated mount plan，再以一个receipt、一个mounted generation、一次publication原子提交；失败可完整rollback。
6. disable/reload/close依次停止admission、取消或drain任务、撤销owner receipts、等待callback/snapshot/job roots归零，然后才允许卸载二进制。

## 静态验证与动态门

- 发生漂移的4个Rust文件逐文件`rustfmt --edition 2021 --check`通过；两棵树的scoped `git diff --check`通过。65文件全树并行format gate在124秒超时且留下不可访问的外部`rustfmt`进程，因此不能声称全树format gate通过。
- 九模块focused Python batch执行39 tests：34通过、3失败、2 error；失败/错误均是旧静态源码锚点（constructor引用形式、generic serialize、snapshot accessor、toolkit save标记、retained tick函数名）与当前owner迁移不一致。
- `test_editor12_plugin_manager_contract`执行24 tests：18通过、6失败；失败锚点覆盖能力配置owner、lifecycle fixture搬迁及native loader函数名。没有动态插件运行断言失败，但这些合同必须由Editor12 owner更新或证明真实语义回归。
- 未运行Rust/Cargo。managed validator session已归档，禁止用raw Cargo或伪造identity绕过；当前没有current-source可执行文件，所以WPR、allocator/RSS、package power、F0/F4和RenderDoc均未执行。

两个模块继续留在`pending`。接受门仍是plugins/contributions、messages/callback stall、history、native batches/dependency depth、toggle/reload/unload矩阵；记录build/clone bytes、queue entries/bytes/age、callback/lock time、RSS、CPU/wakeups、p50/p95/p99和package power。RenderDoc只在extension cutover改变viewport可见输出时用于draw/GPU/pixel parity，不用于证明CPU控制面性能。
