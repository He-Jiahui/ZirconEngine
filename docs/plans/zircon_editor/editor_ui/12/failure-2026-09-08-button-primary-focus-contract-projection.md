---
handoff_kind: failure
status: open
failure_scope: cross_plan
plan_link_mode: child_record_only
created_at: 2026-09-08
summary_slug: button-primary-focus-contract-projection
origin_plan: docs/plans/optimize/zircon_runtime_interface/03-ui-authoring-accessibility-input-diagnostic-status-public-contract-review.md
fixing_plan: docs/plans/zircon_editor/editor_ui/12-unreal-magicavoxel-zui-design-convergence.md
origin_child_dir: docs/plans/optimize/zircon_runtime_interface/03
fixing_child_dir: docs/plans/zircon_editor/editor_ui/12
related_code:
  - zircon_runtime_interface/src/tests/ui_painter_style_contracts.rs
tests:
  - managed Windows static/no-default/locked zircon_runtime_interface --lib tests::ui_painter_style_contracts
---

# EditorUI12: button primary-state assertion calls the scalar focus projection

## 来源执行者

- 来源计划：`docs/plans/optimize/zircon_runtime_interface/03-ui-authoring-accessibility-input-diagnostic-status-public-contract-review.md`
- 来源执行切片：RuntimeInterface current-source library contract regression.
- 修复责任计划：`docs/plans/zircon_editor/editor_ui/12-unreal-magicavoxel-zui-design-convergence.md`
- 交接原因：EditorUI12 owns the composite painter focus contract and this regression.

## 失败现象与复现证据

The immutable `interface-project-identity-3170-20260908` library run executed
754 passing tests and 11 failing tests. Its `interface-library-3170.log` records
`tests::ui_painter_style_contracts::button_interaction_keeps_legacy_focus_without_changing_primary_state`
failing at line 204: actual `Focused`, expected `Normal`.
The current test matches the failing input, SHA-256
`f5f18380391944d976f3a46022dfa163349aa2abb64f45842ed73c9cd251f39c`,
preserved in preimage snapshot 3227. The other ten failures remain separate.

## 最低共享层根因

The test requests keyboard-visible focus, then reads
`resolved_state_for_family`, the existing scalar projection. That API returns
`Focused` while the composite `visual_state_for_family` preserves primary
`Normal` and sets `focus_visible`. Both behaviors are already implemented in
`zircon_runtime_interface/src/ui/style.rs` and asserted by adjacent tests.
The faulty assertion checks composite primary identity through the scalar API.

## 架构修复验收

- Assert `visual_state_for_family(Button).primary == Normal` and independently
  assert its visible focus overlay for the keyboard-focused state.
- Preserve explicit scalar `Focused`, button interaction `Focused`, disabled
  precedence and drop-target precedence assertions.
- Execute the complete `tests::ui_painter_style_contracts` batch on the exact
  frozen source, including every painter family and token composition consumer.
- Require formal validation, independent C0/I0/M0 review, canonical return and
  coordinator closeout before acceptance.

## 禁止临时方案

- Do not change production state selection or add another compatibility API.
- Do not remove focus, disabled or drop-target assertions, or skip the test.
- Do not treat other library failures or product UI gates as passed.
- Do not modify or revalidate external zr_vm.

## 修复结果与回传

Open state: `test-contract-repaired_target-batch-passed_independent-review-zero_formal-binding-pending`.
The source edit selects the composite primary projection and explicitly keeps
the visible overlay and existing scalar focus expectations. No production
source or UI asset was changed.
Audited two-path transfer fingerprint:
`4f2303eca3e5d2423818e8ca9dba2a6a3312a41a6b93b8a57b2a5cc76038d54d`.
Fixing Session `failure-roll-01a07160-editor-ui12` handles this newly discovered
failure after the former `failure-roll-01a07160-editorui12` was automatically
archived. Existing toolbar-density ticket `bc6e869c1cc9416eb80df30ce49a58b0`
retains its original Session and request; this item does not resubmit it.

Source snapshot 3228 has SHA-256
`b4420328d6a8baf2dcb40eecd7f6b7b26dff390c4ddc9c762e6923569e2ead95`.
Frozen input `editorui12-button-focus-3228-20260908` overlays only that test
source onto the immutable Interface02 input; manifest
`e1bb2d3955a335f2f0ba3462155d9dd5c9d6db866d535c760ac1ebd77f5401e6`.
Managed Windows static/no-default/locked job
`d2c982b0f7c14c88a242973d583a17e2` ran
`tests::ui_painter_style_contracts`: 7 passed, 0 failed, 0 ignored, 860 filtered
out. This includes the original button regression, all-family state priority,
focus overlay, scalar consumers, serialization, drop precedence and token
composition. Receipt and log are under the input's results directory as
`editorui12-button-focus-contracts-3228.json` and `.log`.
The operational job is not a formal fixing ticket. Existing external pinning
and reviewer lifecycle restrictions remain; formal binding and canonical
return/closeout are still pending.

Independent review in the existing "优化协调器验证效率" task
`01a07063-6f03-7803-a12d-13ea015ca645` completed at
`2026-09-08T14:01:36Z`: Critical 0, Important 0, Moderate 0 for source snapshot
3228 and record snapshot 3232. The reviewer verified the current bytes,
ObjectStore hashes, exact preimage diff, all seven executed tests, unchanged
production behavior, and absence of reviewed-path ownership conflicts.
The retained report is
`.codex/tmp/editorui12-button-focus-3228-review-20260908-result.txt`.
This is independent source/evidence review, not an accepted formal closeout
review: the existing reviewer Session lifecycle remains archived. No return,
closeout commit or WeCom success is claimed for this item.
