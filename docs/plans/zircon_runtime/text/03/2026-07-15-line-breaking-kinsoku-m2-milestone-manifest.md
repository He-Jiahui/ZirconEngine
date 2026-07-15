Plan: docs/plans/zircon_runtime/text/03-line-breaking-measure-and-layout.md
Milestone: M2
Status: completed
Files: ["docs/plans/zircon_runtime/text/03/2026-07-15-line-breaking-kinsoku-m2-milestone-manifest.md"]

# Text03 M2 line-breaking and kinsoku workflow anchor

## Scope Delivered

This machine-readable anchor records the already accepted Text03 M2 foundation: resolved physical lines and visual runs own wrapping, Unicode line-break and CJK kinsoku outcomes before source geometry is consumed. Detailed historical implementation and framebuffer evidence remain in `2026-07-09-line-breaking-measure-and-layout-output-records.md`; this file does not introduce or reimplement M2 behavior.

## Fresh Testing Evidence

The current M3 source-map tests exercise post-line-break resolved runs, mixed UAX#9 visual order and conservative non-isomorphic replacement mapping without reconstructing wrapping in hit testing or paint.

## Review

The current independent review accepted the hard-cut consumer architecture and found no UI-layer line-breaking fallback, alias or compatibility shim. This predecessor node owns only its workflow anchor; the M3 implementation remains isolated for its own architecture commit.
