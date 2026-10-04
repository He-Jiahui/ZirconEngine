---
handoff_kind: failure
status: open
failure_scope: cross_plan
plan_link_mode: child_record_only
created_at: 2026-09-06
summary_slug: toolbar-popup-offset-density-token-drift
origin_plan: docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
fixing_plan: docs/plans/zircon_editor/editor_ui/12-unreal-magicavoxel-zui-design-convergence.md
origin_child_dir: docs/plans/zircon_editor/editor_ui/08
fixing_child_dir: docs/plans/zircon_editor/editor_ui/12
related_code:
  - zircon_editor/assets/ui/editor/theme/editor_tokens.zui
  - zircon_editor/assets/ui/editor/components/workbench/shell/workbench_top_toolbar.zui
  - zircon_editor/assets/ui/editor/windows/workbench_window.zui
  - tools/tests/test_editor_zui_live_state_authority_contract.py
tests:
  - python -B -m unittest tools.tests.test_editor_zui_live_state_authority_contract.EditorZuiLiveStateAuthorityContractTests.test_toolbar_popup_offsets_follow_the_shared_two_row_chrome_metrics -v
  - python -B -m unittest tools.tests.test_editor_zui_live_state_authority_contract -v
---

# Toolbar popup offsets retain the previous control density

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md`
- 来源执行切片：UI08 workbench test-budget migration and full live-state regression
- 修复责任计划：`docs/plans/zircon_editor/editor_ui/12-unreal-magicavoxel-zui-design-convergence.md`
- 交接原因：The unchanged regression exposed shared theme density offsets owned by EditorUI12.

EditorUI08 discovered this independent shared-token failure while validating
`editorui10-test-budget-workbench-retained-host`. The complete local live-state
suite executed 31 tests: 30 passed and the unchanged popup metric assertion
failed. The source-only toolbar migration does not own theme geometry.
EditorUI12 owns the shared workbench theme and its density convergence.

Stable fixing Session: `failure-roll-01a07160-editorui12`.
Origin record: [EditorUI08 test budget](../08/failure-2026-08-13-editorui10-test-budget-workbench-retained-host.md).

## 失败现象与复现证据

The focused command in frontmatter was executed again before editing. It ran
one test, exited `1`, and reported `AssertionError: 29.0 != 30.0` at line 931.
The exact pre-edit token SHA-256 was
`6d32f681c9828a8db28416a133fc33578b002972de3e2a2e216fb099adb1674f`.
Other current inputs observed during diagnosis:

- Top toolbar: `c20fa48bf745ced8695159d24a58759cc5b912fab91ccfb30b4abe8b4cbaec28`.
- Workbench window: `f55105f3d2d7c2f85d2e12c5a1ad1cc043a4fd7e24f2d97e14b7e756e4190819`.
- Python guard: `f9f04049a299e37fa4fd4b956b461fa4480158a3cd1470dad08a59266776c5b1`.

The guard differs from baseline only in the unrelated UI08 toolbar test-owner
path. Its popup metric test body and TOML loader are unchanged.

## 最低共享层根因

The current theme increased `controls.compact_height` from 30 to 32 logical
pixels while keeping popup offsets derived from the old control height. The
toolbar is 66px high, its command row is 34px, its module row is 28px and both
row and popup placement gaps are 4px. The shared offsets must therefore be:

- Command row: `66 - (34 + 32) / 2 - 4 = 29`.
- Module row: `66 - (34 + 4 + (28 + 32) / 2) - 4 = -6`.

The correction changes only these two shared values. Existing token identity,
layout policy and four menu consumers remain intact. The existing assertion
continues to derive the expected values from the shared metrics.

The complete current token file was transferred from archived Session
`editor-ui12-m2-scope-hold-v2-20260812` using fingerprint
`fffff71f3fef167a3b5857c3de76806599089edb75757ad20890313d5d05f3d7`
and apply request `b16dbbf7be4c4ea88b4a909f8a5a40c8`. Exact lease request
`54c566a0ef734daa986245e34045fc15` had no conflicts, and its current hash was
checked immediately before editing. Existing theme changes are preserved.

## 架构修复验收

1. The unchanged focused metric regression must execute and pass on a
   coordinator-managed immutable snapshot of the current token file.
2. The complete current-worktree live-state suite must execute all 31 tests;
   any other owner failure must remain a separate handoff.
3. Current-source native workbench popup behavior and physical layout must be
   validated before treating the product UI repair as accepted. Rust commands
   retain `--locked`; the user-requested `zr_vm` skip remains in effect.
4. Canonical return, independent review and coordinator closeout remain
   pending until their required evidence is complete.

## 禁止临时方案

- Do not weaken the metric assertion or restore the previous control height.
- Do not duplicate offset values in individual menu consumers.
- Do not treat queued validation or local source checks as native UI acceptance.
- Keep the original request identity while its dependencies are unresolved.

## 修复结果与回传

After correction, the complete live-state suite executed all 31 tests in 1.070
seconds and passed, exit `0`. This includes the unchanged original popup metric
regression and all direct source/asset consumers. The token SHA-256 is now
`2330c221811fb3619b51f5aa97b06405fe5ea878df48b78319c79dc3388c4ec2`.
Scoped whitespace checks passed. The full current token diff includes the
archived owner's 20 earlier replacements plus the two offset replacements;
the earlier density, typography and radius work is preserved.

Managed focused verification uses the immutable baseline Python guard, whose
metric test and TOML loader match the current guard, with only this Session's
current token file overlaid. That test reads no other changed source. The
broader current-worktree 31-test result is recorded separately and is not
represented as a managed full-worktree validation result.

Snapshot `2808` freezes the token correction and local evidence. Managed request
`failure-roll-01a07160-editorui12-popup-density-20260906-r1` returned ticket
`bc6e869c1cc9416eb80df30ce49a58b0`, status `queued`, with
`validation_dependency_failed` blockers from the plan dependency graph,
including `workbench-drawer-header-token-drift`,
`build-editor-product-staging-unregistered` and
`text-retained-layout-split-imports`. No test has run for this ticket. The
existing request is retained without duplicate submission or status polling.

State: `resolving_failure`; local source contracts pass, managed verification
is dependency-blocked and native UI verification remains pending.
No fixed return, integration, commit or WeCom notification is claimed.
