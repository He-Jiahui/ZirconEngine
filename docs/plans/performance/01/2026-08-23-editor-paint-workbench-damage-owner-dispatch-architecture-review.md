---
title: Editor paint-workbench damage-owner dispatch performance review
date: 2026-08-23
module: zircon_editor retained-host host_contract/paint_workbench
priority: MVP-P0 editor workbench paint root
status: source_reviewed_architecture_fix_pending_dynamic_pending
reference_engine: Unreal Engine Slate invalidation root and cached widget path
---

# Goal

Replace clip-driven whole-workbench dispatch with a presentation-generation paint plan that identifies
dirty/intersecting retained owners before entering renderer subtrees.

## Reviewed source

- Rust files: 3/3
- lines: 113
- bytes: 3,888
- joined UTF-8 path, NUL and raw-source-bytes SHA256:
  `ef3d78dce82116a92930e8685da0a62c54aa6d94a49b2da2863c8436f18e47d3`
- owning commit at review: `7762880fd1d8db3d3872888ba8377910177574af`

Scope: `zircon_editor/src/ui/retained_host/host_contract/paint_workbench.rs` and
`paint_workbench/**`.

## Correct foundations to retain

1. Production recording selects exactly one componentized or host workbench route.
2. CPU full-frame tests allocate/fill once and region tests mutate the retained backbuffer under a
   restored paint clip, preserving prior pixels outside damage.
3. Full and region test paths expose separate profiling spans for clear, root-frame resolution,
   skeleton and scene drawing.

## Structural findings

### P0: the production root dispatcher has no damage or owner input

`draw_workbench_presentation_commands` accepts only frame and presentation, then enters the complete
componentized/host renderer. Damage is hidden inside `HostRgbaFrame`; the root cannot select retained
scene owners/ranges from it. Thus every patch starts at the same root and depends on descendant gates
or primitive clip rejection to discover irrelevant work.

### P0: CPU region evidence reproduces whole-root traversal

`repaint_host_frame_region` clips/clears the accepted region, then calls the same complete renderer
route. It is test-only, but it is the Softbuffer/pixel-parity evidence path and therefore can understate
the architectural gain if production later becomes owner-routed while this harness remains clip-only.

### P1: full/region/componentized branch policy is duplicated

The full test path, region test path and production recording path independently choose componentized
versus host rendering and profile it differently. M1 replaces these branches with one prepared
`WorkbenchPaintPlan` consumed by recording and CPU evidence, preventing acceptance from testing a
different traversal policy.

## Unreal source basis

Direct source read:

- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/FastUpdate/SlateInvalidationRoot.cpp`
- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/FastUpdate/WidgetProxy.cpp`
- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Rendering/DrawElements.h`

Unreal builds a retained widget path/update list and chooses fast versus slow paint before traversal.
Cached element data remains attached to the invalidation root/window. The transferable rule is that
the root consumes an explicit update plan; a culling rectangle remains a draw correctness tool, not
the scheduler.

## Target architecture

1. Presentation generation owns a `WorkbenchPaintPlan`: componentized/host mode, typed damage state,
   dirty root/chrome/dock/overlay ranges and spatially intersecting pane/node owners.
2. Production recording and CPU parity consume the same plan. Empty returns before renderer dispatch;
   Regions visit only listed owners; Full visits all owners with an attributed reason.
3. Descendant clips remain correctness guards. Counters distinguish owners scheduled, visited, reused,
   rebuilt and rejected at leaves.

## Milestones and acceptance

| Milestone | Work | Gate |
| --- | --- | --- |
| M0 | Count root dispatch, owner/range visits, leaf clip rejects and plan reasons. | attributable baseline |
| M1 | Introduce one prepared workbench paint plan for recording and CPU parity. | no branch-policy duplication |
| M2 | Connect typed damage/spatial indices and retained ranges. | region traversal proportional to dirty intersections |
| M3 | Run managed Welcome/workbench/plugin scale, WPR and RenderDoc/pixel matrix. | quantified accepted milestone |

## Validation state

- Full direct owner review: passed, 3/3 Rust files.
- Renderer root, recording and CPU retained-backbuffer consumers: traced/read.
- No local implementation was applied because damage/owner identity is not present in this ABI; adding
  another clip check at the root would not eliminate subtree preparation.
- Managed Rust, WPR and RenderDoc validation remain pending under archived managed Cargo Session
  `cargo_session_not_executable`; no elapsed-time, GPU or power claim exists.

This module remains in `pending.md` until M0-M3 pass on one source/executable/workload fingerprint.
