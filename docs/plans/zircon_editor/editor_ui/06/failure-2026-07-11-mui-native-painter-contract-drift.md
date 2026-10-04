---
handoff_kind: failure
status: open
created_at: 2026-07-11
summary_slug: mui-native-painter-contract-drift
origin_plan: docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md
fixing_plan: docs/plans/zircon_editor/editor_ui/06-component-library-mui.md
origin_child_dir: docs/plans/zircon_editor/editor/01
fixing_child_dir: docs/plans/zircon_editor/editor_ui/06
plan_link_mode: child_record_only
related_code:
  - zircon_editor/src/tests/host/retained_window/native_material_painter_mui_primitives
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes
  - zircon_editor/src/ui/retained_host/ui
  - zircon_runtime_interface/src/ui/design_tokens.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_theme/palette_projection.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_node_pipeline.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_node_pipeline/draw.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_node_pipeline/test_support.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_nodes.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_nodes/commands.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_nodes/geometry.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_nodes/fallback.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/material_primitives.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/mui_x_primitives.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/component_showcase_projection.rs
  - zircon_editor/src/ui/retained_host/ui/tests/component_showcase/mod.rs
  - zircon_editor/src/ui/retained_host/ui/tests/component_showcase/contracts.rs
  - zircon_editor/src/ui/retained_host/ui/tests/component_showcase/runtime_projection.rs
  - zircon_editor/src/ui/retained_host/ui/tests/component_showcase/state_projection.rs
  - zircon_editor/src/tests/host/retained_window/native_material_painter_mui_primitives/mod.rs
  - zircon_editor/src/tests/host/retained_window/native_material_painter_mui_primitives/support.rs
  - zircon_editor/src/tests/host/retained_window/native_material_painter_mui_primitives/avatar_badge_chip.rs
  - zircon_editor/src/tests/host/retained_window/native_material_painter_mui_primitives/field_and_icon.rs
  - zircon_editor/src/tests/host/retained_window/native_material_painter_mui_primitives/mui_x_chart_and_chat.rs
  - zircon_editor/src/tests/host/retained_window/native_material_painter_mui_primitives/mui_x_foundation.rs
  - zircon_editor/src/tests/host/retained_window/native_material_painter_mui_primitives/progress_and_overlay.rs
plan_sources:
  - docs/plans/zircon_editor/editor_ui/06-component-library-mui.md
  - docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md
  - docs/plans/engine-code-structure-convention.md
tests:
  - cargo test -p zircon_editor --lib --locked native_material_painter -- --test-threads=1
  - cargo test -p zircon_editor --lib --locked component_showcase -- --test-threads=1
---

