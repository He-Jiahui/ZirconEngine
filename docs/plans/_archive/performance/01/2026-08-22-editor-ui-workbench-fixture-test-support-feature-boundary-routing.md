---
title: Editor workbench fixture test-support feature boundary protected plan routing
date: 2026-08-22
status: routing_requested
owner_record: 2026-08-22-editor-ui-workbench-fixture-test-support-feature-boundary-review.md
---

# Protected plan updates

## Performance ledgers

Keep one concise `pending.md` entry:

`zircon_editor/src/ui/workbench/fixture` - 14/14 Rust files source-reviewed; calls are test-only, but
the default product library still compiles the public fixture module and embeds 8,985 JSON bytes;
feature-gate, managed default/unit/integration matrix and artifact-size acceptance remain pending.

Do not add the folder to `review.md` before M0-M3 pass.

## `docs/plans/performance/01-mvp-performance-audit-and-optimization.md`

Attach the default-product exclusion and test compile/artifact measurements to `PERF-MVP-136` as a
test-support ownership subtask. Keep it P2: it reduces default build/code-size debt but is not a
product frame-time optimization.

## `docs/plans/performance/02-unreal-aligned-engine-system-hard-cutover.md`

Own the hard boundary that prevents fixture/test-support modules and embedded preview JSON from
remaining in default product compilation. Do not add a runtime re-export or compatibility facade.

## `docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md`

Keep preview fixture ownership separate from product workbench state and runtime UI assets. Product
layout/descriptors must not import the test fixture as a fallback.

## `docs/plans/zircon_editor/editor_ui/09-editor-modules-and-design-parity.md`

Preserve visual/screenshot fixture coverage through the explicit integration-contracts/test-support
configuration while product editor modules use real data owners.

## Acceptance handoff

The owner handoff requires the 14/14 source fingerprint, static feature-boundary contract, managed
default lib/unit/integration-contract matrix on D/E/F, proof of zero default fixture module/JSON
inclusion, compile/artifact size delta, milestone commit and quantified WeCom notification. Shared
ledgers remain protected until then.
