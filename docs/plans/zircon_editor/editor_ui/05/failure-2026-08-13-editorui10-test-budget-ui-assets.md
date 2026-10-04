---
handoff_kind: failure
status: open
failure_scope: cross_plan
plan_link_mode: child_record_only
created_at: 2026-08-13
summary_slug: editorui10-test-budget-ui-assets
origin_plan: docs/plans/zircon_editor/editor_ui/10-code-structure-and-module-conventions.md
fixing_plan: docs/plans/zircon_editor/editor_ui/05-ui-asset-management.md
origin_child_dir: docs/plans/zircon_editor/editor_ui/10
fixing_child_dir: docs/plans/zircon_editor/editor_ui/05
related_code:
  - zircon_editor/src/tests/ui/assets_activity/bootstrap_assets
  - zircon_editor/src/tests/ui/boundary/template_assets
  - tools/tests/test_zui_current_layout_wording_convergence.py
tests:
  - python -B .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_editor_structure.py --json --repo-root E:\\Git\\ZirconEngine
  - cargo test -p zircon_editor --lib ui --locked
---

# EditorUI05: UI asset management test owners exceed the 800-line budget

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor_ui/10-code-structure-and-module-conventions.md`
- 来源执行切片：M3.T1 test-file budget gate
- 修复责任计划：`docs/plans/zircon_editor/editor_ui/05-ui-asset-management.md`
- 交接原因：assets activity/bootstrap 和 template assets 是 EditorUI05 UI asset management 责任，不属于 asset-browser、reference 或 MUI component 计划。

## 失败现象与复现证据

准确结构审计在 0 个 test-budget exemption 下报告
`tests/ui/assets_activity/bootstrap_assets.rs` 为 821 行，`tests/ui/boundary/template_assets.rs` 为 1560 行。
它们是 UI asset bootstrap/template management 行为 owner，未拆分使全局 structure gate 继续 RED。

## 最低共享层根因

asset bootstrap/activity 和 template asset contract 场景累积在 flat 测试文件，未将 bootstrap、asset lifecycle、
template fixture 和 contract behavior 建立为 folder-backed 单一 owner。

## 架构修复验收

- 将两项按 UI asset management 行为迁移到 folder-backed tests，薄 `mod.rs` 挂载，测试文件不超过 800 行。
- 保留 bootstrap、activity、template asset 与 fixture/assertion 语义；共享 helper 唯一归属。
- 不得留下 flat compatibility test、`#[path]` mount、duplicate tree 或 budget exemption。
- 重审计不再报告这两项；全局 owner 清零后受管 structure gate 才可 GREEN。

## 禁止临时方案

- 不得提高预算、删除 UI asset 覆盖，或把 asset-browser/reference/MUI 责任混入此切片。

## 修复结果与回传

Open state: `待修复`。本 handoff 只转移拆分责任，未修改 UI asset 测试。

## 产出记录与时间

| 时间 | 里程碑/切片 | 状态 | 完成项目与证据 | 后续门禁 |
| --- | --- | --- | --- | --- |
| 2026-08-13 | M3 UI asset management test-budget handoff | `open` | 从准确 48/0 审计隔离 821 行 bootstrap 与 1560 行 template asset owner。 | 取得源码 lease 后按 asset management 行为 folder-backed 拆分，受管 UI 回归和结构审计复验。 |

## 2026-09-06 current-source folder-backed migration

Session `failure-roll-01a07160-editorui05` acquired the exact source/document lease
`4e7926cfcb094843ab0f9e4931ea12f8` before splitting the two remaining flat owners.
The pre-edit SHA-256 values were
`7b2d74b0ed1cf3f154a99881c569cefe1d03c31f17346639c86f65e9f71f0ba5`
for bootstrap assets and
`cd979ec99d31f05899621d595c68022b99b48b1f1a064763389d5f41ea81634c`
for template assets. The latter includes the prior EditorUI06 source-path update.

- Bootstrap asset declarations, shell projection, responsive layout and content
  geometry now live under `bootstrap_assets/`, with a structural `mod.rs`.
- Template asset inventory, host shells, component contracts, invalidation,
  runtime fixtures and retained projection now live under `template_assets/`.
  Existing `product_binding_fixture.rs` remains mounted and unchanged. File
  traversal helpers have one `support.rs` owner.
- All 34 tests from the two old files are retained. A before/after body comparison
  found no changes beyond Rustfmt whitespace and an optional trailing call comma.
  The largest new file is 623 lines; both old flat files are removed, without a
  compatibility mount or test-budget exemption.
- The Python wording guard now reads the current folder for both positive wording
  and required negative suffix assertions, including an explicit nonempty source
  check.

Local verification: the wording, infrastructure and static-suffix suites passed
31/31; recursive Rustfmt and scoped diff checks passed. The structure audit reports
20 oversized test owners, down from 22, zero exemptions, zero banned module names
and zero UI owner-boundary violations. It no longer reports either migrated owner.

The original managed `cargo test -p zircon_editor --lib ui --locked` acceptance
remains pending. The user requested skipping `E:\Git\zr_vm`; no duplicate Cargo
request is made against that known external admission blocker. This is source and
local static evidence only, with no fixed return, commit or WeCom claim.

### Managed static validation receipt

The exact leaf-file snapshot is `2797`; snapshot 2796 contained directory names
and is superseded for this migration. After explicit leaf attribution, request
`failure-roll-01a07160-editorui05-wording-20260906-r2` was accepted as ticket
`5748c48bead6410083e1dc20f8ec4603` with status `queued` and no admission blockers.
Its command is `python -B -m unittest tools.tests.test_zui_current_layout_wording_convergence -v`.
The ticket retains the two deleted flat-file tombstones and the new source hashes.
After independent documentation work, the result was received once: the ticket
is `passed`, exit `0`, with both targeted tests executed (`2/2`, 0.396 seconds).
The source manifest remains bound to snapshot 2797. This accepts the managed
wording guard only; the UI Cargo gate remains pending.

The two migrated owners also had direct references in 19 functional documents.
Those references now use the current directories or specific leaf modules; dated
historical commands retain their original evidence. All 34 migrated structured
references resolve, and scoped `git diff --check` passes. A wider path audit of
the same documents still reports other pre-existing retired owners, so no global
documentation gate pass is claimed. These exact documentation edits are attributed
to this Session and are included in its next source/document snapshot.

A subsequent direct-reference pass also updated the same functional document set
for the EditorUI06 `global_material_surface_assets/` split and the EditorUI08
`surface_hit_test/template_node/tests/` split. Their source ownership remains with
their respective Sessions. Across this document set and the six EditorUI08
owner/functional documents, all 50 migration-related structured references resolve.
The combined scoped diff check passes; unrelated historical path debt remains
outside that result. Document closeout must retain the corresponding UI06/UI08
source dependency snapshots.
