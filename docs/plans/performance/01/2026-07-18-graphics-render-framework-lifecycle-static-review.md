---
related_code:
  - zircon_runtime/src/graphics/runtime/render_framework/capability_summary
  - zircon_runtime/src/graphics/runtime/render_framework/capture_frame
  - zircon_runtime/src/graphics/runtime/render_framework/compiled_feature_names
  - zircon_runtime/src/graphics/runtime/render_framework/create_viewport
  - zircon_runtime/src/graphics/runtime/render_framework/destroy_viewport
  - zircon_runtime/src/graphics/runtime/render_framework/query_stats
  - zircon_runtime/src/graphics/runtime/render_framework/query_virtual_geometry_debug_snapshot
  - zircon_runtime/src/graphics/runtime/render_framework/viewport_surface
tests:
  - current render-framework lifecycle and query slice 23 of 23 Rust files reviewed, 356 lines
  - create, destroy, surface, capture, stats and debug-snapshot lock scopes traced
  - compiled feature-name call graph confirms two owned projections per submission
  - current-source Cargo, concurrent slow-driver fixture, F2 and RenderDoc pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Graphics render-framework lifecycle/query静态审查（2026-07-18）

## 当前源覆盖

当前23/23个Rust文件、356行已逐文件静态阅读，范围为render-framework root、capability summary、capture、compiled feature names、create/destroy viewport、surface bind、stats/VG debug query、queue/error mapping及runtime-frame facade。小module wiring按逻辑owner合并记录，避免为2-5行文件膨胀计划。

## 全局锁瓶颈

create/destroy/bind/unbind/capture均受全局operation/state锁约束。`bind_viewport_surface`在两把锁内调用renderer创建native surface、查询/配置capabilities，并构造present sampler/layout/shader/pipeline；driver stall会阻塞所有viewport。destroy同样在锁内遍历camera histories并调用renderer release。新增`PERF-MVP-411`要求短锁reservation+generation ticket、专用render-owner command lane执行driver/GPU、短锁条件发布，固定same-viewport顺序而不串行独立viewport查询。

`capture_frame_if_newer`已在RGBA clone前检查generation，stale poll clone=0；但新帧仍在state锁内深clone`CapturedFrame`，归`PERF-MVP-023/411`的Arc capture snapshot与锁外materialization。`query_stats`在锁内clone整份RenderStats，`query_virtual_geometry_debug_snapshot`在锁内clone大payload，分别继续回链`PERF-MVP-324/347/411`；query只应短锁clone generation Arc，深复制或UI投影在锁外。

`compiled_feature_names`每次把enabled features物化为owned String Vec，并在submission context与stats更新各调用一次。它属于`PERF-MVP-362`的compiled immutable artifact和`PERF-MVP-324`的diagnostic delta，不新增重复任务。capability summary只在framework构造时clone短表；queue mapping/error mapping为O(1)或失败边界。

本切片未做局部锁移动：renderer当前嵌在state owner内，单独释放mutex而不先建立ticket/generation会引入destroy/rebind race；把capture clone简单移出锁也需要Arc owner合同。正确修复必须先完成`PERF-MVP-411`的ownership与锁序。

## 验收状态

静态调用图和既有任务边界已核对。Cargo协调器JSON解析失败仍阻止current-source测试；1/2/8/64 viewports、slow surface/history 0/10/100ms、1080p/4K capture、541/10k stats、64MiB debug snapshot的lock wait/hold、command age与race fixture，F2多视口和RenderDoc均未完成。保留`pending.md`，不进入`review.md`。
