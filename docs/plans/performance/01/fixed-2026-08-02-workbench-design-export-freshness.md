---
handoff_kind: fixed
status: fixed
created_at: 2026-08-02
summary_slug: workbench-design-export-freshness
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_editor/editor_layout/06-floating-windows-and-design-parity.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_editor/editor_layout/06
related_code:
  - tools/editor-workbench-preview/design.js
  - tools/editor-workbench-preview/design-manifest.mjs
  - tools/editor-workbench-preview/verify-designs.mjs
  - tools/editor-workbench-preview/export-designs.mjs
  - tools/editor-workbench-preview/export-evidence.mjs
  - tools/editor-workbench-preview/package.json
  - tools/editor-workbench-preview/package-lock.json
  - docs/ui-and-layout/editor-workbench-design-export.md
  - docs/ui-and-layout/editor-workbench-designs
tests:
  - npm --prefix tools/editor-workbench-preview run design:verify
  - npm --prefix tools/editor-workbench-preview run design:verify:reference-negative
resolved_at: 2026-08-02
---


# Editor Layout 06: Workbench design exports fail freshness validation

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行切片：2026-08-02 `docs/plans` / current-source review and obsolete-test cleanup
- 修复责任计划：`docs/plans/zircon_editor/editor_layout/06-floating-windows-and-design-parity.md`
- 交接原因：Editor Layout 06 owns design-parity acceptance and the exported Workbench visual corpus; Performance 01 can reproduce the stale evidence but must not regenerate or redefine that visual baseline.

## 失败现象与复现证据

`npm --prefix tools/editor-workbench-preview run design:verify` reaches the repository-owned verifier and fails all 271 expected PNG outputs: `preview-sheet.png` plus every manifest design is older than `tools/editor-workbench-preview/design.js`. The current timestamps are 2026-06-27 22:46:42 UTC for `design.js` and 2026-05-30 02:24:48 UTC for `preview-sheet.png`; the remaining outputs report the same freshness class.

The same review removed an unrelated dependency on the untracked personal file `.codex/plans/Editor Workbench PNG Design Plan.md`. `node --check tools/editor-workbench-preview/verify-designs.mjs` passes and the verifier now reads only repository-owned documentation; the remaining failure is the real export freshness gate.

## 最低共享层根因

The checked-in visual corpus has not been reconciled with the current renderer source. The gate currently proves this only through filesystem modification times, which are not a durable content identity across Git checkouts. The fixing owner must first determine whether `design.js` changed rendered output; then either regenerate and visually review the corpus or replace timestamp-only freshness with a deterministic source fingerprint recorded by the exporter without weakening pixel/content validation.

## 架构修复验收

- Bind every exported PNG and `preview-sheet.png` to the exact renderer/manifest/style inputs through exporter-owned deterministic evidence.
- If renderer output changed, regenerate all affected images and visually inspect representative shell, drawer, workflow, compact, world-streaming and LiveOps groups.
- Rerun `npm --prefix tools/editor-workbench-preview run design:verify` from a clean checkout and require zero freshness, documentation, manifest, pixel-profile or stale-process failures.
- Keep Editor Layout 06 open until the current visual corpus and its reproducible evidence agree.

## 禁止临时方案

- Do not touch file timestamps, skip freshness checks, delete expected designs, or lower the 270-design / 271-PNG contract to make the gate green.
- Do not restore a dependency on `.codex/plans` or another untracked personal file.
- Do not bulk-regenerate and accept binary assets without visual inspection and deterministic source binding.

## 修复结果与回传

- 根因：Timestamp-only freshness marked all 271 outputs stale after five visible .ui.toml-to-.zui string changes; actual renderer drift was limited to four dependent PNGs, and raw text hashes were checkout-line-ending-sensitive.
- 架构修复：Pinned Playwright 1.62.1, reused one Edge browser, added LF-canonical source plus exact PNG SHA-256 EXPORT-EVIDENCE lock written only by complete exports, and regenerated the four affected PNGs.
- 验证：design:export exit 0; design:verify verified 271 PNGs in both the shared checkout and a detached fresh npm ci checkout; reference-negative guard exit 0; representative shell/drawer/workflow/compact/world-streaming/LiveOps and four changed-output visual QA passed; Git reports exactly four PNG deltas.
- 回传：Performance 01 may resume its review gate; Editor Layout 06 now owns a source-bound, clean-checkout-verifiable Workbench design corpus.
