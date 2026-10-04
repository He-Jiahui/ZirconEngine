---
title: Editor workbench floating window test contract feature boundary protected plan routing
date: 2026-08-22
status: routing_requested
owner_record: 2026-08-22-editor-ui-workbench-floating-window-test-contract-feature-boundary-review.md
---

# Protected plan updates

## Performance ledgers

Keep one concise `pending.md` entry:

`zircon_editor/src/ui/workbench/floating_window.rs` - 1/1 Rust file source-reviewed; symbols are
test-only but the default editor library still compiles/re-exports the parallel serde design
contract; feature gate and managed default/unit/integration artifact acceptance remain pending.

Do not add the file to `review.md` before M0-M3 pass.

## `docs/plans/performance/01-mvp-performance-audit-and-optimization.md`

Attach this to `PERF-MVP-136` with the workbench fixture feature-boundary work. Keep it separate from
real floating-window runtime hotspots such as `PERF-MVP-105/106/130/173/602`.

## `docs/plans/performance/02-unreal-aligned-engine-system-hard-cutover.md`

Own deletion of default-product test schemas and their root re-exports. Do not retain an alias or
compatibility facade after the feature boundary lands.

## `docs/plans/zircon_editor/editor_layout/07-windowing-chrome-tabs-and-dockable-drawers.md`

Real floating-window identity, placement, modal/native behavior and topology remain owned here and
by the retained/native host implementation, not by the gated parity fixture.

## `docs/plans/zircon_editor/editor_ui/09-editor-modules-and-design-parity.md`

Keep `.zui` design parity coverage in the integration-contract feature while removing the parallel
schema from default product compilation.

## Acceptance handoff

Require the 1/1 fingerprint, static module/re-export gate, managed default lib/unit/integration
matrix on D/E/F, proof of zero default exports, artifact/compile size delta, milestone commit and
quantified WeCom notification. Shared ledgers remain protected until then.
