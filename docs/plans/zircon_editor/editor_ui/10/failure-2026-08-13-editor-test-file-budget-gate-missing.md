---
handoff_kind: failure
status: open
failure_scope: local
plan_link_mode: child_record_only
created_at: 2026-08-13
summary_slug: editor-test-file-budget-gate-missing
origin_plan: docs/plans/zircon_editor/editor_ui/10-code-structure-and-module-conventions.md
fixing_plan: docs/plans/zircon_editor/editor_ui/10-code-structure-and-module-conventions.md
origin_child_dir: docs/plans/zircon_editor/editor_ui/10
fixing_child_dir: docs/plans/zircon_editor/editor_ui/10
related_code:
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_editor_structure.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/editor_structure_audits/module_convention_boundary.py
  - zircon_editor/src/tests/structure_convention/mod.rs
  - zircon_editor/src/tests/host/retained_window
  - zircon_editor/src/tests/editing/ui_asset_replay.rs
  - zircon_editor/src/tests/ui/boundary/template_assets.rs
  - zircon_editor/src/ui/layouts/views/view_projection.rs
  - zircon_editor/src/core/editing/engine/transaction.rs
tests:
  - python -B -m unittest tools.tests.test_editorui10_test_file_budget_contract -v
  - python -B .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_editor_structure.py --json --repo-root E:\\Git\\ZirconEngine
  - cargo test -p zircon_editor --lib structure_convention --locked
---

