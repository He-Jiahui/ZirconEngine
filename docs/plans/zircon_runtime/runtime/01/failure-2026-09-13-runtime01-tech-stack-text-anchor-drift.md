---
handoff_kind: failure
status: open
failure_scope: cross_plan
created_at: 2026-09-13
summary_slug: runtime01-tech-stack-text-anchor-drift
origin_plan: docs/plans/optimize/zircon_tooling/13-repository-codex-skill-hook-structural-audit-governance-security-currentness-review.md
fixing_plan: docs/plans/zircon_runtime/runtime/01-tech-stack-and-dependency-governance.md
origin_child_dir: docs/plans/optimize/zircon_tooling/13
fixing_child_dir: docs/plans/zircon_runtime/runtime/01
plan_link_mode: child_record_only
related_code:
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/tech_stack_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/tech_stack_anchor_inventory.py
  - docs/plans/zircon_runtime/runtime/01-tech-stack-and-dependency-governance.md
  - docs/crates/zircon_runtime/ui/text.md
  - zircon_runtime/src/ui/text/shaper.rs
  - zircon_runtime/src/ui/tests/text_shaper.rs
tests:
  - python -B -m unittest tools.tests.test_runtime_tech_stack_boundary.RuntimeTechStackBoundaryTests.test_current_optional_text_and_backend_feature_declarations_are_clean tools.tests.test_runtime_tech_stack_boundary.RuntimeTechStackBoundaryTests.test_jolt_backend_is_feature_gated_and_plugin_owned -v
  - python -B -m unittest tools.tests.test_runtime_tech_stack_boundary -v
  - managed zircon_runtime lib test filter text_shaper_stack_uses_shared_text_service_for_font_backends
---

# Runtime01：技术栈文本行为锚与矩阵标题漂移

## 来源执行者

- 来源计划：`docs/plans/optimize/zircon_tooling/13-repository-codex-skill-hook-structural-audit-governance-security-currentness-review.md`
- 来源执行切片：Tooling13 full Runtime static union。
- 修复责任计划：`docs/plans/zircon_runtime/runtime/01-tech-stack-and-dependency-governance.md`
- 交接原因：完整静态批次发现的两个风险都属于 Runtime01 技术选型/文本栈合同；Tooling13 仅负责审计和调度，不应通过放宽锚点掩盖 Runtime01 当前源漂移。

## 失败现象与复现证据

2026-09-13 在当前工作树直接运行
`python -B -m unittest discover -s tools/tests -p test_runtime_*.py`，实际执行
1075 项，1070 通过、5 项失败。其中两项均为 Runtime01 tech-stack boundary 复现：

- `test_current_optional_text_and_backend_feature_declarations_are_clean`
- `test_jolt_backend_is_feature_gated_and_plugin_owned`

两项都报告同一组风险：`Runtime UI text stack matrix anchors are missing.` 与
`Runtime 01 behavior test anchors are missing.`。精确 focused 重跑仍为 2/2 失败，未把
静态排队回执当作通过。

快照 3504 保留了修复前当前源哈希：`docs/crates/zircon_runtime/ui/text.md` 为
`4f7b2e4728dc0c70b17b0bd8da5c806c38693242fb635fc09b34ab15fcd52977`，
`zircon_runtime/src/ui/tests/text_shaper.rs` 为
`d2cca4927eb2f45dc31a8fc2c2c6df59a099afb7f4e22a6d6536df16a972d912`。

## 最低共享层根因

现行 Runtime01 审计锚仍要求三层矩阵中的精确标题和
`text_shaper_stack_uses_shared_text_service_for_font_backends` 行为回归。提交
`579805160` 的硬切优化正确地移除了已经无职责的 `UiTextShaperStack` wrapper，却同时
删除了该回归函数；同一轮文档机械改名把 `Font registry, raster, and SDF policy` 改为
`Font database, raster, and SDF policy`。因此当前实现没有恢复旧 wrapper 的需要，但合同的
可执行证据与精确文档锚断裂。

## 架构修复验收

- 保持 `UiSharedTextShaper` 作为唯一现行 UI shaper，不恢复 `UiTextShaperStack`、旧
  backend-intent 枚举或兼容 facade。
- 在 `zircon_runtime/src/ui/tests/text_shaper.rs` 恢复同名回归，并实际覆盖 Native 与
  SDF render-mode 请求经过共享 shaper 的 layout/measurement parity。
