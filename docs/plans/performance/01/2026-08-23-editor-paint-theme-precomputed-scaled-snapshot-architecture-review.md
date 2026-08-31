---
title: Editor paint-theme prepared snapshot performance review
date: 2026-08-23
module: zircon_editor retained-host host_contract/paint_theme
priority: MVP-P0 editor retained paint and hit-test metrics
status: source_reviewed_m1_applied_static_pass_dynamic_pending
reference_engine: Unreal Engine Slate style registry and application scale authority
---

# Goal

Make theme reads during retained paint and hit testing constant-time snapshot access. Token projection,
scale validation and full metric-table scaling belong to the low-frequency theme/DPI publication edge;
leaf controls must not rebuild an equivalent scaled table on every read.

## Reviewed source

- Rust files: 6/6
- lines: 826
- bytes: 31,444
- joined UTF-8 path, NUL and raw-source-bytes SHA256:
  `354d62a2c2c4876b10d9a26b45482645afab5ad934fd6deca009913dc830e39e`
- owning commit at review: `7762880fd1d8db3d3872888ba8377910177574af`

Scope: `zircon_editor/src/ui/retained_host/host_contract/paint_theme.rs` and
`zircon_editor/src/ui/retained_host/host_contract/paint_theme/**`.

Supporting production paths traced/read: application appearance and effective-DPI updates,
presentation-generation theme capture and paint-scope entry, globals publication, command-stream text
recording, retained controls, pointer hit testing and native-pane paint consumers.

## Correct foundations to retain

1. `ArcSwap<HostPaintThemeSnapshot>` gives readers one immutable generation and lets publication use
   RCU instead of a global reader mutex.
2. A presentation generation captures an `Arc<HostPaintThemeSnapshot>` and enters one scoped theme for
   paint, so one frame does not observe a mixture of theme generations.
3. `apply_host_appearance_from_tokens` projects metrics, palette and typography into one publication;
   repeated effective scale values are rejected before a new generation is allocated.
4. Metrics and token projection validate non-finite or invalid values at their boundary. Palette is
   Copy, text preferences are shared, and no unbounded queue is present.

## Structural findings

### P0: every metric read rescales the entire metric table

`HostPaintThemeSnapshot.metrics` stores unscaled metrics. `host_metrics_for_read` calls
`metrics.at_scale(scale_factor)` both for the active frame snapshot and the global fallback. One call
performs scale validation plus 26 scalar scale-and-finite transforms, then returns a newly assembled
`HostControlMetrics` value. The editor source contains 124 textual `current_host_metrics(` matches
including its definition, spread across paint, layout and hit-test owners.

The presentation paint scope already captures the stable theme once, but leaf reads repeat equivalent
work. With R metric reads between appearance/DPI changes, the current scale work is `26R`; the required
publication-edge work is 26 once. M1 stores base and prepared scaled metrics in each immutable snapshot,
recomputes prepared metrics only when base metrics or scale changes, and makes reads a direct Copy.

### P0: theme dependency remains implicit at leaf accessors

The active snapshot is held in a thread-local `RefCell`, while 124 metric, 135 palette and 8 text-
preference textual matches including definitions can independently consult that implicit authority.
M1 removes the largest repeated calculation without changing this ABI. M2 passes one borrowed prepared
appearance/context through paint and hit-test traversal, then hard-cuts thread-local access from hot
owners once migration tests prove generation consistency.

### P1: palette and typography access still perform per-read ownership work

`current_host_palette` copies the complete palette value on every access. Text preference reads clone an
`Arc`, adding atomic reference-count traffic even while a paint scope already owns the snapshot. M2
returns borrowed fields from an explicit frame context. This is lower priority than the 26-transform
metric rebuild and must be measured before changing public value semantics.

### P1: legacy component-specific update functions publish separate generations

The application correctly uses `apply_host_appearance_from_tokens`, but exported metrics, palette and
text replacement functions can publish three independent snapshots if used sequentially. Besides extra
allocations, intermediate generations can represent only part of an appearance update. M3 makes the
atomic appearance publication authoritative and removes component-wise production mutation after
call-site migration; focused test helpers may remain explicit.

### P1: no counters attribute theme-read work to frame or generation

There are no counts for metrics/palette/text reads, prepared snapshot rebuilds, appearance generations
or DPI publications. M0 instrumentation must distinguish publication work from stable-frame reads so
the M1 static reduction can be confirmed under Welcome, default workbench, large hierarchy/inspector
and plugin-pane workloads.

## Unreal source basis

Direct source read:

- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Styling/SlateStyleRegistry.cpp`
- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Styling/SlateStyleRegistry.h`
- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Styling/ISlateStyle.h`
- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Application/SlateApplicationBase.h`

`FSlateStyleRegistry` retains registered `ISlateStyle` instances in a name-keyed repository and asks the
renderer to load style resources when registration changes. `FindSlateStyle` returns the retained style
pointer by map lookup. `FSlateApplicationBase::GetApplicationScale` exposes application scale as direct
state; the adjacent clock contract explicitly caches time once per frame instead of calling the slower
platform clock at every leaf.

The transferable rules are stable style ownership, publication-edge resource preparation, direct read
access and frame-scoped cached state. Zircon should retain immutable Rust snapshots and RCU publication;
it should not copy Unreal's raw-pointer repository or assume its process-global ownership model.

## Target architecture

