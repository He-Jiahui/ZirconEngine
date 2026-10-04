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
  - zircon_editor/build.rs
  - zircon_editor/Cargo.toml
  - zircon_editor/src/ui/retained_host/mod.rs
  - zircon_editor/src/ui/retained_host/app/native_windows.rs
  - zircon_editor/src/ui/retained_host/host_contract/window.rs
  - zircon_editor/src/ui/retained_host/host_contract/globals.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/host_components.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/host_root.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/chrome_template_projection.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/chrome_template_projection/activity_rail.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/scene_projection.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/scene_projection/dock_patch.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/host_data.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/shell_presentation.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/projection_cache.rs
  - zircon_editor/assets/ui/editor/workbench_activity_rail.zui
  - zircon_editor/src/tests/host/retained_window/ui_debug_reflector.rs
  - zircon_editor/src/tests/host/retained_window/ui_asset_editor.rs
  - zircon_editor/src/tests/host/retained_window/support.rs
  - zircon_editor/src/tests/host/retained_window/shell_window/window_lifecycle.rs
  - zircon_editor/src/tests/host/retained_window/shell_window/template_paint.rs
  - zircon_editor/src/tests/host/retained_window/shell_window/support.rs
  - zircon_editor/src/tests/host/retained_window/shell_window/scene_snapshots.rs
  - zircon_editor/src/tests/host/retained_window/shell_window/runtime_render.rs
  - zircon_editor/src/tests/host/retained_window/shell_window/pointer_callbacks.rs
  - zircon_editor/src/tests/host/retained_window/shell_window/mod.rs
  - zircon_editor/src/tests/host/retained_window/presenter_store.rs
  - zircon_editor/src/tests/host/retained_window/platform_input_translation.rs
  - zircon_editor/src/tests/host/retained_window/native_workbench_window_menus.rs
  - zircon_editor/src/tests/host/retained_window/native_workbench_reference/text_and_module_input.rs
  - zircon_editor/src/tests/host/retained_window/native_workbench_reference/support.rs
  - zircon_editor/src/tests/host/retained_window/native_workbench_reference/reference_surface.rs
  - zircon_editor/src/tests/host/retained_window/native_workbench_reference/mod.rs
  - zircon_editor/src/tests/host/retained_window/native_workbench_reference/menu_keyboard.rs
  - zircon_editor/src/tests/host/retained_window/native_workbench_reference/dropdown_pointer.rs
  - zircon_editor/src/tests/host/retained_window/native_workbench_reference/dropdown_keyboard.rs
  - zircon_editor/src/tests/host/retained_window/native_window_targets.rs
  - zircon_editor/src/tests/host/retained_window/native_viewport_image.rs
  - zircon_editor/src/tests/host/retained_window/native_template_text.rs
  - zircon_editor/src/tests/host/retained_window/native_runtime_text_painter.rs
  - zircon_editor/src/tests/host/retained_window/native_mode.rs
  - zircon_editor/src/tests/host/retained_window/native_material_painter_paper.rs
  - zircon_editor/src/tests/host/retained_window/native_material_painter_notification_center.rs
  - zircon_editor/src/tests/host/retained_window/native_material_painter_mui_primitives/support.rs
  - zircon_editor/src/tests/host/retained_window/native_material_painter_mui_primitives/progress_and_overlay.rs
  - zircon_editor/src/tests/host/retained_window/native_material_painter_mui_primitives/mui_x_foundation.rs
  - zircon_editor/src/tests/host/retained_window/native_material_painter_mui_primitives/mui_x_chart_and_chat.rs
  - zircon_editor/src/tests/host/retained_window/native_material_painter_mui_primitives/mod.rs
  - zircon_editor/src/tests/host/retained_window/native_material_painter_mui_primitives/field_and_icon.rs
  - zircon_editor/src/tests/host/retained_window/native_material_painter_mui_primitives/avatar_badge_chip.rs
  - zircon_editor/src/tests/host/retained_window/native_material_painter_drag_overlay.rs
  - zircon_editor/src/tests/host/retained_window/native_material_painter_dialog.rs
  - zircon_editor/src/tests/host/retained_window/native_material_painter_command_palette.rs
  - zircon_editor/src/tests/host/retained_window/native_material_painter_alert.rs
  - zircon_editor/src/tests/host/retained_window/native_material_painter.rs
  - zircon_editor/src/tests/host/retained_window/generic_host_layout_paths.rs
  - zircon_editor/src/tests/host/retained_window/generic_host_boundary.rs
  - zircon_editor/src/tests/host/retained_window/callback_source_window.rs
  - zircon_editor/src/tests/host/retained_window/host_page_overflow_keyboard.rs
  - zircon_editor/src/tests/host/retained_window/mod.rs
  - zircon_editor/src/tests/host/retained_window/native_chrome_routing.rs
  - zircon_editor/src/tests/host/retained_window/activity_rail_template_boundary.rs
  - zircon_editor/src/tests/host/retained_window/native_host_contract/viewport_hierarchy_menu/viewport.rs
  - zircon_editor/src/tests/host/retained_window/native_host_contract/viewport_hierarchy_menu/tree_hover.rs
  - zircon_editor/src/tests/host/retained_window/native_host_contract/viewport_hierarchy_menu/mod.rs
  - zircon_editor/src/tests/host/retained_window/native_host_contract/viewport_hierarchy_menu/menu_overlay.rs
  - zircon_editor/src/tests/host/retained_window/native_host_contract/template_input.rs
  - zircon_editor/src/tests/host/retained_window/native_host_contract/support.rs
  - zircon_editor/src/tests/host/retained_window/native_host_contract/mod.rs
  - zircon_editor/src/tests/host/retained_window/native_host_contract/drag_projection.rs
  - zircon_editor/src/tests/host/retained_window/native_host_contract/chrome_pointer.rs