- 将矩阵标题恢复为审计要求的 `Font registry, raster, and SDF policy`，并同步活动
  Runtime01 计划中关于已删除 wrapper 的事实说明。
- focused 两项、完整 `test_runtime_tech_stack_boundary` 与再次 full Runtime static union
  均须在当前源实际执行；Rust `text_shaper` managed gate 仍需单独取得绿色证据。

## 禁止临时方案

- 不得修改或删除 `TEXT_STACK_DOC_ANCHORS` / `TECH_STACK_BEHAVIOR_TEST_ANCHORS` 来制造
  绿色结果。
- 不得只在文档中添加测试名而不恢复可执行 Rust 测试。
- 不得恢复已硬切删除的 `UiTextShaperStack` 兼容层，也不得把外部 Cargo 污染阻断伪装成
  Rust 验收通过。

## 修复结果与回传

- 2026-09-18 successor Session `failure-roll-01a084c8-runtime01-tech-stack-text-anchors-r2`
  通过协调器 ownership transfer 接管了旧归档 Session 的当前源码快照；交接指纹为
  `270a5dfc6d3b2eabcaab13fae74251089c6417e5bed272cf317e8fe8cf259ebc`，随后补接
  `zircon_runtime/src/ui/text/shaper.rs` 的指纹为
  `96f59481e628bfd2b7ad8092fdc8227c9006e3795cf6646dc7945d8ed9a0a301`。未恢复旧
  `UiTextShaperStack` wrapper，也未修改任何审计锚常量。
- 当前源码证据：`text_shaper.rs` 当前 SHA-256
  `3516dc2f605e7aebe3647273b13e4caf9ea52239d312288b1dc3e068fd1b2ac5`，矩阵文档
  `372a49a01bc90787bf8ad9c61623f4466885cd38d4f292e3a64d3bce3ebc6ecd`，Runtime01
  计划 `7ef358a5a1430d8d29010ccf8b3c4fbfc7b91631ae70044c2d5183b4dc800556`；均由
  successor Session 持有 live lease 并显示 `integration_ready`。
- 2026-09-18 实际验证：两个原始聚焦审计测试通过 `2/2`；
  `python -B -m unittest tools.tests.test_runtime_tech_stack_boundary -v` 通过 `8/8`；
  `python -B -m unittest discover -s tools/tests -p "test_runtime_*.py"` 通过
  `1262/1262`（305.752s）；`rustfmt --check --edition 2021
  zircon_runtime/src/ui/tests/text_shaper.rs` 通过。
- Rust 受管门禁已按精确过滤器提交一次，request `b1b87ec2299244d89ad9a6c3b3973359`。
  协调器在 admission 阶段以 `validation_ticket_external_worktree_dirty` 拒绝，原因是
  外部 `E:\Git\zr_vm` 尚有未提交改动；没有生成 validation ticket、Cargo job 或测试
  级结果。该拒绝不能复用为 Rust 通过证据，待外部 owner 提交后必须用同一当前哈希
  重新取得受管验证。
- 当前状态仍为 `open`：源码与 Python/静态证据已完成，但 Rust 受管验收、独立
  Critical/Important/Moderate 审查、canonical `fixed-*`/return 回传与 closeout
  尚未完成。

## 2026-09-18 受管验收准入复核与挂起

- 本轮没有改变源码或测试范围。先有一次本地 CLI 调用误带 Cargo 并行度覆盖参数，
  被项目策略拒绝（`Cargo build parallelism is owned by the coordinator`）；该调用
  没有生成可复用的 coordinator request、ticket 或 job。
- 随后按协调器拥有并行度的规则重新提交精确门禁：
  `cargo +1.94.1 test -p zircon_runtime --lib
  text_shaper_stack_uses_shared_text_service_for_font_backends --locked -- --nocapture
  --test-threads=1`，请求参数为
  `failure-roll-01a084c8-runtime01-tech-stack-text-anchors-20260918-r4`，协调器
  request `be5434c21eea43a19cac271ac977ddd1`。该请求在 admission 阶段以
  `validation_ticket_external_worktree_dirty` 拒绝，repo root 为
  `E:\Git\zr_vm`；没有生成 validation ticket、Cargo job、cargo run 或测试级结果。
- 通过只读 Git 检查保留了阻断的原始证据：
  `git -C E:\Git\zr_vm status --porcelain=v2 --untracked-files=all` 返回大量已跟踪
  修改、修改中的 submodule 以及未跟踪文件；`git -C E:\Git\zr_vm diff --quiet`
  返回退出码 `1`（仅伴随换行格式警告）。该工作树属于外部 owner，本 Session 不清理、
  回滚或提交其内容，也不把准入拒绝解释为 Rust 通过。
