---
handoff_kind: failure
status: open
created_at: 2026-09-07
summary_slug: paragraph-list-prefix-range-contract
plan_link_mode: child_record_only
origin_plan: docs/plans/zircon_runtime/frameworks/01-runtime-crate-decomposition.md
fixing_plan: docs/plans/zircon_runtime/text/03-line-breaking-measure-and-layout.md
origin_child_dir: docs/plans/zircon_runtime/frameworks/01
fixing_child_dir: docs/plans/zircon_runtime/text/03
related_code:
  - zircon_runtime/src/ui/text/layout_engine/paragraph_layout.rs
  - zircon_runtime/src/text/layout/rich_source.rs
tests:
  - managed Runtime target-client production-library check
  - managed Runtime paragraph layout and rich source-range regressions
  - managed Editor target-editor-host check with dev-dynamic linking
---

# Text03: Paragraph List Prefix Range Contract

## 来源执行者

- Origin Session: `failure-roll-01a07160-frameworks01`.
- 来源计划：`docs/plans/zircon_runtime/frameworks/01-runtime-crate-decomposition.md`
- 来源执行切片：current-source Runtime/Editor compilation for the
  [Frameworks01 import failure](../../frameworks/01/failure-2026-09-06-editor-runtime-import-contract.md).
- 修复责任计划：`docs/plans/zircon_runtime/text/03-line-breaking-measure-and-layout.md`
- 交接原因：Text03 owns paragraph prefix measurement and its shared source-range contract.
- Current source owner: `text03-current-proof-r1-bee4c707-20260822`, active.

## 失败现象与复现证据

Managed production-library job `20cb9378ea4f4d8eae2ff4b35464f95a` reported E0308
at `paragraph_layout.rs:486`: `checked_source_range(text, range)` expects
`(u32, u32)`, but `range` is `UiTextRange`. Original log:
`.codex/tmp/app08-runtime-client-lib-20260906.log`, lines 4253-4270. Original
sealed digest: `349d73696525d60d95a58f36373016c13113039aca2253e4b05fa9ce2e54ea4e`.

Current source independently retains this mismatch: `ResolvedParagraphLayoutOverride`
stores `list_prefix: Option<UiTextRange>` and forwards it unchanged to the compact
tuple-range helper in `text/layout/rich_source.rs:78`. The current paragraph file
SHA256 is `43688955fff00fe5f6a6fd525f80ec666c4b8027ceecc2a76b6383b469c5db19`.
Transfer preview `d0318972539548a19ebf7e182a0f81b8` returned
`source_owner_executable`, so Frameworks01 did not claim or modify this source.

Original managed commands to repeat after the source repair and admission recovery:

```powershell
& .codex/skills/zircon-dev/scripts/validate-matrix.ps1 -Package zircon_runtime -Features target-client -NoDefaultFeatures -RuntimeProductDll -SkipTest -LinkMode static
& .codex/skills/zircon-dev/scripts/validate-matrix.ps1 -Package zircon_editor -Features zircon_runtime/target-editor-host -NoDefaultFeatures -LibTests -CheckOnly -LinkMode dev-dynamic
```

## 最低共享层根因

The resolved UI paragraph representation uses native-size source ranges, while
the compact rich-layout helper accepts u32 tuples. The paragraph measurement
owner has not completed that boundary conversion. This is independent of the
existing Text03 `editor-text-export-contract` parser-export failure.

## 架构修复验收

- Align the paragraph prefix consumer with its authoritative range representation.
  Preserve ordering, bounds, UTF-8 boundaries, and typed layout failure.
- Cover valid multibyte list prefixes, empty ranges, reversed ranges, and ranges
  outside the source. Any compact conversion must reject overflow explicitly.
- Run the focused shared source-range and paragraph-layout Rust regressions, then
  the original Runtime and Editor commands through managed Windows validation.
- Return evidence to Frameworks01 before its compilation gate can be accepted.

## 禁止临时方案

- Do not use unchecked narrowing casts, clamp malformed ranges into valid text,
  replace them with empty prefixes, or add a duplicate parser/range authority.
- Do not bypass the active owner, skip paragraph compilation, or claim static
  source inspection as dynamic acceptance.

## 修复结果与回传

Open state: source mismatch confirmed; owner repair and managed acceptance pending.
The separately recorded external `zr_vm` admission prerequisite is excluded by
the user and has not been modified. Frameworks01 continues independent repairs.
