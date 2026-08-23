---
handoff_kind: failure
status: open
created_at: 2026-07-11
summary_slug: retained-window-hard-cutover-expectations
origin_plan: docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md
fixing_plan: docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
origin_child_dir: docs/plans/zircon_editor/editor/01
fixing_child_dir: docs/plans/zircon_editor/editor_ui/08
plan_link_mode: child_record_only
related_code:
  - zircon_editor/src/tests/host/retained_window
  - zircon_editor/src/ui/retained_host
  - zircon_editor/src/ui/layouts/windows/workbench_host_window
plan_sources:
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
tests:
  - cargo test -p zircon_editor --lib --locked tests::host::retained_window -- --test-threads=1
---

# Editor UI 08：Retained-window 硬切后旧合同期望失败交接

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md`
- 来源执行切片：Editor M1 当前源码完整单线程门禁
- 修复责任计划：`docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md`
- 交接原因：失败集中在 Workbench shell/retained host/native painter 的旧结构与视觉合同测试，Plan 01 内核不拥有窗口投影、模板绘制或 retained-window 测试真相。

## 失败现象与复现证据

08:31 当前源码 Editor binary 的完整单线程门禁最终为 2763 passed / 133 failed / 34 ignored（2258.13s）。按功能重分后，Editor UI 08 接管 43 项 Workbench shell/retained-host 投影、pointer、window 与 template-bridge 失败；其中 49 项最初集中出现于 `tests::host::retained_window::*`，后续把明显属于 Editor UI 05/06 的 UI Asset 与 MUI painter 项分别移交对应计划。

| 组 | 当前失败数 |
|---|---:|
| `native_material_painter_mui_primitives` | 22 |
| `native_material_painter` | 8 |
| `generic_host_boundary` | 3 |
| `shell_window` | 3 |
| `ui_asset_editor` | 3 |
| `native_material_painter_alert` | 2 |
| `native_material_painter_paper` | 2 |
| 其余 retained-window 组 | 6 |

两个独立 fully-qualified exact 均为 0/1，且证明不是同一像素断言的重复噪声：

- `activity_rail_template_boundary::host_side_activity_rails_use_projected_toml_template_nodes`：`side dock DTO missing rail_nodes`。
- `generic_host_boundary::rust_owned_host_contract_declares_window_globals_and_projection_data`：仍要求已经不存在的 `UiHostWindow::clone_strong(&self) -> Self` 源码形状。

## 最低共享层根因

该聚类同时包含硬切后的结构守卫漂移与 native painter 产品断言漂移。至少一部分测试仍把旧 TOML projection DTO、旧 `UiHostWindow` 方法形状或旧软件 painter 合同当成当前真相。由于用户明确要求不兼容旧架构，后续修复必须逐组判定“当前 runtime UI 产品合同”与“已退役结构期望”，不能为了让旧测试通过而恢复旧 DTO、旧 helper 或双绘制路径。

## 架构修复验收

- 先按 `generic_host_boundary`、activity-rail DTO、native painter/MUI、shell window、UI Asset host 五类建立当前生产 owner 映射；每类先跑单组 exact，记录真实最低根因。
- 对已退役架构期望，硬切测试到当前 runtime UI/typed projection 合同并增加“旧符号不存在”反向守卫；对仍有效的产品绘制合同，修最低共享生产层。
- `tests::host::retained_window` 组全绿后，再运行 Editor UI 08 完整门禁与 Editor M1 全量门禁。

## 禁止临时方案

- 禁止恢复 `clone_strong`、旧 `rail_nodes` TOML DTO、旧 painter/presentation cache 或同区域双路径，只为满足源码字符串测试。
- 禁止批量 `#[ignore]`、删除像素/命中断言、按测试名或资产路径增加生产特例。

## 产出记录与时间

| 里程碑 | 切片 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| Editor UI 08 M3 / Editor M1 | Retained-window/runtime UI 硬切后合同收束 | `未通过-43项待功能owner处理` | 2026-07-11 | 旧 06:17 binary 的完整门禁最终 2761/133/34（2491.83s），按功能归类后本计划接管 43 项；两个独立 exact 分别证明旧 `rail_nodes` DTO 与已删除 `UiHostWindow::clone_strong` 源码形状仍被测试要求。MUI、UI Asset 与 retained text 项已分别交接 Editor UI 06/05/03，本记录禁止把旧架构复活当成修复。 |
| Editor UI 08 M3 / Editor M1 | 当前源码完整门禁复核 | `未通过-失败集合未变化` | 2026-07-11 | 08:31 当前源码 binary 完整执行 2930 项为 2763/133/34（2258.13s）；与 06:17 门禁逐项比较，133 个失败名 added=0、removed=0，本计划 43 项归属不变。同一 binary 两个 fully-qualified exact 均 0/1（各 0.00s），仍分别要求旧 `rail_nodes` DTO 与已删除 `clone_strong`。 |
| Editor UI 08 M3 / Editor03+08 M1 | 当前全量门 Workbench/retained-host 回归复现 | `未通过-继续由功能owner处理` | 2026-07-12 | 受管 job `520d85713df249afae31661a7697ad07` 再次复现 menu pointer、viewport toolbar/projection、drawer/pane、welcome mount、window contract、ZUI boundary 与 Workbench view-model 失败；代表项包括 `componentized_workbench_window_template_bridge_exports_surface_projection_frames_and_routes`、`shared_viewport_surface_uses_unified_rust_pointer_dispatch`、`root_menu_pointer_click_dispatches_shared_menu_action_in_real_host`。该轮命令 registry/palette 专属测试已通过，故 UI 交互失败继续归本计划而非 Editor08 command registry；完整列表见 `D:/cargo-targets/editor08-m1-rerun4-20260712.log`，禁止恢复旧 host/painter/presentation 双路径。 |
| Editor UI 08 M3 / Editor15 M1 | 当前 editor binary Welcome mount 精确分片 | `未通过-1项待功能owner处理` | 2026-07-12 | `ui::retained_host::ui::tests::welcome_presentation::apply_presentation_projects_welcome_mount_nodes_into_global_context` 精确失败：当前投影节点数 22，断言要求 31；与本文件既有 Welcome mount/hard-cutover 聚类一致，继续由 shell owner 判定当前 projection contract，禁止在 Editor15 恢复旧 mount 节点。 |
| Editor UI 08 M3 / Editor09 M1 | 当前源码完整门停滞前复现 | `未通过-继续由功能owner处理` | 2026-07-13 | job `e81ed19d256f40c28ddb2437e9a18460` 再次记录 retained callback/menu/drawer/window/template-runtime/workbench projection 聚类；第 1755 项 Editor15 外部停滞前已观察 130 个跨功能失败名，故本行只登记本计划失败仍存在，不宣称最终数量。日志 `.codex/tmp/editor09-m1-full-lib-test-r2-20260713.log`。 |

## 修复结果与回传

- 状态：`open / 待修复`。
- 修复后更新本文件，并按交接规范移动到来源计划 `docs/plans/zircon_editor/editor/01/fixed-2026-07-11-retained-window-hard-cutover-expectations.md`；Editor UI 08 只保留相对回链与摘要。
