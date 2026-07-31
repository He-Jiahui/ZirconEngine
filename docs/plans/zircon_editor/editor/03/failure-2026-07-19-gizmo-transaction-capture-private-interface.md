---
handoff_kind: failure
status: open
created_at: 2026-07-19
summary_slug: gizmo-transaction-capture-private-interface
origin_plan: docs/plans/zircon_editor/editor_layout/15-component-standardization-from-primitives.md
fixing_plan: docs/plans/zircon_editor/editor/03-command-transaction-and-undo.md
origin_child_dir: docs/plans/zircon_editor/editor_layout/15
fixing_child_dir: docs/plans/zircon_editor/editor/03
plan_link_mode: child_record_only
related_code:
  - zircon_editor/src/ui/workbench/startup/editor_state_construction.rs
  - zircon_editor/src/ui/workbench/state/editor_state.rs
  - zircon_editor/src/ui/workbench/state/editor_state_viewport.rs
tests:
  - cargo +1.94.1 test -p zircon_editor --lib --locked --jobs 1 --no-run --message-format short --color never
---

# Editor03：Gizmo 事务捕获类型私有接口编译回归

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor_layout/15-component-standardization-from-primitives.md`
- 来源执行切片：Layout15 native-keyboard window contract 修复的受管向上编译门
- 修复责任计划：`docs/plans/zircon_editor/editor/03-command-transaction-and-undo.md`
- 交接原因：最低共享原因位于 Editor03 scene command transaction hardcut；`editor03-scene-transaction-hardcut-m2-20260718` 的写入范围包含全部 3 个相关 EditorState 文件，Layout15 不拥有该事务状态类型或可见性边界。

## 失败现象与复现证据

2026-07-19 的 Windows 受管 Cargo job `a3f58fbc034a47068b8d028eef4e7d97`、run `76e0c89627634bc192781b4a8f7d70a5` 执行：

```text
cargo +1.94.1 test -p zircon_editor --lib --locked --jobs 1 --no-run --message-format short --color never
```

结果为 exit `101`。构建已经越过 Layout15 native-keyboard 原始的 3 个 E0603、1 个 E0063 和 1 个 E0382；当前唯一编译错误为：

```text
zircon_editor\src\ui\workbench\startup\editor_state_construction.rs:159:32: error: type `GizmoTransactionCapture` is private: private type
error: could not compile `zircon_editor` (lib test) due to 1 previous error; 168 warnings emitted
```

受管 job 已按实际 exit `101` finish/release，进程树为空，target 以 `retained` 策略保留。

## 最低共享层根因

`EditorState::gizmo_transaction` 是 `pub(crate)` 字段，但其字段类型 `GizmoTransactionCapture` 在兄弟模块 `editor_state_viewport.rs` 中仅声明为 `pub(super)`。当前 Editor03 迁移让构造模块写入 `gizmo_transaction: None`，从而暴露了字段可见性高于类型可见性的私有接口不变量破坏。该问题不属于 native-keyboard、Layout15 组件样式或布局边界。

## 架构修复验收

- 由 Editor03 在事务状态 owner 内统一 `EditorState::gizmo_transaction` 与 `GizmoTransactionCapture` 的最小必要可见性，字段和类型不得形成 private-interface 失配。
- Editor03 scene transaction hardcut 的静态契约测试、格式检查和相关事务测试通过。
- 原始受管命令重新执行后 exit `0`，并生成当前源 `zircon_editor` lib-test 二进制。
- Layout15 随后运行 native-keyboard focused tests，确认分页、窗口导航和 command mapping 行为通过。

## 禁止临时方案

- 不得在 Layout15/native-keyboard 中加入别名、兼容 shim、静默 fallback、重复类型或调用点例外。
- 不得以 `#[allow(private_interfaces)]`、降低编译诊断、删除 `gizmo_transaction` 初始化或测试专用旁路掩盖可见性不变量。
- 不得削弱 Editor03 或 Layout15 的原始验收门。

## 修复结果与回传

Open state: `源码已修复、向上验证被 Coordinator01 非零空输出截断`; no pass is claimed. `EditorState::gizmo_transaction` 与 `GizmoTransactionCapture` 已共同收窄到 `crate::ui::workbench`；独立 exact46 终审继续要求关闭普通 action/preview 交错、release cleanup latch 与 project/play 两步切换，r7 已以 pre-capture cancel、私有 gizmo commit、finally cleanup 和 exclusive engine transition 整改，静态 TDD 13/13 GREEN。先前 exact31 immutable gate 的 inner Cargo exit 101，但 stdout/stderr 均空且副本已删除，无法证明是否进入 Editor03，因此仍不返回 fixed。