- 因外部 owner 尚未提供干净且可冻结的 revision，本 Session 将保持
  `waiting_validation`，待同一当前源码哈希和同一精确过滤器重新取得 Windows 受管绿色
  证据后再继续独立 C/I/M 审查、canonical `fixed-*`/return 与 closeout。当前 failure
  仍为 `open`；本节不构成验收或关闭。

## 2026-09-21 独立源审查回执（Runtime01 tech-stack text anchors r2）

- 审查 Session：`review-runtime01-tech-stack-r2`；本次仅做当前源码、审计脚本、
  测试锚与文档快照审查，没有编辑生产源码，也没有扩大 primary Session 的归属范围。
- 审查冻结的关键 SHA-256：`docs/plans/zircon_runtime/runtime/01-tech-stack-and-
  dependency-governance.md`=`7ef358a5a1430d8d29010ccf8b3c4fbfc7b91631ae70044c2d5183b4dc800556`；
  `zircon_runtime/src/ui/tests/text_shaper.rs`=`3516dc2f605e7aebe3647273b13e4caf9ea52239d312288b1dc3e068fd1b2ac5`；
  `zircon_runtime/src/ui/text/shaper.rs`=`8e2943365c5da7c79424b72a192f4c7686485e6d477ba58ff996094e3de62ada`；
  `tools/tests/test_runtime_tech_stack_boundary.py`=`3495366f6f394d05c5701bf007cc653297ad31ab8b737f5ebd542cbeedbea5ab`；
  `tech_stack_boundary.py`=`5438f15a9d1baf0f62fe24267bb296a38805f5fd7653e03045fe196054733ab0`；
  `tech_stack_anchor_inventory.py`=`5716e70324bc13a05f152cf684ec96b0f813ade75ff3431881e556bb9f7f7ee4`；
  `docs/crates/zircon_runtime/ui/text.md`=`372a49a01bc90787bf8ad9c61623f4466885cd38d4f292e3a64d3bce3ebc6ecd`。
- 可执行 Python 复核实际通过：聚焦 Runtime01 两项 `2/2`；完整
  `python -B -m unittest tools.tests.test_runtime_tech_stack_boundary -v` 为 `8/8`，
  均在当前工作树执行。直接调用 `tech_stack_boundary_audit` 的受限断言通过，输出
  `RUNTIME01_TECH_STACK_BOUNDARY_AUDIT_PASS`；风险、依赖边界、文本矩阵锚、行为测试
  锚、Cargo 门禁锚、版本/物理/编辑器/Kira 锚、manifest 扫描、Kira owner 违规及 ZIP
  依赖违规均为空；报告计数为 `tech_stack_guard_count=12`、
  `behavior_test_anchor_count=6`、`jolt_feature_slot_count=2`、
  `runtime_jolt_feature_slot_count=1`、`physics_jolt_dependency_feature_slot_count=1`。
- `rustfmt +1.94.1 --edition 2021 --config skip_children=true --check` 针对
  `zircon_runtime/src/ui/text/shaper.rs` 与 `zircon_runtime/src/ui/tests/text_shaper.rs`
  通过；归属范围 `git diff --check` 通过（仅保留换行格式提示）。源码审查确认
  `UiSharedTextShaper` 仍是唯一现行入口，没有恢复 `UiTextShaperStack`，且同名回归实际
  覆盖 Native/SDF layout 与 measurement parity；文档精确标题、审计常量和 Jolt/plugin
  ownership 锚均保持现行合同。
- 独立审查结论：Critical=`0`、Important=`0`、Moderate=`0`。上述结论只覆盖当前源
  静态/脚本证据；外部 `E:\Git\zr_vm` 脏工作树导致的 `validation_ticket_external_worktree_dirty`
  仍没有 Cargo ticket、job、run 或测试级结果，不能冒充 Rust 动态验收。相应 managed
  Rust gate、canonical `fixed-*`/return、closeout 与 WeCom 结果继续 pending；failure
  保持 `open`，本回执不构成关闭。

## 2026-09-26 r3 current-source reconciliation

- Successor Session `failure-roll-01a084c8-runtime01-tech-stack-text-anchors-r3` was
  registered after the archived r2 retention window. Ownership transfer fingerprint
  `16a66599d1d41c9770c2566a3bd99f7d7a660da0424d0f145058a9c498f9281b` reconciled all
  seven plan, audit-script, text-matrix, shaper, test, and failure-record paths; no
  production source was edited.
