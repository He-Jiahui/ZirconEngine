---
handoff_kind: failure
status: open
created_at: 2026-07-19
summary_slug: dynamic-ui-extract-generation
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_runtime/runtime/09
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/dynamic_api/session/extract.rs
  - zircon_runtime/src/dynamic_api/session/ui_extract_cache.rs
  - zircon_runtime/src/dynamic_api/session/state.rs
  - zircon_runtime/src/dynamic_api/session/menu.rs
  - zircon_runtime/src/dynamic_api/session/hud.rs
  - zircon_runtime/src/scene/world/generation.rs
  - zircon_runtime/src/core/framework/render/framework.rs
  - zircon_runtime/src/graphics/types/viewport_render_frame.rs
  - zircon_editor/src/scene/viewport/controller/scene_viewport_controller_build_runtime_overlay_ui.rs
  - zircon_editor/src/ui/retained_host/viewport/world_space_ui.rs
tests:
  - stable UI extract build/visit counters
  - shared Arc identity across Runtime and Editor submission
  - stable project/HUD/world-space aggregate identity
  - menu/HUD F2 pixel parity
---

# Runtime09：dynamic UI extract generation

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行切片：PERF-MVP-433 dynamic UI extract generation
- 修复责任计划：`docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md`
- 交接原因：menu/HUD component generation 与 viewport-owned UI extract 的最低共享 owner 是 Runtime09。

## 失败现象与复现证据

旧复现中的全World `node_records()`扫描已由dynamic-component sparse index删除，HUD分类的临时token Vec也已删除；World当前还发布`dynamic_component_generation(component_id)`，无关transform/scene变化不会推进menu/HUD组件代次。剩余current-source成本是每次capture/present仍重新读取目标component sparse rows，并重新构建owned command Vec、文本与style color Strings。即使session局部缓存extract，`current_ui_extract -> RenderFramework -> ViewportRenderFrame`仍以owned `Option<UiRenderExtract>`传递，cache hit返回时会深克隆同一Vec/String图，因此不能满足稳定帧零build/零allocation。

## 最低共享层根因

最低现成失效权威已经是component generation，缺口位于共享产品所有权：menu/HUD没有按`(目标component generations, viewport size)`发布唯一`Arc<UiRenderExtract>`，Runtime session、RenderFramework queue与`ViewportRenderFrame`仍以owned extract作为跨阶段合同。根因不是缺少另一张String cache，也不是World缺少revision，而是render product边界没有共享不可变generation handle。

## 架构修复验收

- fallback UI cache按menu generation优先、两类HUD component generation及viewport size组成typed key，发布唯一`Arc<UiRenderExtract>`；whole-world replacement继续复用World现有generation carry-forward语义。无关scene/transform变化不得失效。
- `current_ui_extract`、RenderFramework submit/queue和`ViewportRenderFrame`统一传共享handle；stable capture只Arc clone，component-row visits/build/String/Vec alloc均0。不得在返回owned DTO时深克隆缓存。changed key build≤1。
- menu click hit-test继续直接读取sparse component value与算术layout，不为输入构建或克隆presentation extract。
- 1/1k/100k nodes与1/60/240Hz验证menu click、HUD抑制、resize、layout和像素等价；回传PERF-MVP-433并联动Render14/EditorUI08。

## 禁止临时方案

不得恢复每帧World全扫，不得引入可漂移的String cache，不得只在session缓存后返回owned clone，也不得在稳定generation上继续全量重建command Vec。

## 修复结果与回传

Open state: `Runtime fallback/project aggregate、RenderFramework/ViewportRenderFrame与Editor HUD/world-space submission的generation-owned Arc候选已静态实现；focused source contract 7/7及scoped rustfmt/diff通过。Cargo按用户要求暂缓，1/1k/100k规模、1/60/240Hz、F2像素等价和真实allocation/CPU证据未完成，因此failure保持open。`
