---
title: Editor chrome command-stream atlas, resource and retained-range performance review
date: 2026-08-23
module: zircon_editor retained-host chrome_command_stream
priority: MVP-P0 editor invalidated-frame extraction and GPU submission
status: source_reviewed_m1_applied_static_pass_dynamic_pending
reference_engine: Unreal Engine Slate invalidation, window element list, texture atlas and RHI resource manager
---

# Goal

Make invalidated editor chrome work proportional to changed paint ranges and newly admitted resources.
A stable icon, text run or image generation must not repeatedly clone identifiers, rebuild equivalent
command vectors or re-upload pixels merely because another region of the window was invalidated.

## Reviewed source

- Rust files: 42/42
- lines: 3,834
- bytes: 125,284
- joined UTF-8 path, NUL and raw-source-bytes SHA256:
  `4a9a4a8cbd02e1b776a04626e1a02e991aca6a41609be3ed1b5d907aacf8cd7f`
- owning commit at review: `a922089697e41e07fa29e3e42a5e4c9afc1ae31b`

Scope: `zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/{mod.rs,/**}`.

Supporting production paths traced/read: retained-host command recording, presentation snapshot,
GPU presenter normal and cached submission, softbuffer/CPU replay, Runtime draw-list conversion,
Runtime text face capture and image resource upload/residency handling.

## Correct foundations to retain

1. Damage-aware paint recording rejects disjoint primitives before command extraction. The command
   stream preserves monotonic z order, and CPU replay sorts only a malformed fallback stream.
2. Image pixel payloads use `Arc<[u8]>`; resource compaction canonicalizes equal generations and can
   omit pixels already resident in the backend.
3. The icon atlas is generation-qualified, bounded by pages/bytes, uses deterministic packing and LRU
   page eviction, and rewrites commands to atlas subrects without changing visual bounds.
4. Owned Runtime draw-list conversion moves text and resource identifiers. The borrowed cached path
   remains valid for snapshot reuse, and Runtime text face capture occurs once per stream rather than
   once per text command.
5. Replay clipping and visible-frame checks prevent off-window raster work. Statistics and expensive
   uniqueness sets are test/diagnostic paths, not normal GPU submission work.

## Structural findings

### P0: an invalidated frame passes through three command vectors

The producer first records `Vec<HostRecordedPaintCommand>`. Extraction consumes it into a second
`Vec<ChromeCommand>`. `ChromeCommandStream` then creates a third vector and extends it before image
resource compaction. Normal GPU presentation repeats this pipeline for every accepted invalidation,
even when most retained scene generations did not change.

This is O(C) moves plus repeated capacity ownership per invalidated frame. Damage reduces C but does
not retain stable root/chrome/dock/overlay ranges. M2 gives the window one reusable command arena and
generation-qualified ranges; it must integrate with the scene-layer and paint-frame plans rather than
add a fourth cache authority.

### P0: stable icon admission scans commands three times and clones keys under one global lock

`EditorIconAtlas::pack` locks the process atlas and currently performs two immutable icon scans before
one mutable rewrite scan. `icon_source_from_command` creates an owned `IconSourceKey`, so a stable icon
resource key is cloned during active-page discovery, pending discovery and command rewrite. For I icon
commands the stable case is three command scans and about `3I` String clones while holding the mutex.

M1 changes the persistent slot index to a borrowed outer String lookup plus a compact generation/size
version key. Active-page discovery and missing-resource collection share one immutable scan; only a
new unique icon clones its key for admission. Rewriting performs borrowed lookup. Stable work becomes
two scans and zero resource-key String clones without changing page packing, eviction or command ABI.

### P0: atlas mutation and stable lookup share one coarse process mutex

Even after M1, command scans, pending-resource ordering, page allocation, pixel writes and eviction are
serialized by the global atlas mutex. M3 splits immutable generation lookup from mutation/publication,
defines one explicit atlas owner thread and measures lock wait. A lock replacement without contention
evidence is rejected; the ownership boundary must first guarantee that readers never observe a partly
published page.

### P0: cached GPU submission recreates an owned Runtime draw list

The normal presenter rebuilds a chrome stream and converts it to a fresh Runtime command vector.
Snapshot/cached submission can borrow the stream, but borrowed conversion clones text Strings and image
resource keys into another Runtime list. Thus command-stream caching does not yet imply draw-list range
reuse. M4 makes the backend consume stable prepared ranges/handles and preserves one text-face capture
per stream generation.

### P0: text and image identifiers remain command-owned Strings