## 产出记录与时间

| 时间 | 状态 | 完成项目 | 验证与未完成项 |
|---|---|---|---|
| 2026-07-19 07:23 +08:00 | `TDD RED → source-static GREEN` | 新增字段/类型匹配的 workbench 可见性合同；先得到 6 项中 1 项预期失败，再把 `gizmo_transaction` 字段和 `GizmoTransactionCapture` 类型统一为 `pub(in crate::ui::workbench)`，避免 crate-wide 暴露。 | Python 合同 6/6、两源文件 `rustfmt --check`、scoped `git diff --check`、旧 `pub(crate)`/`pub(super)` 失配命中 0。 |
| 2026-07-19 08:04 +08:00 | `managed upward gate / external blocker` | source-bound reservation `aaaaf58fd9fb46fa86bc311a1595c5c5`（fingerprint `d659da750053c6a55b52d73d813857bff6e1e293ecfd9c6a7fc0b7032fd58eb4`）消费为 job `0873f135c7af481fb8d70080387ceab6` / run `bf674e8ca1f24d26a88e24e40bef572f`；按实际 exit 101 finish/release，进程树为空，target retained。 | 编译在 `zircon_runtime` 被 Plugins01 ArcSwap bridge 当前迁移截断：3×E0432（`arc_swap` 未链接）+ E0283 + E0282；尚未进入 Editor03/Layout15 测试二进制阶段。既有 owner `plugins01-bridge-stable-snapshot-r1-20260719` 正等待 Text01 释放 `zircon_runtime/Cargo.toml`，本会话不修改其 bridge/Cargo 文件。待其修复后重跑原命令、focused tests、独立复审与 failure return。 |
| 2026-07-19 10:33 +08:00 | `managed immutable gate / Coordinator01 evidence RED` | exact31 snapshot `608` 与 M2 manifest `4d56cc596a1545c8ade20e56775683c7` 已生成；reservation `df1a1260317643b587564bf78ac030e0` 提升到本 lifecycle 后绑定 job `5682435a212f4921b9959edd5609c7f6`，job released / live PIDs empty。 | inner run `82329a4e961e4ce3ad3894768f9be29c` exit 101、stdout/stderr 空、副本 removed；已交接 `validation-copy-nonzero-cargo-output-missing` 给 Coordinator01。等待其 fixed return 后重跑，不声明编译通过。 |
| 2026-07-19 11:09 +08:00 | `review finding fixed / static 9/9 GREEN` | `apply_viewport_command` 硬切为 `Result`；viewport 单独驱动 begin/record/finish，host 删除重复 gizmo intent；每次可能变换前预检 transaction engine，record/commit 失败恢复初始 transform、重置 drag 并传播错误。 | 强制 engine fault 回归锁定 transform rollback；Python 9/9、rustfmt 与 scoped diff guard GREEN。受管 Cargo 与独立复审仍待办，failure 保持 open。 |
| 2026-07-19 11:57 +08:00 | `lifecycle hardcut / static 12/12 GREEN` | 物理删除 `GizmoDragState`/三类 gizmo intent；capture 改 initial/latest transform；replace/clear/play entry 先 cancel preview，history/context cleanup 失败保持原 world；viewport failure event 带 RenderChanged；state tests 语义拆分。 | Python 12/12、旧符号 0、最大测试 leaf 615 行；faulted replace/clear、跨 play/world 与 render-effect 回归已落。受管 Cargo 与独立终审仍待办，failure 保持 open。 |
| 2026-07-19 12:35 +08:00 | `independent review 0/3/1 fixed / static 13/13 GREEN` | 普通 action 在 capture 前取消 preview，shared executor 拒绝 capture 泄漏；gizmo release 走私有提交，所有失败 finally 清 capture/重置 controller；project/clear/play enter+exit 持有 exclusive engine transition，history finalize 与 Context 清理同 lane。 | 新增 rename/delete-during-drag、missing-target release 与 engine interleaving 回归；Python 13/13、相关 Rust rustfmt GREEN。受管 Cargo 与独立复审仍待办，failure 保持 open。 |
| 2026-07-19 12:49 +08:00 | `independent incremental source review 0/0/0` | 上轮 0/3/1 全部关闭；reviewer 未发现预计的 Rust 类型/隐私/生命周期/借用/move 错误。 | source-only 结论基于 snapshot `623` / exact47 manifest `8bb61cd3...`；Coordinator01 failure 仍 open，未运行 Cargo，故本 failure 不返回 fixed。 |
