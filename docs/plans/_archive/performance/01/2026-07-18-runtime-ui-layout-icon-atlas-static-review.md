---
related_code:
  - zircon_runtime/src/ui/layout
  - zircon_runtime/src/ui/icon_atlas
  - zircon_runtime/src/ui/surface/surface/rebuild.rs
  - zircon_runtime/src/ui/tests/layout_slots.rs
  - zircon_runtime/src/ui/tests/surface_dirty_domains/incremental_layout.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/02-layout-taffy-and-containers.md
  - docs/plans/zircon_editor/editor_ui/01-slate-input-dispatch-core.md
reference_sources:
  - dev/bevy/crates/bevy_ui/src/layout/ui_surface.rs
  - dev/bevy/crates/bevy_ui/src/layout/mod.rs
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Layout/Geometry.h
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Layout/ArrangedChildren.h
tests:
  - layout measure source-level RED to GREEN guard passed
  - incremental geometry snapshot source-level RED to GREEN guard passed
  - rustfmt check and scoped diff check passed
  - current-source Windows zircon_runtime layout tests pending
  - layout stage and virtual-list scale counters pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime UI layout/icon_atlas逐文件性能静态审查（2026-07-18）

## 范围与覆盖

已逐文件完整阅读`zircon_runtime/src/ui/layout` 22/22与`icon_atlas` 3/3，共25/25个Rust文件；两目录均无外部脏文件。连同前两批，`ui`累计生产文件112/783。定向追踪`UiSurface::rebuild_dirty`、arranged tree/hit-test产品调用以及Bevy长期持有Taffy tree的参考实现。

## PERF-MVP-259：增量指标掩盖全树stage

layout入口在dirty subtree discovery前无条件执行responsive MUI：三轮收集node id后逐node解释metadata，另扫全部slots；`incremental_layout_roots`再扫全nodes。任何layout dirty完成后surface仍重建完整`UiArrangedTree`和hit-test。现有`layout_visited_node_count`只计measure/arrange subtree，因此“visited=1”不能证明整体增量。本轮先把geometry snapshot与changed compare从全tree缩到visited set；EditorUI02还需用style/viewport/tree generation索引responsive节点和dirty roots，并让arranged/hit结构接受geometry delta。既有PERF-MVP-032只负责dirty flags/count/clear的三次N扫描，二者共享dirty authority但不重复记账。

## PERF-MVP-260/261：slot全局查找与一次性Taffy树

`slot_for_container_child`线性扫描全局slots，ordering、measure、axis、grid、Taffy资格检查、input构建和arrange对同一child反复调用，规模化后为多轮O(N*S)。布局generation应拥有edge slot索引与一次构建的ordered child/layout input。

Taffy bridge则为每个容器、每次arrange新建`TaffyTree<()>`，插入全部child leaf与parent，求解后立即丢弃。Bevy的`UiSurface`长期保存`TaffyTree<NodeMeasure>`、entity映射和children scratch，只对changed style/context/children做upsert/remove。Zircon应采用同一ownership边界，并让特殊容器和Taffy共同写回单一arranged authority。

## PERF-MVP-262：virtual window仍访问全部行

Scroll/Virtual arrange先为全部children生成positions，再逐项判断窗口，offscreen subtree还递归clone children并清零layout。固定extent可以O(1)求首尾index；动态extent需与Text09测量缓存共用prefix/Fenwick或分块索引。每次scroll只应访问visible+overscan与进出窗口edge，focus/accessibility需要消费range generation，不能用逐帧递归隐藏维持状态。

## PERF-MVP-263：measure/arrange临时工作

本轮源码RED→GREEN已把`ZR_UI_LAYOUT_PROFILE`读取从每node降为每measure subtree一次，删除每node`template_metadata`深clone，并把ordered desired从“排序id后逐id线性find payload”改为携带payload的一次稳定排序。剩余children clone、axis active-index重复分配/重扫、wrap content二次扫描与每frame完整engine selection记录交给EditorUI02统一scratch/generation/diagnostic模式收敛。

## icon_atlas接入前门禁

产品调用检索只命中测试，当前不宣称MVP瓶颈。`build_plan`仍会逐次重新解析SVG并按最大cell排方格；`max_side_px`只clamp宽高而不验证容量/分页，足量icon可能生成越界rect/UV。接入产品前必须以asset generation缓存parsed geometry，并验证capacity、多页/拒绝路径与UV边界；在真实调用和trace出现前不单独增加性能ID。

## 责任计划与验收

EditorUI02收到false-incremental、slot/Taffy与virtualization三份failure。100/1k/10k nodes、1/100 nested containers、1k/10k/100k rows记录各layout stage visits、slot probes、tree create/upsert、transient bytes、visible/offscreen visits与CPU p50/p95。current-source Cargo、MVP workbench layout/scroll trace和RenderDoc像素完成前，25/25仍留pending。
