---
handoff_kind: failure
status: open
created_at: 2026-07-17
summary_slug: notification-center-double-scan-unbounded-paint
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_editor/editor_layout/09-incremental-message-bus-and-refresh.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_editor/editor_layout/09
plan_link_mode: child_record_only
related_code:
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/notifications.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/notifications/history.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/template_nodes/node.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_notification_center/commands.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_notification_center/identity.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_notification_center/instrumentation.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_notification_center/panel.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_notification_center/row.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/notification_center/metadata.rs
  - zircon_editor/src/ui/retained_host/ui/workbench_window_projection.rs
  - zircon_editor/src/ui/retained_host/ui/workbench_window_projection/host_value_toml.rs
  - zircon_editor/src/ui/retained_host/ui/workbench_window_projection/notification_cache.rs
  - zircon_editor/src/ui/retained_host/ui/workbench_window_projection/previous_node_index.rs
---

# Notification center double scan and unbounded paint

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：`template_notification_center*` 11/11 个 Rust 文件逐文件静态审查
- 修复责任计划：`docs/plans/zircon_editor/editor_layout/09-incremental-message-bus-and-refresh.md`
- 交接原因：notification generation、unread metadata、retention/backpressure 与 overflow semantics 属于 EditorLayout09 的消息刷新协议责任；EditorUI08 只负责可见行消费与 compiled paint。

## 失败现象与复现证据

Notification header 每次 paint 都 clone 全部 option 来重新计算 unread count。Row loop 随后再次 clone 全部 option，包括最终被 clip 拒绝的行；可见行还会再次复制 title 和 description。消息风暴下 CPU 与保留内存会随完整 notification history 增长。

## 最低共享层根因

消息/通知 owner 没有发布包含 unread count、retention/overflow 状态的 immutable generation。Painter 因而从完整 options 历史反推 header，并且没有可消费的 bounded visible-row authority。

## 架构修复验收

- 1,000-message burst 有 bounded retention 和显式 overflow semantics；每 message 更新为 amortized O(1) 或 bounded batch。
- Same-generation unread scans/rebuilds 为零；closed notification center paint work 为零。
- Open center 只 visit/clone visible+overscan rows；offscreen row clone/text copy 为零。
- Header/unread、severity/state、ordering、scroll/focus、clip 和 pixel behavior 等价。

## 禁止临时方案

- 不得在 painter 再建立一个不受消息 generation 失效约束的 notification cache。
- 不得仅增加 clip leaf early-return，却继续对全部历史执行 `row_data` 或 unread 全量扫描。
- 不得通过静默丢弃通知规避 backpressure；overflow 必须显式、可计数且保序规则清楚。

## 修复结果与回传

Open state: `实现已完成，运行验证与 failure return 待 coordinator terminal evidence`。

### 2026-08-01 前向修复状态

- `implemented_static / validation_pending`：通知 owner 以 64 条硬上限按“新批次优先、旧项淘汰”保序合并；1,000 条 burst 仅保留 64 条，累计 `overflow_count=936`，toast queue 同样受 64 条上限约束，不再静默形成第二个无界队列。
- 每次真实历史变化推进 `notification_generation`；同一 pending-decision generation 重复同步返回 no-op。`unread_count` 在有界合并时一次计算，generation/unread/overflow 作为 immutable scalar snapshot 投影到 `TemplatePaneNodeData`，painter 不再扫描 options 反推 header。
- notification options/structured options 在 workbench projection 中单次成对解析，`visible_limit` 已前推到 parser，达到上限后不再访问或复制剩余通知；同 workbench document 且 generation/unread/overflow/selection/focus/limit 完整缓存键一致时，才从上一帧 control-id 索引复用同一个 `Rc<Vec<_>>` 行模型和 `Rc<String>` options 摘要，并在 TOML 转换前排除 notification payload，因此同代行解析、模型重建与离屏文本复制均为零；document 切换强制失效。
- open painter 由 panel/clip/fixed row stride 计算 visible + 1-row overscan，以 `ModelRc::get` 借用访问，离屏行不 clone option 或复制文本；closed center 在 metrics/palette/data access 前返回。测试专用计数器实测 closed 的 metrics/palette/row-count/visit/text-copy 全为 0；窄 clip 下 64 行只访问 1 visible + 2 overscan，且只为真正相交的 1 行复制 title/message。
- overflow 标题保持原有 `Notifications (N)` 形式，仅在淘汰发生时追加紧凑的 `+M omitted`；severity、row ordering、focused/selected state、有限 Runtime Text slots 与既有 panel/row geometry 保持原合同。
- 静态证据：目标 Rust 文件 `rustfmt --check` 通过，`git diff --check` 通过，`S09_NOTIFICATION_SOURCE_GUARDS_OK`；新增 burst/累计 overflow、same-generation no-op、metadata projection、parser early-stop、同文档同代 `Rc<Vec<_>>` 复用/notification payload 零复制、完整缓存键与跨文档失效、closed/open paint 工作计数、visible-range/non-finite clip 与无全表循环/clone access regressions。
- 独立二次审查及最后 `Rc<String>` delta 追加审查均为 production finding 0；确认 previous-presentation 生命周期、跨文档失效、完整缓存键、同代 payload/options 摘要零复制、parser early-stop、closed/open paint 计数和模块 re-export 边界成立。残余验证风险仅为 cold/new-generation cache miss 会先对最多 64 条 bounded payload 做一次 TOML 转换，再由 parser 按 `visible_limit` 停止；不把这一点误报为已消除。
- 未声明通过：本 Session 未直接执行 Cargo、未重投 coordinator validation、未生成或更新截图。failure 继续保持 `status: open`，待 terminal current-source evidence 后执行原复现、向上验收及 `failure return`。