Text content and resource keys are copied into command variants at recording/extraction boundaries.
Image compaction additionally builds a resource-key/generation map and clones keys into the resource
table. Pixel ownership is shared correctly, but identifier bytes remain O(total accepted identifier
bytes) per rebuild. M4 replaces duplicated identifiers with canonical generation-qualified handles;
hard cut removes the old owned String variants once all consumers migrate.

### P1: CPU atlas replay materializes a subimage per atlas draw

CPU/softbuffer atlas replay obtains an owned RGBA subimage for each atlas command before rasterization.
This is not the normal GPU path, but snapshot-heavy tests or software presentation pay allocation and
copy cost proportional to atlas draw pixels. M5 adds a borrowed strided atlas view or direct sampled
replay and retains pixel-parity coverage.

### P1: atlas pixel padding allocates temporary row buffers

New icon admission clones first/last pixels into small vectors and creates padded rows for top/bottom
edges. It occurs only for newly admitted icons, not stable frames. M5 replaces these temporaries with
fixed pixels and direct row copies after admission counters prove it is material.

### P1: visibility/intersection ownership is duplicated

Extraction, stream geometry and adjacent paint/replay owners contain equivalent visible-frame and
finite intersection logic. This is not the primary cost, but semantic drift can admit commands that
later layers reject. M5 converges on one typed geometry contract after equivalence tests.

## Unreal source basis

Direct source read:

- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/FastUpdate/SlateInvalidationRoot.cpp`
- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/FastUpdate/WidgetProxy.cpp`
- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Rendering/DrawElements.h`
- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Textures/TextureAtlas.h`
- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Textures/TextureAtlas.cpp`
- `dev/UnrealEngine/Engine/Source/Runtime/SlateRHIRenderer/Private/SlateRHIResourceManager.cpp`
- `dev/UnrealEngine/Engine/Source/Runtime/SlateRHIRenderer/Private/SlateRHIResourceManager.h`

Unreal's invalidation root chooses cached fast-path updates instead of reconstructing all window draw
elements. `FSlateWindowElementList` is a noncopyable window-owned list with cached and uncached element
ranges. `FSlateTextureAtlas` retains CPU atlas data plus used/free slot structures, copies data and marks
the texture dirty only on admission, and records an explicit owner thread. The RHI resource manager
keeps stable-key maps and returns existing dynamic/atlased resources through direct map lookup.

The transferable rules are retained range ownership, persistent slot/resource maps, admission-only
pixel work, explicit mutation ownership and stable-key lookup. Zircon keeps Rust ownership, bounded
LRU eviction and backend generations; it must not reproduce Unreal raw-pointer lists or assume that
Unreal's thread model is directly portable.

## Target architecture

1. One retained window command owner holds a reusable arena and stable root/chrome/dock/overlay ranges.
2. Invalidation patches only ranges whose scene/resource generations changed, preserving z order.
3. Resource registries assign compact generation-qualified handles; commands do not own repeated text
   or image identifier bytes.
4. The icon atlas exposes borrowed stable lookup, admission-only key/pixel ownership, bounded pages and
   atomic generation publication from one measured mutation owner.
5. GPU/softbuffer/snapshot consumers borrow the same prepared command ranges. Backend-specific upload
   state is separate from command identity and reports resident/reused/admitted bytes.
6. CPU atlas replay samples borrowed atlas storage without one temporary subimage per draw.

## Instrumentation and acceptance

Matrix: commands `0/1/1k/10k/100k`, icons `0/1/1k stable/1k new/eviction`, text bytes
`0/1k/1M`, images `0/1/1k resident/new`, damage `none/one-range/full`, backend
`GPU/softbuffer/snapshot`, window `steady/resize`, plugin surfaces `0/16/128`.

| Evidence | Acceptance |
| --- | --- |
| command scans/moves, Vec growth and rebuilt/reused ranges | stable work proportional to dirty ranges |
| icon key clone bytes, lookup/admission count and atlas lock wait | stable icons: two scans, zero key clones; no unexplained wait |
| atlas page alloc/evict/upload bytes | zero stable admission/upload; bounded residency |
| text/resource owned/shared bytes | no duplicate command identifier ownership |
| CPU replay allocations/copied pixels | no per-atlas-draw subimage allocation |
| CPU/allocation/RSS/latency/context switches/power | same executable/workload before and after |
| RenderDoc draw/batch/upload/GPU and pixel/text parity | accepted current backend build |

## Milestones

| Milestone | Work | Gate |
| --- | --- | --- |
| M0 | Add command move/range, key-clone, atlas admission/upload/eviction/lock-wait and resource-residency counters; capture baseline. | attributable baseline |
| M1 | Use borrowed nested atlas lookup and merge active/pending discovery. | stable scans `3 -> 2`, key clones `3I -> 0` |
| M2 | Add reusable per-window command arena and generation-owned ranges. | rebuild proportional to dirty ranges |
| M3 | Split atlas stable lookup from admission/publication under explicit owner. | bounded measured lock wait, atomic generations |
| M4 | Replace command Strings and transient Runtime vectors with canonical handles/prepared ranges. | zero duplicate identifier bytes and steady Vec growth |
| M5 | Remove CPU atlas subimage/row temporaries and converge geometry semantics. | allocation-free stable CPU replay, parity retained |
| M6 | Run managed scale, interaction, WPR/power and RenderDoc/pixel/text matrix. | quantified accepted milestone |

## M1 implementation result

The persistent atlas slot index is now a nested
`BTreeMap<String, BTreeMap<IconSourceVersion, IconAtlasSlot>>`. Stable lookup borrows the outer
resource key as `&str` and uses a compact Copy version containing generation, width and height.
`IconSource` borrows both the command key and `Arc` pixels; it does not construct an owned source
tuple.

One immutable command scan now marks active pages and builds a nested pending-admission table. The
pending table owns a resource name only on its first missing occurrence and shares repeated versions.
Each resource name is moved once into the persistent index after its admitted versions are packed.
The mutable rewrite scan performs borrowed lookup before replacing the command payload with the atlas
page handle. Page allocation, LRU eviction, immutable page publication, generation and UV behavior are
unchanged.

| Static atlas work | Before | After | Change |
| --- | ---: | ---: | ---: |
| command-stream passes in `pack` | 3 | 2 | -33.3% |
| stable source-key String clones for I eligible icons | about `3I` | 0 | eliminated |
| stable source-pixel owner clones before rewrite | `2I` | 0 | eliminated |
| new pending resource-name ownership | per encountered source tuple | once per unique missing name | deduplicated |
| page `Arc` owner written to each rewritten command | `I` | `I` | unchanged ABI |

These are deterministic source/operation counts, not elapsed-time claims. M0 and M6 still must measure
real command distributions and mutex wait before judging frame time or power.

Post-M1 direct owner scope:

- Rust files: 42/42
- lines: 3,862
- bytes: 126,475
- joined UTF-8 path, NUL and raw-source-bytes SHA256:
  `53210644bb01c4fb6e525c3a529bbf94a3ddeca771b147f68e7b51f31403ce4b`
- unchanged direct owner files: 41 retain their pre-M1 fingerprints inside the joined hash above

| Changed direct owner file | Lines | Bytes | SHA256 |
| --- | ---: | ---: | --- |
| `chrome_command_stream/icon_atlas.rs` | 532 | 19,200 | `55ec70ea0ce159f5b21f86777b158a0fdb87aa69350b741a73c45fd70bbb5bd1` |

Focused static contract:
`tools/tests/test_editor_icon_atlas_borrowed_lookup_performance_contract.py`, 49 lines, 1,942 bytes,
SHA256 `8b7312aba39ef574760218f8c506b939bce699a673661768dfc0f9f25082e2ca`.

## Validation state

- Full direct owner review: passed, 42/42 Rust files.
- Normal/cached GPU submission, softbuffer/CPU replay, Runtime conversion and resource upload paths:
  traced/read.
- Relevant Unreal invalidation, draw-element, atlas and resource-manager sources: read and mapped above.
- M1 focused static contract: RED 3/3 before implementation, GREEN 3/3 after implementation.
- Current owned editor performance-contract set: GREEN 77/77 across 32 modules.
- Broad editor performance-contract set: 104/109 passed; its five failures are the unchanged known
  missing `component_showcase_state.rs`, missing `workbench_projection.rs`, missing `available_slots`,
  preview resize `.roots.clone()` and UI asset root dirty-helper `.roots.clone()` findings.
- `rustfmt --check` for the changed Rust file and scoped `git diff --check`: passed.
- Existing icon atlas Rust behavior tests cover stable generation, immutable publication, unrelated
  page generations, bounded LRU eviction and standalone images, but are not claimed passing until
  managed Cargo is executable.
- Managed Rust, WPR and RenderDoc validation remain pending because the managed Cargo Session is
  terminal `archived` with `cargo_session_not_executable`; no elapsed-time, GPU or power claim exists.

This module remains in `pending.md` until M0-M6 pass on one source/executable/workload fingerprint.
