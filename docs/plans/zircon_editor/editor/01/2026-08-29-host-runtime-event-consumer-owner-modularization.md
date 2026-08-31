---
status: structural-owner-hardcut-complete
created_at: 2026-08-29
implementation_status: runtime-event-consumer-orchestration-extracted
managed_validation_status: blocked-before-editor-by-runtime-host
source_baseline_sha256: C5F16868C79043B9D52862A81AB6949E0918944F4813D2F59B5C485FB6741E49
related_code:
  - zircon_editor/src/ui/host/editor_host_event_controller.rs
  - zircon_editor/src/ui/host/editor_host_event_controller/runtime_event_consumers.rs
  - zircon_runtime_host/src/foreign_output/item_count.rs
---

# Editor01 host runtime-event consumer owner modularization

## 结论

current `editor_host_event_controller.rs` 在实施前 SHA-256 为 `C5F16868C79043B9D52862A81AB6949E0918944F4813D2F59B5C485FB6741E49`，共 1130 行。production root 同时承担 host composition、Play gateway identity、runtime-event consumer admission/pump/terminal cleanup、command access 与 Play 日志投影，超过工程结构约定的 1000 行硬审查线。

本轮将 runtime-event consumer 子域硬切至标准子 owner `editor_host_event_controller/runtime_event_consumers.rs`：注册、session begin、frame pump、session end 与 active inspection 五个方法整体迁移，根 owner 只声明 `mod runtime_event_consumers;`。没有保留 forwarding wrapper、alias、第二套 session generation 或旧方法实现。

迁移保持以下顺序不变：pending decision receipt -> backend poll -> consumer retirement -> Play world-sync shutdown -> terminal gateway detach -> backend retirement -> pending decision reconcile -> retained EditorState restore -> reflection refresh；active session 路径仍在 capability reconcile 后 tick play gateway，再 pump consumers。字段、方法可见性、错误类型与调用点均未改变。

## 结构与验证证据

- root：1130 行降至 975 行；其中 `#[cfg(test)]` 前 production 前缀约 236 行。
- new owner：186 行。
- result SHA-256：root `DB01115BF61A439D91BBC1F8DEA8A84FC1C4DFCA552DA4009EDACBB35DECDE93`；child `337367FD6257BCAE1E094D74A983AA2B7A152B53BF9001BDB4923A730AE1D707`。
- 5 个 runtime-event consumer 方法在 production root 中计数均为 0，在 named child 中计数均为 1。
- terminal/pump 关键调用 `poll_backend`、`shutdown_play_world_sync`、`detach_terminal_play_gateway`、`retire_terminal_backend`、`reconcile_pending_play_decision_from_controller`、`exit_play_mode`、`refresh_reflection`、`tick_frame`、consumer `pump` 在 child 中各保留一次。
- 新增结构回归，要求 root 不超过 1000 行且五个方法只能位于 named owner。
- scoped rustfmt 与 scoped `git diff --check`：exit 0。
- D 盘 `cargo check -p zircon_editor --lib --offline --target-dir D:\zt\plan11-journal-product-target -j 4 --message-format short`：exit 101；在编译 `zircon_editor` 前被 `zircon_runtime_host/src/foreign_output/item_count.rs:80` 的 `E0004` 截断，缺失 `WorldQueryResult::TransformSnapshot` 分支。该问题已有 RuntimeHost owner failure，本切片未修改外部文件，也未宣称 Cargo GREEN。

## 产出记录与时间

| 时间 | 状态 | 完成项目与当前门禁 |
|---|---|---|
| 2026-08-29 | `owner-hardcut-complete / structure-guarded / static-green / cargo-blocked-before-editor` | 基于精确 current-source SHA，将 host controller 的 runtime-event consumer admission/pump/terminal cleanup 五方法迁入 186 行 named owner；根文件从 1130 行降至 975 行，不保留 wrapper/alias/双 owner。结构守卫、rustfmt、scoped diff-check 均通过。D 盘 Cargo 在进入 editor 前由 RuntimeHost `TransformSnapshot` 非穷尽匹配 E0004 截断，故本项不作为 accepted milestone，不提交、不推送协调器、不发送企微。 |
