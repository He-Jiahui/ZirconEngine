---
related_code:
  - zircon_runtime/src/ui/surface/
  - zircon_runtime/src/ui/surface/property_mutation/metadata_dirty.rs
  - zircon_runtime/src/ui/surface/surface/rebuild.rs
  - zircon_runtime/src/ui/surface/surface/interaction_state.rs
  - zircon_runtime/src/ui/surface/surface/pointer_component_events.rs
  - zircon_runtime/src/ui/tests/surface_node_pool.rs
  - zircon_runtime/src/ui/tests/surface_dirty_domains.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/01-slate-input-dispatch-core.md
  - docs/plans/zircon_editor/editor_ui/02-layout-taffy-and-containers.md
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - docs/plans/zircon_editor/editor_ui/09-editor-modules-and-design-parity.md
reference_sources:
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/FastUpdate/WidgetProxy.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/FastUpdate/SlateInvalidationWidgetHeap.h
  - dev/UnrealEngine/Engine/Source/Runtime/UMG/Public/Blueprint/UserWidgetPool.h
  - dev/slint/internal/core/partial_renderer.rs
tests:
  - node-pool subtree slot-filter source-level RED to GREEN guard passed
  - surface dirty-summary source-level RED to GREEN guard passed
  - rustfmt check and scoped diff check passed
  - current-source Windows surface tests pending behind shared Cargo FIFO
  - arranged/frame/pool/diagnostics/input scale counters pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime UI surface核心逐文件性能静态审查（2026-07-18）

## 范围与覆盖

本批逐文件完整阅读21个生产文件：`zircon_runtime/src/ui/surface`顶层17/17，`property_mutation/metadata_dirty.rs` 1/1，以及`surface/{rebuild,interaction_state,pointer_component_events}.rs` 3/21；`input`、`render`和其余default-interaction/event-routing仍明确pending。除本会话新增修改外，生产范围审查前无外部脏文件；连同前批，`ui`累计208/783。产品调用追踪覆盖asset preview、template hit surfaces、viewport toolbar、workbench reference、Runtime Diagnostics与virtual rows。

## PERF-MVP-277/281：arranged无索引与下游全量stage

arranged构建对每node重复祖先clip/visibility和全slots查找，canvas额外做child contains；`UiArrangedTree::get`在z/paint排序Vec上始终线性find，focus/hit/render/reflector再放大为O(N²)。rebuild虽调用incremental layout，但只要layout/style/text/visible-range任一dirty，仍全量重建arranged/hit并跑render extract；input/hit也重建全部arranged。

本轮源码RED→GREEN把dirty flags与dirty node count两次全树scan合为一次。EditorUI02/08仍需generation-owned dense node/slot index、一次DFS继承effective属性，并让changed nodes/ranges贯穿arranged/hit/render stage。UE Slate unique invalidation heap只处理dirty widgets并按需要向parent传播；Slint PartialRenderer以cache generation+property tracker计算dirty region，均支持此处的增量边界。

## PERF-MVP-278：SurfaceFrame每次全复制并重建ECS快照

`surface_frame()`深clone arranged/render/hit/focus等大对象，重新构建带owned note Strings的pipeline report，并无条件全树生成ECS node DTO及render/hit count maps。所有ECS delta/schedule helper也先物化完整current snapshot。编辑器toolbar与临时hit surface会真实消费frame；已有EditorUI08 toolbar storm还确认unchanged recompute重复临时surface、rebuild与frame clone。

EditorUI08需让每个stage generation发布一次immutable Arc artifact，consumer只借用/clone handle；轻量window/report访问不应携带tree/render/hit/ECS。ECS changed set应由dirty transaction直接产生，不能以两份full snapshot求diff。

## PERF-MVP-279：node pool无界且维护超线性

pool以component+control id+完整path String为key，保存完整UiTreeNode且没有count/bytes/age预算。virtual rows批量prune原先每node都对全部slots retain，grow时每insert全树找max paint order。本轮源码RED→GREEN用detached BTreeSet在recycle loop前只过滤slots一次。

EditorUI02需把pool收敛为family/type reusable object+identity rebind，批量分配paint-order range并更新slot/index；active/inactive均受class/global byte预算、idle age和owner shutdown管理。UE `FUserWidgetPool`按class复用active/inactive实例并提供Release/Reset/ReleaseResources，尤其强调owner释放资源，当前surface pool缺失这些生命周期门槛。

## PERF-MVP-280：诊断与timeline默认全物化

默认debug options开启commands/hit cells/overdraw/overlay；调用先承担PERF278 frame copy，再为所有nodes/properties/actions与counts建owned maps。overdraw按窗口网格分配数组、每command创建cell-index Vec并在线性Vec中去重node；4K和长command list会显著放大。timeline虽限制frame数，仍持完整snapshot，读取时深clone全部历史。reflector同样每次为所有节点重建属性与action maps。

EditorUI08/09需改成轻量summary+显式section request，selected-node属性按需或delta；overdraw有独立采集预算，timeline持Arc且同时受entry/byte/age上限，导出流式序列化。诊断关闭或generation未变时这些工作必须为0。

## PERF-MVP-282：focus/input历史无界

focus change与每次focused keyboard/gamepad/accessibility input都向`Vec` push owned event/route，生产代码没有drain、cap或frame reset；仅测试手动clear。长编辑器会话内存持续增长，PERF278还在每份frame中复制全部历史。

EditorUI01需把事件交给dispatch outcome/observer并在消费后释放，state只保留current/previous/pending/capture；可选诊断历史进入有entry+byte+age预算和drop counter的ring。1M输入风暴必须证明live bytes有界且交付顺序不变。

## 回链而不重复立项

component state property同步继续重复String/value并把单一动作写到metadata/state flags/component values，回链PERF-MVP-265；每hover/focus/press调用runtime style subtree回链PERF-MVP-275。`shape_text_line`每调用新建layout session且editor paint按line调用，回链PERF-MVP-235/236的single-shape/session ownership。slot mutation线性position回链PERF-MVP-261。

## 责任计划与验收

EditorUI02收到arranged/stage和pool两份failure，EditorUI08收到frame/ECS及diagnostics两份，EditorUI01收到focus history一份。以1/100/1k/10k nodes、depth 1/16/64、10k virtual scroll、1M input、1080p/4K diagnostics记录node/slot/ancestor probes、stage visits、full-copy bytes、pool/timeline/history resident bytes与CPU p95；current-source Cargo、MVP product trace与像素/hit parity完成前，本批21文件仍留pending。
