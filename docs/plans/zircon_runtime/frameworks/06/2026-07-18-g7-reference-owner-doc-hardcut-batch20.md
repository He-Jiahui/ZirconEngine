---
related_code:
  - dev/bevy/crates/bevy_text/src/text_edit.rs
  - dev/bevy/crates/bevy_text/src/editing.rs
  - dev/material-ui/packages/mui-system/src/createTheme/createTheme.js
  - dev/material-ui/packages/mui-system/src/createTheme/shape.ts
  - tools/check_conventions.py
implementation_files:
  - docs/ui-and-layout/bevy-ui-text-widgets-focus-a11y-m0-gap-audit.md
  - docs/ui-and-layout/material-ui-token-component-audit.md
plan_sources:
  - docs/plans/zircon_runtime/frameworks/06-development-conventions-and-guardrails.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
tests:
  - python tools/check_conventions.py --only docs --json
  - git diff --check -- docs/ui-and-layout/bevy-ui-text-widgets-focus-a11y-m0-gap-audit.md docs/ui-and-layout/material-ui-token-component-audit.md
---

# Frameworks06 G7 Reference Owner 文档硬切 Batch 20

Status: focused_g7_passed_global_g7_red_review_passed
Date: 2026-07-18
Session: `frameworks06-g7-reference-owner-doc-hardcut-batch20-20260718`

## 完成项目

- 将 Bevy M0 gap audit 中已删除的 `bevy_text/src/text_editable.rs` 硬切到 current `editing.rs`，保留 `text_edit.rs` actions owner，并同步正文责任拆分。
- 将 Material UI token audit 中已删除的 mui-system `createTheme/shape.js` 硬切到 current `shape.ts`，记录 `Shape` / `ShapeOptions` 与默认 `borderRadius` owner。
- 保留原 audit baseline/设计结论作为历史取样，但 current machine paths 只指向当前检出的 reference tree；不创建旧 reference 文件、alias、shim 或兼容路径。

## Fresh Testing Evidence

- 修改前 fresh G7：两份所选文档合计 `2` 个 missing-path violations。
- 修改后 fresh `python tools/check_conventions.py --only docs --json`：两份所选文档合计 `0` violations；共享 current-source 全局快照为 `468` violations / `121` documents，G7 继续保持 RED。
- Bevy/MUI current reference owner 机器路径全部存在；两条退役 reference front-matter 路径均为 `0`。
- exact-scope `git diff --check` 通过，staged_total 为 `0`。

## Review

独立只读复审为 **Critical 0 / Important 0 / Minor 0**：Bevy `EditableText`/filter/limits/apply/change trigger 与 `TextEdit` actions 的 owner 拆分、MUI `Shape`/`ShapeOptions`/default `borderRadius` 及 `createTheme.js` 消费关系均与当前 reference source 一致；历史 baseline 与 current refresh 未混淆。

## 里程碑判定

本批只关闭所选 reference current-owner 文档漂移。Frameworks06 M1、全局 G7 和计划 06 均保持 `in_progress`；本批不构成独立里程碑提交。
