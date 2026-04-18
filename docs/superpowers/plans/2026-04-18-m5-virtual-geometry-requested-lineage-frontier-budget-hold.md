---
related_code:
  - zircon_graphics/src/visibility/planning/build_virtual_geometry_plan/build.rs
  - zircon_graphics/src/tests/visibility.rs
implementation_files:
  - zircon_graphics/src/visibility/planning/build_virtual_geometry_plan/build.rs
plan_sources:
  - user: 2026-04-18 下一步是更深的 unified-indirect / residency-manager cascade，把同一套 frontier truth 继续推进到真实 GPU uploader / page-table / split-merge frontier policy
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - .codex/sessions/20260417-1415-m5-vg-slot-assignment-ownership.md
  - docs/superpowers/plans/2026-04-18-m5-virtual-geometry-pending-cascade-descendant-hold.md
  - docs/superpowers/plans/2026-04-18-m5-virtual-geometry-frontier-priority-and-active-request-lineage.md
tests:
  - zircon_graphics/src/tests/visibility.rs
  - cargo test -p zircon_graphics --offline --locked visibility_context_only_holds_requested_virtual_geometry_lineage_when_frontier_budget_collapses -- --nocapture
  - cargo test -p zircon_graphics --offline --locked visibility -- --nocapture
  - cargo test -p zircon_graphics --offline --locked virtual_geometry -- --nocapture
doc_type: milestone-detail
---

# M5 Virtual Geometry Requested-Lineage Frontier Budget Hold

## Goal

把 `Virtual Geometry` 的 split-merge frontier policy 再往前推进一层：当当前帧 budget 已经收窄、frontier 被迫塌回 coarse root 时，系统只持续保热真正仍然 relevant 的 requested lineage，而不再把同一 visible frontier 下的整个 sibling subtree 一起钉住。

## Non-Goal

- 本轮不改写 cluster refine 排分本身。
- 本轮不引入新的 GPU-side residency algorithm。
- 本轮不移除既有的 merge-back child hold / pending-cascade descendant hold；只是把它们的作用域收得更精确。

## Delivered Slice

### 1. Requested lineage 持续保热，不再依赖 sibling subtree 一起被 pin

- `build_virtual_geometry_plan(...)` 现在会显式计算：
  - `requested_lineage_targets`
  - `streaming_target_lineage_targets`
- resident hidden page 只有在满足以下之一时才继续被排除在 `evictable_pages` 外：
  - 它属于当前 pending request 自己那条 lineage
  - 或者它属于当前 `streaming_target_clusters` 仍然想继续回到的更深 lineage

于是 repeated collapsed frame 里：

- requested branch 自己仍然保持 hot
- 但 unrelated sibling branch 不会只因为“共享同一个 visible frontier ancestor”就被一并 pin 住

### 2. Budget collapse 不再把 full sibling subtree 当成 implicit frontier truth

旧逻辑的保护条件本质上是：

- 只要 hidden resident cluster 的 visible frontier ancestor 出现在 requested frontier 里
- 就继续保热

这在 cluster budget 收窄时会过宽：

- 当前 streaming target 已经不再打算回到 sibling subtree
- 但 sibling pages 仍会被同一个 visible root 祖先顺手保护

新逻辑改成：

- requested lineage 自己始终保热
- sibling 只有在 current `streaming_target_clusters` 仍然覆盖它时才继续保热

这样 split-merge policy 就开始真正消费当前帧 frontier truth，而不是继续把 “shared root ancestor” 当成隐式代理。

### 3. 旧的更宽 cascade hold 仍然保留在它该生效的地方

这次不是简单把保护范围缩小，而是把它收成两层：

- `requested_lineage_targets`
  - 负责保证 pending request 自己那条恢复路径不会断
- `streaming_target_lineage_targets`
  - 负责在当前 budget 仍允许更深 frontier 时，继续保住那些仍然 relevant 的 sibling / cousin lineages

因此原有回归仍然成立：

- children visible while requesting nonresident grandchildren
- merge-back child hysteresis
- multi-level collapse descendant hold
- wider cascade hysteresis

但新回归也成立：

- 当 budget 已经把 frontier 真正收窄到 root 时，不再继续 pin unrelated sibling subtree

## Why This Slice Exists

上一轮已经把 split-merge policy 扩到了：

- split hold
- merge-side coarse-parent hold
- merge-back child hold
- pending-cascade descendant hold

但它还缺一个“frontier 真值作用域”问题：

- protection 仍然主要看 shared visible frontier ancestor
- 没有分清：
  - requested lineage 自己必须保热
  - sibling lineage 只有在当前 streaming target 仍然想回去时才该保热

这会导致 budget collapse 时：

- requested branch 正常 pending
- unrelated sibling subtree 却仍然被整片 pin 住
- 进而把 eviction pressure 和 slot recycling 压到错误的 resident pages 上

本轮补上的就是这条 requested-lineage / streaming-target-lineage 的精确边界。

## Validation Summary

- `visibility_context_only_holds_requested_virtual_geometry_lineage_when_frontier_budget_collapses`
  - 证明 repeated collapsed frame 里 requested branch 会继续保持 hot，但 unrelated sibling subtree 会重新进入 eviction pressure
- `visibility`
  - 证明更早的 split/merge / cascade regressions 没有被这条更精确的 frontier policy 打穿
- `virtual_geometry`
  - 证明这条 visibility-side policy 仍与 runtime prepare、GPU uploader、unified indirect、cluster raster 主链兼容

## Remaining Route

- 继续把这条 lineage-scoped frontier policy 下沉到更真实的 runtime residency-manager cascade / unified-indirect authority。
- 继续推进更深的 cluster raster consumption、unified indirect ownership 和 residency-manager cascade。
