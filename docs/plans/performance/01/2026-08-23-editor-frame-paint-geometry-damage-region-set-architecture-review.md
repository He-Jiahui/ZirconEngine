---
title: Editor frame/paint geometry and damage-region-set performance review
date: 2026-08-23
module: zircon_editor retained-host frame_geometry and paint_geometry
priority: MVP-P0 editor damage propagation, hit routing and primitive clipping
status: source_reviewed_architecture_pending_dynamic
reference_engine: Unreal Engine Slate invalidation root, clipping and SlateRect contracts
---

# Goal

Make editor redraw cost proportional to the actual changed regions instead of the bounding rectangle
between them. Logical layout validity, stable paint coverage and device-pixel clipping must have named,
canonical semantics so a region accepted by invalidation is not silently rejected by paint.

## Reviewed source

- Rust files: 9/9
- lines: 320
- bytes: 9,541
- joined normalized UTF-8 path, NUL and raw-source-bytes SHA256:
  `696d6ed7995f078a1fd4f619b3f13d24176656f3d755010be9ab7a9dd8ff0ec8`
- owning commit at review: `a922089697e41e07fa29e3e42a5e4c9afc1ae31b`

Scope:

- `frame_geometry.rs` and `frame_geometry/**`
- `paint_geometry.rs` and `paint_geometry/**`

Supporting production paths traced/read: redraw request constructors/merge, native pointer hover/menu/
dock/resize damage, paint recording damage, paint-frame clip, chrome extraction/stream visibility,
surface hit-test, profiling geometry, primitive pixel bounds and `FrameRect` definitions.

## Correct foundations to retain

1. Geometry operators are allocation-free fixed-size math. Finite/positive checks prevent NaN and
   infinite values from reaching pixel casts in the canonical paint path.
2. `PixelRect::from_frame` intersects first, floors minima, ceils maxima and clamps to surface bounds;
   no negative or oversized index reaches the raster loop.
3. Inward pixel alignment only removes coverage. Radius and extent helpers reject non-finite/negative
   values and bound radii to half the smallest extent.
4. Redraw requests preserve whether a presentation frame update is required and retain the latest
   scenario attribution through merges.
5. Frame hit tests and Unreal `FSlateRect::ContainsPoint` both use inclusive edges. This convention is
   acceptable only while candidate order/z resolution is authoritative; it is not changed as a local
   performance tweak.

## Structural findings

### P0: every pair of dirty regions is collapsed to one bounding rectangle

`HostRedrawRequest` can represent only `None`, one `Region`, `Full` or frame-update-only. Merging two
regions calls `union_frame`, and native hover/menu/tab/dock helpers also union old/new frames before the
request boundary. Two distant small changes therefore repaint every pixel and rebuild every accepted
command between them.

For two 16x16 regions at opposite corners of a 1,920x1,080 window, useful dirty area is 512 pixels but
the bounding union is 2,073,600 pixels: an amplification of 4,050x before upper-layer traversal and
command work. This is an algorithm/ownership defect, not an intersection micro-optimization.

M1 introduces a bounded `DamageRegionSet` owned by the redraw/invalidation context. It retains disjoint
regions, clips them to the window, merges only when measured cost decreases and promotes to full redraw
when region count/covered-area/command overhead crosses a measured budget. Damage consumers iterate the
same set; no compatibility path may immediately collapse it back to one rectangle.

### P0: logical visibility and paintability use incompatible anonymous thresholds

`frame_geometry::visible_frame` accepts finite extents greater than zero. `paint_geometry::is_visible_frame`
requires both extents to exceed 0.5, and intersection requires more than half a device pixel. The latter
has an explicit stable-coverage rationale, but callers and wrappers use nearly identical names.

Consequently a `0 < extent <= 0.5` frame can enter damage merge/hit/layout paths, schedule a region
redraw and later be rejected by paint. Repeated fractional hover/layout churn can produce no-op redraws.
Changing every threshold to 0.5 would incorrectly erase logical layout/hit semantics.

M2 defines separate typed predicates: finite positive logical rect, device-scale-aware paintable rect,
non-empty mathematical intersection and non-empty raster pixel rect. Each subsystem declares which
policy it consumes; the device scale is part of paintability rather than a fixed logical-pixel guess.

### P0: visibility, intersection and union have many parallel owners

The current host contract contains at least 12 production `visible_frame`/`is_visible_frame` definitions
or wrappers, five intersection owners including the canonical `intersect`, and local optional-union
implementations in pointer paths. Some use `>0`, some `>0.5`, and some construct the intersection before
validating inputs.

This duplication is already behavioral drift, not merely maintenance cost. M2/M3 hard-cut consumers to
one geometry module and delete local copies after semantic tests are attached. Thin forwarding wrappers
that preserve old ambiguity are rejected.

### P0: union accepts invalid/negative inputs and can poison damage bounds

