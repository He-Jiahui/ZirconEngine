---
related_code:
  - zircon_runtime_interface/src/ui/template
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md
  - docs/plans/zircon_editor/editor_ui/04-style-theme-and-painter-selector.md
  - docs/plans/zircon_editor/editor_ui/05-ui-asset-management.md
reference_sources:
  - dev/slint/internal/compiler/llr/lower_to_item_tree.rs
  - dev/bevy/crates/bevy_asset/src/server/info.rs
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/FastUpdate/SlateInvalidationWidgetList.cpp
tests:
  - zircon_runtime_interface/src/tests/contracts.rs
  - zircon_runtime_interface/tests/ui_binding_control_prop_ref.rs
  - current-source Windows template contract and runtime consumer tests pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime interface UI template 性能静态审查（2026-07-23）

## 范围与覆盖

`zircon_runtime_interface/src/ui/template/**`当前源 **56/56** 个受跟踪且clean的 Rust 文件、**2,589** 行已逐文件阅读，并反查runtime package/cache/action/resource/surface消费者、interface合同及`dev/`参考。本轮未修改Rust源码。

## 性能结论

- `UiCompileCacheKey::invalidation_snapshot()`每次clone widget/style fingerprint maps；cache record又拥有header、cache key及其完整snapshot。package manifest在外层计算artifact fingerprint后，内层cache record对同一bytes再次全量hash，同时重复clone header/cache metadata。归 **PERF-MVP-308**：单次package build只计算一次fingerprint，manifest持single generation metadata owner，snapshot/cache/report通过handle或共享不可变owner引用。
- `UiBindingExpression::parse()`先把输入收集为`Vec<char>`，token中的identifier/String再次分配，parser `next()`再clone token；递归unary/parentheses、token/node/string均无预算。`UiSelector::parse()`又为compound建String、再建char Vec和token String，且同selector在document/style/component/invalidation路径重复parse。归 **PERF-MVP-311**：source generation只lex/parse一次到有深度、节点、token和字符串预算的typed arena，各validator借用handle。
- `UiResourceKind::infer_from_path_and_uri()`对path/URI做lowercase String、segments Vec及相邻segment `format!`；resource collector与surface index还分别遍历同一TOML树。action side-effect推断也构造两份lowercase String和joined String；localized ref即使成功也先把path转owned String。均归 **PERF-MVP-311**，要求共享node/path/resource index、ASCII不区分大小写的borrowed识别及错误时才物化诊断文本。
- `UiInvalidationImpact`把resource dependency直接标为`rebuild_required`，selector/style value无条件扩散到style/layout/hit/render/text五域；source/import/descriptor/component变化则全域dirty。归 **PERF-MVP-309**：由compiled dependency、selector候选和asset-to-node reverse index产精确node/domain set，resource-only更新不得默认重建树。
- `UiRawAssetPrototype`已使用`UiPrototypeNodeHandle(u32)`和Vec O(1)节点定位，是正向基线；但document/prototype/package DTO仍以recursive String/BTreeMap/Vec拥有完整authoring payload。继续回链 **PERF-MVP-306/312**：compiled generation持canonical typed arena，runtime tree与editor preview只持dense handle和mutable override/delta。
- Slint lowering以typed index、sub-component mapping和单次`LoweringState`发布item tree；Bevy `AssetInfos`维护asset info及loader-dependent反向索引；UE Slate invalidation list以widget index/range修复受影响区间。Zircon采用其owner/index/invalidation原则，不复制具体实现。

## 动态验收

1. imports/artifacts各1/100/10k、artifact 1 KiB/1 MiB/100 MiB：记录fingerprint passes/bytes、map/header/cache metadata clone bytes、owners、RSS与package p95；每artifact hash=1，generation metadata owner=1。
2. expressions/selectors/resources/actions各1/100/10k，depth 1/64/1k、string 0/1 KiB/1 MiB：记录char/token/path/lowercase allocations、parse/index builds、tree passes与p95；同generation parse/index≤1，超预算返回typed error且无栈溢出。
3. assets/nodes/dependencies/rules各1/100/10k：记录invalidated edges/nodes/domains、root rebuilds、layout/hit/render visits与reload p95；resource-only和selector-local变化只触达真实consumer。
4. current interface合同、runtime package/cache/resource/style tests通过；运行current-source Windows managed Cargo及F4 asset preview/edit/hot-reload产品trace。

current-source Cargo、规模counter与F4产品trace未完成，因此该目录继续保留在 `pending.md`，不进入 `review.md`。
