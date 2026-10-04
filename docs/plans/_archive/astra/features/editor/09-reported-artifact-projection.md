---
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/269-editor-build-export-preset-pipeline-cook-pack-platform-bundle-publishing-resume-determinism-current-working-tree-review.md
---

# Reported artifact projection

## Current source and repair

The artifact panel merged planned paths with reported artifacts, while stage rows
and the pipeline report section substituted planned report paths when execution
had reported none. A fresh, failed, or cancelled export could therefore display
an output/report that had never been produced.

Keep planned artifacts in their existing explicit plan field. Artifact entries,
stage report paths, and the report body now consume only execution-reported
paths. Removing the merge also removes its temporary artifact clone and pairwise
duplicate scan. UI projection does not perform synchronous filesystem probes.

This repairs the planned/reported conflation in EXPORT-GATE-31. Existing stdout
announcements are not verified publication receipts: typed artifact verification
and report consumption in EXPORT-GATE-11/12/32 remain pending, so this record
does not claim complete artifact authenticity or release performance acceptance.

## Plan completion list

| Batch | Work | Status | Validation evidence |
|---|---|---|---|
| M23 | Remove planned artifact/report fallbacks from execution result projection | implemented_pending_validation | Two regressions cover all six statuses and announced-path replacement, including stdout-only JSON without a reported artifact. Focused import/export contracts `21/21`, the combined Runtime02/08/19 + Editor09 source batch `39/39`, and scoped diff checks pass. Combined coordinator batch `1969ede38024473f9e960fcb1a55a9e9` was queued without polling; managed Editor Cargo and release performance evidence remain pending. |

Regression scope: pending, running, failed, cancelled, and finished jobs without
reported output; announced report paths and path replacement; preserve explicit
planned-artifact information. Existing success fixtures that announce no report
must assert absence instead of relying on the removed fallback.

## Local completion record

The remaining stdout/report conflation is closed locally: report-body summary
entries are parsed only when the Report execution has announced a `report` or
`pipeline_report` artifact. A stdout-only JSON regression now proves that fresh
or terminal jobs cannot display an unmaterialized report body. Rustfmt and
scoped diff checks pass; Cargo and managed release evidence remain deferred to
the parent batch.
