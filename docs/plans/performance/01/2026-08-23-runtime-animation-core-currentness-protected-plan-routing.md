---
related_code:
  - zircon_runtime/src/animation
canonical_review:
  - docs/plans/performance/01/2026-08-23-runtime-animation-core-currentness-revalidation.md
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
  - docs/plans/zircon_plugins/04-animation.md
doc_type: protected-plan-routing
status: update_requested_not_applied
---

# Runtime animation core保护计划路由（2026-08-23）

## Performance01接纳请求

把`zircon_runtime/src/animation/**`记录为17/17 Rust文件、2,459 physical lines、86,237 bytes、17 inline
tests，ordered path + NUL + raw bytes + NUL SHA256
`74f429d2a613ae1c0cfc4861d2e40a9b147b67432ca175cab0ba2fdc945df82d`；状态为
`static_current_revalidated / structural_and_dynamic_pending`。

旧结论应纠正为：event merge已是heap；channel区间定位已二分但每sample有限性扫描使总复杂度仍为`O(K)`；
compiled sequence已有writer binding；linked与fallback互斥，但implementation owner仍重复。

## 现有owner计划接纳请求

- Performance02和Runtime14：按hard cut保留一个animation production owner；未链接插件时返回typed
  unavailable，删除runtime fallback raw evaluator和重复module/manager，不留兼容facade。
- Runtime04：定义versioned rig/clip/graph/state-machine/sequence prepared artifact、profile/cook key、residency
  lease、last-good generation和quality/error receipt；frame path禁止同步owned asset load。
- Runtime03/07/11：定义`Update -> Sample/Decompress -> Blend/IK -> Commit` phase DAG、world epoch、
  deadline/cancel/typed reject、queue age和worker scratch；删除每帧sync channel和owner立即阻塞的组织方式。
- Runtime08：提供animation instance registry、change/revision admission、dense rig slot和required-bone/LOD
  query，不再每帧扫描所有candidate并clone参数/状态容器。
- Plugins04：成为唯一manager/system/instance/diagnostic owner，复用Runtime prepared contract；当前
  `zircon_plugins/animation/runtime/src/**`需单独逐文件currentness验收，不能继承本报告17/17状态。
- Editor animation 75/76/77：authoring、preview和runtime共享同一个compiler/artifact/receipt；Editor不得
  建第二套graph/sequence求值器或以descriptor脚手架宣称能力完成。
- Graphics/physics owner：只通过dense pose generation、palette/deformation request和body subset bridge消费，
  禁止依赖逐bone scene entity双写。

## 受保护索引状态请求

- `pending.md`：链接canonical review和Optimize08c，标记unique owner、prepared artifact、instance registry、
  dense pose/task DAG、renderer/physics bridge、动态数据全部开放。
- `review.md`：在current-source managed Cargo、linked/unlinked hard-cut回归、1/100/1,000角色规模测试、至少
  31次WPR CPU/RSS/power与真实skinned RenderDoc capture通过前不得加入。
- Performance01主计划：把main-thread update/commit、worker sample/blend/IK、asset stall、pose bytes、
  compressed bytes、queue age和GPU deformation分别计量；不得用总frame time掩盖串行阶段。

本Session不修改受保护索引、主计划或owner计划。当前没有动态验收里程碑，因此不提交git commit，也不发送
企微量化通知。
