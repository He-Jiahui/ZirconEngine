---
title: Editor Workbench page tabs single strip layout authority performance review
date: 2026-08-22
module: zircon_editor/src/ui/workbench/page_tabs
priority: MVP-P1
status: source_reviewed_static_pass_dynamic_pending
reference_engine: Unreal Engine Slate docking tab well
---

# Goal

Preserve the current page-tab sizing and responsive overflow policy, but compute visible tabs, exact
frames, close frames, hidden-page identity and overflow geometry once per committed generation.
Projection, pointer dispatch and overflow presentation must consume that same result.

## Reviewed source

- folder: `zircon_editor/src/ui/workbench/page_tabs`
- Rust files: 2/2
- lines: 159
- bytes: 5,807
- joined UTF-8 SHA256: `202ae2e201697f992c893458c644d7a1727f50804091f98b0fe192765d766bb2`
- `metrics.rs`: `5cf276746aa147a89167741c2914e4ccffc9f1f4361a098af46dc7769bf70b02`
- `mod.rs`: `ff2622c8471ca3bce77ebdbe07774e4953cbc4498668496349a3fd92311c200c`
- owning commit before review: `08094b9b9e17f6c80372e15c17b01204038b305b`

Both files were read in full. Related algorithms were read in:

- `chrome_template_projection.rs:409-595` and `page_tabs.rs`;
- `host_page_pointer/build_host_page_pointer_layout.rs`;
- `host_page_pointer/tab_strip_geometry.rs`;
- existing 20/20 host-page pointer and 11/11 pointer-layout performance reviews;
- host recompute pointer-surface call sites.

## Result

### Source module

All production functions in `metrics.rs` are bounded constant-time policy/math. They sanitize
non-finite sizes, bound tab/close/project-path geometry and map a logical width to a responsive
visible cap. Four tests cover typography, title-width clamping, close geometry and project-path
collapse. `rustfmt --edition 2021 --check` passes for 2/2 files. No isolated edit is justified.

The narrow/ultra cap of two tabs is an explicit readability policy, not a performance algorithm. It
may be changed only with layout/interaction evidence, not as a micro-optimization.

### P1: projection and pointer own duplicate strip allocators

`visible_page_tab_indices` and `host_page_pointer::visible_indices` independently perform the same
work:

- derive project-path reserve and responsive visible cap from width;
- iterate candidates and measure Runtime Text title widths;
- reserve overflow space and stop at the available boundary;
- replace the last visible item with the active page when necessary;
- derive hidden indices and overflow placement.

The projection repeats title measurement again while emitting nodes. The pointer builder clones every
page ID/title/optional close-instance ID before allocation, builds a `BTreeSet` of visible indices and
then scans all pages to derive hidden indices. The bridge equality guard runs only after this owned
layout is built.

The asymptotic cost is small for current human-scale page counts, but ownership is structurally wrong:
two implementations can drift and an unchanged unrelated Host recompute still pays page clones,
measurements and allocation before equality. The existing `PERF-MVP-106` review already identifies
this compare-after-build pattern; this record supplies the current `page_tabs` owner boundary.

### Autolayout dependency

`main_page_tab_visible_cap_for_width` reclassifies logical width through autolayout on every caller.
The operation is constant-time, but the future single runtime layout generation should publish the
resolved `WorkbenchLayoutTier`. Page tabs should consume that tier receipt rather than independently
derive another layout authority.

## Unreal source basis

Direct source read:

- `dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Docking/SDockingTabWell.cpp`
- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Widgets/SWidget.cpp`
- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Widgets/SWidget.h`

`SDockingTabWell::OnArrangeChildren` computes the child-size policy once and performs one linear
arrangement pass (`100-189`). Paint consumes arranged children. `ComputeDesiredSize` reads cached child
desired sizes (`268-295`), and Slate's `SWidget::CacheDesiredSize` owns those cached values. The
relevant standard is a persistent tab-well/layout owner with cached child metrics, not Unreal's
uniform-width choice or exact pixels.

## Target architecture

Create one immutable committed page-strip layout owned by the Host chrome/layout generation. Its key
contains:

- page topology/title/active/close-target generation;
- committed autolayout tier and strip frame;
- Runtime Text metrics generation;
- chrome style/density generation;
- project-path/preset presentation generation when it changes available width.

Its value contains visible tab identities and exact frames, close frames, hidden page indices,
overflow frame, project-path frame and total content width. Chrome projection publishes it;
host-page pointer and overflow menu borrow/share it. The pointer layer must not clone titles or rerun
responsive allocation.

Complexity target:

- invalidated layout: one `O(P + glyph_measurement)` pass for `P` pages;
- unchanged unrelated recompute: zero page visits/clones/measures/layout builds;
- pointer/overflow lookup: consume committed route/frame identity without rebuilding;
- memory: one current artifact per live Host surface, bounded by current pages.

## Instrumentation and acceptance

Add counters before migration:

| Counter | Unchanged unrelated recompute target |
| --- | --- |
| page-strip layout builds | 0 |
| page title measurements | 0 |
| page ID/title/close-target cloned bytes | 0 |
| hidden-index set/vector builds | 0 |
| pointer layouts built then rejected by equality | 0 |
| projection/pointer/overflow generation mismatch | 0 always |

Matrix: pages `1/2/8/100/1k`, widths `320/640/900/1260/1920`, scale `1/1.25/1.5/2`, active page
first/middle/last, closeable `0/1/100%`, stable/unrelated/title/active/topology/resize/font changes and
overflow keyboard/pointer selection. Record median/p95 main-thread time, visits, allocations/bytes,
input-to-pixel latency, RSS and package energy before/after on the same current-source profile.

WPR/ETW and temporary build artifacts belong on `D:`, `E:` or `F:`. RenderDoc is only for final
draw/pixel parity after a launchable current-source renderer exists; it cannot validate CPU layout or
allocation removal.

## Milestones

| Milestone | Work | Gate |
| --- | --- | --- |
| M0 | Add counters and capture page/width/invalidation baseline. | Current-source counter and WPR artifacts |
| M1 | Consume committed autolayout tier and define immutable page-strip layout. | Key/invalidation tests |
| M2 | Make projection, pointer and overflow consume the artifact. | Focused behavior/source ownership tests |
| M3 | Delete duplicate production allocators and compare-after-build path. | Zero duplicate owner scan |
| M4 | Run scale matrix, real-window screenshots and interaction parity. | Quantified WPR/allocation/power evidence |

## Validation state

- Full folder review: passed, 2/2 files.
- Static formatting: passed, 2/2 files.
- Existing local tests identified: 4.
- Managed Cargo: pending while shared Runtime Cargo lanes remain active.
- Dynamic profiling, current-source real-window and renderer parity: pending.

The folder remains in `pending.md` and must not enter `review.md` until M0-M4 pass on one fingerprint.
