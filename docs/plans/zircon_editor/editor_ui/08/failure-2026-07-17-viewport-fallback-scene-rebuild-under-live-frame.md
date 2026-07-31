---
handoff_kind: failure
status: open
created_at: 2026-07-17
summary_slug: viewport-fallback-scene-rebuild-under-live-frame
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_editor/editor_ui/08
plan_link_mode: child_record_only
related_code:
  - zircon_editor/assets/ui/editor/components/workbench/shell/workbench_viewport_panel.zui
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_viewport_scene.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_viewport_scene/commands.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_viewport_scene/identity/classify/entry.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_viewport_scene_floor/grate/entry.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_viewport_scene_props/rails/rack.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/scene_layers/overlay/componentized.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_node_pipeline/draw.rs
---

# Viewport fallback scene rebuild under live frame

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：`template_viewport_scene*` 105/105 个 Rust 文件及89-control ZUI入口
- 修复责任计划：`docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md`
- 交接原因：live/fallback visibility、presentation generation和compiled segment属于Workbench shell；Render16/13只提交viewport texture和typed overlay资源。

## 失败现象与复现证据

Viewport ZUI声明89个WorkbenchViewport控件，多数为装饰fallback节点。Painter逐node执行多段substring classification、style/theme解析和command build；floor grate/rack/cargo按pixel尺寸循环扩quad。真实viewport image由另一条路径发布，当前painter边界没有live frame覆盖时跳过fallback subtree的契约。

## 最低共享层根因

Presentation没有typed live/fallback viewport mode，也没有layout/theme generation-owned fallback segment；leaf painter无法知道装饰场景是否被live frame完全遮挡。

## 已落地的局部止损

性能计划已在组件化布局帧缺失的全窗口fallback入口加入有效live image guard。该入口以现有`ViewportSceneKind`过滤装饰scene kinds，保留toolbar、selection、axis、gizmo与无关节点，并删除transform路径对owned node DTO的第二次clone。缺失/非法image仍绘制完整fallback。该修复只减少这一异常live路径的command/style工作，仍会访问和分类nodes，不能替代本计划的typed generation契约。

## 架构修复验收

- Live frame完整覆盖时fallback nodes visited/classify/style/theme/build/commands全部为0，仅viewport handle与必要overlay可见。
- No-frame fallback按layout/theme generation最多编译/栅格一次，stable frame build为0。
- 89-node product trace报告host/compiled/RHI commands、CPU p95、alloc、theme reads；grate/rack/cargo不按pixel展开高层commands。
- Startup/first frame/stale/error/resize/device loss恢复和toolbar/hit/gizmo/selection/order/clip/pixels等价。
- RenderDoc证明live frame不提交被完全遮挡的fallback draw workload。

## 禁止临时方案

- 不得只隐藏单个高命令节点而保留其余遮挡场景遍历。
- 不得在leaf painter建立无法感知viewport generation的第二份cache。
- 不得以永久删除fallback破坏无frame、启动或device-loss体验。

## 修复结果与回传

Open state: `局部missing-layout live guard已落地；仍待EditorUI08回传覆盖全部live路径的typed live/fallback generation、zero-visit/classify/build counters，并由Render16/13回传viewport handle与RenderDoc draw证据`。