# EditorUI10: editor test file budget gate missing

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor_ui/10-code-structure-and-module-conventions.md`
- 来源执行切片：M3.T1 `单一 owner / 拆 > 800 行测试`
- 修复责任计划：`docs/plans/zircon_editor/editor_ui/10-code-structure-and-module-conventions.md`
- 交接原因：R1.4/R4.3 的通用 test-owner 预算 gate 由 EditorUI10 结构审计唯一拥有；每个被报告的行为测试仍由其所属功能计划分批 folder-backed 硬切。

## 失败现象与复现证据

`audit_editor_structure.py --json` 原先只输出 `oversized_production_files`。因此计划中声明的
`editor_ui_10_no_oversized_test_files` 没有实现，`structure_convention` 也无法拒绝任何超过 800 行的测试 owner。

2026-08-13 当前源码审计在新增通用扫描后报告：

- `oversized_test_file_count = 48`，`oversized_test_file_exemptions = []`。
- 最大项为 `src/tests/host/retained_window/native_host_contract.rs`（3123 行）；同一 retained-window
  owner 还有 `shell_window.rs`（1465）、`native_workbench_reference.rs`（1187）与
  `native_material_painter_mui_primitives.rs`（984）。
- UI asset / scene authoring 测试仍有 `ui_asset_replay.rs`（1910）、
  `ui_asset_preview_binding_authoring.rs`（1798）、`ui_asset_theme_authoring.rs`（1381）等超限 owner。
- component/ZUI boundary 测试仍有 `template_assets.rs`（1560）、
  `zui_asset_governance/workbench_primitives.rs`（1174）等超限 owner。
- 同一次审计还报告生产 owner `core/editing/engine/transaction.rs`（1030）与
  `ui/layouts/views/view_projection.rs`（1213）超过 R1.4 的生产预算；它们必须由 Editor03
  transaction 和 EditorUI10 projection owner 分别前向修复，不能以测试豁免覆盖。

新增隔离回归 `test_editorui10_test_file_budget_contract` 已证明：801 行测试必须被报告，800 行边界不报告，
显式 fixture 豁免只以路径、行数和理由单独输出。旧实现对该 API 返回 `TypeError`，因此该回归构成 gate 缺失的 RED 证据。

## 最低共享层根因

EditorUI10 的 `EditorModuleConventionAudit` 以“`tests` 不属于 production”过滤了所有测试 owner，
却没有建立对应的 800 行测试扫描、计数、显式豁免台账和 Rust gate。此前结构记录中
`classified-and-clear` 只表示 production audit 清零，不能表示 R1.4/R4.3 已满足。

## 二次审查补充

2026-08-13 的独立复审确认通用 gate 还需要三个精确修复，均属于本 EditorUI10 结构审计 owner：

- `editor_ui_10_no_oversized_test_files` 必须是实际 Rust 零容忍断言，而不只是 JSON shape 检查；该断言已补入
  `structure_convention/mod.rs`，故当前 48 项债务会显式保持 RED。
- `test_owner_class_for` 不能先把所有 `src/tests/**` 归为 `editor-tests`；必须先匹配 retained-host、host、UI、
  editing 等功能目录，保证 handoff 能路由到对应功能计划。
- `is_test_owner` 不能仅按 `tests.rs` / `*_tests.rs` 文件名判断；普通生产模块可能合法使用这些名称。只有 `tests/`
  目录或实际 test/cfg(test) 属性才可归类为测试 owner。显式豁免还必须拒绝空白 remediation reason。

初次修复尝试因 NTFS ACL 拒绝而没有写入；在协调器返回精确 lease
`f7bbc136a4b74d8790debf02aea409ac` 后，最小补丁已前向完成，未改变 ACL。新增隔离合同还确认：
带内联 `#[cfg(test)]` 的生产模块仍必须进入 production audit，只有 `tests/` 目录或带真实测试属性的专用
`tests.rs`/`*_tests.rs` 才属于 test-file budget。该边界排除了 67 项的误报，当前准确审计恢复为 48 项、0 豁免。
在新的静态合同 GREEN、重新审计和二次审查完成后，helper 修复可作为已完成子项；整个 failure 仍不得在
48 项测试拆分归零前回传为 fixed。

## 架构修复验收

- `EditorModuleConventionAudit` 必须持续输出 `oversized_test_file_count`、完整
  `oversized_test_files` 及 `oversized_test_file_exemptions`；豁免必须逐路径写明理由，不能使用目录或 glob 白名单。
- `editor_ui_10_no_oversized_test_files` 必须在所有 >800 行测试 owner 清零前保持失败；不得删除或弱化该断言。
- 每个报告项必须按行为 owner 实施 folder-backed 拆分，薄 `mod.rs` 只挂载子 owner；保留测试函数语义，
  不复制测试树、不留旧 flat 文件或 compatibility module。
- 分批责任：UI asset/editor-domain 测试由 Editor07/Editor03/Editor06 owner 处理；MUI/ZUI boundary
  测试由 EditorUI06/11/12 owner 处理；retained-host/workbench 测试由 EditorUI08/10 owner 处理；
  core recovery/gateway/event tests 由各 core 功能计划处理。EditorUI10 负责 gate、retained-window 首批及最终汇总复验。
- 上游重跑：`python -B -m unittest tools.tests.test_editorui10_test_file_budget_contract -v`、
  `audit_editor_structure.py --json`、受管 `cargo test -p zircon_editor --lib structure_convention --locked`；
  所有运行时/产品测试保持各功能 owner 的现有验收路径。

## 功能 Owner 前向路由

下表以当前 `audit_editor_structure.py` 的 48 项输出为准。它在 audit helper 修复前仍把大多数
`src/tests/**` 标为 `editor-tests`；此表按行为域消除该临时分类错误，**不构成完成声明**。每个目标计划
必须先在自己的子目录建立/更新 `failure-*.md`，在维持功能语义的前提下完成 folder-backed 拆分并回传
新的审计计数。

| 目标计划 | 数量 | 精确超限测试 owner |
| --- | ---: | --- |
| Editor17 `editor/17-editor-services-and-recovery.md` | 1 | `core/recovery/tests/autosave_adapter.rs` |
| Editor03 `editor/03-command-transaction-and-undo.md` | 1 | `tests/editing/reflected_command.rs` |
| Editor07 `editor/07-domain-editors-and-graph-foundation.md` | 6 | `tests/editing/ui_asset/{inspector,tree_and_undo}.rs`; `tests/editing/{ui_asset_palette_drop,ui_asset_preview_binding_authoring,ui_asset_replay,ui_asset_theme_authoring}.rs` |
| Editor10 `editor/10-project-and-asset-reference-management.md` | 1 | `tests/editing/ui_asset/reference_and_promotion.rs` |
| Editor02 `editor/02-data-sync-and-messaging.md` | 3 | `tests/editor_event/runtime/integration.rs`; `tests/editor_message/bus/backpressure.rs`; `tests/runtime_event_consumer_bounded_pump.rs` |
| Editor01 `editor/01-editor-kernel-and-runtime-interaction.md` | 1 | `tests/gateway/session.rs` |
| EditorUI08 `editor_ui/08-workbench-shell-on-runtime-ui.md` | 21 | `tests/host/{binding_dispatch,pane_presentation}.rs`; `tests/host/manager/{bootstrap_and_startup,ui_asset_reference_and_promotion}.rs`; `tests/host/retained_callback_dispatch/template_bridge/{workbench_toolbar_breakpoints/mod,workbench_window_menus}.rs`; `tests/host/retained_menu_pointer/visual_screenshot/blend_space_workspace.rs`; `tests/host/retained_window/{native_host_contract,native_material_painter_mui_primitives,native_workbench_reference,shell_window}.rs`; `tests/host/template_runtime/{component_showcase_state,pane_body_documents}.rs`; `tests/{ui/boundary/workbench_projection_cutover,workbench/layout/editor_layout_contracts}.rs`; `ui/host/play_pending_decision/tests.rs`; `ui/layouts/windows/workbench_host_window/chrome_template_projection/tests.rs`; `ui/retained_host/app/tests/drag_sources.rs`; `ui/retained_host/host_contract/surface_hit_test/template_node_tests.rs`; `ui/retained_host/ui/tests/{component_showcase,host_scene_projection}.rs` |
| EditorUI05 `editor_ui/05-ui-asset-management.md` | 2 | `tests/ui/assets_activity/bootstrap_assets.rs`; `tests/ui/boundary/template_assets.rs` |
| EditorUI06 `editor_ui/06-component-library-mui.md` | 7 | `tests/ui/boundary/global_material_surface_assets.rs`; `tests/ui/boundary/material_component_lab/{feedback,inventory,lab_theme,shell}.rs`; `tests/ui/boundary/material_meta_component_contracts.rs`; `tests/ui/component_adapter.rs` |
| EditorUI11 `editor_ui/11-zui-suffix-convergence-and-ui-toml-retirement.md` | 2 | `tests/ui/boundary/zui_asset_governance.rs`; `tests/ui/boundary/zui_asset_governance/workbench_primitives.rs` |
| EditorUI03 `editor_ui/03-text-and-font-stack.md` | 2 | `ui/retained_host/host_contract/paint_text/draw/layout/tests.rs`; `ui/retained_host/host_contract/paint_text_tests.rs` |
| Editor09 `editor/09-editor-asset-management.md` | 1 | `ui/layouts/views/asset_browser/tests.rs` |

## 禁止临时方案

- 不得将超限测试加入 blanket exemption、提高 800 行预算、从 audit 输出中排除 `src/tests/**`，或删除 Rust gate。
- 不得为避免迁移而保留旧 flat test 文件、`#[path]` compatibility mount、重复 helper/test tree 或测试专用行为分支。
- 不得把 `transaction.rs`、`view_projection.rs` 的生产超限问题伪装为 test-budget 修复，或吸收到无关 retained-window 提交。

## 修复结果与回传

Open state: `待修复`。本记录仅完成通用 gate 的 RED/GREEN 基础设施建立；48 个测试 owner 与两个生产 owner
尚未全部拆分，未声明 EditorUI10 M3 或任何下游功能计划通过。

## 产出记录与时间

| 时间 | 里程碑/切片 | 状态 | 完成项目与证据 | 后续门禁 |
| --- | --- | --- | --- | --- |
| 2026-08-13 | M3 test-budget audit RED/forward handoff | `resolving_failure` | 新增 801/800/显式豁免隔离回归；旧 audit API 因不接受 test budget 参数而失败。通用 audit 与 Rust contract 已前向补齐，当前主树报告 48 个超限测试、0 个豁免。 | 各功能 owner 按行为拆分；只有 audit count 归零、受管 structure gate 与二次审查完成后才可 fixed return。 |
| 2026-08-13 | 二次审查与 gate 加固 | `resolving_failure` | 已补 `editor_ui_10_no_oversized_test_files` 零容忍 Rust gate。复审发现的 test owner 分类、文件名误判和空白豁免校验缺陷均已在精确 helper lease 下前向修复；新增内联 `cfg(test)` production-boundary 合同。 | 4/4 静态合同、48/0 重新审计与独立二审后，helper 子项才可记为完成；48 个业务 owner 继续保持 open。 |
| 2026-08-13 | 48 个 test owner 功能路由 | `open` | 以当前审计逐项建立 12 个目标计划的前向拆分表：EditorUI08 21 项、EditorUI06 7 项、Editor07 6 项，其余 14 项分配给 Editor01/02/03/09/10/17 与 EditorUI03/05/11。 | 每个目标计划先落自己的 child failure 与 exact lease；EditorUI10 仅在计数归零后汇总复验。 |
| 2026-08-13 | 最终独立二审 | `open` | 复核 `C/I/M = 0/0/0`：12 个路由组总计 48、覆盖当前全部超限路径；Rust gate 直接断言 `oversized_test_files` 为空，failure 仍为 `open`，没有 fixed/closeout 误标。 | 保持 RED；等待审计 helper 三项修复和各功能 owner 的 folder-backed 拆分回传。 |
| 2026-08-13 | helper 修复验证 | `resolving_failure` | 精确 lease `f7bbc136a4b74d8790debf02aea409ac` 下完成分类/检测/空白豁免修复；`test_editorui10_test_file_budget_contract` 4/4 GREEN，重审计为 48 个超限、0 豁免，排除内联测试造成的 67 项虚报。 | 对 helper 修复做独立二审；随后各功能 owner 继续拆分，Rust zero-tolerance gate 在计数归零前必须 RED。 |
| 2026-08-13 | helper 属性边界二审修复 | `resolving_failure` | 二审指出裸字符串会把注释/字符串误归类；在 lease `76456b8289be48e29b1f97e79adb89fe` 下改为行首 Rust 属性识别，并补普通 `tests.rs`、`*_tests.rs`、注释/字符串及内联 `cfg(test)` 回归。静态合同 5/5 GREEN，审计稳定为 48 个超限、0 豁免。 | 最终独立二审后关闭 helper 子项；48 个业务 owner 仍保持 open，未达到 fixed return。 |
| 2026-08-13 | `cfg(test)` regex 复审修复 | `resolving_failure` | 最终复审发现旧 regex 漏真 `#[cfg(test)]` 且误接 `#[test::fixture]`；已收紧为完整属性匹配，新增两项正反例。`test_editorui10_test_file_budget_contract` 5/5 GREEN，审计保持 48 个超限、0 豁免。 | 再次独立复审完整属性规则；仅 helper 子项可在复审后完成，根 failure 仍等待 48 个拆分。 |
| 2026-08-13 | helper 最终独立二审 | `completed` | 独立复审 `C/I/M = 0/0/0`：完整 `#[test]`/`#[cfg(test)]` 行首属性可识别；`#[test::fixture]`、注释、字符串和 `cfg(testing)` 不会误匹配；专用测试模块与内联生产测试的边界均有合同覆盖。 | helper 子项完成；根 failure 保持 `open`，等待 48 个功能 owner 的 folder-backed 拆分与受管 zero-tolerance gate GREEN。 |
| 2026-08-13 | cross-plan child failure 路由 | `open` | 已在具备精确 lease 的 11 个目标子计划创建 open handoff，逐路径覆盖准确审计的 46/48 owner；所有 handoff 均为 cross-plan，不含业务源码改动。 | EditorUI03 的 retained-text 两项因 lease 请求 `database is locked` 尚未创建子记录：`ui/retained_host/host_contract/paint_text/draw/layout/tests.rs`、`ui/retained_host/host_contract/paint_text_tests.rs`。下一个可用 EditorUI03 会话须先建 child failure，再完成拆分。 |
| 2026-08-13 | EditorUI08 native-host contract forward repair | `resolving_failure` | `native_host_contract.rs` 已从 3123 行 flat owner 硬切为 4 个行为模块和唯一 support，43 个测试保留；结构审计当前为 47 个超限测试、0 个豁免。 | 该项须经受管 Rust test；其余 47 个 owner 仍保持根 failure `open`。 |
| 2026-08-13 | native-host contract independent second review | `completed` | 独立二审 `C/I/M = 0/0/0`：43 个基线 `#[test]` 与新目录映射完全一致（13/8/10/12）；旧 flat 文件删除、目录挂载正确、无 compatibility mount/re-export，全部新文件低于 800 行。 | EditorUI08 子项剩余 20 个 owner；根 failure 仍为 `open`，当前总债务 47 项、0 豁免。 |
| 2026-08-13 | native-host contract managed Rust gate admission | `pending_environment_repair` | 为 7 个 retained-window source path 生成 SHA-256 manifest，申请受管 `cargo acquire test`（目标 `E:/cargo-targets/zircon-editor/editorui10-native-host-contract-20260813`，过滤器 `native_host_contract`）。协调器在 job 创建前拒绝：`Coordinator-managed work is blocked until unregistered D/E/F artifacts are removed`。 | 该准入故障由 Coordinator 环境 owner 前向修复；不得以未受管 Cargo 替代。恢复后以新的 current-source manifest 重建本局部 Rust gate。 |
| 2026-08-13 | EditorUI08 shell-window forward repair | `resolving_failure` | `shell_window.rs` 已从 1465 行 flat owner 硬切为窗口生命周期、场景快照、模板绘制、运行时渲染、指针回调 5 个行为模块与唯一 494 行 support；16 个测试逐项保留，定向 `rustfmt` 与 `diff --check` 通过，结构审计当前为 46 个超限测试、0 个豁免。 | 该项须经独立二审；EditorUI08 剩余 19 项，根 failure 仍为 `open`，不得以局部计数下降回传 fixed。 |
| 2026-08-13 | shell-window independent second review | `completed` | 独立二审 `C/I/M = 0/0/0`：基线 16 个 `#[test]` 和迁移模块映射完整一致（4/2/5/4/1）；旧 flat 文件删除，support 只用 `pub(super)`，无 compatibility mount/re-export，全部新文件低于 800 行。 | 根 failure 仍为 `open`，当前总债务 46 项、0 豁免；EditorUI08 还有 19 项行为 owner 待处理。 |
| 2026-08-13 | EditorUI08 native-workbench-reference forward repair | `resolving_failure` | `native_workbench_reference.rs` 已从 1204 行 flat owner 硬切为 reference surface、文本/模块输入、下拉指针、下拉键盘、菜单键盘 5 个行为模块与唯一 211 行 support。迁移前 current source 的预存 hit-index 测试被按 25-test 基线完整保留；定向 `rustfmt`/`diff --check` 通过，结构审计当前为 45 个超限测试、0 豁免。 | 独立二审；根 failure 保持 `open`，EditorUI08 剩余 18 项，不能把预存测试或局部计数下降归为全局 fixed。 |