`union_frame` assumes well-formed frames and applies floating min/max directly. Several callers filter
through `union_visible_frame`, but direct redraw, menu and hover callers union trusted frames without a
shared invariant. NaN propagation can create a region that later disappears; negative extents can make
the bounding result unrelated to either input.

The new damage set accepts only canonical finite paintable regions and records rejected-region counters.
Logical union remains separately available for layout bounds and documents its preconditions.

### P1: damage amplification is invisible to current counters

Existing counters count full/region requests but not input region count, useful area, bounding-union
area, post-clip area or promotion reason. A system reporting one region can therefore look efficient
while repainting most of the window. M0 adds deterministic counters before changing representation.

### P1: pixel conversion and intersection coverage are mixed

Mathematical intersection can be positive while stable paint coverage or clamped pixel coverage is
empty. Current `intersect` rejects at 0.5 before device scale and `PixelRect` performs another conversion.
M2 moves coverage policy to the raster/paint boundary and keeps exact intersections for layout/damage
set operations, preventing threshold-dependent union/clip drift.

## Unreal source basis

Direct source read:

- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Layout/SlateRect.h`
- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Rendering/DrawElementTypes.cpp`
- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Input/HittestGrid.cpp`
- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/FastUpdate/SlateInvalidationRoot.cpp`
- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/FastUpdate/WidgetProxy.cpp`
- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Rendering/DrawElements.h`

Unreal separates `FSlateRect` exact expand/intersection/containment math from draw culling based on clip
area. Its hit-test grid keeps candidate cells/order and tests clip masks rather than treating one unioned
rectangle as the invalidation model. The invalidation root and widget proxies retain per-widget cached
state/elements, so disjoint widget invalidations need not rebuild the bounding space between them.

The transferable constraints are exact typed rectangle math, policy-specific clipping/culling, retained
invalid owners and multiple dirty elements. Zircon should not copy raw-pointer widget proxies or assume
Unreal's inclusive hit edge alone resolves candidate priority.

## Target architecture

1. `FrameRect` exposes canonical finite-positive, exact intersection and logical union operations from
   one owner. Layout/hit code consumes logical semantics.
2. Paint geometry adds device-scale-aware coverage and pixel conversion; stable coverage thresholds do
   not leak into logical geometry.
3. A retained `DamageRegionSet` carries disjoint clipped regions through scene routing, command recording
   and backend presentation, with measured merge/full-promotion policy.
4. Scene/range generations attach to damage regions so command rebuild/replay remains proportional to
   affected roots, not merely affected pixels.
5. Profiling records input/output region count, useful/union/clipped area, amplification, promotion reason
   and rebuilt/reused command ranges.

## Instrumentation and acceptance

Matrix: regions `0/1/2/8/64/1k`, placement `overlap/adjacent/opposite corners/random`, size
`1/16/256/full`, extent `0/0.25/0.5/0.51/1`, scale `1/1.25/2`, source
`hover/menu/tab/resize/plugin`, backend `GPU/softbuffer/snapshot`.

| Evidence | Acceptance |
| --- | --- |
| input/retained/merged region counts and promotion reason | bounded, deterministic policy |
| useful, bounding-union, clipped and presented area | amplification reported; disjoint work retained |
| no-op region redraws | zero accepted-damage/zero-paint mismatches |
| traversal and rebuilt/reused command ranges | work proportional to damaged scene ranges |
| geometry policy parity | logical/hit/paint/pixel thresholds explicit and tested |
| CPU/allocation/RSS/latency/context switches/power | same executable/workload before and after |
| RenderDoc draw/batch/scissor/GPU and pixel parity | accepted current backend build |

## Milestones

| Milestone | Work | Gate |
| --- | --- | --- |
| M0 | Add region count/area/amplification/promotion/no-op and command-range counters; capture. | attributable baseline |
| M1 | Introduce bounded retained `DamageRegionSet` through redraw, scene and backend boundaries. | disjoint regions do not bounding-union by default |
| M2 | Define canonical logical, exact-intersection, paintable and pixel-coverage policies. | no anonymous threshold drift |
| M3 | Hard-cut local visibility/intersection/union owners and attach scene generations. | one owner per semantic operation |
| M4 | Run managed interaction/scale/WPR/power and RenderDoc/scissor/pixel matrix. | quantified accepted milestone |

## Validation state

- Full direct owner review: passed, 9/9 Rust files.
- Redraw merge, pointer damage, paint/frame/command/profiling and hit-test consumers: traced/read.
- Relevant Unreal rect, culling, hit-grid and invalidation sources: read and mapped.
- No code change applied: a single-region ABI change without counters/backend propagation would preserve
  or hide the amplification instead of solving it.
- Current owned editor performance-contract set remains GREEN 79/79; broad set remains 106/111 with the
  five unchanged known failures documented by adjacent reports.
- Managed Rust, WPR and RenderDoc validation remain pending because the managed Cargo Session is
  terminal `archived` with `cargo_session_not_executable`; no elapsed-time, GPU or power claim exists.

These modules remain in `pending.md` until M0-M4 pass on one source/executable/workload fingerprint.