plan_sources:
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
tests:
  - cargo +1.94.1 test -p zircon_editor --lib --locked --jobs 1 --no-run --message-format short --color never
  - cargo +1.94.1 test -p zircon_editor --lib --locked --jobs 1 tests::host::retained_window::activity_rail_template_boundary::host_side_activity_rails_use_projected_toml_template_nodes -- --exact --test-threads=1
  - cargo +1.94.1 test -p zircon_editor --lib --locked --jobs 1 tests::host::retained_window::generic_host_boundary::rust_owned_host_contract_declares_window_globals_and_projection_data -- --exact --test-threads=1
  - cargo +1.94.1 test -p zircon_editor --lib --locked --jobs 1 tests::host::retained_window::generic_host_layout_paths::editor_ui_toml_assets_replace_former_workbench_source_roles -- --exact --test-threads=1
  - cargo +1.94.1 test -p zircon_editor --lib --locked --jobs 1 tests::host::retained_window::native_mode::native_floating_window_mode_uses_rust_owned_host_window_contract -- --exact --test-threads=1
  - cargo +1.94.1 test -p zircon_editor --lib --locked --jobs 1 tests::host::retained_window::shell_window::window_lifecycle::workbench_shell_window_starts_at_reference_size_and_can_resize -- --exact --test-threads=1
  - cargo +1.94.1 test -p zircon_editor --lib --locked --jobs 1 tests::host::retained_window::native_chrome_routing::native_host_activity_rail_click_wins_over_overlapping_drawer_header -- --exact --test-threads=1
  - cargo +1.94.1 test -p zircon_editor --lib --locked --jobs 1 tests::host::retained_window -- --test-threads=1
  - cargo +1.94.1 test -p zircon_editor --lib --locked --jobs 1 -- --test-threads=1
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

### 2026-09-25 current-source r2 reconciliation

