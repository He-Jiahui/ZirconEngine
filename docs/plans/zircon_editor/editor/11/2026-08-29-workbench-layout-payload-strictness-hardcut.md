---
related_code:
  - zircon_editor/src/ui/workbench/layout/activity_window_layout.rs
  - zircon_editor/src/ui/workbench/layout/activity_drawer_layout.rs
  - zircon_editor/src/ui/workbench/layout/document_node.rs
  - zircon_editor/src/ui/workbench/layout/tab_stack_layout.rs
  - zircon_editor/src/ui/workbench/layout/floating_window_layout.rs
  - zircon_editor/src/ui/workbench/layout/main_host_page_layout.rs
  - zircon_editor/src/ui/workbench/autolayout/axis_constraint_override.rs
  - zircon_editor/src/ui/workbench/autolayout/pane_constraint_override.rs
  - zircon_editor/fixtures/workbench/default-layout.json
related_tests:
  - tools/tests/test_editor11_workbench_layout_payload_strictness.py
  - zircon_editor/src/tests/workbench/layout/window_drawer_ownership.rs
  - zircon_editor/src/tests/workbench/layout/payload_strictness.rs
plan_sources:
  - docs/plans/zircon_editor/editor/11-serialization-and-versioning.md
  - docs/plans/zircon_editor/editor/06-ui-extension-framework.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
status: in_progress
---

# Editor11 Workbench layout nested payload strictness hard cut

## 架构裁决

Workbench layout 的版本壳负责识别 schema/version 并在进入当前 typed payload 前执行显式迁移；
`ActivityWindowLayout`、`FloatingWindowLayout` 与 `MainHostPageLayout` 只描述当前格式。因此当前格式的
未知字段、缺失字段必须失败，禁止通过 `serde(default)` 把不完整或历史 payload 静默解释为新版状态。
默认 fixture 同步携带完整字段，不保留旧 reader、字段 alias、双写或兼容 fallback。

本切片只收敛持久化正确性边界，不属于性能优化；没有 profile 数据时不声明耗时、功耗或算法收益。

## 产出记录与时间

| 时间 | 状态 | 完成项目 | 证据与后续 |
|---|---|---|---|
| 2026-08-29 14:57 +08:00 | `implementation-complete / static-contract-green / managed-rust-pending` | 三个嵌套 layout payload 增加 `serde(deny_unknown_fields)`；删除 ActivityWindow 的 `menu_overflow_mode/region_overrides/view_overrides` 和 FloatingWindow `frame` 的默认解码；默认 Workbench fixture 显式补齐当前字段；新增缺失 ActivityWindow 字段，以及 ActivityWindow/MainHostPage/FloatingWindow 未知字段 Rust 负例。 | TDD RED 为 0/4（3 fail、1 error，0.079s），GREEN 为 4/4（0.033s）；与 Editor06 合并源码契约 13/13（15.453s）；9 个相关 Rust 文件定向 `rustfmt --check` 通过；三个类型默认回退计数均为 0；全仓仅 1 个 `activity_windows` JSON，缺失必需字段数 0。Windows 受管 `zircon_editor -SkipTest` 未进入 Cargo：`cargo.acquire` 请求 `9907f0a493b14045a3a5cfa1279932e2` 被接受但协调器在 15s 内无可确认终态；不轮询、不旁路。待受管 build 与 focused Rust tests 通过前保持 `in_progress`，不提交、不发送企微。 |
| 2026-08-29 15:09 +08:00 | `implementation-complete / static-contract-green / managed-rust-pending` | 完成持久化图深层收口：`ActivityDrawerLayout`、`DocumentNode`、`TabStackLayout`、`PaneConstraintOverride`、`AxisConstraintOverride` 拒绝未知字段；`FloatingWindowLayout.frame` 通过 editor-owned strict wire adapter 保持 `{x,y,width,height}` 形状并拒绝未知 frame 键；新增独立 `payload_strictness.rs` 深层负例，保留稀疏 override 字段的合法默认语义。 | 合并 Editor06/11 源码合同 14/14（25.142s）；相关 8 个新增/修改 Rust owner 定向 `rustfmt --check` 通过；根级旧字段访问、旧槽位、旧同步符号 0 命中；当前唯一 Workbench JSON fixture 的必需 ActivityWindow 字段缺失数 0。`floating_window_layout.rs` 曾因 Windows mapped section OS 1224 暂时无法格式化，锁释放后已完成定向 `rustfmt`。受管 Rust build/test 仍未进入 Cargo，保持 `in_progress`，不提交、不发送企微。 |
