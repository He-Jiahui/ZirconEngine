---
handoff_kind: failure
status: open
created_at: 2026-07-23
summary_slug: ui-text-edit-state-dead-code-plan-anchor-drift
origin_plan: docs/plans/zircon_runtime/text/01-font-resource-faces-and-database.md
fixing_plan: docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md
origin_child_dir: docs/plans/zircon_runtime/text/01
fixing_child_dir: docs/plans/zircon_runtime/runtime/15
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/runtime_dead_code/ui_text.rs
  - docs/plans/zircon_runtime/runtime/15/2026-07-10-priority-plan-doc-current-owner-inventory.md
  - docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-code-structure-and-module-conventions-output-records.md
  - docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-runtime-index-output-records.md
  - docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-review-findings-output-records.md
  - docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-structure-output-records.md
tests:
  - cargo +1.94.1 test -p zircon_runtime --lib runtime_15_ui_text_edit_state_dead_code_suppression_cleanup --locked --jobs 1 -- --nocapture --test-threads=1
---

# Runtime15：UI text edit-state 守卫仍读取已迁移根文档

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/text/01-font-resource-faces-and-database.md`
- 来源执行切片：Text01 current-source default/UI lib-test gate
- 修复责任计划：`docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md`
- 交接原因：Runtime15 的结构守卫仍把四个已完成 hard-cut 的根文档当作历史状态镜像；Text01 只在上行 lib-test gate 中发现该漂移，不应恢复第二份计划事实源。

## 失败现象与复现证据

Text01 managed job `f9f5581fb83b40c2a3cc81aa15f5bcaa` / run `b98dc769094b4bd9b96fc445fd8a1332` 执行 `runtime_15_ui_text_edit_state_dead_code_suppression_cleanup` 时以 `exit_code=101` 失败。生产 UI text edit-state 消费链已通过，失败发生在文档断言：守卫从 Runtime15 主计划、runtime index、review findings 和 structure convention 根文档读取以下历史切片锚点：

- `Runtime 15 F12 UI text edit-state dead-code suppression cleanup`
- `runtime_15_ui_text_edit_state_dead_code_suppression_cleanup_static_passed_cargo_deferred`
- `runtime_15_ui_text_edit_state_dead_code_suppression_cleanup`

job 于 `2026-07-22T19:24:42.482382+00:00` 自然结束并 release，exit `101`、live PIDs 为空；
原始日志位于 `.codex/state/session-coordinator/cargo-runs/f9f5581fb83b40c2a3cc81aa15f5bcaa/b98dc769094b4bd9b96fc445fd8a1332/`。

## 最低共享层根因

提交 `f7a320904d681fb30dede6d5b222fc943cdeb3a7` 已按 Runtime15 的 plan-output archive hard-cut 从四个根文档删除具体历史状态。后续 2026-08-02 receipt-tree hard cut 又把计划 lifecycle/status mirrors 移出 Runtime Rust lib-test。`ui_text.rs` 的生产断言仍然有效，但 current source 仍读取四个 canonical archive、module doc 与 UI doc，却完全没有消费这些变量；这些读取既不提供验证，也继续扩大 lib-test 的文档编译契约。

## 架构修复验收

- 守卫保留 `ui/text/mod.rs` 无 dead-code suppression、`edit_state.rs` 状态机和四条生产消费链断言。
- 删除不参与断言的四个 archive 与 module/UI 文档读取；计划 lifecycle/schema 由 Coordinator/Python tooling 持有，不向 live 根计划、总索引或全局 review/structure 文档回填历史镜像。
- 精确 current-source command 实际执行目标 test 恰好 1 个并通过。

## 禁止临时方案

- 不删除整个生产结构守卫，也不把 live `edit_state` owner 降为 test-only。
- 不向四个 live 根文档恢复已迁移的历史切片锚点或 duplicated truth，也不保留只读取但不验证的 archive dependency。
- 不弱化生产 `allow(dead_code)` 扫描，不用 skip/ignore/cfg 绕过该测试。

## 修复结果与回传

Open state: `resolving_failure`。2026-08-14 current-source 前向修复已删除 `ui_text.rs` 中四个 archive、module doc 与 UI doc 的六个未使用读取及多余 exact helper import；`ui/text/mod.rs` dead-code suppression、`edit_state.rs` 状态机和四条生产消费链断言全部保留。2026-08-28 Windows managed validator 已越过 workspace closure 并开始编译 `zircon_runtime`，但目标 test 尚未执行即被既有 Render17 下层 handoff `docs/plans/zircon_runtime/render/17/failure-2026-08-28-rhi-diagnostic-readback-layout-width-drift.md` 的三处 `u32`/`u64` E0308 挡住；无 Cargo/rustc 残留。精确 managed current-source 1/1 尚未完成，因此本记录继续保持 `open`，不声明 fixed/accepted。
