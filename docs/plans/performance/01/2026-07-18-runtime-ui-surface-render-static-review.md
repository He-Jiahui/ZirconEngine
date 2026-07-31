---
related_code:
  - zircon_runtime/src/ui/surface/render/**/*.rs
  - zircon_runtime/src/ui/tests/render_*.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/03-text-and-font-stack.md
  - docs/plans/zircon_editor/editor_ui/04-style-theme-and-painter-selector.md
  - docs/plans/zircon_editor/editor_ui/06-component-library-mui.md
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - docs/plans/zircon_runtime/render/17-performance-and-profiling.md
reference_sources:
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Rendering/ElementBatcher.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/FastUpdate/WidgetProxy.cpp
  - dev/slint/internal/core/item_rendering.rs
  - dev/slint/internal/core/partial_renderer.rs
  - dev/slint/internal/core/model.rs
  - dev/slint/internal/core/model/adapters.rs
tests:
  - fourteen source-level RED to GREEN performance guards passed
  - rustfmt check and scoped diff check passed
  - current-source Windows UI tests pending behind shared Cargo FIFO
  - extract/cache/model/text/command scale counters pending
  - current-source product GPU and RenderDoc capture pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime UI surface render逐文件性能静态审查（2026-07-18）

## 范围与覆盖

本批逐文件完整阅读`zircon_runtime/src/ui/surface/render/**`全部32/32生产文件：extract/cache/resolve/text核心8、collection rows 5、buttons/dropdowns/segmented/selection/sliders/text fields 6、popup 4、feedback 4以及command palette/chrome/dialog/drag overlay/notification 5。`surface`当前批累计71/128，整个`ui`累计258/783。

## PERF-MVP-288：全量extract后cache仍不能表达多command节点

每次render stage先遍历全部draw order并完整解析所有visible node，再对每node依次调用14类specialized renderer；每个非目标renderer仍做component probe并返回临时Vec。cache只在完整command构建后比较，无法节省extract；更严重的是entries只以`node_id`为键，而button/tree/table等一个node生成多条command，前序command会相互覆盖，稳定帧reused统计与damage都失真。owner base command与未声明surface suppression的专用控件还可能重复绘制surface，需像素/RenderDoc确认。

本轮RED→GREEN让不可见node在视觉解析前退出，并让stable cache hit直接移动已拥有的incoming command，不再丢弃它后深clone cached command。EditorUI08应发布generation-owned per-node command range并用`node_id + local command identity`或range handle精确缓存；dirty changed range直接patch，不先全量extract。Slint ItemCache以item identity+dependency tracker只在dirty时update，PartialRenderer再按dirty region过滤；当前post-build equality不是同级缓存。

## PERF-MVP-289：每帧从TOML重复恢复视觉契约

`resolve_style`为每node逐字段多次查询attributes/style overrides并复制font/color/language String；owner suppression的text/image/surface与special renderer又分别重复component分类。每条quad/text/icon都新建`UiResolvedStyle`并复制常量或metadata颜色String。Button原先每帧最多5次variant分类且每次collect/join/lowercase，popup/dialog/chrome/feedback/text enum也重复分配normalize String。

本轮14组止损中已把7类style enum、Button/Feedback tone、popup placement/origin、notification/dialog/chrome分类改成borrowed ASCII compare；Button kind只解析一次，Dropdown label只取一次，Segmented在非目标node上先退出并借用selected值。EditorUI04仍需让compiled style/painter descriptor携带family、behavior mask、state tokens与interned brush/font handles，command只引用handle+dynamic state delta。

## PERF-MVP-290：集合控件render每帧重建owned模型

Table cells、Dropdown/Segmented options、Popup menu/options、CommandPalette与NotificationCenter每帧从TOML/`UiValue`重建owned row DTO、多个BTreeSet和String。CommandPalette未提供filtered list时为每row每field lowercase；提供filtered ids时又逐id线性find+clone完整row，最坏O(F*N)。Notification `visible_limit`在全量递归parse后才take；popup/notification/command rows也没有基于frame的可见范围上限。

本轮已借用collection row label、移动table cell与popup/menu/notification/palette已拥有文本、用BTreeSet直接命中，并让popup options复用一次position结果而非每row重算。EditorUI06应让PERF-MVP-266/267/268/269的generation-owned typed model直接产出visible row handles；render只访问visible+overscan，filtered mapping和state sets增量维护。

## PERF-MVP-291：command数量与每command所有权无预算

TreeRow为每个depth生成一条guide quad，Slider按外部`tick_count/steps`生成无上限quad；popup、notification、palette为所有rows生成多条command，即使超出clip。每个原子command复制颜色String与完整style，且都复用同一node id。大量层级/刻度/行会先在CPU制造command风暴，再把batch/vertex压力传到RHI。

EditorUI06应为visible rows和decorations建立明确上限/聚合primitive，Render17负责compact brush/style handles、instance/mesh batching与command-count/bytes预算。UE ElementBatcher按batch key合并render batches并区分cached/uncached source arrays；Zircon需在保留z/clip/state语义下让repeated guides/ticks/rows走可实例化数据。必须用当前源码RenderDoc验证draw/pass/overdraw，不能用旧capture代替。

## PERF-MVP-292：文本prewarm、resolve与editable正文重复拥有

owner prewarm先对visible node完整解析visual text/style并构造owned requests，extract随后再次解析；生成command后又做第二轮prewarm与missing-layout pass。全局私有prewarm pool固定2线程，未纳入统一任务预算。TextField同步layout时clone base style、composition text、command text和editable state，focus路径再把整份editable复制进layout；一份长文本在同帧多份所有权。

EditorUI03联动Text09发布generation-owned compiled text/layout handle：prewarm/extract/render共享一次source/style identity，editable text/selection/composition用Arc source+ranges，只有changed generation shape；worker使用Runtime11统一budget/cancel/age。1/100/10k文本node与1/10k/100k字符要记录parse/shape/layout passes、String/style/editable clone bytes、worker depth/age和UI CPU p95。

## 责任计划与验收

EditorUI08收到extract/cache command identity failure，EditorUI04收到visual descriptor/style allocation failure，EditorUI06收到collection model/command fanout failure，EditorUI03收到text ownership/prewarm failure，Render17收到command batching/GPU capture failure。以1/100/1k/10k nodes、1k/10k/100k rows、depth/ticks 1/100/10k、稳定300帧记录node/renderer probes、command/style/String bytes、cache hit identity、visible rows、shape/layout passes、CPU p50/p95/p99、draw/pass/vertex/upload/overdraw；current-source Cargo、F4 workbench交互trace、像素/hit与RenderDoc验收完成前仍留`pending.md`。
