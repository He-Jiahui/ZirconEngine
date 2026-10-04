---
handoff_kind: failure
status: open
created_at: 2026-09-06
summary_slug: editor-text-export-contract
plan_link_mode: child_record_only
origin_plan: docs/plans/optimize/zircon_app/08-product-host-bootstrap-loop-dynamic-runtime-shutdown-current-source-review.md
fixing_plan: docs/plans/zircon_runtime/text/03-line-breaking-measure-and-layout.md
origin_child_dir: docs/plans/optimize/zircon_app/08
fixing_child_dir: docs/plans/zircon_runtime/text/03
related_code:
  - zircon_runtime/src/ui/text/mod.rs
  - zircon_runtime/src/ui/text/rich_text.rs
tests:
  - managed Editor target-editor-host check with dev-dynamic linking
---

# Text03: Editor text export contract

## 来源执行者

- 来源计划：`docs/plans/optimize/zircon_app/08-product-host-bootstrap-loop-dynamic-runtime-shutdown-current-source-review.md`
- 来源执行切片：App08 Editor development-DLL acceptance
- 修复责任计划：`docs/plans/zircon_runtime/text/03-line-breaking-measure-and-layout.md`
- 交接原因：the Editor check reaches the shared Runtime text surface owned by Text03.

## 失败现象与复现证据

The managed Windows target-editor-host check failed before code generation with `unresolved import crate::ui::text::parse_source_text`. Rustc reports that the item exists only behind `#[cfg(test)]` in `zircon_runtime/src/ui/text/mod.rs`; the sealed copy and working file have the same SHA-256.

Reproduce with `& .codex/skills/zircon-dev/scripts/validate-matrix.ps1 -Package zircon_editor -Features zircon_runtime/target-editor-host -NoDefaultFeatures -LibTests -CheckOnly -LinkMode dev-dynamic`. Evidence: `.codex/tmp/app08-editor-dynamic-20260906-r2.log`. The independent Runtime product build with `-Package zircon_runtime -Features target-client -NoDefaultFeatures -RuntimeProductDll -SkipTest -LinkMode static` reproduced the same parser-export error in `.codex/tmp/app08-product-client-20260906-r3.log` and exited 101.

## 最低共享层根因

The Runtime text module exports a production caller's parser only in test configuration. This is a shared text boundary defect, not an Editor test problem.

## 架构修复验收

- Restore the production text export with the Text03 ownership and visibility rules.
- Run the focused text layout/parser contract checks.
- Rerun the exact managed Editor target-editor-host check and then its focused test.

## 禁止临时方案

- Do not add an Editor-local parser or cfg-specific duplicate.
- Do not skip the Runtime text module from the Editor closure.

## 修复结果与回传

Open state: `待修复`; no Editor pass or DLL-reuse claim is made. The production
parser definition and re-export are no longer gated by `cfg(test)`. Current
target-client test-profile job `9e45fc19097d4bfb8267d1647e40cc2a` no longer reports
the parser import error, but its 503 remaining source errors prevent test
execution and upward Editor/product validation. Log:
`.codex/tmp/app08-taffy-client-20260906-r2.log`. The ownership record stays open
until the original commands and focused parser/layout tests pass.

## Additional Consumer Evidence

Designment02 reproduced the production `parse_source_text` import error in `ui/surface/render/inline_widgets.rs:8` on 2026-09-06, managed job `e3513aa0f91b4d4dafc39bf68f58bbff`. The `zui_native_visual_acceptance` Runtime screenshot target did not execute. Log: `docs/_data/layout/evidence/windows-runtime-validation.log`. The same existing failure also blocks native ZUI visual acceptance; no alternate parser or font/layout fallback is accepted.