1. One immutable prepared appearance snapshot owns base metrics, scaled metrics, palette, typography,
   generation and effective scale.
2. Token or scale mutation validates and prepares the complete new snapshot once, then atomically
   publishes one generation.
3. Presentation generations capture that snapshot once. Paint, layout and hit testing receive an
   explicit borrowed appearance context and read fields without rescaling, Arc cloning or global loads.
4. Metrics/palette/text component-wise mutation is removed from production once the atomic appearance
   path owns all call sites.
5. Counters report snapshot publications, reads by field, generations per frame and preparation time;
   they do not add per-control logging.

## Instrumentation and acceptance

Matrix: theme `stable/change`, scale `1.0/1.25/1.5/2.0/invalid`, scene
`Welcome/default workbench/10k hierarchy/10k inspector`, plugin panes `0/16/128`, damage
`none/one-region/full`, backend `GPU/softbuffer/snapshot`.

| Evidence | Acceptance |
| --- | --- |
| metrics reads and scaled-table rebuilds | stable frame: zero rebuilds; one rebuild per effective base/scale change |
| scalar scale transforms | stable R reads: `26R -> 0`; effective update: exactly 26 |
| palette bytes and text Arc clones | M2 hot traversal: borrowed, zero per-leaf ownership traffic |
| snapshot publications/generations | one atomic generation per appearance or effective-scale change |
| CPU/allocation/RSS/frame latency/context switches/power | same executable/workload before and after |
| visual/pointer/text parity | all scale/theme combinations retain paint and hit-test agreement |

## Milestones

| Milestone | Work | Gate |
| --- | --- | --- |
| M0 | Add prepared-snapshot publication/read/rebuild counters and capture baseline. | attributable base/theme/DPI costs |
| M1 | Store base plus precomputed scaled metrics and make metric reads direct. | stable transforms `26R -> 0`; focused contract green |
| M2 | Pass borrowed appearance context through paint/layout/hit testing. | zero hot palette copies/text Arc clones/global loads |
| M3 | Make atomic appearance update authoritative and remove component publications. | one generation per logical change |
| M4 | Run managed scale, interaction, WPR/power and visual/pointer parity matrix. | quantified accepted milestone |

## M1 implementation result

`HostPaintThemeSnapshot` now retains both token-projected `base_metrics` and the prepared scaled
`metrics`. Appearance and component-metric publication prepare the scaled value against the current
effective scale. Effective-scale publication prepares it against the retained base. Palette and text
updates preserve both values unchanged, and the default/test constructors initialize a consistent
scale-1 pair.

The active paint-scope path and global fallback now return the prepared Copy value directly. All three
remaining `at_scale` call sites are publication boundaries; the hot reader contains none. The extra
snapshot payload is one 27-f32 metric value (108 logical data bytes before Rust layout), paid once per
immutable generation without adding another allocation.

| Static metric work | Before | After | Change |
| --- | ---: | ---: | ---: |
| scale validation per stable metric read | `R` | 0 | eliminated |
| scalar scale-and-finite transforms per stable metric read set | `26R` | 0 | eliminated |
| reconstructed metric values per stable metric read set | `R` | 0 | eliminated |
| transforms per effective base/scale publication | deferred to reads | 26 | prepared once |
| immutable snapshot allocations per effective publication | 1 | 1 | unchanged |

These are deterministic source/operation counts, not elapsed-time or power claims. M0 and M4 still
must measure real R distributions and same-executable frame/power behavior.

Post-M1 direct owner scope:

- Rust files: 6/6
- lines: 827
- bytes: 31,718
- joined UTF-8 path, NUL and raw-source-bytes SHA256:
  `a2ddac60a6399870eeb3d6a5ce6e4a682ab57994db56d33eacc7f30a1d5ab33a`
- unchanged direct owner files: 5 retain their pre-M1 fingerprints inside the joined hash above

| Changed direct owner file | Lines | Bytes | SHA256 |
| --- | ---: | ---: | --- |
| `paint_theme.rs` | 199 | 7,273 | `63524b92f1fd3d86ca9ca4947929f1a012705a8368b61c7618d2ed4abae5b585` |

Focused static contract:
`tools/tests/test_editor_paint_theme_precomputed_metrics_performance_contract.py`, 47 lines,
2,215 bytes, SHA256
`4ede25b0f5b03f2196135c2669a84e2327e604c5bebad8c68443a4510ec6c389`.

## Validation state

- Full direct owner review: passed, 6/6 Rust files.
- Appearance/DPI publication, presentation capture/scope, paint and hit-test consumers: traced/read.
- Relevant Unreal registry, style and scale authority sources: read and mapped above.
- M1 focused static contract: RED 3/3 before implementation, GREEN 3/3 after implementation.
- Current owned editor performance-contract set: GREEN 82/82 across 34 modules.
- Broad editor performance-contract set: 109/114 passed; its five failures are the unchanged known
  missing `component_showcase_state.rs`, missing `workbench_projection.rs`, missing `available_slots`,
  preview resize `.roots.clone()` and UI asset root dirty-helper `.roots.clone()` findings.
- `rustfmt --check` for the changed Rust file and scoped `git diff --check`: passed.
- Managed Rust, WPR and RenderDoc validation remain pending because the managed Cargo Session is
  terminal `archived` with `cargo_session_not_executable`; no elapsed-time, GPU or power claim exists.

This module remains in `pending.md` until M0-M4 pass on one source/executable/workload fingerprint.
