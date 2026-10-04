---
title: Editor runtime UI render command transient extraction protected plan routing
date: 2026-08-22
status: routing_requested
owner_record: 2026-08-22-editor-runtime-ui-render-command-transient-extraction-architecture-review.md
---

# Protected plan updates

## Performance ledgers

Keep one concise `pending.md` module entry:

`zircon_editor retained-host render_commands/** + render_command_conversion/**`
- 40/40 Rust files source-reviewed. Immediate host conversion serializes each complete runtime command
  for unused cache generation, allocates unused per-element debug labels and clones payloads through
  consecutive DTO layers; partial-opacity images copy/rewrite full RGBA. M1 removes host-route JSON
  generation 1 -> 0 and debug labels per element -> 0 (focused GREEN 2/2; owned contracts GREEN
  54/54). M0/M2-M6 retained generation/text/image/batch/profile/power acceptance remain pending.

Do not add these folders to `review.md` before M0-M6 pass.

## `docs/plans/performance/01-mvp-performance-audit-and-optimization.md`

Attach M0-M6 to the shared MVP editor paint path. Record serialized hash bytes/calls, debug/string/
payload clones, retained reuse/rebuild bytes, text shape/layout, image alpha copies, sort/batch counts,
CPU/allocation/RSS/latency/context switches, RenderDoc GPU/draw and WPR power/energy.

## `docs/plans/performance/02-unreal-aligned-engine-system-hard-cutover.md`

Own removal of per-frame `command -> paint element -> host command` materialization and whole-DTO JSON
hot-path generations after prepared retained consumers migrate.

## `docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md`

Own per-surface retained native/CPU display-list generations and granular dirty-node/batch updates.

## `docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md`

Own monotonic domain generations, prior-entry comparison, shared prepared render-list/text/image
contracts, canonical batch order and per-draw opacity instance state.

## Acceptance handoff

The handoff requires 40/40 post-change fingerprints plus supporting public-contract fingerprints,
managed Rust behavior tests, the scale/dirty/payload matrix, same-executable WPR/power artifacts on
D/E/F, RenderDoc draw/GPU and pixel/text parity, milestone commit and quantified WeCom notification.
Protected ledgers remain unchanged until then.
