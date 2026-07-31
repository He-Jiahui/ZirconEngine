---
related_code:
  - zircon_runtime_interface/src/ui/skin
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md
  - docs/plans/zircon_editor/editor_ui/04-style-theme-and-painter-selector.md
reference_sources:
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Styling/SlateStyle.h
tests:
  - zircon_runtime_interface/src/tests/ui_skin_contracts.rs
  - current-source Windows skin tests pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime interface UI skin 性能静态审查（2026-07-23）

## 范围与覆盖

`zircon_runtime_interface/src/ui/skin/**`当前源 **2/2** 个 Rust 文件、**259** 行已逐文件阅读，并完成全仓Rust consumer反查。当前只有`ui_skin_contracts.rs`消费四个preset构造器，runtime/editor产品路径尚未接线；本轮未修改源码。

## 性能结论

- enums与preset ID均为Copy/static合同，无运行时成本。
- `material_dark()`每次重新分配descriptor id/display、31组token name/value、visual states和component roles；其他三个preset也重建owned String/Vec。`token()`线性扫描tokens。当前无产品consumer，因此没有现行帧瓶颈，不新增PERF-MVP编号。
- 接线时必须由single static或theme-generation owner一次构建immutable preset catalog，按interned/static token id索引并共享descriptor；禁止paint、pane projection或每次theme lookup调用preset构造器。该门并入 **PERF-MVP-264** 与EditorUI04 theme generation。
- Unreal `FSlateStyleSet`以`FName`键的长期TMap保存颜色、margin、brush、font和widget styles，查询返回引用/共享对象；Zircon只采用“长期registry + compact identity + borrowed lookup”原则，不照搬其类型分表。

## 动态验收

1. presets 1/4、tokens 31/1k/10k、lookups 1/1M记录constructor calls、String/Vec bytes、token comparisons和p95；stable theme generation constructor=0、owned clone bytes=0、lookup近O(1)。
2. theme reload只构建一次候选catalog并原子发布generation，旧generation由共享handle自然回收；失败保留last-good。
3. current 2条skin合同、serde、preset ID/reference/state/token值保持等价；F4 Material/Workbench paint trace确认stable frame preset/token projection=0。

current-source Cargo、规模counter与F4 theme产品接线未完成，因此该目录继续保留在 `pending.md`，不进入 `review.md`。
