---
related_code:
  - zircon_plugins/plugin_sdk/src/registration.rs
  - zircon_plugins/plugin_sdk/src/prelude.rs
  - zircon_runtime/src/plugin/extension_registry/register/runtime_scene_system_registration.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/performance/02-unreal-aligned-engine-system-hard-cutover.md
  - docs/plans/performance/01/2026-08-23-plugin-sdk-static-declaration-runtime-projection-current-architecture-review.md
  - docs/plans/zircon_plugins/01-plugin-architecture-core.md
  - docs/plans/zircon_plugins/12-plugin-dx-and-structure-framework.md
reference_sources:
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/TickTaskManager.cpp
tests:
  - current plugin SDK registration and prelude slice 2 of 2 Rust files and 3 tests reviewed
  - Python source contract RED then GREEN
  - minimal Rust generic-inference probe compiled and ran from E drive, then removed
  - focused rustfmt and diff check passed
  - current-source Cargo, allocator counters, WPR and power pending
doc_type: implementation-evidence
status: source_reviewed_m0_implemented_dynamic_blocked
---

# Plugin SDK runtime registration factory ownership复审（2026-08-23）

## 范围与当前性

已逐行复读`registration.rs`和`prelude.rs`当前**2/2**文件、**475物理行、16,842 B、3 tests**；SHA-256分别为
`6bed42ce...b0ff1`和`477758fd...387e31`。连同前三片，Plugin SDK累计复审**17/21**。

`registration.rs`在本轮开始前已有import/test格式化dirty；本轮保留这些并在同一当前文件上完成工厂所有权M0，没有回退或覆盖
他人逻辑。新增Python源码契约先在旧结构上RED，修改后GREEN；focused rustfmt与diff check通过。另用stdin编译的最小Rust探针
验证`Builder<F>::register<S>`可以通过`F: Fn() -> S`正确推断`S`，探针只生成于E盘，运行后已精确删除。这不是工作区
Cargo测试的替代品。

## 当前源码判定

### 正确边界

module owner只intern一次；system set在最终注册时intern，ordering constraint按builder顺序转交；resource/component/event/interface
注册直接进入runtime owner registry。这里没有I/O、线程创建、帧callback或全图重建。`prelude.rs`仅重导出，不产生运行时成本。

### 已实现M0：SDK不再提前擦除system factory

旧SDK builder把调用者的具体`F`立即装入`Arc<dyn Fn() -> S>`；`register()`再创建一个调用该Arc的closure，并交给runtime
registry。runtime registry随后把该closure装入第二个Arc，最终又创建返回boxed system的erased build closure。因此SDK层每注册
一个system多一次Arc heap object，每构造一个system instance多一次不必要的动态调用。

当前M0让`RuntimePluginRuntimeSceneSystemBuilder<'registry, F>`直接拥有具体`F`，到runtime registry边界才进行第一次type erasure。
静态差值为：**SDK factory Arc allocations/system registration 1 -> 0**，**extra SDK dynamic dispatch/system instance build
1 -> 0**。数据、ordering、clock-domain与per-instance callback state语义不变。该路径发生在插件激活/scene system实例构建，不是
每帧callback，因此不得推导为frame time已经下降。

runtime registry自身仍有`Arc<dyn Fn() -> S>`和`Arc<dyn Fn() -> BoxedRuntimeSceneSystem>`两层factory/build擦除，并在注册时clone
sets/constraints构造metadata。这是更大owner的下一候选：完整复审registry build/clone/unload/reload与调度消费后，评估合并为一个
erased build closure；不能只删Arc而破坏registration clone、跨scene实例私有state或owner revoke。

## Unreal源码依据

`TickTaskManager.cpp:2406-2425`的`FTickFunction::RegisterTickFunction`只在未注册时创建`FInternalData`并把稳定tick对象加入manager；
重复调用只检查既有注册。`2910-2938`还提供1000 tick的cache-coherent连续存储与indirect allocation对照入口，明确把注册布局和
缓存局部性作为调度基础设施性能变量。

可转移原则是：插件/系统注册阶段建立稳定描述和一次性类型擦除，运行时实例与帧调度消费已发布结构；不在SDK、registry和scheduler
每层重复heap indirection。Zircon不应照搬UE对象指针模型，Rust侧仍要保留per-instance callback所有权和generation/revoke安全。

## 量化验收

矩阵为systems S=0/1/100/1k、instances/system I=1/2/100、sets/constraints C=0/1/8/100、reload/unload R=0/1/100。
记录factory/build Arc allocations与bytes、dynamic dispatch count、registration wall p50/p95、instance build wall p50/p95、RSS、
revoke残留和frame scheduler consumption。M0源码门为SDK factory allocation=0、extra SDK dispatch=0；语义门为factory builds=I、
callback state跨instance共享=0、ordering/clock-domain/revoke parity=100%。

受管release allocator receipt与current-source Cargo尚未执行；当前validator session不可执行且不得改用未经登记target。F0/F4
WPR/RSS/power也仍pending，无launchable current-source executable，因此不运行WPR；本切片非渲染，不要求RenderDoc。本轮不把
17/21 SDK迁入`review.md`，不提交milestone，不发送完成企微。
