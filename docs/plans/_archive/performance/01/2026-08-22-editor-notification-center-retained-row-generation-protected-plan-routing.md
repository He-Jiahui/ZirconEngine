---
title: Editor notification-center retained row-generation protected plan routing
date: 2026-08-22
status: routing_requested
owner_record: 2026-08-22-editor-notification-center-retained-row-generation-architecture-review.md
---

# Protected plan updates

## Performance ledgers

Keep one concise `pending.md` module entry:

`zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/notification_center/**`
- 8/8 Rust files source-reviewed. Visible-limit early stop and workbench generation reuse are correct,
  but changed typed notification rows still cross pipe-string/TOML parsing and retain parallel
  legacy/structured/joined titles. M1 removes per-production-row default-title cloning and lowercase
  tone temporaries (focused contract GREEN 3/3; owned contracts GREEN 38/38). M0/M2-M5 typed-
  generation/profile/power/interaction acceptance remain pending.

Do not add these files to `review.md` before M0-M5 pass.

## `docs/plans/performance/01-mvp-performance-audit-and-optimization.md`

Attach M0-M5 to the existing P0 unified activity-notification item. Distinguish upstream stable-tick
snapshot/localization/encoding work from changed-generation retained parsing and row construction.

## `docs/plans/performance/02-unreal-aligned-engine-system-hard-cutover.md`

Own deletion of pipe/TOML notification row transport and legacy options/join owners after the shared
typed generation is live. Preserve visible-limit early stop and unversioned stale-row protection.

## `docs/plans/zircon_editor/editor/17-editor-services-and-recovery.md`

Own one immutable `ActivityNotificationProjection`, source/locale revision tuple, shared typed rows,
selected identity, unread/overflow and next-expiry receipt.

## `docs/plans/zircon_editor/editor/14-threading-and-job-scheduling.md`

Publish Progress revisions/shared rows without per-tick duplicate map/snapshot materialization.

## `docs/plans/zircon_editor/editor/04-pie-and-simulation.md`

Publish indexed Decision revisions and localized typed choices without nested resnapshot matching.

## `docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md`

Carry and apply one changed typed generation at most once; coalesce tick and dispatch dirtiness.

## `docs/plans/zircon_editor/editor_ui/01-slate-input-dispatch-core.md`

Share stable notification row IDs across paint, hit, keyboard and accessibility and patch exact
focus/selection rows.

## `docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md`

Own a retained typed list-generation contract that can carry notification and other searchable/list
rows without generic presentation codecs.

## Acceptance handoff

The handoff requires 8/8 post-change fingerprints, managed focused and behavior tests, the full
source/visible-limit/row-width matrix, current-source WPR/power artifacts on D/E/F, interaction and
accessibility parity, RenderDoc parity where notification paint changes, milestone commit and
quantified WeCom notification. Protected ledgers remain unchanged until then.