- Successor Session `failure-roll-01a084c8-editorui08-retained-window-r2` owns this failure record only. The exact retained-window test tree (55 files) and the direct Rust/TOML production owners are listed in `related_code`; no source path is edited or re-attributed by this session.
- The current direct owner hashes are: `zircon_editor/build.rs` `695cc4f8369f0ffe37042e0831e16267027795c8c4216f987f3e8498faafe6c9`, `zircon_editor/Cargo.toml` `77d44b7ac1d4d3b6990ea7e111512a506738857d03d377b9f5cb34eae50d50b8`, retained-host `mod.rs` `9b1688cbbccfc0c25a7c3a3f28273b18d9a6b57ab8b6ade9ceb507d88c47912b`, `app/native_windows.rs` `6b1cc9389301ab48f2893b2de406a8234edd7931522c6abbe3a4cd2eb7be02d1`, `host_contract/window.rs` `fe472f5d3c45108ea8b6072e5acfbf72521edd34bea7ede2695a900a5ba93984`, `host_contract/globals.rs` `f6290cadd83e33aca3eddd9259fa0229c12faf6c7c891df7ea89e459c4de50a1`, `host_components.rs` `b34b7865ae12ec16ca392730af1e4995c164f1fa215da06e8fbb9141c19e2e53`, `host_root.rs` `3959845404109854816fb358d354bbed99d7829fc4fd9c887b71625d639ad34d`, `chrome_template_projection.rs` `218ae54079ad49363080dd476294dc3421ca8f16003c367cf90fa0b2a71649e0`, `chrome_template_projection/activity_rail.rs` `5cdad98aacc67e82f6b93f96aaf0e967d60949015307f65432d170fb5e4f2109`, `scene_projection.rs` `305b9d8b36e9fbf6f521e49b4cc752904b126145aa0d97eab9c4342669dde31a`, `scene_projection/dock_patch.rs` `9cf5f60387210eea6f7ed46bb0fe5135573cc658260e93f561376fc99470f8a5`, `host_data.rs` `8a79933b4c0869886787d5e4b7999f9c0a4afb948baa9689580d630f47131ce5`, `shell_presentation.rs` `b428a32a7b7b615268475b06caf406c84c867e268f3f19c4ba2a570fdd2f8405`, `projection_cache.rs` `19c3cd30c0a35e633d7649c609206adf44de4ee53e07f5210d354cc64fb568aa`, and `workbench_activity_rail.zui` `76a729cae7a1f6048339c58aedf9d1ee0eef2d21d684a7a780bba2fef80f6e50`.
- `git status` reports foreign dirty edits in these direct owners (Cargo/build, retained-host, projection, and activity-rail asset); this Session preserves that provenance. All related paths exist, and all six focused filters resolve to real test functions. Managed Cargo, the full retained-window gate, Editor UI 08/M1 upward acceptance, and independent Critical/Important/Moderate review remain pending; no fixed return or closeout is claimed.

### 2026-09-26 successor intake (failure-roll-01a084c8-editorui08-retained-window-r3)

- The stale r2 lifecycle was cancelled through the coordinator after its
  heartbeat expired with no active lease. Successor
  `failure-roll-01a084c8-editorui08-retained-window-r3` now owns only this
  failure record. Ownership transfer fingerprint is
  `8944fe04f184df1c5c3414424dccd9320342e6adea7e240d3a629711f075a60a`, and
  pre-review snapshot `3917` sealed the record at SHA
  `63b73acba74eff05b85a8f6bcf7ec39fce614f92b4491fc0a1ca82382ee1914e`.
- The sixteen direct owner hashes listed in the r2 reconciliation were
  rechecked against the current working tree and still match. The checkout
  remains broadly dirty across retained-host, projection, test, and asset
  paths; those changes retain their foreign provenance and are not claimed,
  edited, or leased by r3. The 55-file related-code tree and six focused
  filters remain evidence references only.
- No prior supervisor or full-gate result is promoted to acceptance. This
  successor is an evidence-only reconciliation while the current owners
  stabilize their source chain. A fresh managed `zircon_editor --lib`
  retained-window gate, complete Editor UI 08/M1 upward gate, and an
  independent review are required before any `fixed-*` return; closeout and
  WeCom notification remain pending. The failure stays open.

### 2026-09-26 independent successor review receipt

- Read-only independent reviewer `/root/review_editor03_gizmo_private`
  rechecked snapshot `3918` and its failure-record SHA
  `188f3da0f34eda3919cf3445c2cc381926072a90af25523fd913f6682b1775cf`.
  The stale r2 cancellation/expired lease, successor fingerprint
  `8944fe04f184df1c5c3414424dccd9320342e6adea7e240d3a629711f075a60a`, all
  sixteen direct-owner hashes, and foreign dirty provenance are explicit.
- The review found no source absorption and no promotion of supervisor or
  full-gate evidence. Managed retained-window/Editor UI 08/M1/upward gates,
  canonical return, closeout, and WeCom remain pending. C/I/M result:
  **Critical=0 / Important=0 / Moderate=0**. The failure remains open.
