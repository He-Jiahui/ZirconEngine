---
related_code:
  - zircon_runtime/src/navigation
canonical_review:
  - docs/plans/performance/01/2026-08-23-runtime-navigation-fallback-currentness-and-production-owner-revalidation.md
protected_targets:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/performance/pending.md
  - docs/plans/performance/review.md
owner_plans:
  - docs/plans/performance/02-unreal-aligned-engine-system-hard-cutover.md
  - docs/plans/zircon_runtime/runtime/03-schedule-and-frame-loop-alignment.md
  - docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md
  - docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md
  - docs/plans/zircon_runtime/runtime/08-ecs-kernel-data-alignment.md
  - docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
  - docs/plans/zircon_runtime/runtime/14-runtime-module-family-closeout.md
  - docs/plans/zircon_plugins/05-navigation.md
doc_type: protected-plan-routing
status: update_requested_not_applied
---

# Runtime navigation fallback保护计划路由（2026-08-23）

## Performance01接纳请求

把`zircon_runtime/src/navigation/**`记录为15/15 Rust文件、3,351 physical lines、116,220 bytes、26 inline
tests，ordered path + NUL + raw bytes + NUL SHA256
`067c4f0966dd40ed20ae7f1ed9b5555248f4f65887f9410e81e152253bec6e0a`；状态为
`static_current_revalidated / structural_and_dynamic_pending`。

旧PERF-MVP-437/438应纠正：builtin已完成typed projection、bounded spatial avoidance、shared-edge adjacency、
polygon BVH、epoch scratch、area table和repath budget/cache；这些局部问题不得继续作为open current finding。
现存production瓶颈是plugin全world scan/JSON、每query asset clone与native owner重建、silent fallback、三套
authority、global lifecycle、Transform直写、Bake stub和frame demand缺失。

## 现有owner计划接纳请求

- Performance02/Runtime14/Plugins05：插件Recast成为唯一production owner；unlinked为typed Unavailable；
  删除builtin独立算法、plugin legacy自动路由和native failure fallback，不留facade或双写。
- Runtime04：定义真实geometry/collision cook lease、Recast profile receipt、Detour tile artifact、tile attach/detach
  generation、last-good publish和retire lease；禁止query时从raw asset重建。
- Runtime08：提供NavWorldKey、typed dense scene projection和change admission；stable frame的全world node scan、
  JSON decode和容器重建必须为0。
- Runtime03/11：active agent/pending bake接reactive frame demand与wake；build/query使用唯一bounded task runtime、
  sync/batch/async lanes、deadline/cancel/fairness/queue age和shutdown receipt。
- Runtime07：分别计量asset clone bytes、navmesh builds/query、query pool wait、node visits、projection rows、Crowd/
  avoidance、bake queue/apply和overlay bytes，不用总frame time掩盖backend切换。
- AI/Script owner：改用request-id command/outcome与capability status；禁止以event storage推断可用性，禁止脚本
  单次host call绕过scene cadence直接驱动完整navigation tick。
- Editor/Graphics owner：async bake ticket/progress/commit与static generation tile pages + dynamic agent delta；
  stable navmesh不得每帧物化全部triangle DTO。

## 受保护索引状态请求

- `pending.md`：链接canonical review与Optimize08d，标记unique owner、NavWorld lifecycle、real tile artifact、
  persistent query pool、typed projection、movement/AI/Editor和动态规模数据全部开放。
- `review.md`：current-source managed Cargo、linked/unlinked/obstacle/off-mesh hard-cut回归、1/100/10k规模、
  至少31次WPR CPU/RSS/power、native fault/sanitizer和真实产品overlay capture通过前不得加入。
- Performance01主计划：明确“builtin局部算法已改善但不是plugin产品热点”，停止向fallback增加功能；性能门
  必须证明steady-state `navmesh builds/query=0`与`asset clone bytes/query=0`。

本Session不修改受保护索引、主计划或owner计划。当前没有动态验收里程碑，因此不提交git commit，也不发送
企微量化通知。
