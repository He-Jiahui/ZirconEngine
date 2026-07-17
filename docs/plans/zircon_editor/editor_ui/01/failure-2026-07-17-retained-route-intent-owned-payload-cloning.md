---
handoff_kind: failure
status: open
created_at: 2026-07-17
summary_slug: retained-route-intent-owned-payload-cloning
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_editor/editor_ui/01-slate-input-dispatch-core.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_editor/editor_ui/01
plan_link_mode: child_record_only
related_code:
  - zircon_editor/src/ui/retained_host/route_intent/map.rs
  - zircon_editor/src/ui/retained_host/activity_rail_pointer/host_activity_rail_pointer_route.rs
  - zircon_editor/src/ui/retained_host/document_tab_pointer/host_document_tab_pointer_route.rs
  - zircon_editor/src/ui/retained_host/drawer_header_pointer/host_drawer_header_pointer_route.rs
  - zircon_editor/src/ui/retained_host/menu_pointer/host_menu_pointer_route_intent.rs
  - zircon_editor/src/ui/retained_host/hierarchy_pointer/hierarchy_pointer_route.rs
  - zircon_editor/src/ui/retained_host/host_page_pointer/host_page_pointer_route.rs
  - zircon_editor/src/ui/retained_host/viewport_toolbar_pointer/viewport_toolbar_pointer_route.rs
  - zircon_editor/src/ui/retained_host/welcome_recent_pointer/welcome_recent_pointer_route_intent.rs
reference_sources:
  - dev/bevy/crates/bevy_ui/src/layout/ui_surface.rs
  - dev/bevy/crates/bevy_ui/src/layout/debug.rs
tests:
  - 1k move/click route payload clone-count regression
  - handled-by/target fallback and typed route parity matrix
---

# EditorUI01：retained route intent 在每次指针分发深克隆 owned payload

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：`zircon_editor/src/ui/retained_host/route_intent` 2/2 与 retained pointer consumers 静态审查
- 修复责任计划：`docs/plans/zircon_editor/editor_ui/01-slate-input-dispatch-core.md`
- 交接原因：稳定 pointer route identity 与 payload lifetime 属于 EditorUI01 共享 input dispatch owner。

## 失败现象与复现证据

性能计划逐文件审查 `route_intent` 2/2 后，已把无有序迭代语义的 node/route `BTreeMap` 直接改为 `HashMap`。最低层剩余问题是 typed accessors 为返回 owned route，逐事件 clone surface、slot、instance、action、node、page、project path 等 `String`，菜单/overflow 还 clone `Vec<usize>`；move-only 事件也会先物化完整 route。

## 最低共享层根因

route map 只以 node/route id定位 owned enum，公开 typed accessor却返回 clone 后的 payload；pointer dispatch没有可跨 callback借用的 generation handle，因此 move/hover也承担最终 click DTO 的 String/Vec ownership成本。

## 架构修复验收

EditorUI01 应让 pointer dispatch 返回稳定 route handle 或 generation-owned immutable payload；move/hover 只传轻量 node/route id，最终 click/drag/action consumer 需要 owned DTO 时再解引用或物化。不得让 bridge/app/painter各建一份 route owner，也不得依赖 hash iteration 改变确定顺序。

- 1k move 的 route payload String/Vec clone 与 heap allocation为 0；click/drag只在最终消费边界至多物化一次。
- `handled_by` 优先与 target fallback、missing route、document/drawer/menu/activity/hierarchy/detail/page/toolbar/welcome variants保持等价。
- generation 替换、surface rebuild、capture/release 后 stale handle 不得命中新 payload。

## 禁止临时方案

- 不得用泄漏或进程永久表把 route payload 强行变成 `'static`。
- 不得让每个 pointer bridge各自维护 route generation或通过省略 typed route破坏 click/drag语义。
- 不得依赖 `HashMap` iteration产生 route优先级或确定顺序。

## 修复结果与回传

Open state: `待 EditorUI01 建立 generation-owned stable route handle/immutable payload 并回传 clone-count 与完整 route parity 证据`。
