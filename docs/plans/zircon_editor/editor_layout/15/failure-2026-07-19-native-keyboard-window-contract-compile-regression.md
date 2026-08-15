---
handoff_kind: failure
status: open
created_at: 2026-07-19
summary_slug: native-keyboard-window-contract-compile-regression
origin_plan: docs/plans/zircon_editor/editor/15-build-export-and-publishing.md
fixing_plan: docs/plans/zircon_editor/editor_layout/15-component-standardization-from-primitives.md
origin_child_dir: docs/plans/zircon_editor/editor/15
fixing_child_dir: docs/plans/zircon_editor/editor_layout/15
plan_link_mode: child_record_only
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/native_keyboard/target.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_keyboard/target/model.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_keyboard/target/page_overflow.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_keyboard/target/selection.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_keyboard/dispatch.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_keyboard/dispatch/actions.rs
tests:
  - cargo test -p zircon_editor --lib host_page_overflow_keyboard --locked --jobs 1 --color never -- --test-threads=1
  - cargo test -p zircon_editor --lib native_keyboard --locked --jobs 1 --color never -- --test-threads=1
---

# Layout15：native keyboard window contract 编译回归

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/15-build-export-and-publishing.md`
- 来源执行者：`editor15-export-report-parse-once-r7-20260718`
- 来源执行切片：受管 focused gate `f5cd31cd719042ce88cb133cde113cef` / run `b510c09846f94d5aaf63e43985a31d9a`
- 修复责任计划：`docs/plans/zircon_editor/editor_layout/15-component-standardization-from-primitives.md`
- 交接原因：popup keyboard target、分页窗口与 host-page overflow 均由 Layout15 组件标准化计划持有，Editor15 不应修改该 UI 合同。

## 失败现象与复现证据

Editor15 的受管 focused gate `f5cd31cd719042ce88cb133cde113cef` / run `b510c09846f94d5aaf63e43985a31d9a` 在当前 `zircon_editor --lib` 编译中暴露 5 条 Layout15 根因：

- `dispatch/actions.rs` 与 `dispatch.rs` 直接导入私有 `target::model`，产生 3 条 E0603；`PopupKeyboardMove`、`PopupKeyboardWindowFocus`、`PopupKeyboardWindowRequest` 尚未从 `target` 窄边界回导出。
- `target/page_overflow.rs:54` 构造 `PopupKeyboardTarget` 时漏填 `window_offset`、`window_count`、`total_count`、`window_navigation_enabled`、`window_query`，产生 E0063。
- `target/selection.rs:38` 先把 `rows` 移入目标，再在 `unwrap_or(rows.len())` 借用，产生 E0382。

原始 stderr：`.codex/state/session-coordinator/cargo-runs/f5cd31cd719042ce88cb133cde113cef/b510c09846f94d5aaf63e43985a31d9a/stderr.log`。该作业 tests 0 且有外部 Runtime 源竞态，只作为编译诊断，不作为任何功能验收。

## 最低共享层根因

`PopupKeyboardTarget` 新增分页窗口合同后，模块边界、host-page overflow producer 与通用 selection producer 未原子迁移：调用方穿透私有 `model`，旧 producer 漏填新字段，另一个 producer 在 move 后才计算 fallback。

## 架构修复验收

- 由 Layout15 在 `native_keyboard::target` 内建立窄可见性边界，调用方不得穿透私有 `model` 模块。
- host-page overflow 明确声明为非窗口分页目标或按现有窗口合同填写全部字段；不得用无语义默认值掩盖分页行为。
- 在移动 `rows` 前计算 `total_count` fallback，不得为解决 E0382 clone 整个行列表。
- 运行 host-page overflow、native-keyboard focused tests与新鲜 `zircon_editor --lib` 门禁，并完成独立复审。

## 禁止临时方案

- 不得将 `model` 整模块改为 crate-wide public；只从 `target` 窄边界回导出所需类型。
- 不得 clone `rows` 规避 move error，也不得以无语义零值掩盖 overflow 的分页合同。
- 不得把 source-raced Editor15 作业登记为 Layout15 验收 GREEN。

## 修复结果与回传

Resolving state: `Layout15 已完成原子合同迁移与 source/static 复核；三轮受管向上编译均证明原始 5 条诊断消失。Plugins01 E0631/E0308 已修复并在 warm gate 中消失；当前唯一阻断为 Text01 UI font asset cache 的 E0502，focused binary tests、独立复审、fixed return 与 owner milestone commit 仍待完成。`

## 产出记录与时间

| 时间 | 状态 | 完成项目 | 验证与未完成项 |
|---|---|---|---|
| 2026-07-19 01:19 +08:00 | `open / Layout15 待修复` | 已把 private-model 可见性、overflow 目标字段缺失和 `rows` 移动后借用共 5 条编译错误回传到 Layout15；未修改 native-keyboard 业务代码。 | 受管作业 exit 101、tests 0；待 Layout15 完成 TDD 修复、focused/broad current-source 验证和独立复审。 |
| 2026-07-19 04:43 +08:00 | `resolving_failure / source-static GREEN` | `target` 仅向 `host_contract` 窄回导出 window move/request/focus；host-page overflow 明确 `offset=0`、`count=total_count`、禁用 window navigation 且保留空 query；通用 selection 在移动 `rows` 前计算 `window_count`/`total_count` fallback，没有 clone 行列表。调用方不再穿透私有 `target::model`。 | Python paged-keyboard 合同 3/3、六文件 `rustfmt --check`、scoped `git diff --check`、private-import/producer source guard 全部通过。受管 default-feature no-run reservation `64cf5ca438be4cf4baeecafc9aefe64a` 已按 FIFO 创建；focused host-page/native-keyboard、broad current-source、独立复审、fixed return 与 owner commit 仍待完成。 |
| 2026-07-19 05:22 +08:00 | `resolving_failure / managed validation queued` | 前序 Frameworks06 作业 `683eb23631aa4364a6cdbc82de80dddd` 已自然 GREEN/release（exit 0、进程树空），未被 Layout15 干预；Layout15 的 main warm `9b17452355354a8a99d55089c50e4691` 与 repair `64cf5ca438be4cf4baeecafc9aefe64a` 均保持 pending。 | 当前合法 FIFO 头为 Render18 `95926ba08e5f4ed68572cbd44b113084`；其 Session 自 2026-07-18 21:41 +08:00 无 heartbeat，但 recovery 仍把 reservation 续到约 06:20。WOC 已恢复 heartbeat 但位于其后。按已批准 FIFO 方案不释放、替代消费或绕过外部 reservation；Cargo/focused/broad/独立复审仍 pending。 |
| 2026-07-19 05:33 +08:00 | `resolving_failure / priority validation next` | 已释放空 source-manifest 的 repair reservation `64cf5ca438be4cf4baeecafc9aefe64a`，以 6/6 `related_code` 当前 SHA-256 重建 source-bound reservation `4e673f0497ae4ed4a71da8f521256ef9`；manifest fingerprint `00145188740dd839f47184ea6c5d4df49fed0862f030aba12e356dc3f60a62f3`，coordinator 正式提升为 priority 0。main warm `9b17452355354a8a99d55089c50e4691` 保持不变。 | 更早的 Runtime12 priority-0 作业 `c5d6303ce4334f3995b2b5073af7569b` 正在运行；Layout15 repair 已合法排在它之后、Render18/WOC 普通 FIFO 之前。尚未获得 Layout15 job/exit，focused/broad、独立复审、fixed return 与 owner commit 仍 pending。 |
| 2026-07-19 07:01 +08:00 | `resolving_failure / original compile symptoms removed` | source-bound reservation 已消费为受管 Windows job `a3f58fbc034a47068b8d028eef4e7d97`、run `76e0c89627634bc192781b4a8f7d70a5`；命令为 `cargo +1.94.1 test -p zircon_editor --lib --locked --jobs 1 --no-run --message-format short --color never`，`CARGO_INCREMENTAL=0`。编译完整越过 Runtime 与 native-keyboard，原始 3×E0603、E0063、E0382 均未再出现。job 已按实际 exit 101 finish/release，进程树为空，target retained。 | 当前唯一错误为外部 `editor_state_construction.rs:159:32`：`GizmoTransactionCapture` private type；tests binary 未生成，因此不能伪报 focused tests 通过。已建立 Editor03 child-record-only 交接：[gizmo transaction capture private interface](../../editor/03/failure-2026-07-19-gizmo-transaction-capture-private-interface.md)。待 Editor03 return 后复用保留 target 重跑向上编译与 focused tests；独立复审、fixed return、owner commit 仍 pending。 |
| 2026-07-22 05:12 +08:00 | `resolving_failure / fresh locked upward gate reached workspace compile` | 根 lockfile ArcSwap consumer edge 已由 Plugins01 child return 修复；snapshot `680` 的受管 Windows reservation `997d474015c3403eb27a5b56fbf99833` 消费为 job `8e229f6cd2c749f495b0f701e0c07bc0` / run `b410b0de35d14f2d9980be50241c640e`，重跑同一 `zircon_editor --lib --locked --no-run` 命令。日志明确编译 `arc-swap v1.9.2`、`zircon_runtime` 与 `zircon_editor`，原始 3×E0603、E0063、E0382 及旧 Editor03 `GizmoTransactionCapture` 诊断均未出现。job 按实际 exit `101` 由 coordinator 自动 finish/release，进程树为空，target retained。 | 最新仅有既存 Plugins01 owner 的 `plugin/bridge/import.rs:66` E0631 与 `native_plugin_live_host/registration_replay.rs:392` E0308；测试二进制未生成，故 focused filters 仍未计通过。lockfile 子失败已 fixed return 至本目录；待 Plugins01 既有 bridge-import / registration-replay failures return 后复用 retained target 重跑，不在 Layout15 混修。 |
| 2026-07-22 05:51 +08:00 | `resolving_failure / Plugins blockers removed, Text01 blocker isolated` | Plugins01 已分别以 `ArcSwapOption::load().as_deref()` 与 typed error `to_string()` adapter 修复 E0631/E0308；同一 snapshot `680` warm reservation `0905384dda9c4f57836d0cf4329ee2c0` 复用 retained target 为 job `4576d0ee13194594a5dfe684bec27c13` / run `3d7907529ce14245a00399c50e4eff57`。编译越过两处 Plugins 源码和全部 native-keyboard，未出现原始 5 错，job 按实际 exit `101` 自动 finish/release。 | 当前唯一错误为 Text01 `scene_renderer/ui/text/font_assets.rs:151` E0502；测试二进制仍未生成。已建立 child-only 交接：[UI font asset cache borrow regression](../../../zircon_runtime/text/01/failure-2026-07-22-ui-font-asset-cache-borrow-regression.md)，并由 active Text01 owner 续租源文件。待其 fixed return 后再次复用 warm target；focused tests、独立复审与 native failure return 仍 pending。 |
| 2026-08-13 | `open / implemented_static / validation_pending` | 当前 `native_keyboard::target` 仅向 `host_contract` 窄回导出 move/request/focus DTO，dispatch 不穿透私有 `target::model`；host-page overflow producer 明确本地窗口 `offset=0/count=total`、禁用窗口导航，通用 selection producer 在移动 `rows` 前计算 `window_count/total_count`，没有为规避 E0382 克隆行表。 | 本轮只做 current-source 静态审计，原始 E0603/E0063/E0382 的源码根因仍保持消除；未运行受管 host-page/native-keyboard focused Cargo，不声称 fixed return 或 accepted closeout。 |