# Editor UI 06：MUI/native painter 当前合同漂移失败交接

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md`
- 来源执行切片：Editor M1 当前源码完整单线程门禁；Editor 15 M1 追加 retained UI 精确分片
- 修复责任计划：`docs/plans/zircon_editor/editor_ui/06-component-library-mui.md`
- 交接原因：失败集中在 MUI 组件 token、状态优先级、几何与 native painter parity，最低共享原因不属于 Editor 内核或导出流水线。

## 失败现象与复现证据

Editor M1 当前源码 08:31 binary 的完整门禁中，MUI/component/native-painter 聚类仍为 73 项失败：其中 `native_material_painter_mui_primitives` 22 项、基础 painter 8 项，其余集中于 alerts/buttons/list rows/metrics/showcase。独立 circular-progress exact 为 0/1（0.00s）：实际 RGBA `[42, 166, 184, 255]`，旧断言为 `[53, 199, 208, 255]`。

该聚类归 Editor UI 06 的组件 token、state priority、geometry 与 native painter parity。后续必须先确认 design token 当前单源，再区分产品绘制回归和旧 palette 快照漂移；禁止复制旧颜色常量、恢复旧 painter 路径或按测试名称特判。

## 最低共享层根因

最低已证实边界是 Editor UI 06 的共享 design token、component state projection 与 native painter
消费合同存在整体漂移；component showcase 的结构测试还要求退役的源码形状。功能 owner 必须先裁决当前 typed
组件 DTO 和中央 token，再同步生产实现与测试，不能从上层 retained host 添加兼容结构。

## 架构修复验收

- 共享 token/state/geometry focused tests 与 component showcase 精确组全绿。
- `native_material_painter` 与 `component_showcase` 原始复现命令全绿。
- 重跑 Editor M1 分区与完整门禁，确认没有恢复旧 painter 或旧 DTO。

## 禁止临时方案

- 禁止恢复旧 painter 路径、旧 DTO、局部 palette 或按测试名特例。
- 禁止批量改像素断言、忽略失败或降低组件合同覆盖。

## 产出记录与时间

| 里程碑 | 切片 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| Editor UI 06 / Editor M1 | MUI/native painter/component showcase parity | `未通过-73项待功能owner处理` | 2026-07-11 | 完整门禁按功能归类 73 项；circular-progress exact 0/1，当前/旧期望 RGBA 分别为 `[42,166,184,255]` 与 `[53,199,208,255]`。先从共享 design token/palette 最低层向上复验。 |
| Editor UI 06 / Editor M1 | 当前源码完整门禁复核 | `未通过-失败集合未变化` | 2026-07-11 | 08:31 当前源码 binary 完整执行 2930 项为 2763/133/34（2258.13s）；与 06:17 门禁逐项比较，133 个失败名 added=0、removed=0，本计划 73 项归属不变。同一 binary circular-progress exact 0/1，当前/旧 RGBA 仍为 `[42,166,184,255]` / `[53,199,208,255]`。 |
| Editor UI 06 / Editor03+08 M1 | 当前全量门 component/native painter 回归复现 | `未通过-继续由功能owner处理` | 2026-07-12 | 受管 job `520d85713df249afae31661a7697ad07` 再次复现 MUI primitive、alert/paper/button/list/tree/status、component-showcase state 与 reference-well 投影失败；代表项包括 `native_template_painter_draws_mui_circular_progress_ring`、`workbench_toast_paints_status_mark_action_and_close`、`component_showcase_option_and_action_callbacks_are_rust_wired`。原始失败列表见 `D:/cargo-targets/editor08-m1-rerun4-20260712.log`；必须从共享 token/state/geometry owner 自底向上修复，不得复制旧色值或增加按测试名特例。 |
| Editor UI 06 / Editor15 M1 | 当前 editor binary retained showcase 精确分片 | `未通过-6项组件合同待owner处理` | 2026-07-12 | `ui::retained_host::ui::` 为 94/102，8 项失败；其中 6 项归 component showcase/reference/structure owner：`reference_component_tests` 1、`structure_component_tests` 2、`component_showcase` 3。代表 panic 为测试仍要求 `TemplatePaneActionData` 源码声明，owner 应按当前 typed component DTO 硬切断言，不得恢复退役结构。另 2 项 Build/Export pane 投影已单独写入 Editor15 子计划。 |
| Editor UI 06 / Editor09 M1 | 当前源码完整门停滞前复现 | `未通过-继续由功能owner处理` | 2026-07-13 | job `e81ed19d256f40c28ddb2437e9a18460` 在外部停滞前再次记录基础 native painter 8 项、MUI primitives 22 项以及 alert/paper/notification/material-lab 等组件失败；日志 `.codex/tmp/editor09-m1-full-lib-test-r2-20260713.log`。该轮只证明聚类仍 open，不用部分执行数替代完整门。 |

## 修复结果与回传

- 状态：`open / 待修复`；先跑 native painter 与 component showcase focused groups，再向上重跑 Editor M1。

## 2026-08-27 test-owner module anchor repair

Commit `7a20f921bb97ed428ae248cbcaf3c2fac5442ddf` removed the monolithic
`native_material_painter_mui_primitives.rs` leaf. Its existing same-name module tree now
owns `mod.rs`, primitive groups, shared support, and the circular-progress regression,
so `related_code` names that directory. No painter or test implementation changed and
no current managed result is added; the MUI parity failure remains open.

## 2026-09-19 受管静态合同回执

- 受管 ticket `b46ba7043acd4cd3a5bfa63074bb8cc8`（job/run 同 ID）已通过，退出码 `0`；
  输出为 `EDITORUI06_MUI_NATIVE_PAINTER_SOURCE_CONTRACT_PASS`、`CHECKED_PATHS=11`，
  source manifest `1d7782e4c40d18cae22b41a348b516deedb53dfcdf0a8c14e0c77ca035d8b371`。
- 本票据只验证当前 MUI/native painter、共享 `EditorPaletteTokens`、material primitive
  dispatch 与 component showcase 的源码合同；`ROOT_FAILURE_REMAINS_OPEN=true` 是有意保留的
  failure 状态。此前的失败复现和源码形状纠正证据不删除、不覆盖。
- 尚未获得 Cargo focused groups、Editor M1 全量、产品/性能门槛、独立 Critical/Important/
  Moderate 审查、failure return 或 closeout 的通过证据；本 failure 继续保持 `open`。

## 2026-09-21 independent current-source review receipt

- Reviewer Session `review-editorui06-mui-painter-r2` re-read the full failure record and the
  current UI-06 plan, then inspected the current painter, primitive-test, showcase-structure,
  and `EditorPaletteTokens` owners. The seven changed source/test paths were checked at their
  current bytes; the unrelated/unattributed `native_material_painter_mui_primitives/mod.rs`
  overlay was intentionally excluded and not absorbed.
- `EDITORUI06_RUSTFMT_PASS` covered 10 current Rust paths and `EDITORUI06_DIFF_CHECK_PASS`
  completed with only Git's LF-to-CRLF warnings. The source probe passed as
  `EDITORUI06_MUI_NATIVE_PAINTER_CURRENT_SOURCE_REVIEW_PASS`, covering shared token ownership,
  state-priority painter contracts, MUI/MUI-X primitive regressions, material dispatch, and
  Rust-owned component-showcase structure. Independent C/I/M review is `0/0/0`; no foreign
  source change was incorporated.
- This is static evidence only. Focused managed Cargo for `native_material_painter` and
  `component_showcase`, the Editor M1/full product and performance gates, canonical fixed
  return, managed closeout, and WeCom receipt remain pending. The failure therefore remains
  `open`.

## 2026-09-21 current-source ticket receipt

- Current-source static ticket `c20dac1b15c340fe89c539ea827f0a5a` passed under managed job
  `0ed4aad2d76843509c01a5f92e5ae9dc` (run ID equal to the ticket), exit code `0`, with
  `EDITORUI06_MUI_NATIVE_PAINTER_SOURCE_CONTRACT_PASS`, `CHECKED_PATHS=11`, and source
  manifest `f8863dba07557092cba72f78ad47e72cfc099d7a653bc47613d1c87399895503`.
- The ticket is static parse/source-contract evidence only. The earlier one-path probe ticket
  `9899ca73b91d4a0c97eac23f64d667ca` passed a non-acceptance command and is explicitly not
  reused for this failure.
- Focused managed Cargo, Editor M1/full product and performance gates, fixed return, closeout,
  and WeCom receipt remain pending; `status: open` is intentionally preserved.

## 2026-09-25 current-source owner-chain reconciliation

- The current production/test chain is now indexed explicitly: the
  `native_material_painter_mui_primitives` module tree (its `mod.rs`, support helpers, and
  primitive groups) calls the retained-host test bridge in
  `paint_template_nodes/template_node_pipeline/{test_support.rs,draw.rs}`, then the
  `template_nodes.rs` export and `template_nodes/{commands.rs,geometry.rs}` helpers route
  `push_template_node_commands` through `template_nodes/fallback.rs` to the
  `material_primitives.rs` and `mui_x_primitives.rs` families. Painter colors and geometry consume the host projection in
  `paint_theme/palette_projection.rs`, whose canonical inputs are
  `zircon_runtime_interface/src/ui/design_tokens.rs` (`EditorDesignTokens`/
  `EditorPaletteTokens`). The component-showcase path is separately indexed from
  `ui/pane_data_conversion/component_showcase_projection.rs` into the Rust-owned
  `ui/tests/component_showcase/{mod.rs,contracts.rs,runtime_projection.rs,state_projection.rs}`
  tests. This is the minimum direct chain for the two focused Cargo filters; no retired
  monolithic `native_material_painter_mui_primitives.rs` path is reintroduced.
- The current bytes were rechecked before this receipt. The frontmatter now contains 26 explicit
  existing related-code paths (3 directory anchors plus 23 concrete files); no path count is
  implied beyond that manifest. Key source hashes are:
  `design_tokens.rs` `1adaaf791f0593c1cef5ee06565d4055d770e71c2910663ee19d2f671116d3bc`,
  `palette_projection.rs` `2db767572bc0d3c9fc3859b0cdcf5ef6081cd36475719d9c555b14b0193dd7be`,
  `template_node_pipeline.rs` `7e2e7c0e2619372016d4b55c27dec265820afc9cd7dc039cff2b011e70c6818e`,
  `template_node_pipeline/test_support.rs` `3a06785041c28fee1298b2c5608e1f91eebd45dd48d3664cc5c70c9bfd164dbb`,
  `component_showcase_projection.rs` `5f7665a9c79e26572d9c0a13378ae6c9334b5555faa4b5531cdb0a51c0e3a858`,
  and `native_material_painter_mui_primitives/progress_and_overlay.rs`
  `35abeb7be788b6976a4de4437dd6505c87281ffa5702ceada91bc59922d24d9f`.
  Coordinator ownership inspection reports the retained-host painter tree as foreign or
  unattributed (including an archived owner for `material_primitives.rs`); this session does
  not claim, modify, or validate those dirty source bytes.
 - The existing static source-contract receipts remain static only. Focused managed Cargo for
   `native_material_painter` and `component_showcase`, the Editor M1/full product and performance
   gates, and the canonical return/closeout are still required. The failure therefore remains
   `open`.

## 2026-09-26 rolling successor source handoff

- Successor Session `failure-roll-01a084c8-editorui06-mui-painter-r4` owns only this
  failure document under audited ownership-transfer fingerprint
  `c1d33762958969b227143db284cd0ae2a9c441827242f0cb7c08e37b26b3fe6f`.
  The pre-review document boundary is snapshot `3907`, with document SHA
  `d5594c1d87addc721975e098a68c83c76de05ab6543a84955ddc83d911467ddc`.
  The retained-host painter and token source paths remain foreign or unattributed;
  this Session claims, edits, and absorbs none of those dirty bytes.
- At the pre-review boundary, the exact twelve-path source manifest (plan, failure
  record, and the ten existing painter/token source/test files used by the passed
  static contract) was
  `debeb29fefb9ce5a47e84115e718e3e6747c231f35b33dfda2cd7c6021162b02`. The
  final doc-only review receipt changes only the failure-record entry; the final
  twelve-path manifest is
  `4786deee4c4f673a5ec60480240ffbc649c06b69eed68ec621ea9ad829f529a5`.
  The plan
  SHA is `520b992814dc533547411e83a7e831b331670f85645cf52ad677c264651963e9`;
  the ten code/test hashes are unchanged from the corrected source ticket:
  `native_material_painter.rs` `c189a421346ddaf5b2ac140f0f42ee1e402e7edb21fe6300d13db6cafcb59511`,
  `avatar_badge_chip.rs` `40f8af6a92fcbfdd36617aec0862a84da5ffd13a5af10bfb6204ab2e00014a1e`,
  `field_and_icon.rs` `e9c2729529277d3b4decd7d569bffe6bc00185f84bf51b3750dc0cda3014c4c9`,
  `mui_x_chart_and_chat.rs` `6d4cdab7ef9ea4acfa30c9d52177fc0497d4b5102b14526645ad55ca42c38928`,
  `mui_x_foundation.rs` `b52205147456e09376064012a8fba2cd1fe00560b8cffe97a2c54f39f3e5647d`,
  `progress_and_overlay.rs` `35abeb7be788b6976a4de4437dd6505c87281ffa5702ceada91bc59922d24d9f`,
  `support.rs` `1a50b08b18f27653e163a2e8985aaba4cfd48a9cdfdc5ad828946b48771fa626`,
  `material_primitives.rs` `aebea782b21c20a40629fc831f845d221c7c1625489f9496ba6addd612645405`,
  `structure_component_tests.rs` `70efe82fcd16311e0485b92d81e4d79d45ba3f01418d63278872e8b277599bcb`,
  and `design_tokens.rs` `1adaaf791f0593c1cef5ee06565d4055d770e71c2910663ee19d2f671116d3bc`.
- Current-source static ticket `c20dac1b15c340fe89c539ea827f0a5a` is the sole
  reusable acceptance for this boundary and emitted
  `EDITORUI06_MUI_NATIVE_PAINTER_SOURCE_CONTRACT_PASS` with `CHECKED_PATHS=11`
  and exit code `0`. Failed checker ticket `817cfc2d67454b23a728ea1a05e49760`
  and non-acceptance probe `9899ca73b91d4a0c97eac23f64d667ca` remain diagnostic
  evidence and are not reused. The existing independent review is static
  `Critical=0 / Important=0 / Moderate=0`; no foreign source change was incorporated.
- Focused managed Cargo for `native_material_painter` and `component_showcase`,
  Editor M1/full product and performance gates, canonical `fixed-*` return,
  closeout, and WeCom receipt remain pending. External `E:\Git\zr_vm` cleanliness
  remains a validation admission blocker. Failure status remains `open`.
- Final r4 review receipt: reviewer `review_editor03_gizmo_private` verified
  snapshot `3909` (document SHA
  `6ec843bfd9ad84dea46e61cd2757b995f367f5ba5d540f727982461c84b687ef`) and
  the pre-review/final-manifest distinction, with **Critical=0 /
  Important=0 / Moderate=0**. The review-boundary manifest
  `4786deee4c4f673a5ec60480240ffbc649c06b69eed68ec621ea9ad829f529a5`
  remains the sealed source index; this receipt does not promote any deferred
  Cargo or product gate.
