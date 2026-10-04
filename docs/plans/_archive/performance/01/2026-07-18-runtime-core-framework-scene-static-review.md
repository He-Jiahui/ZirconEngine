---
related_code:
  - zircon_runtime/src/core/framework/scene
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/runtime/05-scene-editor-boundary-closeout.md
  - docs/plans/zircon_runtime/runtime/08-ecs-kernel-data-alignment.md
tests:
  - twenty-four of twenty-four Rust files reviewed
  - current-source Cargo, allocation counters and scene/editor traces pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime core framework scene逐文件性能静态审查（2026-07-18）

## 范围与覆盖

已完整阅读`zircon_runtime/src/core/framework/scene/**`当前Rust文件24/24，覆盖entity/component property path、component descriptors/property values、level/world/stage contracts、mobility reflection及scene-facing physics metadata/serde/tests。除路径DTO与reflection转换外，本目录主要是Copy小值、serde schema和trait边界，没有锁、线程、channel、I/O或逐帧调度实现；physics axis固定3项，线性visitor不是规模热点。

## PERF-MVP-329：路径同时拥有raw与逐段String

`EntityPath`同时拥有完整`raw: String`和`Vec<String>`，parse先为每段分配再join第二份正文；`ComponentPropertyPath`同样拥有raw、component与property segment Strings。当前构造已对component path单pass计算容量，避免了更早的重复parse，但不可变路径被clone进animation track、ScenePropertyEntry、reflection/property access与editor actions时仍按总字符和segment数复制两套owner。动态component写入还以`type_path.to_string()`建立component owner。

生产检索显示路径主要在clip/graph compile、scene property entry和显式反射访问构建，而不是每帧重新parse；因此本轮不以仓促的borrowed lifetime改写公共serde/Hash/Clone契约。Runtime05/08应建立interned `PathId`或`Arc<PathStorage>`：规范化raw单owner，segments存range/small index，clone只增Arc；compiled animation/property access以dense path ID缓存entity/component/property resolution，scene generation变化才失效。作者输入与错误报告仍可按需保留raw文本。

## 验收要求

对paths 1/100/10k、segments 1/8/128、text 16 B/1 KiB/1 MiB、clone/lookup 1/100/1M及scene generations记录String owners/bytes、segment alloc、hash/probe、entity/component resolution visits、cache hit与p95/RSS：normalized正文owner=1、path clone正文bytes=0、segment access不分配、stable generation resolve≤1；serde/display/hash/equality、empty/trim/error、animation/scene/editor action parity、Cargo/F2/F4 trace通过前，本目录留在`pending.md`。
