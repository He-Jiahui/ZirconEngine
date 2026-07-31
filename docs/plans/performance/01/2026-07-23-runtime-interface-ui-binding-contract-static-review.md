---
related_code:
  - zircon_runtime_interface/src/ui/binding
  - zircon_runtime/src/ui/event_ui/manager
  - zircon_runtime/src/ui/binding/update_report.rs
  - zircon_runtime/src/ui/surface/property_mutation.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md
  - docs/plans/zircon_editor/editor_ui/01-slate-input-dispatch-core.md
  - docs/plans/zircon_editor/editor_ui/06-component-library-mui.md
reference_sources:
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Framework/Commands/UICommandList.h
tests:
  - zircon_runtime/src/ui/tests/binding.rs
  - zircon_runtime_interface/src/tests/ui_contract_spine.rs
  - current-source Windows binding tests pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime interface UI binding 性能静态审查（2026-07-23）

## 范围与覆盖

`zircon_runtime_interface/src/ui/binding/**` 当前源 **10/10** 个 Rust 文件、**874** 行已逐文件阅读，并反查runtime event manager、binding update builder、property mutation与input/pointer/navigation result消费者。runtime 4条binding tests与interface 1条binding update合同覆盖native roundtrip、drop payload、headless typed router、update分类/dirty union与serde默认。目录当前无工作区改动，本轮未修改源码。

## 性能结论

- `UiEventKind`和dirty/status/kind枚举为Copy静态合同，headless `UiEventRouter`直接以typed `UiEventPath`查表，不需要native String；这是应保留的正向基线。
- 产品`UiEventManager::invoke_binding()`却在每次调用执行`binding.native_binding()`，完整格式化view/control/event/action/arguments并转义字符串后用新String查`routes_by_binding`；命中后又clone arguments，构造context时clone binding，success/failure再clone binding，broadcast还clone完整result。registration查询也重复native formatting。新增 **PERF-MVP-572**。
- `native_repr`对call/array先为每个值生成String、collect Vec再join，字符串escape连续执行5次replace；parser为view/control/symbol/string创建owned String，array递归且没有input bytes、argument count或nesting hard limit。它可以保留为authoring/serde边界，但不得处于稳定输入事件热路，外部文本必须在递归/分配前受预算约束。
- `UiBindingUpdate`同时拥有source/target property或path String、previous与current `UiValue`、dirty Vec和message；`UiValue`可递归拥有String/Array/BTreeMap。runtime property mutation还clone request property到report/source/target、clone value、失败message双份，并在追加update或dirty变化后重扫report、把同一dirty Vec clone到每个update。该证据补强 **PERF-MVP-265** 的single typed patch，文本多字段动作继续联动258/295，不另建重复任务。
- report dirty union的`Vec::contains`最多10个domain，单独不是主因；先消除每字段transaction和wide payload复制，再以counter决定是否换bitmask/inline storage。

## PERF-MVP-572 设计

1. template/route generation冻结时为binding发布稳定`UiRouteId`或generation-scoped typed binding handle；产品input、route-intent与component event直接传handle，不在invoke时格式化native String或解析参数。
2. native binding String仅用于authoring、serde/codec和unknown-binding错误；文本解析增加input bytes、arguments、array nodes、nesting和single/total string hard limits，且在递归/owned graph形成前拒绝。
3. route entry持有immutable shared binding/default-argument artifact；invoke消费caller arguments或借用shared defaults，context/result/broadcast共享同一binding/result owner。慢/死subscriber队列和fanout clone继续由PERF-MVP-252治理。
4. reload/register原子发布route generation，stale handle明确拒绝或重新resolve；不得保留产品String lookup兼容旁路。

## PERF-MVP-265 补充验收

1. `UiComponentStatePatch`一次提交canonical field identity、previous/current ownership、dirty bitmask与实际changed set；alias只在外部投影，source/target相同property不得双String ownership。
2. live success receipt默认只携带node/property handle、status、dirty与必要value handle；previous/current wide value和message仅在有界诊断capture或serialization边界物化。
3. 1/100/10k fields、0/1KiB/1MiB value、单action 1/8/18 mutations记录property/value/message clone bytes、report recompute、dirty copies、transactions和input p95；每action transaction=1、stable/unchanged write=0、wide value authoritative owner=1。

## 参考引擎对照

Unreal Slate `FUICommandList`的binding map以`TSharedPtr<const FUICommandInfo>`作为key，执行入口接收同一shared command identity，不在每次输入时把完整命令与参数重新格式化成字符串。Zircon采用相同的“编译期/注册期身份、运行时handle”原则，但保留自身`UiRouteId`、native codec与serde格式。

current-source Cargo、route/binding规模allocation counter与F4 input/property产品trace未完成，因此该目录继续保留在 `pending.md`，不进入 `review.md`。
