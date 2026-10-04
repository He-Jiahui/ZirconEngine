---
title: Editor workbench host data shared-generation and selection protected plan routing
date: 2026-08-22
status: routing_requested
owner_record: 2026-08-22-editor-ui-workbench-host-data-shared-generation-selection-architecture-review.md
---

# Protected plan updates

## Performance ledgers

Keep one concise `pending.md` module entry:

`zircon_editor/src/ui/layouts/windows/workbench_host_window/{host_data.rs,shell_content_selection.rs,frame_rect.rs,mod.rs}`
- 4/4 Rust files source-reviewed; pane/domain DTOs are deeply cloned across model/scene/host and the
workbench plus retained host own duplicate complete scene shapes; M1 reduces side selection from up to
3S to S lookups, while M0/M2-M5 shared-artifact/profile/power/visual acceptance remains pending.

Do not add these files to `review.md` before M0-M5 pass.

## `docs/plans/performance/01-mvp-performance-audit-and-optimization.md`

Attach M0-M5 to editor retained ownership MVP work. Record deep-clone calls/bytes, simultaneous scene
tree bytes, segment builds, selection visits, allocations, main-thread CPU, latency, RSS and energy.

## `docs/plans/performance/02-unreal-aligned-engine-system-hard-cutover.md`

Own deletion of flat native-body, deep-Clone transfer DTOs and duplicate workbench/host scene trees
after all consumers use immutable typed artifacts and exact generation receipts.

## `docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md`

Own shared shell/chrome/pane/dock/floating/scene artifacts. Stable assembly clones handles; one-domain
invalidation replaces one owner.

## `docs/plans/zircon_editor/editor_layout/03-jetbrains-docking-workbench.md`

Own one-pass priority selection and persistent foreground pane content ownership for all dock stacks.

## `docs/plans/zircon_editor/editor_layout/09-incremental-message-bus-and-refresh.md`

Carry exact dirty-domain generations through surface, scene, partial patch and host apply. Do not
escalate one pane/dock change into a full duplicate scene reconstruction.

## `docs/plans/zircon_editor/editor_layout/15-component-standardization-from-primitives.md`

Own typed artifact contracts and module split by stable ownership domain. A `Clone`-derived monolithic
DTO is not a retained component contract.

## `docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md`

Own immutable runtime UI artifact handles and generation receipts across the editor/runtime host
boundary, with final ABI ownership performed once.

## Acceptance handoff

The owner handoff requires 4/4 post-change fingerprints, managed focused and behavior tests, the full
scale matrix, current-source WPR/power artifacts on D/E/F, interaction and screenshot checks,
RenderDoc parity for GPU content, milestone commit and quantified WeCom notification. Shared ledgers
remain protected until then.