- Snapshot `3881` (baseline epoch `611`) froze the current manifest:
  `tech_stack_anchor_inventory.py`=`5716e70324bc13a05f152cf684ec96b0f813ade75ff3431881e556bb9f7f7ee4`;
  `tech_stack_boundary.py`=`5438f15a9d1baf0f62fe24267bb296a38805f5fd7653e03045fe196054733ab0`;
  Runtime01 plan=`7ef358a5b1430d8d29010ccf8b3c4fbfc7b91631ae70044c2d5183b4dc800556`;
  this failure record=`7506d933c508966b6720b2ee06e11d1c30af7090dd775933dc9c784a601b2913`;
  text matrix=`372a49a01bc90787bf8ad9c61623f4466885cd38d4f292e3a64d3bce3ebc6ecd`;
  `text_shaper.rs`=`3516dc2f605e7aebe3647273b13e4caf9ea52239d312288b1dc3e068fd1b2ac5`;
  and `shaper.rs`=`8e2943365c5da7c79424b72a192f4c7686485e6d477ba58ff996094e3de62ada`.
- On the current source, the exact focused Python anchors ran `2/2` and the complete
  `tools.tests.test_runtime_tech_stack_boundary` module ran `8/8`; both exited `0`.
  `rustfmt +1.94.1 --edition 2021 --check --config skip_children=true` over the two
  Rust paths exited `0`, and scoped `git diff --check` had no whitespace errors (only
  existing line-ending warnings). The current source still uses `UiSharedTextShaper`,
  contains the executable shared-service behavior anchor, and does not restore the
  retired `UiTextShaperStack` wrapper.
- Fresh coordinator request `runtime01-tech-stack-text-anchors-current-static-20260926-r3`
  created ticket `ffde3c3e5fcc4c23ac1011843717d5f3` with the snapshot manifest and a
  command that repeats the two focused tests. Admission recorded
  `validation_dependency_failed` through the unresolved Runtime01 → Runtime06 →
  Plugins01 and downstream failure chain; the ticket is `queued` with
  `executionKind=pending`, and no managed job, run, or terminal test result exists.
  This receipt therefore records current-source/local evidence and the durable blocker,
  not a coordinator pass.
- Existing r2 Cargo requests remain rejected by the external `E:\Git\zr_vm` dirty
  worktree. Managed Runtime01 Rust, independent C/I/M review for this r3 receipt,
  canonical `fixed-*`/return, closeout, and WeCom remain pending; this failure stays
  `open`.

## 2026-09-26 independent r3 review receipt

- Reviewer Session `review-editor03-gizmo-private` reconciled the seven current hashes,
  transfer fingerprint, local focused `2/2` and full-module `8/8` results, rustfmt and
  diff-check evidence, and queued ticket `ffde3c3e5fcc4c23ac1011843717d5f3`. The source
  evidence itself is clean and the ticket's `validation_dependency_failed` / no-job state
  is accurately recorded.
- The review found one documentation boundary issue (`Critical=0`, `Important=1`,
  `Moderate=0`): the receipt named source snapshot `3881` but did not explicitly bind the
  post-receipt manifest to snapshot `3883`. Snapshot `3883` is the authoritative
  post-receipt boundary; it carries the same six source/document hashes plus this record's
  post-receipt hash `95578217a0c79073aa28eb927b4c9ef3d0b32b74230bf4d9d642fe05611d832b`.
  This correction is documentation-only and does not change source evidence or the
  pending dependency blocker. A final post-correction review is required.

## 2026-09-26 final r3 review resolution

- The wording-only correction is sealed by post-correction snapshot `3884`, whose
  current failure-record hash is
  `bed7915e4f58efe0ad30c56de25dc0dd8b794e917eed9c39c39b4eb9ff25178a`. Snapshot `3883`
  remains the authoritative post-receipt source boundary, while `3884` records the
  explicit resolution of its documentation reference; all seven source hashes remain
  unchanged.
- The reviewer rechecked this resolution and confirmed the source manifest, local test
  evidence, queued dependency-failed ticket, and snapshot chain. Final independent
  result: `Critical=0`, `Important=0`, `Moderate=0`. No production source or test was
  edited by the review.
- Managed Runtime01 Rust, external `E:\Git\zr_vm` clean-revision admission, canonical
  `fixed-*`/return, closeout, and WeCom remain pending; this final review receipt does not
  close the failure.
